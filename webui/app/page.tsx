'use client';

import { useEffect } from 'react';
import { useAuth } from '@/lib/auth';
import ChatLayout from '@/components/ChatLayout';
import LoginScreen from '@/components/desktop/login-screen';

// Check if running in Tauri environment
function isTauriApp(): boolean {
  return typeof window !== 'undefined' && (window as any).__TAURI__ !== undefined;
}

export default function Home() {
  const { isAuthenticated, isLoading, initialize, setAuth } = useAuth();

  useEffect(() => {
    console.log('Home component mounted');
    console.log('Is Tauri?', isTauriApp());
    console.log('Is authenticated?', isAuthenticated);
    console.log('Is loading?', isLoading);
    initialize();
  }, [initialize]);

  if (isLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gray-50">
        <div className="text-center">
          <div className="mb-4 inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-blue-600 border-r-transparent"></div>
          <p className="text-gray-600">Loading OpenChat...</p>
        </div>
      </div>
    );
  }

  if (!isAuthenticated) {
    const isTauri = isTauriApp();
    console.log('Not authenticated, is Tauri?', isTauri);

    // Show desktop login screen if running in Tauri
    if (isTauri) {
      console.log('Showing desktop login screen');
      return (
        <LoginScreen
          onLoginSuccess={async (authResponse) => {
            // Note: Credentials are already stored in OS keychain by verify_pairing_code
            // Just need to set the auth state in the UI
            const user = {
              id: authResponse.user.id,
              org_id: authResponse.user.org_id,
              tv_user_id: authResponse.user.id,
              email: authResponse.user.email,
              display_name: authResponse.user.name || authResponse.user.email.split('@')[0],
              status: 'online' as const,
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
              roles: [],
            };
            setAuth(authResponse.access_token, user);
          }}
        />
      );
    }

    // Web flow - show redirect message (OAuth redirect happens in auth.tsx initialize())
    console.log('Showing web redirect message');
    return (
      <div className="flex min-h-screen items-center justify-center bg-gray-50">
        <div className="text-center">
          <h1 className="text-2xl font-bold text-gray-900 mb-2">Redirecting to login...</h1>
          <p className="text-gray-600">Please wait while we redirect you to the login page.</p>
          <p className="text-xs text-gray-400 mt-4">DEBUG: isTauri={String(isTauri)}</p>
        </div>
      </div>
    );
  }

  return <ChatLayout />;
}
