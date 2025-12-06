import apiClient from './client';
import type { User } from '../../types';

export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  user: User;
  accessToken: string;
  refreshToken: string;
}

export interface RegisterRequest {
  email: string;
  password: string;
  name: string;
}

export interface RegisterResponse {
  user: User;
  accessToken: string;
  refreshToken: string;
}

export interface RefreshTokenRequest {
  refreshToken: string;
}

export interface RefreshTokenResponse {
  accessToken: string;
}

// Backend response wrapper type
interface BackendResponse<T> {
  success: boolean;
  data: T;
  error: { code: string; message: string; details: unknown } | null;
  timestamp: string;
}

// Backend auth data format
interface BackendAuthData {
  userId: string;
  email: string;
  displayName: string;
  token: string;
  refreshToken: string;
  expiresAt: string;
  createdAt: string;
}

/**
 * Login user with email and password
 */
export const login = async (data: LoginRequest): Promise<LoginResponse> => {
  const response = await apiClient.post<BackendResponse<BackendAuthData>>('/auth/login', data);
  const backendData = response.data.data;

  // Transform backend response to frontend format
  return {
    user: {
      id: backendData.userId,
      email: backendData.email,
      name: backendData.displayName,
      createdAt: backendData.createdAt,
    },
    accessToken: backendData.token,
    refreshToken: backendData.refreshToken,
  };
};

/**
 * Register new user
 */
export const register = async (data: RegisterRequest): Promise<RegisterResponse> => {
  const response = await apiClient.post<BackendResponse<BackendAuthData>>('/auth/register', data);
  const backendData = response.data.data;

  // Transform backend response to frontend format
  return {
    user: {
      id: backendData.userId,
      email: backendData.email,
      name: backendData.displayName,
      createdAt: backendData.createdAt,
    },
    accessToken: backendData.token,
    refreshToken: backendData.refreshToken,
  };
};

// Backend refresh token response format
interface BackendRefreshData {
  token: string;
  expiresAt: string;
}

/**
 * Refresh access token
 */
export const refreshToken = async (refreshToken: string): Promise<RefreshTokenResponse> => {
  const response = await apiClient.post<BackendResponse<BackendRefreshData>>('/auth/refresh', {
    refreshToken,
  });
  return {
    accessToken: response.data.data.token,
  };
};

/**
 * Logout user
 */
export const logout = async (): Promise<void> => {
  await apiClient.post('/auth/logout');
  localStorage.removeItem('accessToken');
  localStorage.removeItem('refreshToken');
};

/**
 * Get current user profile
 */
export const getCurrentUser = async (): Promise<User> => {
  const response = await apiClient.get<User>('/auth/me');
  return response.data;
};
