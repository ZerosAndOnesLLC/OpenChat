'use client';

import StatusIndicator from './StatusIndicator';
import type { User } from '@/lib/types';

interface UserAvatarProps {
  user: User | { display_name: string; status?: 'online' | 'offline' | 'away' | 'dnd'; avatar_url?: string };
  size?: 'sm' | 'md' | 'lg' | 'xl';
  showStatus?: boolean;
}

const SIZE_CLASSES = {
  sm: 'h-6 w-6 text-xs',
  md: 'h-8 w-8 text-sm',
  lg: 'h-10 w-10 text-base',
  xl: 'h-16 w-16 text-2xl',
};

const STATUS_INDICATOR_SIZES = {
  sm: 'sm' as const,
  md: 'sm' as const,
  lg: 'md' as const,
  xl: 'lg' as const,
};

const STATUS_INDICATOR_POSITIONS = {
  sm: 'bottom-0 right-0',
  md: 'bottom-0 right-0',
  lg: 'bottom-0 right-0',
  xl: 'bottom-1 right-1',
};

export default function UserAvatar({ user, size = 'md', showStatus = true }: UserAvatarProps) {
  const sizeClass = SIZE_CLASSES[size];
  const statusSize = STATUS_INDICATOR_SIZES[size];
  const statusPosition = STATUS_INDICATOR_POSITIONS[size];

  return (
    <div className="relative inline-block">
      <div
        className={`${sizeClass} flex flex-shrink-0 items-center justify-center rounded-full bg-blue-600 font-semibold text-white`}
      >
        {user.avatar_url ? (
          <img
            src={user.avatar_url}
            alt={user.display_name}
            className="h-full w-full rounded-full object-cover"
          />
        ) : (
          user.display_name?.charAt(0).toUpperCase() || '?'
        )}
      </div>
      {showStatus && user.status && (
        <div className={`absolute ${statusPosition}`}>
          <StatusIndicator status={user.status} size={statusSize} />
        </div>
      )}
    </div>
  );
}
