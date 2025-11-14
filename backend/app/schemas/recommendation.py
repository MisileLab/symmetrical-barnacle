"""Recommendation schemas"""
from pydantic import BaseModel, Field
from typing import Optional, List, Any
from datetime import datetime
from uuid import UUID

from app.schemas.game import GameDetailResponse


class GenerateRecommendationsRequest(BaseModel):
    """Schema for generating recommendations"""
    exclude_game_ids: Optional[List[UUID]] = Field(default=[], description="Games to exclude")
    count: Optional[int] = Field(default=10, description="Number of recommendations")


class RecommendationRequest(BaseModel):
    """Schema for recommendation request"""
    session_id: UUID
    exclude_game_ids: Optional[List[UUID]] = []
    count: Optional[int] = 10


class RecommendationResponse(BaseModel):
    """Schema for recommendation response"""
    id: UUID
    session_id: UUID
    game: GameDetailResponse
    rank: int
    ownership_category: str  # G1, G2, G3
    score: Optional[float] = None
    explanation: Optional[str] = None
    pros: Optional[List[str]] = None
    cons: Optional[List[str]] = None
    best_for: Optional[List[str]] = None
    metadata_snapshot: Optional[dict] = None
    created_at: datetime

    class Config:
        from_attributes = True


class RecommendationListResponse(BaseModel):
    """Schema for list of recommendations"""
    session_id: UUID
    recommendations: List[RecommendationResponse]
    total_count: int
    generated_at: datetime
