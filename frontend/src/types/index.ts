// Type definitions for the application

export interface User {
  id: string;
  steam_id: string;
  nickname: string;
  avatar_url?: string;
  profile_url?: string;
  created_at: string;
}

export interface Game {
  id: string;
  steam_app_id: number;
  title: string;
  description?: string;
  short_description?: string;
  header_image?: string;
  tags: string[];
  genres: string[];
  categories: string[];
  is_multiplayer: boolean;
  is_coop: boolean;
  is_pvp: boolean;
  metacritic_score?: number;
  release_date?: string;
  developers: string[];
  publishers: string[];
}

export interface SessionUser {
  user_id: string;
  nickname: string;
  avatar_url?: string;
  joined_at: string;
  is_ready: boolean;
}

export interface Session {
  id: string;
  session_code: string;
  name?: string;
  owner_user_id?: string;
  status: 'waiting' | 'ready' | 'completed';
  expected_players?: number;
  created_at: string;
  completed_at?: string;
  participants: SessionUser[];
}

export interface Recommendation {
  id: string;
  session_id: string;
  game: Game;
  rank: number;
  ownership_category: 'G1' | 'G2' | 'G3';
  score?: number;
  explanation?: string;
  pros?: string[];
  cons?: string[];
  best_for?: string[];
  created_at: string;
}

export interface Feedback {
  id: string;
  session_id: string;
  user_id: string;
  game_id: string;
  recommendation_id?: string;
  feedback: 1 | -1;
  reason_code?: string;
  reason_text?: string;
  created_at: string;
}

export interface FeedbackCreate {
  game_id: string;
  recommendation_id?: string;
  feedback: 1 | -1;
  reason_code?: string;
  reason_text?: string;
}
