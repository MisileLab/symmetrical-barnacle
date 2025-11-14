"""Game recommendation service using embeddings and GPT"""
import logging
from typing import List, Dict, Any, Optional, Tuple
import openai
import json

from app.config import settings
from app.services.embedding_service import embedding_service
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select
from app.models.user import User, UserGame, UserPreference
from app.models.game import Game
from app.models.feedback import Feedback

logger = logging.getLogger(__name__)


class RecommendationService:
    """Service for generating game recommendations"""

    def __init__(self):
        self.openai_client = openai.AsyncOpenAI(api_key=settings.openai_api_key)
        self.embedding_service = embedding_service

    async def generate_recommendations(
        self,
        session_id: str,
        user_ids: List[str],
        db: AsyncSession,
        exclude_game_ids: List[str] = None,
        count: int = None
    ) -> List[Dict[str, Any]]:
        """
        Generate game recommendations for a group

        Args:
            session_id: Session ID
            user_ids: List of user IDs in the session
            db: Database session
            exclude_game_ids: Games to exclude from recommendations
            count: Number of recommendations to generate
        """
        try:
            if count is None:
                count = settings.recommendation_final_count

            # Step 1: Get user data
            users_data = await self._get_users_data(user_ids, db)
            if not users_data:
                logger.error("No user data found")
                return []

            # Step 2: Create preference vectors for each user
            user_vectors = []
            for user_data in users_data:
                vector = await self._create_user_vector(user_data, db)
                if vector:
                    user_vectors.append(vector)

            if not user_vectors:
                logger.error("Could not create user vectors")
                return []

            # Step 3: Create group vector
            group_vector = await self.embedding_service.create_group_vector(user_vectors)
            if not group_vector:
                logger.error("Could not create group vector")
                return []

            # Step 4: Search for candidate games
            candidate_count = settings.recommendation_candidate_count
            candidates = await self.embedding_service.search_similar_games(
                query_embedding=group_vector,
                n_results=candidate_count
            )

            if not candidates:
                logger.error("No candidate games found")
                return []

            # Step 5: Get detailed game information
            game_ids = [c["game_id"] for c in candidates]
            result = await db.execute(
                select(Game).where(Game.id.in_(game_ids))
            )
            games = result.scalars().all()
            games_dict = {str(game.id): game for game in games}

            # Step 6: Categorize games by ownership
            categorized_games = await self._categorize_by_ownership(
                candidates, users_data, games_dict, exclude_game_ids or []
            )

            # Step 7: Use GPT to select final recommendations
            final_recommendations = await self._gpt_select_games(
                categorized_games, users_data, count
            )

            return final_recommendations

        except Exception as e:
            logger.error(f"Error generating recommendations: {e}")
            return []

    async def _get_users_data(self, user_ids: List[str], db: AsyncSession) -> List[Dict[str, Any]]:
        """Get detailed user data including games and preferences"""
        users_data = []

        for user_id in user_ids:
            # Get user
            result = await db.execute(
                select(User).where(User.id == user_id)
            )
            user = result.scalar_one_or_none()
            if not user:
                continue

            # Get user's games
            result = await db.execute(
                select(UserGame).where(UserGame.user_id == user_id)
            )
            user_games = result.scalars().all()

            # Get user's preferences
            result = await db.execute(
                select(UserPreference).where(UserPreference.user_id == user_id)
            )
            preferences = result.scalar_one_or_none()

            # Get user's feedback history
            result = await db.execute(
                select(Feedback)
                .where(Feedback.user_id == user_id)
                .order_by(Feedback.created_at.desc())
                .limit(50)
            )
            feedbacks = result.scalars().all()

            users_data.append({
                "user": user,
                "games": user_games,
                "preferences": preferences,
                "feedbacks": feedbacks
            })

        return users_data

    async def _create_user_vector(self, user_data: Dict[str, Any], db: AsyncSession) -> Optional[List[float]]:
        """Create preference vector for a user"""
        try:
            # Get user's games with embeddings
            games_data = []
            for user_game in user_data["games"]:
                result = await db.execute(
                    select(Game).where(Game.id == user_game.game_id)
                )
                game = result.scalar_one_or_none()
                if game:
                    games_data.append({
                        "game_id": str(game.id),
                        "playtime_forever": user_game.playtime_forever
                    })

            # Get liked/disliked game IDs from feedback
            liked_games = []
            disliked_games = []
            for feedback in user_data["feedbacks"]:
                if feedback.feedback > 0:
                    liked_games.append(str(feedback.game_id))
                else:
                    disliked_games.append(str(feedback.game_id))

            # Create user vector
            user_vector = await self.embedding_service.create_user_preference_vector(
                user_games=games_data,
                liked_games=liked_games if liked_games else None,
                disliked_games=disliked_games if disliked_games else None
            )

            return user_vector

        except Exception as e:
            logger.error(f"Error creating user vector: {e}")
            return None

    async def _categorize_by_ownership(
        self,
        candidates: List[Dict[str, Any]],
        users_data: List[Dict[str, Any]],
        games_dict: Dict[str, Game],
        exclude_game_ids: List[str]
    ) -> Dict[str, List[Dict[str, Any]]]:
        """Categorize games by ownership (G1/G2/G3)"""
        categorized = {
            "G1": [],  # All users own
            "G2": [],  # Some users own
            "G3": []   # No users own
        }

        # Build ownership map
        ownership_map = {}
        for candidate in candidates:
            game_id = candidate["game_id"]

            if game_id in exclude_game_ids:
                continue

            if game_id not in games_dict:
                continue

            game = games_dict[game_id]

            owners_count = 0
            for user_data in users_data:
                user_games_ids = [str(ug.game_id) for ug in user_data["games"]]
                if game_id in user_games_ids:
                    owners_count += 1

            game_info = {
                "game_id": game_id,
                "game": game,
                "distance": candidate.get("distance"),
                "owners_count": owners_count,
                "total_users": len(users_data)
            }

            if owners_count == len(users_data):
                categorized["G1"].append(game_info)
            elif owners_count > 0:
                categorized["G2"].append(game_info)
            else:
                categorized["G3"].append(game_info)

        return categorized

    async def _gpt_select_games(
        self,
        categorized_games: Dict[str, List[Dict[str, Any]]],
        users_data: List[Dict[str, Any]],
        count: int
    ) -> List[Dict[str, Any]]:
        """Use GPT to select final recommendations"""
        try:
            # Prepare context for GPT
            context = self._prepare_gpt_context(categorized_games, users_data)

            # Create prompt
            prompt = self._create_selection_prompt(context, count)

            # Call GPT
            response = await self.openai_client.chat.completions.create(
                model=settings.openai_model,
                messages=[
                    {
                        "role": "system",
                        "content": """You are an expert game recommender for groups of friends.
                        Your goal is to recommend games that the entire group will enjoy together.
                        Consider each player's preferences, play history, and the group dynamics.
                        Provide thoughtful explanations for each recommendation."""
                    },
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                temperature=0.7,
                response_format={"type": "json_object"}
            )

            # Parse response
            result_text = response.choices[0].message.content
            result = json.loads(result_text)

            # Format recommendations
            recommendations = []
            for rec in result.get("recommendations", []):
                game_id = rec.get("game_id")
                game_info = None

                # Find game in categorized games
                for category in ["G1", "G2", "G3"]:
                    for game in categorized_games[category]:
                        if game["game_id"] == game_id:
                            game_info = game
                            break
                    if game_info:
                        break

                if game_info:
                    recommendations.append({
                        "game_id": game_id,
                        "game": game_info["game"],
                        "ownership_category": rec.get("category"),
                        "rank": rec.get("rank"),
                        "score": rec.get("score"),
                        "explanation": rec.get("explanation"),
                        "pros": rec.get("pros", []),
                        "cons": rec.get("cons", []),
                        "best_for": rec.get("best_for", [])
                    })

            return recommendations

        except Exception as e:
            logger.error(f"Error in GPT selection: {e}")
            # Fallback: return top games from each category
            return self._fallback_selection(categorized_games, count)

    def _prepare_gpt_context(
        self,
        categorized_games: Dict[str, List[Dict[str, Any]]],
        users_data: List[Dict[str, Any]]
    ) -> Dict[str, Any]:
        """Prepare context information for GPT"""
        # Summarize users
        users_summary = []
        for user_data in users_data:
            user = user_data["user"]
            top_games = sorted(
                user_data["games"],
                key=lambda x: x.playtime_forever,
                reverse=True
            )[:5]

            users_summary.append({
                "nickname": user.nickname,
                "top_games_count": len(user_data["games"]),
                "recent_likes": [f.game_id for f in user_data["feedbacks"] if f.feedback > 0][:5],
                "recent_dislikes": [f.game_id for f in user_data["feedbacks"] if f.feedback < 0][:5]
            })

        # Summarize candidate games
        games_summary = {}
        for category in ["G1", "G2", "G3"]:
            games_summary[category] = []
            for game_info in categorized_games[category][:30]:  # Limit to avoid token overflow
                game = game_info["game"]
                games_summary[category].append({
                    "game_id": str(game.id),
                    "title": game.title,
                    "genres": game.genres,
                    "tags": game.tags[:10] if game.tags else [],
                    "is_multiplayer": game.is_multiplayer,
                    "is_coop": game.is_coop,
                    "is_pvp": game.is_pvp,
                    "short_description": game.short_description[:200] if game.short_description else "",
                    "owners_count": game_info["owners_count"]
                })

        return {
            "users": users_summary,
            "games": games_summary
        }

    def _create_selection_prompt(self, context: Dict[str, Any], count: int) -> str:
        """Create prompt for GPT"""
        return f"""
You are helping a group of {len(context['users'])} friends find games to play together.

USER PROFILES:
{json.dumps(context['users'], indent=2)}

CANDIDATE GAMES:
The games are categorized into three groups:
- G1: All players own these games
- G2: Some players own these games
- G3: No players own these games (new purchases)

{json.dumps(context['games'], indent=2)}

YOUR TASK:
Select {count} games that this group should play together. Consider:
1. Group dynamics - will everyone enjoy this?
2. Accessibility - do they already own it?
3. Multiplayer features - can they play together?
4. Variety - mix of genres and playstyles
5. Balance between familiar (G1/G2) and new (G3) games

Aim for a distribution like:
- 50% from G1 (everyone owns)
- 30% from G2 (some own)
- 20% from G3 (new discoveries)

Return your selection in JSON format:
{{
  "recommendations": [
    {{
      "game_id": "uuid",
      "title": "Game Title",
      "rank": 1,
      "category": "G1|G2|G3",
      "score": 0.95,
      "explanation": "Why this group will love this game (2-3 sentences)",
      "pros": ["Pro 1", "Pro 2"],
      "cons": ["Con 1", "Con 2"],
      "best_for": ["Player1", "Player2"]
    }}
  ]
}}
"""

    def _fallback_selection(
        self,
        categorized_games: Dict[str, List[Dict[str, Any]]],
        count: int
    ) -> List[Dict[str, Any]]:
        """Fallback selection if GPT fails"""
        recommendations = []
        rank = 1

        # Select games proportionally
        g1_count = int(count * 0.5)
        g2_count = int(count * 0.3)
        g3_count = count - g1_count - g2_count

        for game_info in categorized_games["G1"][:g1_count]:
            recommendations.append({
                "game_id": game_info["game_id"],
                "game": game_info["game"],
                "ownership_category": "G1",
                "rank": rank,
                "score": 1.0 - (game_info.get("distance", 0) or 0),
                "explanation": "Everyone in the group owns this game, making it easy to start playing together immediately."
            })
            rank += 1

        for game_info in categorized_games["G2"][:g2_count]:
            recommendations.append({
                "game_id": game_info["game_id"],
                "game": game_info["game"],
                "ownership_category": "G2",
                "rank": rank,
                "score": 1.0 - (game_info.get("distance", 0) or 0),
                "explanation": f"Some players own this game ({game_info['owners_count']}/{game_info['total_users']}). Great opportunity to try something new together."
            })
            rank += 1

        for game_info in categorized_games["G3"][:g3_count]:
            recommendations.append({
                "game_id": game_info["game_id"],
                "game": game_info["game"],
                "ownership_category": "G3",
                "rank": rank,
                "score": 1.0 - (game_info.get("distance", 0) or 0),
                "explanation": "A new game that matches your group's preferences. Perfect for a fresh shared experience."
            })
            rank += 1

        return recommendations


# Singleton instance
recommendation_service = RecommendationService()
