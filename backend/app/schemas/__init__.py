"""Pydantic schemas for API"""
from app.schemas.user import UserCreate, UserResponse, UserGameResponse
from app.schemas.session import (
    SessionCreate,
    SessionResponse,
    SessionUserResponse,
    SessionJoinRequest
)
from app.schemas.game import GameResponse, GameDetailResponse
from app.schemas.recommendation import (
    RecommendationRequest,
    RecommendationResponse,
    GenerateRecommendationsRequest
)
from app.schemas.feedback import FeedbackCreate, FeedbackResponse

__all__ = [
    "UserCreate",
    "UserResponse",
    "UserGameResponse",
    "SessionCreate",
    "SessionResponse",
    "SessionUserResponse",
    "SessionJoinRequest",
    "GameResponse",
    "GameDetailResponse",
    "RecommendationRequest",
    "RecommendationResponse",
    "GenerateRecommendationsRequest",
    "FeedbackCreate",
    "FeedbackResponse",
]
