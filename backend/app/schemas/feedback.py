"""Feedback schemas"""
from pydantic import BaseModel, Field
from typing import Optional
from datetime import datetime
from uuid import UUID


class FeedbackCreate(BaseModel):
    """Schema for creating feedback"""
    game_id: UUID
    recommendation_id: Optional[UUID] = None
    feedback: int = Field(..., description="Feedback value: +1 (like) or -1 (dislike)")
    reason_code: Optional[str] = Field(None, description="Reason code: genre_mismatch, too_complex, pvp_dislike, horror, already_played")
    reason_text: Optional[str] = Field(None, description="Additional reason text")


class FeedbackResponse(BaseModel):
    """Schema for feedback response"""
    id: UUID
    session_id: UUID
    user_id: UUID
    game_id: UUID
    recommendation_id: Optional[UUID] = None
    feedback: int
    reason_code: Optional[str] = None
    reason_text: Optional[str] = None
    created_at: datetime

    class Config:
        from_attributes = True
