'use client';

import { create } from 'zustand';
import { apiClient } from './api';
import { useWebSocketStore } from './websocket';
import type { User } from './types';

// PKCE helper functions
function generateCodeVerifier(): string {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return btoa(String.fromCharCode.apply(null, Array.from(array)))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const digest = await crypto.subtle.digest('SHA-256', data);
  return btoa(String.fromCharCode.apply(null, Array.from(new Uint8Array(digest))))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

interface AuthStore {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;

  setAuth: (token: string, user: User) => void;
  logout: () => void;
  initialize: () => Promise<void>;
}

export const useAuthStore = create<AuthStore>((set, get) => ({
  user: null,
  token: null,
  isAuthenticated: false,
  isLoading: true,

  setAuth: (token: string, user: User) => {
    apiClient.setToken(token);
    set({ token, user, isAuthenticated: true, isLoading: false });

    // Connect to WebSocket
    const wsStore = useWebSocketStore.getState();
    wsStore.connect(token);
  },

  logout: () => {
    apiClient.clearToken();
    const wsStore = useWebSocketStore.getState();
    wsStore.disconnect();
    set({ token: null, user: null, isAuthenticated: false, isLoading: false });

    // Redirect to home (which will trigger SSO flow)
    if (typeof window !== 'undefined') {
      window.location.href = '/';
    }
  },

  initialize: async () => {
    try {
      if (typeof window !== 'undefined') {
        // Check for existing token in localStorage
        const existingToken = apiClient.getToken();
        if (existingToken) {
          try {
            // Verify token is still valid by making an API call
            apiClient.setToken(existingToken);
            const users = await apiClient.listUsers();

            // Get current user info
            const userInfo = await apiClient.getUserInfo(existingToken);
            const currentUser = users.find((u) => u.tv_user_id === userInfo.sub);

            if (currentUser) {
              get().setAuth(existingToken, currentUser);
              return;
            }
          } catch (error) {
            console.error('Token validation failed:', error);
            apiClient.clearToken();
          }
        }

        // No valid token, redirect to TitaniumVault applications page
        // User must access OpenChat through TitaniumVault's applications portal
        // which will initiate the proper OAuth flow with authorization code
        const tvUiUrl = process.env.NEXT_PUBLIC_TV_UI_URL || 'https://titanium-vault.com';

        window.location.href = `${tvUiUrl}/applications`;
      }
    } catch (error) {
      console.error('Auth initialization error:', error);
      set({ isLoading: false });
    }
  },
}));

export function useAuth() {
  return useAuthStore();
}
