"""User schemas"""
from pydantic import BaseModel, Field
from typing import Optional
from datetime import datetime
from uuid import UUID


class UserCreate(BaseModel):
    """Schema for creating a user"""
    steam_profile_url: str = Field(..., description="Steam profile URL or Steam ID")
    nickname: Optional[str] = Field(None, description="User nickname (optional, will be fetched from Steam)")


class UserResponse(BaseModel):
    """Schema for user response"""
    id: UUID
    steam_id: str
    nickname: str
    avatar_url: Optional[str] = None
    profile_url: Optional[str] = None
    created_at: datetime

    class Config:
        from_attributes = True


class UserGameResponse(BaseModel):
    """Schema for user's game"""
    game_id: UUID
    game_title: str
    playtime_forever: int
    playtime_2weeks: int
    last_played: Optional[datetime] = None

    class Config:
        from_attributes = True


class UserLibraryResponse(BaseModel):
    """Schema for user's game library"""
    user: UserResponse
    games: list[UserGameResponse]
    total_games: int
    total_playtime: int
