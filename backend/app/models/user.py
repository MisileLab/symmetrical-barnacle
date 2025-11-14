"""User models"""
from sqlalchemy import Column, String, DateTime, ForeignKey, Integer, JSON
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import relationship
from sqlalchemy.sql import func
import uuid

from app.database import Base


class User(Base):
    """Steam user"""
    __tablename__ = "users"

    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    steam_id = Column(String(100), unique=True, nullable=False, index=True)
    nickname = Column(String(100), nullable=False)
    avatar_url = Column(String)
    profile_url = Column(String)
    created_at = Column(DateTime(timezone=True), server_default=func.now())
    updated_at = Column(DateTime(timezone=True), onupdate=func.now())

    # Relationships
    games = relationship("UserGame", back_populates="user", cascade="all, delete-orphan")
    preferences = relationship("UserPreference", back_populates="user", uselist=False, cascade="all, delete-orphan")
    session_participations = relationship("SessionUser", back_populates="user")
    feedbacks = relationship("Feedback", back_populates="user")


class UserGame(Base):
    """User's game library"""
    __tablename__ = "user_games"

    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    user_id = Column(UUID(as_uuid=True), ForeignKey("users.id", ondelete="CASCADE"), nullable=False)
    game_id = Column(UUID(as_uuid=True), ForeignKey("games.id", ondelete="CASCADE"), nullable=False)
    playtime_forever = Column(Integer, default=0)  # minutes
    playtime_2weeks = Column(Integer, default=0)  # minutes
    last_played = Column(DateTime(timezone=True))
    added_at = Column(DateTime(timezone=True), server_default=func.now())

    # Relationships
    user = relationship("User", back_populates="games")
    game = relationship("Game", back_populates="user_games")


class UserPreference(Base):
    """User's learned preferences from feedback"""
    __tablename__ = "user_preferences"

    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    user_id = Column(UUID(as_uuid=True), ForeignKey("users.id", ondelete="CASCADE"), unique=True, nullable=False)
    preference_data = Column(JSON, nullable=False, default=dict)
    liked_tags = Column(JSON, default=list)
    disliked_tags = Column(JSON, default=list)
    liked_genres = Column(JSON, default=list)
    disliked_genres = Column(JSON, default=list)
    updated_at = Column(DateTime(timezone=True), onupdate=func.now())

    # Relationships
    user = relationship("User", back_populates="preferences")
