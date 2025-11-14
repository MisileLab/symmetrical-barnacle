"""Game models"""
from sqlalchemy import Column, String, DateTime, Integer, Boolean, Date, JSON, Text
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import relationship
from sqlalchemy.sql import func
import uuid

from app.database import Base


class Game(Base):
    """Steam game"""
    __tablename__ = "games"

    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    steam_app_id = Column(Integer, unique=True, nullable=False, index=True)
    title = Column(String(500), nullable=False)
    description = Column(Text)
    short_description = Column(Text)
    header_image = Column(String)
    tags = Column(JSON, default=list)
    genres = Column(JSON, default=list)
    categories = Column(JSON, default=list)
    release_date = Column(Date)
    developers = Column(JSON, default=list)
    publishers = Column(JSON, default=list)
    price_info = Column(JSON)
    metacritic_score = Column(Integer)
    is_multiplayer = Column(Boolean, default=False, index=True)
    is_coop = Column(Boolean, default=False)
    is_pvp = Column(Boolean, default=False)
    player_count_min = Column(Integer)
    player_count_max = Column(Integer)
    embedding_id = Column(String(255))  # Reference to ChromaDB
    created_at = Column(DateTime(timezone=True), server_default=func.now())
    updated_at = Column(DateTime(timezone=True), onupdate=func.now())

    # Relationships
    user_games = relationship("UserGame", back_populates="game")
    recommendations = relationship("Recommendation", back_populates="game")
    feedbacks = relationship("Feedback", back_populates="game")
    group_favorites = relationship("GroupFavorite", back_populates="game")
