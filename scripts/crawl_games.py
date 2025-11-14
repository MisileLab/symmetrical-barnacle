#!/usr/bin/env python3
"""
Game crawling script for Steam Party Picker
This script fetches popular Steam games and stores their metadata with embeddings.
"""

import asyncio
import sys
import os
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent / "backend"))

from app.services.steam_service import steam_service
from app.services.embedding_service import embedding_service
from app.database import AsyncSessionLocal
from app.models.game import Game
from sqlalchemy import select
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


async def crawl_top_games(limit: int = 500):
    """
    Crawl top popular Steam games and add them to database with embeddings

    Args:
        limit: Number of games to crawl
    """
    logger.info(f"Starting game crawl for {limit} games")

    # List of popular Steam app IDs (you can expand this list)
    # These are some of the most popular multiplayer/co-op games
    popular_app_ids = [
        # Multiplayer favorites
        730,    # Counter-Strike 2
        440,    # Team Fortress 2
        570,    # Dota 2
        578080, # PUBG
        252490, # Rust
        271590, # GTA V
        386070, # Lethal Company
        1172470, # Apex Legends
        359550,  # Rainbow Six Siege
        413150,  # Stardew Valley (co-op)

        # Co-op games
        892970,  # Valheim
        1145360, # Hades
        646570,  # Slay the Spire
        391540,  # Undertale
        582010,  # Monster Hunter World
        367520,  # Hollow Knight
        881100,  # Noita
        1203220, # NARAKA: BLADEPOINT
        1174180, # Red Dead Redemption 2
        1426210, # It Takes Two

        # Party games
        1092790, # Among Us
        674940,  # Stick Fight
        1466860, # Content Warning
        457140,  # Oxygen Not Included
        838380,  # Ultimate Chicken Horse
        620980,  # Beat Saber
        945360,  # Among Us
        383120,  # Golf With Your Friends
        1426210, # It Takes Two
        1621690, # Ready or Not
    ]

    async with AsyncSessionLocal() as db:
        processed = 0
        for app_id in popular_app_ids[:limit]:
            try:
                logger.info(f"Processing game {app_id}...")

                # Check if game already exists
                result = await db.execute(
                    select(Game).where(Game.steam_app_id == app_id)
                )
                existing_game = result.scalar_one_or_none()

                if existing_game and existing_game.embedding_id:
                    logger.info(f"Game {app_id} already exists with embedding, skipping")
                    continue

                # Fetch game details from Steam
                game_details = await steam_service.get_game_details(app_id)
                if not game_details:
                    logger.warning(f"Could not fetch details for game {app_id}")
                    continue

                # Parse metadata
                metadata = steam_service.parse_game_metadata(game_details)

                # Create embedding
                embedding = await embedding_service.create_game_embedding(metadata)
                if not embedding:
                    logger.warning(f"Could not create embedding for game {app_id}")
                    continue

                # Save or update game
                if existing_game:
                    # Update existing game
                    for key, value in metadata.items():
                        setattr(existing_game, key, value)
                    game = existing_game
                else:
                    # Create new game
                    game = Game(
                        steam_app_id=app_id,
                        **metadata
                    )
                    db.add(game)

                await db.flush()

                # Add embedding to vector DB
                embedding_id = f"game_{app_id}"
                success = await embedding_service.add_game_to_vector_db(
                    game_id=embedding_id,
                    embedding=embedding,
                    metadata={
                        "game_id": str(game.id),
                        "steam_app_id": app_id,
                        "title": game.title,
                        "is_multiplayer": game.is_multiplayer,
                        "is_coop": game.is_coop,
                    }
                )

                if success:
                    game.embedding_id = embedding_id
                    await db.commit()
                    logger.info(f"✓ Successfully processed game: {game.title}")
                    processed += 1
                else:
                    logger.error(f"Failed to add embedding for {game.title}")
                    await db.rollback()

                # Rate limiting
                await asyncio.sleep(1.5)

            except Exception as e:
                logger.error(f"Error processing game {app_id}: {e}")
                await db.rollback()
                continue

        logger.info(f"Crawling complete. Processed {processed} games.")


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description="Crawl Steam games")
    parser.add_argument("--limit", type=int, default=50, help="Number of games to crawl")
    args = parser.parse_args()

    asyncio.run(crawl_top_games(args.limit))
