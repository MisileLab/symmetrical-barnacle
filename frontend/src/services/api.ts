// API service for communicating with backend

import axios from 'axios';
import type { Session, User, Recommendation, Feedback, FeedbackCreate } from '../types';

const API_URL = process.env.REACT_APP_API_URL || 'http://localhost:8000/api';

const api = axios.create({
  baseURL: API_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Session API
export const sessionAPI = {
  create: async (name?: string, expectedPlayers?: number): Promise<Session> => {
    const response = await api.post('/sessions/', {
      name,
      expected_players: expectedPlayers,
    });
    return response.data;
  },

  get: async (sessionCode: string): Promise<Session> => {
    const response = await api.get(`/sessions/${sessionCode}`);
    return response.data;
  },

  join: async (sessionCode: string, steamProfileUrl: string, nickname?: string): Promise<User> => {
    const response = await api.post(`/sessions/${sessionCode}/join`, {
      steam_profile_url: steamProfileUrl,
      nickname,
    });
    return response.data;
  },

  updateStatus: async (sessionCode: string, status: string): Promise<Session> => {
    const response = await api.patch(`/sessions/${sessionCode}/status`, { status });
    return response.data;
  },
};

// Recommendation API
export const recommendationAPI = {
  generate: async (
    sessionCode: string,
    excludeGameIds: string[] = [],
    count: number = 10
  ): Promise<{ recommendations: Recommendation[]; total_count: number }> => {
    const response = await api.post(`/recommendations/${sessionCode}/generate`, {
      exclude_game_ids: excludeGameIds,
      count,
    });
    return response.data;
  },

  get: async (sessionCode: string): Promise<{ recommendations: Recommendation[]; total_count: number }> => {
    const response = await api.get(`/recommendations/${sessionCode}`);
    return response.data;
  },

  trackClick: async (recommendationId: string): Promise<void> => {
    await api.post(`/recommendations/${recommendationId}/click`);
  },
};

// Feedback API
export const feedbackAPI = {
  create: async (sessionCode: string, userId: string, feedback: FeedbackCreate): Promise<Feedback> => {
    const response = await api.post(`/feedback/${sessionCode}/${userId}`, feedback);
    return response.data;
  },

  getUserFeedback: async (sessionCode: string, userId: string): Promise<Feedback[]> => {
    const response = await api.get(`/feedback/${sessionCode}/${userId}`);
    return response.data;
  },
};

export default api;
