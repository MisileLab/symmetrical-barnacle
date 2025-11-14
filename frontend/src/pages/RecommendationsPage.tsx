import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { recommendationAPI, feedbackAPI, sessionAPI } from '../services/api';
import type { Recommendation, Session } from '../types';

const RecommendationsPage: React.FC = () => {
  const { sessionCode } = useParams<{ sessionCode: string }>();

  const [session, setSession] = useState<Session | null>(null);
  const [recommendations, setRecommendations] = useState<Recommendation[]>([]);
  const [isGenerating, setIsGenerating] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState('');
  const [excludedGames, setExcludedGames] = useState<Set<string>>(new Set());
  const [userFeedback, setUserFeedback] = useState<Map<string, 1 | -1>>(new Map());
  const [currentUserId, setCurrentUserId] = useState<string | null>(null);

  useEffect(() => {
    if (sessionCode) {
      loadData();
    }
  }, [sessionCode]);

  const loadData = async () => {
    try {
      if (!sessionCode) return;

      const sessionData = await sessionAPI.get(sessionCode);
      setSession(sessionData);

      // Get current user (last participant for demo)
      if (sessionData.participants.length > 0) {
        setCurrentUserId(sessionData.participants[0].user_id);
      }

      // Try to load existing recommendations
      try {
        const data = await recommendationAPI.get(sessionCode);
        setRecommendations(data.recommendations);
        setIsLoading(false);
      } catch (err) {
        // No recommendations yet, generate them
        await generateRecommendations();
      }
    } catch (err: any) {
      setError(err.response?.data?.detail || 'Failed to load data');
      setIsLoading(false);
    }
  };

  const generateRecommendations = async (exclude: string[] = []) => {
    if (!sessionCode) return;

    setIsGenerating(true);
    setError('');

    try {
      const data = await recommendationAPI.generate(
        sessionCode,
        exclude,
        10
      );
      setRecommendations(data.recommendations);
    } catch (err: any) {
      setError(err.response?.data?.detail || 'Failed to generate recommendations');
    } finally {
      setIsGenerating(false);
      setIsLoading(false);
    }
  };

  const handleFeedback = async (recommendation: Recommendation, feedback: 1 | -1) => {
    if (!sessionCode || !currentUserId) return;

    try {
      await feedbackAPI.create(sessionCode, currentUserId, {
        game_id: recommendation.game.id,
        recommendation_id: recommendation.id,
        feedback,
      });

      setUserFeedback(new Map(userFeedback.set(recommendation.game.id, feedback)));
    } catch (err) {
      console.error('Failed to submit feedback:', err);
    }
  };

  const handleExcludeGame = (gameId: string) => {
    const newExcluded = new Set(excludedGames);
    newExcluded.add(gameId);
    setExcludedGames(newExcluded);
  };

  const handleRegenerateWithExclusions = () => {
    generateRecommendations(Array.from(excludedGames));
  };

  const getCategoryColor = (category: string) => {
    switch (category) {
      case 'G1':
        return '#34c759'; // Green - All own
      case 'G2':
        return '#ff9500'; // Orange - Some own
      case 'G3':
        return '#5856d6'; // Purple - None own
      default:
        return '#8e8e93';
    }
  };

  const getCategoryLabel = (category: string) => {
    switch (category) {
      case 'G1':
        return 'Everyone Owns';
      case 'G2':
        return 'Some Own';
      case 'G3':
        return 'New Discovery';
      default:
        return category;
    }
  };

  if (isLoading) {
    return (
      <div className="container">
        <div className="loading">
          <div className="loading-spinner"></div>
          <p>{isGenerating ? 'Generating recommendations...' : 'Loading...'}</p>
          {isGenerating && (
            <p style={{ fontSize: '14px', opacity: 0.7, marginTop: '10px' }}>
              This may take a few moments as we analyze your group's preferences
            </p>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="container">
      <div className="card">
        <h1>🎮 Game Recommendations</h1>
        {session && (
          <p style={{ opacity: 0.8 }}>
            For session: <strong>{session.name || session.session_code}</strong>
          </p>
        )}
      </div>

      {error && (
        <div className="error">{error}</div>
      )}

      {excludedGames.size > 0 && (
        <div className="card">
          <h3>Excluded Games: {excludedGames.size}</h3>
          <button
            className="button"
            onClick={handleRegenerateWithExclusions}
            disabled={isGenerating}
          >
            Regenerate Without Excluded Games
          </button>
        </div>
      )}

      <div className="grid grid-2">
        {recommendations.map((rec) => (
          <div key={rec.id} className="card">
            {/* Game Header Image */}
            {rec.game.header_image && (
              <img
                src={rec.game.header_image}
                alt={rec.game.title}
                style={{
                  width: '100%',
                  borderRadius: '10px',
                  marginBottom: '15px'
                }}
              />
            )}

            {/* Title and Category */}
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '10px' }}>
              <h2 style={{ margin: 0, flex: 1 }}>{rec.game.title}</h2>
              <span style={{
                padding: '6px 12px',
                borderRadius: '15px',
                background: getCategoryColor(rec.ownership_category),
                fontSize: '12px',
                fontWeight: '600',
                marginLeft: '10px',
                whiteSpace: 'nowrap'
              }}>
                {getCategoryLabel(rec.ownership_category)}
              </span>
            </div>

            {/* Rank and Score */}
            <div style={{ opacity: 0.7, marginBottom: '15px', fontSize: '14px' }}>
              Rank: #{rec.rank} {rec.score && `• Score: ${(rec.score * 100).toFixed(0)}%`}
            </div>

            {/* Genres */}
            <div style={{ marginBottom: '15px' }}>
              {rec.game.genres.slice(0, 3).map((genre) => (
                <span
                  key={genre}
                  style={{
                    display: 'inline-block',
                    padding: '4px 10px',
                    marginRight: '8px',
                    marginBottom: '8px',
                    background: 'rgba(255, 255, 255, 0.1)',
                    borderRadius: '12px',
                    fontSize: '12px'
                  }}
                >
                  {genre}
                </span>
              ))}
              {rec.game.is_multiplayer && (
                <span style={{
                  display: 'inline-block',
                  padding: '4px 10px',
                  marginRight: '8px',
                  background: 'rgba(52, 199, 89, 0.3)',
                  borderRadius: '12px',
                  fontSize: '12px'
                }}>
                  Multiplayer
                </span>
              )}
              {rec.game.is_coop && (
                <span style={{
                  display: 'inline-block',
                  padding: '4px 10px',
                  background: 'rgba(52, 199, 89, 0.3)',
                  borderRadius: '12px',
                  fontSize: '12px'
                }}>
                  Co-op
                </span>
              )}
            </div>

            {/* Explanation */}
            {rec.explanation && (
              <p style={{ fontSize: '14px', opacity: 0.9, marginBottom: '15px', lineHeight: '1.5' }}>
                {rec.explanation}
              </p>
            )}

            {/* Pros/Cons */}
            {(rec.pros || rec.cons) && (
              <div style={{ fontSize: '13px', marginBottom: '15px' }}>
                {rec.pros && rec.pros.length > 0 && (
                  <div style={{ marginBottom: '10px' }}>
                    <strong style={{ color: '#34c759' }}>✓ Pros:</strong>
                    <ul style={{ marginTop: '5px', paddingLeft: '20px' }}>
                      {rec.pros.map((pro, i) => (
                        <li key={i} style={{ opacity: 0.9 }}>{pro}</li>
                      ))}
                    </ul>
                  </div>
                )}
                {rec.cons && rec.cons.length > 0 && (
                  <div>
                    <strong style={{ color: '#ff3b30' }}>✗ Cons:</strong>
                    <ul style={{ marginTop: '5px', paddingLeft: '20px' }}>
                      {rec.cons.map((con, i) => (
                        <li key={i} style={{ opacity: 0.9 }}>{con}</li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            )}

            {/* Feedback Buttons */}
            <div style={{ display: 'flex', gap: '10px', marginTop: '15px' }}>
              <button
                className={`button ${userFeedback.get(rec.game.id) === 1 ? '' : 'button-secondary'}`}
                onClick={() => handleFeedback(rec, 1)}
                style={{ flex: 1 }}
              >
                👍 Like
              </button>
              <button
                className={`button ${userFeedback.get(rec.game.id) === -1 ? '' : 'button-secondary'}`}
                onClick={() => handleFeedback(rec, -1)}
                style={{ flex: 1 }}
              >
                👎 Pass
              </button>
              <button
                className="button button-secondary"
                onClick={() => handleExcludeGame(rec.game.id)}
                title="Exclude from future recommendations"
              >
                ✕
              </button>
            </div>
          </div>
        ))}
      </div>

      {recommendations.length === 0 && !isGenerating && (
        <div className="card" style={{ textAlign: 'center', padding: '50px' }}>
          <h2>No recommendations available</h2>
          <p>Click the button below to generate recommendations</p>
          <button
            className="button"
            onClick={() => generateRecommendations()}
            style={{ marginTop: '20px' }}
          >
            Generate Recommendations
          </button>
        </div>
      )}
    </div>
  );
};

export default RecommendationsPage;
