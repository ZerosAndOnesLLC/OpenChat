'use client';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useState, type ReactNode, useEffect } from 'react';
import { ErrorBoundary } from '@/components/ErrorBoundary';
import ToastProvider from '@/components/ToastProvider';
import NotificationManager from '@/components/NotificationManager';

export function Providers({ children }: { children: ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 60 * 1000, // 1 minute
            refetchOnWindowFocus: false,
            retry: (failureCount, error) => {
              // Don't retry on authentication errors
              if (error instanceof Error && error.message === 'Authentication required') {
                return false;
              }
              // Don't retry on 401 errors
              if (error instanceof Error && error.message.includes('401')) {
                return false;
              }
              // Don't retry on rate limit errors (429) - user needs to wait
              if (error instanceof Error && error.message.includes('Rate limit exceeded')) {
                return false;
              }
              // Retry up to 2 times for other errors
              return failureCount < 2;
            },
            retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
          },
          mutations: {
            retry: false, // Don't retry mutations by default
          },
        },
      })
  );

  // Initialize toast manager
  useEffect(() => {
    // Import the toast manager to ensure global function is registered
    import('@/lib/toast');
  }, []);

  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        {children}
        <ToastProvider />
        <NotificationManager />
      </QueryClientProvider>
    </ErrorBoundary>
  );
}
