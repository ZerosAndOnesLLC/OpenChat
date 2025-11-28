'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/lib/auth';
import type { User } from '@/lib/types';
import StatusPicker from './StatusPicker';
import UserAvatar from './UserAvatar';

interface UserProfileProps {
  user: User | null;
}

export default function UserProfile({ user }: UserProfileProps) {
  const { logout } = useAuth();
  const router = useRouter();
  const [showMenu, setShowMenu] = useState(false);

  if (!user) return null;

  const isAdmin = user.roles?.includes('openchat-admin') || false;

  return (
    <div className="relative border-t border-gray-800 p-4">
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
        userId={user.id}
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
                onClick={() => {
                  router.push('/settings/');
                  setShowMenu(false);
                }}
                className="w-full rounded px-3 py-2 text-left text-sm text-gray-300 hover:bg-gray-700 flex items-center gap-2"
              >
                <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                </svg>
                Desktop App
              </button>
              <button
                onClick={() => {
                  router.push('/settings/');
                  setShowMenu(false);
                }}
                className="w-full rounded px-3 py-2 text-left text-sm text-gray-300 hover:bg-gray-700 flex items-center gap-2"
              >
                <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
                Settings
              </button>
              {isAdmin && (
                <>
                  <div className="my-1 border-t border-gray-700" />
                  <div className="px-2 py-1 text-xs font-semibold text-gray-500 uppercase">
                    Admin
                  </div>
                  <button
                    onClick={() => {
                      router.push('/admin/storage/');
                      setShowMenu(false);
                    }}
                    className="w-full rounded px-3 py-2 text-left text-sm text-gray-300 hover:bg-gray-700"
                  >
                    Storage Settings
                  </button>
                  <button
                    onClick={() => {
                      router.push('/admin/audit-logs/');
                      setShowMenu(false);
                    }}
                    className="w-full rounded px-3 py-2 text-left text-sm text-gray-300 hover:bg-gray-700"
                  >
                    Audit Logs
                  </button>
                  <button
                    onClick={() => {
                      router.push('/admin/retention/');
                      setShowMenu(false);
                    }}
                    className="w-full rounded px-3 py-2 text-left text-sm text-gray-300 hover:bg-gray-700"
                  >
                    Retention Policies
                  </button>
                </>
              )}
              <div className="my-1 border-t border-gray-700" />
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
