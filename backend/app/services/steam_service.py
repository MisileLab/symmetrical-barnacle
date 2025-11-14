"""Steam API integration service"""
import aiohttp
from typing import List, Dict, Optional, Any
import logging
from datetime import datetime

from app.config import settings

logger = logging.getLogger(__name__)


class SteamService:
    """Service for interacting with Steam Web API"""

    BASE_URL = "https://api.steampowered.com"
    STORE_API_URL = "https://store.steampowered.com/api"

    def __init__(self):
        self.api_key = settings.steam_api_key

    async def get_steam_id_from_url(self, profile_url: str) -> Optional[str]:
        """
        Extract Steam ID from profile URL or custom URL
        Supports:
        - https://steamcommunity.com/profiles/[steamid64]
        - https://steamcommunity.com/id/[customurl]
        """
        try:
            if "/profiles/" in profile_url:
                # Direct Steam ID
                steam_id = profile_url.split("/profiles/")[1].rstrip("/").split("/")[0]
                return steam_id
            elif "/id/" in profile_url:
                # Custom URL - need to resolve
                custom_url = profile_url.split("/id/")[1].rstrip("/").split("/")[0]
                return await self.resolve_vanity_url(custom_url)
            else:
                # Assume it's already a Steam ID
                return profile_url
        except Exception as e:
            logger.error(f"Failed to extract Steam ID from URL: {e}")
            return None

    async def resolve_vanity_url(self, vanity_url: str) -> Optional[str]:
        """Resolve custom vanity URL to Steam ID"""
        url = f"{self.BASE_URL}/ISteamUser/ResolveVanityURL/v0001/"
        params = {
            "key": self.api_key,
            "vanityurl": vanity_url
        }

        async with aiohttp.ClientSession() as session:
            try:
                async with session.get(url, params=params) as response:
                    if response.status == 200:
                        data = await response.json()
                        if data.get("response", {}).get("success") == 1:
                            return data["response"]["steamid"]
                    logger.error(f"Failed to resolve vanity URL: {vanity_url}")
                    return None
            except Exception as e:
                logger.error(f"Error resolving vanity URL: {e}")
                return None

    async def get_player_summary(self, steam_id: str) -> Optional[Dict[str, Any]]:
        """Get player summary information"""
        url = f"{self.BASE_URL}/ISteamUser/GetPlayerSummaries/v0002/"
        params = {
            "key": self.api_key,
            "steamids": steam_id
        }

        async with aiohttp.ClientSession() as session:
            try:
                async with session.get(url, params=params) as response:
                    if response.status == 200:
                        data = await response.json()
                        players = data.get("response", {}).get("players", [])
                        if players:
                            return players[0]
                    return None
            except Exception as e:
                logger.error(f"Error getting player summary: {e}")
                return None

    async def get_owned_games(self, steam_id: str, include_free: bool = True) -> Optional[List[Dict[str, Any]]]:
        """Get list of games owned by user"""
        url = f"{self.BASE_URL}/IPlayerService/GetOwnedGames/v0001/"
        params = {
            "key": self.api_key,
            "steamid": steam_id,
            "include_appinfo": 1,
            "include_played_free_games": 1 if include_free else 0,
            "format": "json"
        }

        async with aiohttp.ClientSession() as session:
            try:
                async with session.get(url, params=params) as response:
                    if response.status == 200:
                        data = await response.json()
                        games = data.get("response", {}).get("games", [])
                        return games
                    return None
            except Exception as e:
                logger.error(f"Error getting owned games: {e}")
                return None

    async def get_recently_played_games(self, steam_id: str, count: int = 10) -> Optional[List[Dict[str, Any]]]:
        """Get recently played games"""
        url = f"{self.BASE_URL}/IPlayerService/GetRecentlyPlayedGames/v0001/"
        params = {
            "key": self.api_key,
            "steamid": steam_id,
            "count": count,
            "format": "json"
        }

        async with aiohttp.ClientSession() as session:
            try:
                async with session.get(url, params=params) as response:
                    if response.status == 200:
                        data = await response.json()
                        games = data.get("response", {}).get("games", [])
                        return games
                    return None
            except Exception as e:
                logger.error(f"Error getting recently played games: {e}")
                return None

    async def get_game_details(self, app_id: int) -> Optional[Dict[str, Any]]:
        """Get detailed information about a game from Steam Store API"""
        url = f"{self.STORE_API_URL}/appdetails"
        params = {
            "appids": app_id,
            "l": "english"
        }

        async with aiohttp.ClientSession() as session:
            try:
                async with session.get(url, params=params) as response:
                    if response.status == 200:
                        data = await response.json()
                        app_data = data.get(str(app_id), {})
                        if app_data.get("success"):
                            return app_data.get("data")
                    return None
            except Exception as e:
                logger.error(f"Error getting game details for app {app_id}: {e}")
                return None

    async def get_game_schema(self, app_id: int) -> Optional[Dict[str, Any]]:
        """Get game schema (achievements, stats)"""
        url = f"{self.BASE_URL}/ISteamUserStats/GetSchemaForGame/v2/"
        params = {
            "key": self.api_key,
            "appid": app_id
        }

        async with aiohttp.ClientSession() as session:
            try:
                async with session.get(url, params=params) as response:
                    if response.status == 200:
                        data = await response.json()
                        return data.get("game")
                    return None
            except Exception as e:
                logger.error(f"Error getting game schema for app {app_id}: {e}")
                return None

    def parse_game_metadata(self, game_data: Dict[str, Any]) -> Dict[str, Any]:
        """Parse game data from Steam API into our format"""
        try:
            # Extract multiplayer/coop information
            categories = game_data.get("categories", [])
            category_ids = [cat.get("id") for cat in categories if isinstance(cat, dict)]

            is_multiplayer = any(cat in category_ids for cat in [1, 9, 36, 37, 38])  # Multi-player categories
            is_coop = 9 in category_ids  # Co-op category
            is_pvp = 1 in category_ids  # Multi-player PVP

            # Parse player count
            player_count = game_data.get("recommendations", {}).get("total", 0)

            # Parse release date
            release_date = None
            try:
                release_date_str = game_data.get("release_date", {}).get("date")
                if release_date_str:
                    # Try to parse various date formats
                    for fmt in ["%b %d, %Y", "%d %b, %Y", "%Y"]:
                        try:
                            release_date = datetime.strptime(release_date_str, fmt).date()
                            break
                        except ValueError:
                            continue
            except Exception:
                pass

            return {
                "title": game_data.get("name", "Unknown"),
                "description": game_data.get("detailed_description", ""),
                "short_description": game_data.get("short_description", ""),
                "header_image": game_data.get("header_image", ""),
                "tags": [genre.get("description") for genre in game_data.get("genres", [])],
                "genres": [genre.get("description") for genre in game_data.get("genres", [])],
                "categories": [cat.get("description") for cat in categories if isinstance(cat, dict)],
                "release_date": release_date,
                "developers": game_data.get("developers", []),
                "publishers": game_data.get("publishers", []),
                "price_info": {
                    "currency": game_data.get("price_overview", {}).get("currency"),
                    "initial": game_data.get("price_overview", {}).get("initial"),
                    "final": game_data.get("price_overview", {}).get("final"),
                    "discount_percent": game_data.get("price_overview", {}).get("discount_percent"),
                },
                "metacritic_score": game_data.get("metacritic", {}).get("score"),
                "is_multiplayer": is_multiplayer,
                "is_coop": is_coop,
                "is_pvp": is_pvp,
            }
        except Exception as e:
            logger.error(f"Error parsing game metadata: {e}")
            return {}

    async def check_profile_visibility(self, steam_id: str) -> bool:
        """Check if user's profile and game library are public"""
        try:
            games = await self.get_owned_games(steam_id)
            return games is not None
        except Exception:
            return False


# Singleton instance
steam_service = SteamService()
