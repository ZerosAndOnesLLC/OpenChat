'use client';

import { Suspense, useEffect, useState, useRef } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { apiClient } from '@/lib/api';
import { useAuthStore } from '@/lib/auth';
import type { User } from '@/lib/types';

function SSOCallbackContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { setAuth } = useAuthStore();
  const [status, setStatus] = useState<'processing' | 'success' | 'error'>('processing');
  const [error, setError] = useState<string | null>(null);
  const hasAttemptedExchange = useRef(false);

  useEffect(() => {
    const handleSSOCallback = async () => {
      // Prevent multiple exchanges of the same authorization code
      if (hasAttemptedExchange.current) {
        console.log('SSO Callback - Already attempted exchange, skipping');
        return;
      }
      hasAttemptedExchange.current = true;

      try {
        // Clear any existing auth token immediately to prevent validation loops
        localStorage.removeItem('openchat_token');

        // Get authorization code from URL query parameter
        console.log('SSO Callback - Full URL:', window.location.href);
        console.log('SSO Callback - Search params:', window.location.search);

        let code = searchParams.get('code');
        let state = searchParams.get('state');
        console.log('SSO Callback - Authorization code from searchParams:', code);

        // Fallback to reading from window.location.search for S3 static hosting
        if (!code && typeof window !== 'undefined') {
          const urlParams = new URLSearchParams(window.location.search);
          code = urlParams.get('code');
          state = urlParams.get('state');
          console.log('SSO Callback - Authorization code from window.location:', code);
        }

        if (!code) {
          console.error('SSO Callback - No authorization code found in URL');
          setError('No authorization code provided');
          setStatus('error');
          return;
        }

        // Verify state parameter for CSRF protection
        const savedState = sessionStorage.getItem('oauth_state');
        if (savedState && state !== savedState) {
          console.error('SSO Callback - State mismatch (CSRF protection)');
          setError('Invalid state parameter - possible CSRF attack');
          setStatus('error');
          return;
        }
        // Clear the saved state
        sessionStorage.removeItem('oauth_state');

        // Exchange authorization code for access token
        console.log('SSO Callback - Exchanging code for access token');
        const tokenData = await apiClient.exchangeSSOCode(code);
        console.log('SSO Callback - Received token data with user_claims');

        // Use user_claims from token exchange response (already includes user info from ID token)
        const userInfo = tokenData.user_claims;
        if (!userInfo) {
          throw new Error('No user claims in token response');
        }
        console.log('SSO Callback - Using user claims:', userInfo);

        // Create user object for OpenChat
        const user: User = {
          id: userInfo.sub || 'unknown',
          org_id: userInfo.org_id || '',
          tv_user_id: userInfo.sub || '',
          email: userInfo.email || 'user@openchat.local',
          display_name: userInfo.name || userInfo.email?.split('@')[0] || 'User',
          status: 'online',
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        };

        console.log('SSO Callback - Setting auth state');
        setAuth(tokenData.access_token, user);

        setStatus('success');

        // Redirect to home after a short delay
        setTimeout(() => {
          console.log('SSO Callback - Redirecting to home');
          router.push('/');
        }, 1000);
      } catch (err) {
        console.error('SSO callback error:', err);
        setError(err instanceof Error ? err.message : 'SSO authentication failed');
        setStatus('error');
      }
    };

    handleSSOCallback();
  }, [searchParams, router, setAuth]);

  return (
    <div className="flex min-h-screen items-center justify-center bg-gray-50">
      <div className="w-full max-w-md space-y-8 p-8">
        <div className="text-center">
          {status === 'processing' && (
            <>
              <div className="mx-auto mb-4 h-16 w-16 animate-spin rounded-full border-b-2 border-blue-600"></div>
              <h2 className="text-2xl font-bold text-gray-900">
                Completing SSO Login...
              </h2>
              <p className="mt-2 text-gray-600">
                Please wait while we log you in.
              </p>
            </>
          )}

          {status === 'success' && (
            <>
              <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-green-100">
                <svg className="h-8 w-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <h2 className="text-2xl font-bold text-gray-900">
                Login Successful!
              </h2>
              <p className="mt-2 text-gray-600">
                Redirecting to OpenChat...
              </p>
            </>
          )}

          {status === 'error' && (
            <>
              <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-red-100">
                <svg className="h-8 w-8 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </div>
              <h2 className="text-2xl font-bold text-gray-900">
                SSO Login Failed
              </h2>
              <p className="mt-2 text-red-600">
                {error || 'An error occurred during SSO authentication'}
              </p>
              <button
                onClick={() => router.push('/')}
                className="mt-4 rounded-md bg-blue-600 px-4 py-2 text-white transition-colors hover:bg-blue-700"
              >
                Try Again
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

export default function SSOCallbackPage() {
  return (
    <Suspense fallback={
      <div className="flex min-h-screen items-center justify-center bg-gray-50">
        <div className="h-16 w-16 animate-spin rounded-full border-b-2 border-blue-600"></div>
      </div>
    }>
      <SSOCallbackContent />
    </Suspense>
  );
}
