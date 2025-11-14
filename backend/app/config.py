"""Application configuration"""
from pydantic_settings import BaseSettings
from typing import Optional


class Settings(BaseSettings):
    """Application settings loaded from environment variables"""

    # Application
    app_name: str = "Steam Party Picker"
    environment: str = "development"
    debug: bool = True

    # URLs
    frontend_url: str = "http://localhost:3000"
    backend_url: str = "http://localhost:8000"

    # Database
    database_url: str

    # Redis
    redis_url: str = "redis://redis:6379"

    # Steam API
    steam_api_key: str

    # OpenAI
    openai_api_key: str
    openai_model: str = "gpt-4-turbo-preview"
    openai_embedding_model: str = "text-embedding-3-small"

    # ChromaDB
    chroma_host: str = "chromadb"
    chroma_port: int = 8000

    # JWT
    secret_key: str
    algorithm: str = "HS256"
    access_token_expire_minutes: int = 60 * 24 * 7  # 7 days

    # Recommendation settings
    recommendation_candidate_count: int = 100
    recommendation_final_count: int = 10
    embedding_dimension: int = 1536

    # Discord (Phase 3)
    discord_bot_token: Optional[str] = None
    discord_client_id: Optional[str] = None

    class Config:
        env_file = ".env"
        case_sensitive = False


settings = Settings()
