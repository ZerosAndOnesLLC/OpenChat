'use client';

import { useEffect, useState } from 'react';
import { apiClient } from '@/lib/api';
import { useWebSocketStore } from '@/lib/websocket';
import type { DirectMessage } from '@/lib/types';

interface DirectMessageListProps {
  dms: DirectMessage[];
  activeDm: DirectMessage | null;
  onSelectDm: (dm: DirectMessage) => void;
}

function DirectMessageItem({
  dm,
  isActive,
  onSelect
}: {
  dm: DirectMessage;
  isActive: boolean;
  onSelect: () => void;
}) {
  const [initiallyLoaded, setInitiallyLoaded] = useState(false);

  // Get unread count and initial state status from WebSocket store
  const wsUnreadCount = useWebSocketStore((state) => state.unreadCounts[dm.id]);
  const initialStateLoaded = useWebSocketStore((state) => state.initialStateLoaded);
  const unreadCount = wsUnreadCount ?? 0;

  // Load initial unread count only if WebSocket initial state hasn't loaded
  useEffect(() => {
    if (!initiallyLoaded && !initialStateLoaded) {
      apiClient.getDmUnreadCount(dm.id).then((data) => {
        useWebSocketStore.setState((state) => ({
          unreadCounts: {
            ...state.unreadCounts,
            [dm.id]: data.unread_count,
          },
        }));
        setInitiallyLoaded(true);
      }).catch((error) => {
        console.error('Failed to load initial DM unread count:', error);
      });
    } else if (initialStateLoaded) {
      setInitiallyLoaded(true);
    }
  }, [dm.id, initiallyLoaded, initialStateLoaded]);

  const hasUnread = unreadCount > 0;

  const getDmName = (dm: DirectMessage) => {
    if (!dm.participants || dm.participants.length === 0) {
      return 'Unknown';
    }
    return dm.participants.map((p) => p.display_name).join(', ');
  };

  return (
    <button
      onClick={onSelect}
      className={`w-full rounded px-2 py-1.5 text-left text-sm transition-colors ${
        isActive
          ? 'bg-blue-600 text-white'
          : 'text-gray-300 hover:bg-gray-800'
      }`}
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center min-w-0 flex-1">
          <span className="mr-1.5">💬</span>
          <span className={`truncate ${hasUnread && !isActive ? 'font-bold' : ''}`}>
            {getDmName(dm)}
          </span>
        </div>
        {hasUnread && !isActive && (
          <span className="ml-2 flex-shrink-0 rounded-full bg-red-500 px-2 py-0.5 text-xs font-semibold text-white">
            {unreadCount > 99 ? '99+' : unreadCount}
          </span>
        )}
      </div>
    </button>
  );
}

export default function DirectMessageList({
  dms,
  activeDm,
  onSelectDm,
}: DirectMessageListProps) {
  return (
    <div className="space-y-1">
      {dms.map((dm) => (
        <DirectMessageItem
          key={dm.id}
          dm={dm}
          isActive={activeDm?.id === dm.id}
          onSelect={() => onSelectDm(dm)}
        />
      ))}
      {dms.length === 0 && (
        <p className="px-2 py-2 text-xs text-gray-500">No direct messages yet</p>
      )}
    </div>
  );
}
