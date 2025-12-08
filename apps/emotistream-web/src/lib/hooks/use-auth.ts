import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useAuthStore } from '../stores/auth-store';
import * as authApi from '../api/auth';
import type { LoginRequest, RegisterRequest } from '../api/auth';

/**
 * Hook for user login
 */
export const useLogin = () => {
  const { login: storeLogin } = useAuthStore();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: LoginRequest) => authApi.login(data),
    onSuccess: (response) => {
      storeLogin(response.user, response.accessToken, response.refreshToken);
      queryClient.invalidateQueries({ queryKey: ['user'] });
    },
  });
};

/**
 * Hook for user registration
 */
export const useRegister = () => {
  const { login: storeLogin } = useAuthStore();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: RegisterRequest) => authApi.register(data),
    onSuccess: (response) => {
      storeLogin(response.user, response.accessToken, response.refreshToken);
      queryClient.invalidateQueries({ queryKey: ['user'] });
    },
  });
};

/**
 * Hook for user logout
 */
export const useLogout = () => {
  const { logout: storeLogout } = useAuthStore();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => authApi.logout(),
    onSuccess: () => {
      storeLogout();
      queryClient.clear();
    },
  });
};

/**
 * Hook to get current user
 */
export const useCurrentUser = () => {
  const { isAuthenticated } = useAuthStore();

  return useQuery({
    queryKey: ['user', 'current'],
    queryFn: () => authApi.getCurrentUser(),
    enabled: isAuthenticated,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};

/**
 * Hook to refresh access token
 */
export const useRefreshToken = () => {
  const { refreshToken } = useAuthStore();

  return useMutation({
    mutationFn: () => {
      if (!refreshToken) {
        throw new Error('No refresh token available');
      }
      return authApi.refreshToken(refreshToken);
    },
  });
};
