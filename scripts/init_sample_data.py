#!/usr/bin/env python3
"""
Initialize sample data for testing
"""

import asyncio
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent / "backend"))

from app.database import AsyncSessionLocal
from app.models.game import Game
from app.services.embedding_service import embedding_service
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


SAMPLE_GAMES = [
    {
        "steam_app_id": 730,
        "title": "Counter-Strike 2",
        "short_description": "For over two decades, Counter-Strike has offered an elite competitive experience.",
        "genres": ["Action", "FPS"],
        "tags": ["Multiplayer", "Competitive", "Shooter", "Team-Based"],
        "is_multiplayer": True,
        "is_pvp": True,
        "header_image": "https://cdn.akamai.steamstatic.com/steam/apps/730/header.jpg"
    },
    {
        "steam_app_id": 413150,
        "title": "Stardew Valley",
        "short_description": "You've inherited your grandfather's old farm plot in Stardew Valley.",
        "genres": ["RPG", "Simulation"],
        "tags": ["Farming", "Casual", "Relaxing", "Co-op"],
        "is_multiplayer": True,
        "is_coop": True,
        "header_image": "https://cdn.akamai.steamstatic.com/steam/apps/413150/header.jpg"
    },
    {
        "steam_app_id": 1426210,
        "title": "It Takes Two",
        "short_description": "Embark on the craziest journey of your life in It Takes Two.",
        "genres": ["Action", "Adventure"],
        "tags": ["Co-op", "Multiplayer", "Adventure", "Platformer"],
        "is_multiplayer": True,
        "is_coop": True,
        "header_image": "https://cdn.akamai.steamstatic.com/steam/apps/1426210/header.jpg"
    },
]


async def init_sample_data():
    """Initialize sample game data"""
    logger.info("Initializing sample data...")

    async with AsyncSessionLocal() as db:
        for game_data in SAMPLE_GAMES:
            try:
                game = Game(**game_data)
                db.add(game)
                await db.flush()

                # Create embedding
                embedding = await embedding_service.create_game_embedding(game_data)
                if embedding:
                    embedding_id = f"game_{game.steam_app_id}"
                    await embedding_service.add_game_to_vector_db(
                        game_id=embedding_id,
                        embedding=embedding,
                        metadata={
                            "game_id": str(game.id),
                            "steam_app_id": game.steam_app_id,
                            "title": game.title
                        }
                    )
                    game.embedding_id = embedding_id

                logger.info(f"✓ Added: {game.title}")

            except Exception as e:
                logger.error(f"Error adding {game_data['title']}: {e}")
                await db.rollback()
                continue

        await db.commit()
        logger.info("Sample data initialized successfully!")


if __name__ == "__main__":
    asyncio.run(init_sample_data())
