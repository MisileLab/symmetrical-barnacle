"""Game schemas"""
from pydantic import BaseModel
from typing import Optional, List, Any
from datetime import date, datetime
from uuid import UUID


class GameResponse(BaseModel):
    """Schema for game response"""
    id: UUID
    steam_app_id: int
    title: str
    short_description: Optional[str] = None
    header_image: Optional[str] = None
    tags: List[str] = []
    genres: List[str] = []
    is_multiplayer: bool = False
    is_coop: bool = False
    is_pvp: bool = False

    class Config:
        from_attributes = True


class GameDetailResponse(BaseModel):
    """Schema for detailed game response"""
    id: UUID
    steam_app_id: int
    title: str
    description: Optional[str] = None
    short_description: Optional[str] = None
    header_image: Optional[str] = None
    tags: List[str] = []
    genres: List[str] = []
    categories: List[str] = []
    release_date: Optional[date] = None
    developers: List[str] = []
    publishers: List[str] = []
    price_info: Optional[dict] = None
    metacritic_score: Optional[int] = None
    is_multiplayer: bool = False
    is_coop: bool = False
    is_pvp: bool = False
    player_count_min: Optional[int] = None
    player_count_max: Optional[int] = None
    created_at: datetime
    updated_at: Optional[datetime] = None

    class Config:
        from_attributes = True
