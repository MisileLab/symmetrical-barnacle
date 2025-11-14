"""Feedback API endpoints"""
from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select
import logging

from app.database import get_db
from app.models.session import Session
from app.models.feedback import Feedback
from app.models.recommendation import RecommendationMetric
from app.models.user import UserPreference
from app.schemas.feedback import FeedbackCreate, FeedbackResponse

logger = logging.getLogger(__name__)
router = APIRouter(prefix="/feedback", tags=["feedback"])


@router.post("/{session_code}/{user_id}", response_model=FeedbackResponse, status_code=status.HTTP_201_CREATED)
async def create_feedback(
    session_code: str,
    user_id: str,
    feedback_data: FeedbackCreate,
    db: AsyncSession = Depends(get_db)
):
    """Create feedback for a game recommendation"""
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

        # Check if feedback already exists
        result = await db.execute(
            select(Feedback).where(
                Feedback.session_id == session.id,
                Feedback.user_id == user_id,
                Feedback.game_id == feedback_data.game_id
            )
        )
        existing = result.scalar_one_or_none()

        if existing:
            # Update existing feedback
            existing.feedback = feedback_data.feedback
            existing.reason_code = feedback_data.reason_code
            existing.reason_text = feedback_data.reason_text
            feedback = existing
        else:
            # Create new feedback
            feedback = Feedback(
                session_id=session.id,
                user_id=user_id,
                game_id=feedback_data.game_id,
                recommendation_id=feedback_data.recommendation_id,
                feedback=feedback_data.feedback,
                reason_code=feedback_data.reason_code,
                reason_text=feedback_data.reason_text
            )
            db.add(feedback)

        # Update recommendation metrics
        if feedback_data.recommendation_id:
            result = await db.execute(
                select(RecommendationMetric)
                .where(RecommendationMetric.recommendation_id == feedback_data.recommendation_id)
            )
            metric = result.scalar_one_or_none()

            if metric:
                if feedback_data.feedback > 0:
                    metric.positive_feedback_count += 1
                else:
                    metric.negative_feedback_count += 1

        # Update user preferences (Phase 2 feature)
        result = await db.execute(
            select(UserPreference).where(UserPreference.user_id == user_id)
        )
        user_pref = result.scalar_one_or_none()

        if not user_pref:
            user_pref = UserPreference(
                user_id=user_id,
                preference_data={}
            )
            db.add(user_pref)

        await db.commit()
        await db.refresh(feedback)

        logger.info(f"Created feedback for user {user_id}, game {feedback_data.game_id}: {feedback_data.feedback}")

        return FeedbackResponse(
            id=feedback.id,
            session_id=feedback.session_id,
            user_id=feedback.user_id,
            game_id=feedback.game_id,
            recommendation_id=feedback.recommendation_id,
            feedback=feedback.feedback,
            reason_code=feedback.reason_code,
            reason_text=feedback.reason_text,
            created_at=feedback.created_at
        )

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error creating feedback: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to create feedback: {str(e)}"
        )


@router.get("/{session_code}/{user_id}", response_model=list[FeedbackResponse])
async def get_user_feedback(
    session_code: str,
    user_id: str,
    db: AsyncSession = Depends(get_db)
):
    """Get all feedback from a user in a session"""
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

        # Get feedback
        result = await db.execute(
            select(Feedback).where(
                Feedback.session_id == session.id,
                Feedback.user_id == user_id
            )
        )
        feedbacks = result.scalars().all()

        return [
            FeedbackResponse(
                id=f.id,
                session_id=f.session_id,
                user_id=f.user_id,
                game_id=f.game_id,
                recommendation_id=f.recommendation_id,
                feedback=f.feedback,
                reason_code=f.reason_code,
                reason_text=f.reason_text,
                created_at=f.created_at
            )
            for f in feedbacks
        ]

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error getting feedback: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to get feedback: {str(e)}"
        )
