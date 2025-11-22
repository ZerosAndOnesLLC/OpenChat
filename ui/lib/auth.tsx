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

  logout: async () => {
    apiClient.clearToken();
    const wsStore = useWebSocketStore.getState();
    wsStore.disconnect();
    set({ token: null, user: null, isAuthenticated: false, isLoading: false });

    // If in Tauri desktop app, clear keychain token
    if (typeof window !== 'undefined' && (window as any).__TAURI__) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('clear_token');
      } catch (error) {
        console.error('Failed to clear desktop token:', error);
      }
    }

    // Redirect to home (which will trigger SSO flow or desktop login)
    if (typeof window !== 'undefined') {
      window.location.href = '/';
    }
  },

  initialize: async () => {
    try {
      if (typeof window !== 'undefined') {
        // Check if we're already authenticated (set by SSO callback)
        const currentState = get();
        if (currentState.isAuthenticated && currentState.user) {
          console.log('Already authenticated, skipping initialization');
          set({ isLoading: false });
          return;
        }

        // Check if running in Tauri desktop app
        if ((window as any).__TAURI__) {
          console.log('Detected Tauri environment, using desktop auth flow');
          try {
            const { invoke } = await import('@tauri-apps/api/core');

            // Try to get stored token from OS keychain
            const storedToken = await invoke<string | null>('get_stored_token');

            if (storedToken) {
              // Verify token is still valid
              const isValid = await invoke<boolean>('validate_token', { token: storedToken });

              if (isValid) {
                // Get user info with the stored token
                apiClient.setToken(storedToken);
                const userInfo = await apiClient.getUserInfo(storedToken);

                if (userInfo && userInfo.sub) {
                  const user: User = {
                    id: userInfo.sub,
                    org_id: userInfo.org_id || '',
                    tv_user_id: userInfo.sub,
                    email: userInfo.email || 'user@openchat.local',
                    display_name: userInfo.name || userInfo.email?.split('@')[0] || 'User',
                    status: 'online',
                    created_at: new Date().toISOString(),
                    updated_at: new Date().toISOString(),
                    roles: userInfo.roles || [],
                  };

                  console.log('Desktop token validated, setting auth state');
                  get().setAuth(storedToken, user);
                  return;
                }
              } else {
                // Token invalid, clear it
                await invoke('clear_token');
              }
            }

            // No valid token found, user needs to show login screen
            // This will be handled by the desktop app's routing
            console.log('No valid desktop token found, showing login screen');
            set({ isLoading: false });
            return;
          } catch (error) {
            console.error('Desktop auth error:', error);
            set({ isLoading: false });
            return;
          }
        }

        // Web flow: Check for existing token in localStorage
        const existingToken = apiClient.getToken();
        if (existingToken) {
          try {
            // Verify token is still valid by getting user info
            apiClient.setToken(existingToken);
            const userInfo = await apiClient.getUserInfo(existingToken);

            if (userInfo && userInfo.sub) {
              // Create a minimal user object from userinfo
              const user: User = {
                id: userInfo.sub,
                org_id: userInfo.org_id || '',
                tv_user_id: userInfo.sub,
                email: userInfo.email || 'user@openchat.local',
                display_name: userInfo.name || userInfo.email?.split('@')[0] || 'User',
                status: 'online',
                created_at: new Date().toISOString(),
                updated_at: new Date().toISOString(),
                roles: userInfo.roles || [],
              };

              console.log('Token validated, setting auth state');
              get().setAuth(existingToken, user);
              return;
            }
          } catch (error) {
            console.error('Token validation failed:', error);
            apiClient.clearToken();
          }
        }

        // No valid token, initiate OAuth flow with TitaniumVault
        const tvApiUrl = process.env.NEXT_PUBLIC_TV_API_URL || 'https://api.titanium-vault.com';
        const clientId = process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'openchat-api';
        const redirectUri = `${window.location.origin}/sso/callback/`;

        // Build OAuth authorization URL
        const authUrl = new URL(`${tvApiUrl}/oauth/authorize`);
        authUrl.searchParams.set('response_type', 'code');
        authUrl.searchParams.set('client_id', clientId);
        authUrl.searchParams.set('redirect_uri', redirectUri);
        authUrl.searchParams.set('scope', 'openid email profile');

        // Generate and store state for CSRF protection
        const state = generateCodeVerifier();
        sessionStorage.setItem('oauth_state', state);
        authUrl.searchParams.set('state', state);

        console.log('Redirecting to OAuth authorization:', authUrl.toString());
        window.location.href = authUrl.toString();
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
