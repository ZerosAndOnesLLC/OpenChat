'use client';

import { useState } from 'react';
import { useAuth } from '@/lib/auth';
import type { User } from '@/lib/types';
import StatusPicker from './StatusPicker';
import UserAvatar from './UserAvatar';

interface UserProfileProps {
  user: User | null;
}

export default function UserProfile({ user }: UserProfileProps) {
  const { logout } = useAuth();
  const [showMenu, setShowMenu] = useState(false);

  if (!user) return null;

  return (
    <div className="border-t border-gray-800 p-4">
      <div className="mb-3 flex items-center gap-3">
        <UserAvatar user={user} size="md" showStatus={true} />
        <div className="flex-1 text-left">
          <p className="text-sm font-medium text-white">{user.display_name}</p>
          <p className="text-xs text-gray-400">{user.email}</p>
        </div>
        <button
          onClick={() => setShowMenu(!showMenu)}
          className="rounded-lg p-2 text-gray-400 hover:bg-gray-800 hover:text-white"
        >
          <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"
            />
          </svg>
        </button>
      </div>

      <StatusPicker
        currentStatus={user.status}
        currentCustomMessage={user.user_status?.custom_message}
        currentEmoji={user.user_status?.emoji}
      />

      {showMenu && (
        <>
          <div
            className="fixed inset-0 z-10"
            onClick={() => setShowMenu(false)}
          />
          <div className="absolute bottom-full left-0 right-0 z-20 mb-2 rounded-lg border border-gray-700 bg-gray-800 shadow-lg">
            <div className="p-2">
              <button
                onClick={logout}
                className="w-full rounded px-3 py-2 text-left text-sm text-red-400 hover:bg-gray-700"
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
