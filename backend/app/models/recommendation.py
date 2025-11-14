"""Recommendation models"""
from sqlalchemy import Column, String, DateTime, ForeignKey, Integer, Float, Boolean, JSON, Text
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import relationship
from sqlalchemy.sql import func
import uuid

from app.database import Base


class Recommendation(Base):
    """Game recommendations for sessions"""
    __tablename__ = "recommendations"

    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    session_id = Column(UUID(as_uuid=True), ForeignKey("sessions.id", ondelete="CASCADE"), nullable=False)
    game_id = Column(UUID(as_uuid=True), ForeignKey("games.id", ondelete="CASCADE"), nullable=False)
    rank = Column(Integer, nullable=False)
    ownership_category = Column(String(10), nullable=False)  # G1, G2, G3
    score = Column(Float)
    explanation = Column(Text)
    metadata_snapshot = Column(JSON)  # Snapshot of game info at recommendation time
    created_at = Column(DateTime(timezone=True), server_default=func.now())

    # Relationships
    session = relationship("Session", back_populates="recommendations")
    game = relationship("Game", back_populates="recommendations")
    metrics = relationship("RecommendationMetric", back_populates="recommendation", uselist=False, cascade="all, delete-orphan")
    feedbacks = relationship("Feedback", back_populates="recommendation")


class RecommendationMetric(Base):
    """Metrics for recommendation quality tracking"""
    __tablename__ = "recommendation_metrics"

    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    recommendation_id = Column(UUID(as_uuid=True), ForeignKey("recommendations.id", ondelete="CASCADE"), nullable=False, unique=True)
    clicks = Column(Integer, default=0)
    positive_feedback_count = Column(Integer, default=0)
    negative_feedback_count = Column(Integer, default=0)
    was_selected = Column(Boolean, default=False)
    was_played = Column(Boolean, default=False)
    created_at = Column(DateTime(timezone=True), server_default=func.now())
    updated_at = Column(DateTime(timezone=True), onupdate=func.now())

    # Relationships
    recommendation = relationship("Recommendation", back_populates="metrics")
