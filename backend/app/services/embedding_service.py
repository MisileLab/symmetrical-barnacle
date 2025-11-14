"""Embedding and vector database service"""
import chromadb
from chromadb.config import Settings as ChromaSettings
from typing import List, Dict, Any, Optional
import logging
from sentence_transformers import SentenceTransformer
import numpy as np
import openai

from app.config import settings

logger = logging.getLogger(__name__)


class EmbeddingService:
    """Service for managing game embeddings and vector search"""

    def __init__(self):
        self.openai_client = openai.AsyncOpenAI(api_key=settings.openai_api_key)

        # Initialize ChromaDB client
        try:
            self.chroma_client = chromadb.HttpClient(
                host=settings.chroma_host,
                port=settings.chroma_port,
                settings=ChromaSettings(anonymized_telemetry=False)
            )

            # Get or create collection
            self.collection = self.chroma_client.get_or_create_collection(
                name="game_embeddings",
                metadata={"description": "Steam game embeddings"}
            )
            logger.info("ChromaDB client initialized successfully")
        except Exception as e:
            logger.error(f"Failed to initialize ChromaDB: {e}")
            self.chroma_client = None
            self.collection = None

        # Initialize local embedding model as fallback
        try:
            self.local_model = SentenceTransformer('all-MiniLM-L6-v2')
            logger.info("Local embedding model loaded successfully")
        except Exception as e:
            logger.error(f"Failed to load local embedding model: {e}")
            self.local_model = None

    async def create_game_embedding(self, game_data: Dict[str, Any], use_openai: bool = True) -> Optional[List[float]]:
        """
        Create embedding for a game based on its metadata

        Args:
            game_data: Dictionary containing game information
            use_openai: Whether to use OpenAI API or local model
        """
        try:
            # Create text representation of the game
            text_parts = []

            if game_data.get("title"):
                text_parts.append(f"Title: {game_data['title']}")

            if game_data.get("short_description"):
                text_parts.append(f"Description: {game_data['short_description']}")

            if game_data.get("genres"):
                genres = ", ".join(game_data["genres"]) if isinstance(game_data["genres"], list) else game_data["genres"]
                text_parts.append(f"Genres: {genres}")

            if game_data.get("tags"):
                tags = ", ".join(game_data["tags"][:10]) if isinstance(game_data["tags"], list) else game_data["tags"]
                text_parts.append(f"Tags: {tags}")

            if game_data.get("categories"):
                categories = ", ".join(game_data["categories"][:5]) if isinstance(game_data["categories"], list) else game_data["categories"]
                text_parts.append(f"Categories: {categories}")

            # Add multiplayer information
            if game_data.get("is_multiplayer"):
                text_parts.append("Multiplayer: Yes")
            if game_data.get("is_coop"):
                text_parts.append("Co-op: Yes")
            if game_data.get("is_pvp"):
                text_parts.append("PVP: Yes")

            text = " | ".join(text_parts)

            if use_openai:
                return await self._create_openai_embedding(text)
            else:
                return self._create_local_embedding(text)

        except Exception as e:
            logger.error(f"Error creating game embedding: {e}")
            return None

    async def _create_openai_embedding(self, text: str) -> Optional[List[float]]:
        """Create embedding using OpenAI API"""
        try:
            response = await self.openai_client.embeddings.create(
                model=settings.openai_embedding_model,
                input=text
            )
            return response.data[0].embedding
        except Exception as e:
            logger.error(f"Error creating OpenAI embedding: {e}")
            # Fallback to local model
            return self._create_local_embedding(text)

    def _create_local_embedding(self, text: str) -> Optional[List[float]]:
        """Create embedding using local model"""
        try:
            if self.local_model is None:
                return None
            embedding = self.local_model.encode(text, convert_to_numpy=True)
            return embedding.tolist()
        except Exception as e:
            logger.error(f"Error creating local embedding: {e}")
            return None

    async def add_game_to_vector_db(
        self,
        game_id: str,
        embedding: List[float],
        metadata: Dict[str, Any]
    ) -> bool:
        """Add game embedding to vector database"""
        try:
            if self.collection is None:
                logger.error("ChromaDB collection not initialized")
                return False

            self.collection.add(
                ids=[game_id],
                embeddings=[embedding],
                metadatas=[metadata]
            )
            return True
        except Exception as e:
            logger.error(f"Error adding game to vector DB: {e}")
            return False

    async def search_similar_games(
        self,
        query_embedding: List[float],
        n_results: int = 100,
        where_filter: Optional[Dict[str, Any]] = None
    ) -> List[Dict[str, Any]]:
        """
        Search for similar games using vector similarity

        Args:
            query_embedding: The query embedding vector
            n_results: Number of results to return
            where_filter: Optional metadata filter
        """
        try:
            if self.collection is None:
                logger.error("ChromaDB collection not initialized")
                return []

            results = self.collection.query(
                query_embeddings=[query_embedding],
                n_results=n_results,
                where=where_filter
            )

            # Format results
            formatted_results = []
            if results and results.get("ids"):
                for i, game_id in enumerate(results["ids"][0]):
                    formatted_results.append({
                        "game_id": game_id,
                        "distance": results["distances"][0][i] if results.get("distances") else None,
                        "metadata": results["metadatas"][0][i] if results.get("metadatas") else {}
                    })

            return formatted_results
        except Exception as e:
            logger.error(f"Error searching similar games: {e}")
            return []

    async def create_user_preference_vector(
        self,
        user_games: List[Dict[str, Any]],
        liked_games: List[str] = None,
        disliked_games: List[str] = None,
        alpha: float = 0.3,
        beta: float = 0.3
    ) -> Optional[List[float]]:
        """
        Create user preference vector from their game library and feedback

        Args:
            user_games: List of user's games with playtime
            liked_games: Game IDs user liked
            disliked_games: Game IDs user disliked
            alpha: Weight for liked games
            beta: Weight for disliked games
        """
        try:
            if self.collection is None:
                return None

            # Get embeddings for user's most played games
            top_games = sorted(
                user_games,
                key=lambda x: x.get("playtime_forever", 0),
                reverse=True
            )[:20]  # Top 20 most played games

            game_ids = [str(game["game_id"]) for game in top_games]

            # Get embeddings from vector DB
            base_embeddings = []
            try:
                results = self.collection.get(ids=game_ids, include=["embeddings"])
                if results and results.get("embeddings"):
                    base_embeddings = results["embeddings"]
            except Exception as e:
                logger.error(f"Error fetching embeddings: {e}")

            if not base_embeddings:
                return None

            # Calculate base vector (weighted by playtime)
            total_playtime = sum(game.get("playtime_forever", 0) for game in top_games)
            if total_playtime == 0:
                weights = [1.0 / len(top_games)] * len(top_games)
            else:
                weights = [game.get("playtime_forever", 0) / total_playtime for game in top_games]

            v_base = np.average(base_embeddings[:len(weights)], axis=0, weights=weights)

            # Adjust based on liked/disliked games
            v_user = v_base.copy()

            if liked_games:
                try:
                    liked_results = self.collection.get(ids=liked_games, include=["embeddings"])
                    if liked_results and liked_results.get("embeddings"):
                        v_like = np.mean(liked_results["embeddings"], axis=0)
                        v_user += alpha * v_like
                except Exception as e:
                    logger.error(f"Error processing liked games: {e}")

            if disliked_games:
                try:
                    disliked_results = self.collection.get(ids=disliked_games, include=["embeddings"])
                    if disliked_results and disliked_results.get("embeddings"):
                        v_dislike = np.mean(disliked_results["embeddings"], axis=0)
                        v_user -= beta * v_dislike
                except Exception as e:
                    logger.error(f"Error processing disliked games: {e}")

            # Normalize
            v_user = v_user / np.linalg.norm(v_user)

            return v_user.tolist()

        except Exception as e:
            logger.error(f"Error creating user preference vector: {e}")
            return None

    async def create_group_vector(self, user_vectors: List[List[float]]) -> Optional[List[float]]:
        """Create group preference vector by averaging user vectors"""
        try:
            if not user_vectors:
                return None

            group_vector = np.mean(user_vectors, axis=0)
            # Normalize
            group_vector = group_vector / np.linalg.norm(group_vector)

            return group_vector.tolist()
        except Exception as e:
            logger.error(f"Error creating group vector: {e}")
            return None

    async def update_game_embedding(self, game_id: str, embedding: List[float], metadata: Dict[str, Any]) -> bool:
        """Update existing game embedding"""
        try:
            if self.collection is None:
                return False

            self.collection.update(
                ids=[game_id],
                embeddings=[embedding],
                metadatas=[metadata]
            )
            return True
        except Exception as e:
            logger.error(f"Error updating game embedding: {e}")
            return False

    async def delete_game_embedding(self, game_id: str) -> bool:
        """Delete game embedding from vector DB"""
        try:
            if self.collection is None:
                return False

            self.collection.delete(ids=[game_id])
            return True
        except Exception as e:
            logger.error(f"Error deleting game embedding: {e}")
            return False


# Singleton instance
embedding_service = EmbeddingService()
