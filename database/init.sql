-- Steam Party Picker Database Schema

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    steam_id VARCHAR(100) UNIQUE NOT NULL,
    nickname VARCHAR(100) NOT NULL,
    avatar_url TEXT,
    profile_url TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Games table
CREATE TABLE IF NOT EXISTS games (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    steam_app_id INTEGER UNIQUE NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    short_description TEXT,
    header_image TEXT,
    tags JSONB DEFAULT '[]',
    genres JSONB DEFAULT '[]',
    categories JSONB DEFAULT '[]',
    release_date DATE,
    developers JSONB DEFAULT '[]',
    publishers JSONB DEFAULT '[]',
    price_info JSONB,
    metacritic_score INTEGER,
    is_multiplayer BOOLEAN DEFAULT FALSE,
    is_coop BOOLEAN DEFAULT FALSE,
    is_pvp BOOLEAN DEFAULT FALSE,
    player_count_min INTEGER,
    player_count_max INTEGER,
    embedding_id VARCHAR(255),  -- Reference to ChromaDB
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_games_steam_app_id ON games(steam_app_id);
CREATE INDEX idx_games_multiplayer ON games(is_multiplayer);
CREATE INDEX idx_games_tags ON games USING GIN(tags);

-- User's game library
CREATE TABLE IF NOT EXISTS user_games (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    game_id UUID REFERENCES games(id) ON DELETE CASCADE,
    playtime_forever INTEGER DEFAULT 0,  -- in minutes
    playtime_2weeks INTEGER DEFAULT 0,   -- in minutes
    last_played TIMESTAMP,
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, game_id)
);

CREATE INDEX idx_user_games_user_id ON user_games(user_id);
CREATE INDEX idx_user_games_playtime ON user_games(playtime_forever DESC);

-- Sessions (party groups)
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_code VARCHAR(20) UNIQUE NOT NULL,
    name VARCHAR(200),
    owner_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    status VARCHAR(50) DEFAULT 'waiting',  -- waiting, ready, completed
    expected_players INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);

CREATE INDEX idx_sessions_code ON sessions(session_code);
CREATE INDEX idx_sessions_status ON sessions(status);

-- Session participants
CREATE TABLE IF NOT EXISTS session_users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES sessions(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_ready BOOLEAN DEFAULT FALSE,
    UNIQUE(session_id, user_id)
);

CREATE INDEX idx_session_users_session_id ON session_users(session_id);

-- Recommendations generated for sessions
CREATE TABLE IF NOT EXISTS recommendations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES sessions(id) ON DELETE CASCADE,
    game_id UUID REFERENCES games(id) ON DELETE CASCADE,
    rank INTEGER NOT NULL,
    ownership_category VARCHAR(10) NOT NULL,  -- G1, G2, G3
    score FLOAT,
    explanation TEXT,
    metadata_snapshot JSONB,  -- Snapshot of game info at recommendation time
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(session_id, game_id)
);

CREATE INDEX idx_recommendations_session_id ON recommendations(session_id);
CREATE INDEX idx_recommendations_rank ON recommendations(session_id, rank);

-- User feedback on recommendations
CREATE TABLE IF NOT EXISTS feedback (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES sessions(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    game_id UUID REFERENCES games(id) ON DELETE CASCADE,
    recommendation_id UUID REFERENCES recommendations(id) ON DELETE CASCADE,
    feedback INTEGER NOT NULL,  -- +1 (like) or -1 (dislike)
    reason_code VARCHAR(50),  -- genre_mismatch, too_complex, pvp_dislike, horror, already_played
    reason_text TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(session_id, user_id, game_id)
);

CREATE INDEX idx_feedback_user_game ON feedback(user_id, game_id);
CREATE INDEX idx_feedback_session ON feedback(session_id);

-- User preference vectors (derived from feedback)
CREATE TABLE IF NOT EXISTS user_preferences (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE UNIQUE,
    preference_data JSONB NOT NULL,  -- Stores preference weights, liked/disliked features
    liked_tags JSONB DEFAULT '[]',
    disliked_tags JSONB DEFAULT '[]',
    liked_genres JSONB DEFAULT '[]',
    disliked_genres JSONB DEFAULT '[]',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Group favorites
CREATE TABLE IF NOT EXISTS group_favorites (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES sessions(id) ON DELETE CASCADE,
    game_id UUID REFERENCES games(id) ON DELETE CASCADE,
    added_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(session_id, game_id)
);

-- Recommendation history metadata
CREATE TABLE IF NOT EXISTS recommendation_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    recommendation_id UUID REFERENCES recommendations(id) ON DELETE CASCADE,
    clicks INTEGER DEFAULT 0,
    positive_feedback_count INTEGER DEFAULT 0,
    negative_feedback_count INTEGER DEFAULT 0,
    was_selected BOOLEAN DEFAULT FALSE,
    was_played BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Phase 3: Discord integration
CREATE TABLE IF NOT EXISTS discord_integrations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    guild_id VARCHAR(100) NOT NULL,
    channel_id VARCHAR(100),
    session_id UUID REFERENCES sessions(id) ON DELETE CASCADE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(guild_id, session_id)
);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply updated_at triggers
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_games_updated_at BEFORE UPDATE ON games
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_preferences_updated_at BEFORE UPDATE ON user_preferences
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_recommendation_metrics_updated_at BEFORE UPDATE ON recommendation_metrics
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
