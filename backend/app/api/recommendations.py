"""Recommendation API endpoints"""
from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, delete
from typing import List
import logging
from datetime import datetime

from app.database import get_db
from app.models.session import Session, SessionUser
from app.models.recommendation import Recommendation, RecommendationMetric
from app.schemas.recommendation import (
    GenerateRecommendationsRequest,
    RecommendationResponse,
    RecommendationListResponse
)
from app.schemas.game import GameDetailResponse
from app.services.recommendation_service import recommendation_service

logger = logging.getLogger(__name__)
router = APIRouter(prefix="/recommendations", tags=["recommendations"])


@router.post("/{session_code}/generate", response_model=RecommendationListResponse)
async def generate_recommendations(
    session_code: str,
    request: GenerateRecommendationsRequest,
    db: AsyncSession = Depends(get_db)
):
    """Generate game recommendations for a session"""
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

        # Get session users
        result = await db.execute(
            select(SessionUser).where(SessionUser.session_id == session.id)
        )
        session_users = result.scalars().all()

        if len(session_users) < 2:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Session must have at least 2 participants"
            )

        user_ids = [str(su.user_id) for su in session_users]

        # Delete existing recommendations for this session
        await db.execute(
            delete(Recommendation).where(Recommendation.session_id == session.id)
        )

        # Generate recommendations
        exclude_ids = [str(gid) for gid in request.exclude_game_ids] if request.exclude_game_ids else []
        recommendations = await recommendation_service.generate_recommendations(
            session_id=str(session.id),
            user_ids=user_ids,
            db=db,
            exclude_game_ids=exclude_ids,
            count=request.count
        )

        if not recommendations:
            raise HTTPException(
                status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
                detail="Failed to generate recommendations"
            )

        # Save recommendations to database
        saved_recommendations = []
        for rec in recommendations:
            new_rec = Recommendation(
                session_id=session.id,
                game_id=rec["game_id"],
                rank=rec["rank"],
                ownership_category=rec["ownership_category"],
                score=rec.get("score"),
                explanation=rec.get("explanation"),
                metadata_snapshot={
                    "pros": rec.get("pros", []),
                    "cons": rec.get("cons", []),
                    "best_for": rec.get("best_for", [])
                }
            )
            db.add(new_rec)
            await db.flush()

            # Create metrics entry
            metric = RecommendationMetric(
                recommendation_id=new_rec.id
            )
            db.add(metric)

            saved_recommendations.append(new_rec)

        await db.commit()

        # Update session status
        if session.status == "waiting":
            session.status = "ready"
            await db.commit()

        # Format response
        response_recs = []
        for rec in saved_recommendations:
            game = rec.game
            metadata = rec.metadata_snapshot or {}

            response_recs.append(RecommendationResponse(
                id=rec.id,
                session_id=rec.session_id,
                game=GameDetailResponse.from_orm(game),
                rank=rec.rank,
                ownership_category=rec.ownership_category,
                score=rec.score,
                explanation=rec.explanation,
                pros=metadata.get("pros"),
                cons=metadata.get("cons"),
                best_for=metadata.get("best_for"),
                metadata_snapshot=metadata,
                created_at=rec.created_at
            ))

        logger.info(f"Generated {len(response_recs)} recommendations for session {session_code}")

        return RecommendationListResponse(
            session_id=session.id,
            recommendations=response_recs,
            total_count=len(response_recs),
            generated_at=datetime.utcnow()
        )

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error generating recommendations: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to generate recommendations: {str(e)}"
        )


@router.get("/{session_code}", response_model=RecommendationListResponse)
async def get_recommendations(
    session_code: str,
    db: AsyncSession = Depends(get_db)
):
    """Get recommendations for a session"""
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

        # Get recommendations
        result = await db.execute(
            select(Recommendation)
            .where(Recommendation.session_id == session.id)
            .order_by(Recommendation.rank)
        )
        recommendations = result.scalars().all()

        # Format response
        response_recs = []
        for rec in recommendations:
            game = rec.game
            metadata = rec.metadata_snapshot or {}

            response_recs.append(RecommendationResponse(
                id=rec.id,
                session_id=rec.session_id,
                game=GameDetailResponse.from_orm(game),
                rank=rec.rank,
                ownership_category=rec.ownership_category,
                score=rec.score,
                explanation=rec.explanation,
                pros=metadata.get("pros"),
                cons=metadata.get("cons"),
                best_for=metadata.get("best_for"),
                metadata_snapshot=metadata,
                created_at=rec.created_at
            ))

        return RecommendationListResponse(
            session_id=session.id,
            recommendations=response_recs,
            total_count=len(response_recs),
            generated_at=recommendations[0].created_at if recommendations else datetime.utcnow()
        )

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error getting recommendations: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to get recommendations: {str(e)}"
        )


@router.post("/{recommendation_id}/click")
async def track_click(
    recommendation_id: str,
    db: AsyncSession = Depends(get_db)
):
    """Track a click on a recommendation"""
    try:
        result = await db.execute(
            select(RecommendationMetric)
            .where(RecommendationMetric.recommendation_id == recommendation_id)
        )
        metric = result.scalar_one_or_none()

        if metric:
            metric.clicks += 1
            await db.commit()

        return {"status": "success"}

    except Exception as e:
        logger.error(f"Error tracking click: {e}")
        return {"status": "error"}
