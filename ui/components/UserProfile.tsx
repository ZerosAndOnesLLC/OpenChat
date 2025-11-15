'use client';

import { useState } from 'react';
import { useAuth } from '@/lib/auth';
import { useWebSocketStore } from '@/lib/websocket';
import type { User } from '@/lib/types';

interface UserProfileProps {
  user: User | null;
}

export default function UserProfile({ user }: UserProfileProps) {
  const { logout } = useAuth();
  const { updateStatus } = useWebSocketStore();
  const [showMenu, setShowMenu] = useState(false);

  if (!user) return null;

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online':
        return 'bg-green-500';
      case 'away':
        return 'bg-yellow-500';
      case 'offline':
        return 'bg-gray-500';
      default:
        return 'bg-gray-500';
    }
  };

  const handleStatusChange = (status: 'online' | 'offline' | 'away') => {
    updateStatus(status);
    setShowMenu(false);
  };

  return (
    <div className="relative">
      <button
        onClick={() => setShowMenu(!showMenu)}
        className="flex w-full items-center gap-3 px-4 py-3 hover:bg-gray-800 transition-colors"
      >
        <div className="relative">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-blue-600 text-sm font-semibold">
            {user.display_name.charAt(0).toUpperCase()}
          </div>
          <div
            className={`absolute bottom-0 right-0 h-3 w-3 rounded-full border-2 border-gray-900 ${getStatusColor(
              user.status
            )}`}
          />
        </div>
        <div className="flex-1 text-left">
          <p className="text-sm font-medium">{user.display_name}</p>
          <p className="text-xs text-gray-400 capitalize">{user.status}</p>
        </div>
        <svg
          className="h-4 w-4 text-gray-400"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </button>

      {showMenu && (
        <>
          <div
            className="fixed inset-0 z-10"
            onClick={() => setShowMenu(false)}
          />
          <div className="absolute bottom-full left-0 right-0 z-20 mb-2 rounded-lg bg-gray-800 shadow-lg">
            <div className="p-2">
              <div className="mb-2 border-b border-gray-700 pb-2">
                <p className="px-2 py-1 text-xs font-semibold text-gray-400">
                  Set Status
                </p>
                <button
                  onClick={() => handleStatusChange('online')}
                  className="flex w-full items-center gap-2 rounded px-2 py-1.5 text-sm hover:bg-gray-700"
                >
                  <div className="h-2 w-2 rounded-full bg-green-500" />
                  Online
                </button>
                <button
                  onClick={() => handleStatusChange('away')}
                  className="flex w-full items-center gap-2 rounded px-2 py-1.5 text-sm hover:bg-gray-700"
                >
                  <div className="h-2 w-2 rounded-full bg-yellow-500" />
                  Away
                </button>
                <button
                  onClick={() => handleStatusChange('offline')}
                  className="flex w-full items-center gap-2 rounded px-2 py-1.5 text-sm hover:bg-gray-700"
                >
                  <div className="h-2 w-2 rounded-full bg-gray-500" />
                  Offline
                </button>
              </div>
              <button
                onClick={logout}
                className="w-full rounded px-2 py-1.5 text-left text-sm text-red-400 hover:bg-gray-700"
              >
                Logout
              </button>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
