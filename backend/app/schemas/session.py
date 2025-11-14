"""Session schemas"""
from pydantic import BaseModel, Field
from typing import Optional
from datetime import datetime
from uuid import UUID


class SessionCreate(BaseModel):
    """Schema for creating a session"""
    name: Optional[str] = Field(None, description="Session name")
    expected_players: Optional[int] = Field(None, description="Expected number of players")


class SessionUserResponse(BaseModel):
    """Schema for session participant"""
    user_id: UUID
    nickname: str
    avatar_url: Optional[str] = None
    joined_at: datetime
    is_ready: bool

    class Config:
        from_attributes = True


class SessionResponse(BaseModel):
    """Schema for session response"""
    id: UUID
    session_code: str
    name: Optional[str] = None
    owner_user_id: Optional[UUID] = None
    status: str
    expected_players: Optional[int] = None
    created_at: datetime
    completed_at: Optional[datetime] = None
    participants: list[SessionUserResponse] = []

    class Config:
        from_attributes = True


class SessionJoinRequest(BaseModel):
    """Schema for joining a session"""
    steam_profile_url: str = Field(..., description="Steam profile URL or Steam ID")
    nickname: Optional[str] = Field(None, description="User nickname")


class SessionStatusUpdate(BaseModel):
    """Schema for updating session status"""
    status: str = Field(..., description="New status: waiting, ready, completed")
