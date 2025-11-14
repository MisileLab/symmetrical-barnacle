"""Feedback models"""
from sqlalchemy import Column, String, DateTime, ForeignKey, Integer, Text
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import relationship
from sqlalchemy.sql import func
import uuid

from app.database import Base


class Feedback(Base):
    """User feedback on game recommendations"""
    __tablename__ = "feedback"

    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    session_id = Column(UUID(as_uuid=True), ForeignKey("sessions.id", ondelete="CASCADE"), nullable=False)
    user_id = Column(UUID(as_uuid=True), ForeignKey("users.id", ondelete="CASCADE"), nullable=False)
    game_id = Column(UUID(as_uuid=True), ForeignKey("games.id", ondelete="CASCADE"), nullable=False)
    recommendation_id = Column(UUID(as_uuid=True), ForeignKey("recommendations.id", ondelete="CASCADE"))
    feedback = Column(Integer, nullable=False)  # +1 (like) or -1 (dislike)
    reason_code = Column(String(50))  # genre_mismatch, too_complex, pvp_dislike, horror, already_played
    reason_text = Column(Text)
    created_at = Column(DateTime(timezone=True), server_default=func.now())

    # Relationships
    session = relationship("Session", back_populates="feedbacks")
    user = relationship("User", back_populates="feedbacks")
    game = relationship("Game", back_populates="feedbacks")
    recommendation = relationship("Recommendation", back_populates="feedbacks")
