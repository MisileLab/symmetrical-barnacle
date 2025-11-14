"""Session management API endpoints"""
from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select
from typing import List
import logging
from datetime import datetime

from app.database import get_db
from app.models.session import Session, SessionUser
from app.models.user import User, UserGame
from app.schemas.session import (
    SessionCreate,
    SessionResponse,
    SessionJoinRequest,
    SessionUserResponse,
    SessionStatusUpdate
)
from app.schemas.user import UserResponse
from app.services.steam_service import steam_service
from app.utils.code_generator import generate_session_code

logger = logging.getLogger(__name__)
router = APIRouter(prefix="/sessions", tags=["sessions"])


@router.post("/", response_model=SessionResponse, status_code=status.HTTP_201_CREATED)
async def create_session(
    session_data: SessionCreate,
    db: AsyncSession = Depends(get_db)
):
    """Create a new game session"""
    try:
        # Generate unique session code
        while True:
            session_code = generate_session_code()
            result = await db.execute(
                select(Session).where(Session.session_code == session_code)
            )
            if result.scalar_one_or_none() is None:
                break

        # Create session
        new_session = Session(
            session_code=session_code,
            name=session_data.name,
            expected_players=session_data.expected_players,
            status="waiting"
        )

        db.add(new_session)
        await db.commit()
        await db.refresh(new_session)

        logger.info(f"Created session: {session_code}")

        return SessionResponse(
            id=new_session.id,
            session_code=new_session.session_code,
            name=new_session.name,
            owner_user_id=new_session.owner_user_id,
            status=new_session.status,
            expected_players=new_session.expected_players,
            created_at=new_session.created_at,
            completed_at=new_session.completed_at,
            participants=[]
        )

    except Exception as e:
        logger.error(f"Error creating session: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="Failed to create session"
        )


@router.get("/{session_code}", response_model=SessionResponse)
async def get_session(
    session_code: str,
    db: AsyncSession = Depends(get_db)
):
    """Get session by code"""
    result = await db.execute(
        select(Session).where(Session.session_code == session_code)
    )
    session = result.scalar_one_or_none()

    if not session:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Session not found"
        )

    # Get participants
    result = await db.execute(
        select(SessionUser).where(SessionUser.session_id == session.id)
    )
    session_users = result.scalars().all()

    participants = []
    for su in session_users:
        result = await db.execute(
            select(User).where(User.id == su.user_id)
        )
        user = result.scalar_one_or_none()
        if user:
            participants.append(SessionUserResponse(
                user_id=user.id,
                nickname=user.nickname,
                avatar_url=user.avatar_url,
                joined_at=su.joined_at,
                is_ready=su.is_ready
            ))

    return SessionResponse(
        id=session.id,
        session_code=session.session_code,
        name=session.name,
        owner_user_id=session.owner_user_id,
        status=session.status,
        expected_players=session.expected_players,
        created_at=session.created_at,
        completed_at=session.completed_at,
        participants=participants
    )


@router.post("/{session_code}/join", response_model=UserResponse)
async def join_session(
    session_code: str,
    join_data: SessionJoinRequest,
    db: AsyncSession = Depends(get_db)
):
    """Join a session with Steam profile"""
    try:
        # Get session
        result = await db.execute(
            select(Session).where(Session.session_code == session_code)
        )
        session = result.scalar_one_or_none()

        if not session:
            raise HTTPException(
                status_code=status.HTTP_404_NOT_FOUND,
                detail="Session not found"
            )

        if session.status == "completed":
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Session is already completed"
            )

        # Get Steam ID from URL
        steam_id = await steam_service.get_steam_id_from_url(join_data.steam_profile_url)
        if not steam_id:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Invalid Steam profile URL"
            )

        # Check if profile is public
        is_public = await steam_service.check_profile_visibility(steam_id)
        if not is_public:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Steam profile must be public to use this service"
            )

        # Get or create user
        result = await db.execute(
            select(User).where(User.steam_id == steam_id)
        )
        user = result.scalar_one_or_none()

        if not user:
            # Create new user
            player_summary = await steam_service.get_player_summary(steam_id)
            if not player_summary:
                raise HTTPException(
                    status_code=status.HTTP_400_BAD_REQUEST,
                    detail="Could not fetch Steam profile"
                )

            user = User(
                steam_id=steam_id,
                nickname=join_data.nickname or player_summary.get("personaname", "Unknown"),
                avatar_url=player_summary.get("avatarfull"),
                profile_url=player_summary.get("profileurl")
            )
            db.add(user)
            await db.flush()

            # Fetch and store user's games
            owned_games = await steam_service.get_owned_games(steam_id)
            if owned_games:
                for game_data in owned_games:
                    app_id = game_data.get("appid")

                    # Get or create game entry
                    from app.models.game import Game
                    result = await db.execute(
                        select(Game).where(Game.steam_app_id == app_id)
                    )
                    game = result.scalar_one_or_none()

                    if not game:
                        # Create basic game entry (will be enriched later)
                        game = Game(
                            steam_app_id=app_id,
                            title=game_data.get("name", f"Game {app_id}"),
                            header_image=f"https://steamcdn-a.akamaihd.net/steam/apps/{app_id}/header.jpg"
                        )
                        db.add(game)
                        await db.flush()

                    # Add to user's library
                    user_game = UserGame(
                        user_id=user.id,
                        game_id=game.id,
                        playtime_forever=game_data.get("playtime_forever", 0),
                        playtime_2weeks=game_data.get("playtime_2weeks", 0)
                    )
                    db.add(user_game)

        # Check if user already in session
        result = await db.execute(
            select(SessionUser).where(
                SessionUser.session_id == session.id,
                SessionUser.user_id == user.id
            )
        )
        existing = result.scalar_one_or_none()

        if not existing:
            # Add user to session
            session_user = SessionUser(
                session_id=session.id,
                user_id=user.id,
                is_ready=True
            )
            db.add(session_user)

            # Set owner if first user
            if session.owner_user_id is None:
                session.owner_user_id = user.id

        await db.commit()
        await db.refresh(user)

        logger.info(f"User {user.nickname} joined session {session_code}")

        return UserResponse(
            id=user.id,
            steam_id=user.steam_id,
            nickname=user.nickname,
            avatar_url=user.avatar_url,
            profile_url=user.profile_url,
            created_at=user.created_at
        )

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error joining session: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to join session: {str(e)}"
        )


@router.patch("/{session_code}/status", response_model=SessionResponse)
async def update_session_status(
    session_code: str,
    status_update: SessionStatusUpdate,
    db: AsyncSession = Depends(get_db)
):
    """Update session status"""
    result = await db.execute(
        select(Session).where(Session.session_code == session_code)
    )
    session = result.scalar_one_or_none()

    if not session:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Session not found"
        )

    session.status = status_update.status
    if status_update.status == "completed":
        session.completed_at = datetime.utcnow()

    await db.commit()
    await db.refresh(session)

    # Get participants
    result = await db.execute(
        select(SessionUser).where(SessionUser.session_id == session.id)
    )
    session_users = result.scalars().all()

    participants = []
    for su in session_users:
        result = await db.execute(
            select(User).where(User.id == su.user_id)
        )
        user = result.scalar_one_or_none()
        if user:
            participants.append(SessionUserResponse(
                user_id=user.id,
                nickname=user.nickname,
                avatar_url=user.avatar_url,
                joined_at=su.joined_at,
                is_ready=su.is_ready
            ))

    return SessionResponse(
        id=session.id,
        session_code=session.session_code,
        name=session.name,
        owner_user_id=session.owner_user_id,
        status=session.status,
        expected_players=session.expected_players,
        created_at=session.created_at,
        completed_at=session.completed_at,
        participants=participants
    )
