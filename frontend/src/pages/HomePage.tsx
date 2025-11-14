import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { sessionAPI } from '../services/api';

const HomePage: React.FC = () => {
  const [sessionName, setSessionName] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const navigate = useNavigate();

  const handleCreateSession = async () => {
    setIsLoading(true);
    setError('');

    try {
      const session = await sessionAPI.create(sessionName || undefined);
      navigate(`/session/${session.session_code}`);
    } catch (err: any) {
      setError(err.response?.data?.detail || 'Failed to create session');
    } finally {
      setIsLoading(false);
    }
  };

  const [joinCode, setJoinCode] = useState('');

  const handleJoinSession = () => {
    if (joinCode.trim()) {
      navigate(`/session/${joinCode.toUpperCase()}`);
    }
  };

  return (
    <div className="container">
      <div style={{ textAlign: 'center', padding: '50px 0' }}>
        <h1 style={{ fontSize: '48px', marginBottom: '10px' }}>🎮 Steam Party Picker</h1>
        <p style={{ fontSize: '20px', opacity: 0.9 }}>
          Find the perfect game for your group
        </p>
      </div>

      <div className="grid grid-2">
        {/* Create Session */}
        <div className="card">
          <h2>Create New Session</h2>
          <p style={{ opacity: 0.8, margin: '10px 0 20px' }}>
            Start a new game recommendation session for your friends
          </p>

          <input
            type="text"
            className="input"
            placeholder="Session name (optional)"
            value={sessionName}
            onChange={(e) => setSessionName(e.target.value)}
            disabled={isLoading}
          />

          {error && <div className="error">{error}</div>}

          <button
            className="button"
            onClick={handleCreateSession}
            disabled={isLoading}
            style={{ width: '100%', marginTop: '10px' }}
          >
            {isLoading ? 'Creating...' : 'Create Session'}
          </button>
        </div>

        {/* Join Session */}
        <div className="card">
          <h2>Join Existing Session</h2>
          <p style={{ opacity: 0.8, margin: '10px 0 20px' }}>
            Enter the session code shared by your friend
          </p>

          <input
            type="text"
            className="input"
            placeholder="Enter session code"
            value={joinCode}
            onChange={(e) => setJoinCode(e.target.value.toUpperCase())}
            maxLength={8}
          />

          <button
            className="button button-secondary"
            onClick={handleJoinSession}
            disabled={!joinCode.trim()}
            style={{ width: '100%', marginTop: '10px' }}
          >
            Join Session
          </button>
        </div>
      </div>

      {/* Features */}
      <div className="card" style={{ marginTop: '50px' }}>
        <h2>How It Works</h2>
        <div className="grid grid-3" style={{ marginTop: '30px' }}>
          <div>
            <div style={{ fontSize: '40px', marginBottom: '10px' }}>1️⃣</div>
            <h3>Create or Join</h3>
            <p style={{ opacity: 0.8 }}>
              Create a new session or join with a code
            </p>
          </div>
          <div>
            <div style={{ fontSize: '40px', marginBottom: '10px' }}>2️⃣</div>
            <h3>Connect Steam</h3>
            <p style={{ opacity: 0.8 }}>
              Everyone adds their Steam profile (public profiles required)
            </p>
          </div>
          <div>
            <div style={{ fontSize: '40px', marginBottom: '10px' }}>3️⃣</div>
            <h3>Get Recommendations</h3>
            <p style={{ opacity: 0.8 }}>
              AI analyzes your libraries and suggests perfect games for your group
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default HomePage;
