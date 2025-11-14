import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { sessionAPI } from '../services/api';
import type { Session, User } from '../types';

const SessionPage: React.FC = () => {
  const { sessionCode } = useParams<{ sessionCode: string }>();
  const navigate = useNavigate();

  const [session, setSession] = useState<Session | null>(null);
  const [steamProfileUrl, setSteamProfileUrl] = useState('');
  const [nickname, setNickname] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [currentUser, setCurrentUser] = useState<User | null>(null);

  useEffect(() => {
    if (sessionCode) {
      loadSession();
      const interval = setInterval(loadSession, 5000); // Refresh every 5 seconds
      return () => clearInterval(interval);
    }
  }, [sessionCode]);

  const loadSession = async () => {
    try {
      if (!sessionCode) return;
      const data = await sessionAPI.get(sessionCode);
      setSession(data);
    } catch (err) {
      console.error('Failed to load session:', err);
    }
  };

  const handleJoinSession = async () => {
    if (!sessionCode || !steamProfileUrl.trim()) return;

    setIsLoading(true);
    setError('');

    try {
      const user = await sessionAPI.join(sessionCode, steamProfileUrl, nickname || undefined);
      setCurrentUser(user);
      await loadSession();
      setSteamProfileUrl('');
      setNickname('');
    } catch (err: any) {
      setError(err.response?.data?.detail || 'Failed to join session');
    } finally {
      setIsLoading(false);
    }
  };

  const handleGenerateRecommendations = () => {
    navigate(`/session/${sessionCode}/recommendations`);
  };

  if (!session) {
    return (
      <div className="container">
        <div className="loading">
          <div className="loading-spinner"></div>
          <p>Loading session...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="container">
      <div className="card">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div>
            <h1>{session.name || 'Game Session'}</h1>
            <p style={{ opacity: 0.8 }}>Session Code: <strong>{session.session_code}</strong></p>
          </div>
          <div>
            <span style={{
              padding: '8px 16px',
              borderRadius: '20px',
              background: session.status === 'ready' ? 'rgba(52, 199, 89, 0.3)' : 'rgba(255, 159, 10, 0.3)',
              fontSize: '14px',
              fontWeight: '600'
            }}>
              {session.status.toUpperCase()}
            </span>
          </div>
        </div>
      </div>

      {/* Participants */}
      <div className="card">
        <h2>Participants ({session.participants.length})</h2>
        <div className="grid grid-3" style={{ marginTop: '20px' }}>
          {session.participants.map((participant) => (
            <div
              key={participant.user_id}
              style={{
                background: 'rgba(255, 255, 255, 0.05)',
                padding: '15px',
                borderRadius: '10px',
                display: 'flex',
                alignItems: 'center',
                gap: '15px'
              }}
            >
              {participant.avatar_url && (
                <img
                  src={participant.avatar_url}
                  alt={participant.nickname}
                  style={{ width: '50px', height: '50px', borderRadius: '50%' }}
                />
              )}
              <div>
                <div style={{ fontWeight: '600' }}>{participant.nickname}</div>
                <div style={{ fontSize: '12px', opacity: 0.7 }}>
                  {participant.is_ready ? '✓ Ready' : 'Waiting...'}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Join Form */}
      {!currentUser && (
        <div className="card">
          <h2>Join This Session</h2>
          <p style={{ opacity: 0.8, marginBottom: '20px' }}>
            Enter your Steam profile URL to join. Your profile must be public.
          </p>

          <input
            type="text"
            className="input"
            placeholder="Steam Profile URL (e.g., https://steamcommunity.com/id/yourprofile)"
            value={steamProfileUrl}
            onChange={(e) => setSteamProfileUrl(e.target.value)}
            disabled={isLoading}
          />

          <input
            type="text"
            className="input"
            placeholder="Nickname (optional)"
            value={nickname}
            onChange={(e) => setNickname(e.target.value)}
            disabled={isLoading}
          />

          {error && <div className="error">{error}</div>}

          <button
            className="button"
            onClick={handleJoinSession}
            disabled={isLoading || !steamProfileUrl.trim()}
            style={{ width: '100%', marginTop: '10px' }}
          >
            {isLoading ? 'Joining...' : 'Join Session'}
          </button>

          <div style={{ marginTop: '20px', padding: '15px', background: 'rgba(255, 255, 255, 0.05)', borderRadius: '10px' }}>
            <p style={{ fontSize: '14px', opacity: 0.8 }}>
              <strong>Note:</strong> Your Steam profile and game library must be set to public for this service to work.
              You can change this in your Steam Privacy Settings.
            </p>
          </div>
        </div>
      )}

      {/* Generate Recommendations Button */}
      {currentUser && session.participants.length >= 2 && (
        <div className="card">
          <h2>Ready to Find Games?</h2>
          <p style={{ opacity: 0.8, marginBottom: '20px' }}>
            {session.participants.length} participants have joined. Generate game recommendations now!
          </p>

          <button
            className="button"
            onClick={handleGenerateRecommendations}
            style={{ width: '100%' }}
          >
            Generate Recommendations 🎮
          </button>
        </div>
      )}

      {/* Share Link */}
      <div className="card">
        <h3>Share This Session</h3>
        <p style={{ opacity: 0.8, marginBottom: '15px' }}>
          Share this link with your friends:
        </p>
        <div style={{
          background: 'rgba(255, 255, 255, 0.1)',
          padding: '15px',
          borderRadius: '10px',
          wordBreak: 'break-all'
        }}>
          {window.location.href}
        </div>
      </div>
    </div>
  );
};

export default SessionPage;
