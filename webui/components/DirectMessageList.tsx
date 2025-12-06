'use client';

import { useEffect, useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import { useWebSocketStore } from '@/lib/websocket';
import type { DirectMessage } from '@/lib/types';

interface DirectMessageListProps {
  dms: DirectMessage[];
  activeDm: DirectMessage | null;
  onSelectDm: (dm: DirectMessage) => void;
  onHideDm?: (dmId: string) => void;
}

function DirectMessageItem({
  dm,
  isActive,
  onSelect,
  onHide,
}: {
  dm: DirectMessage;
  isActive: boolean;
  onSelect: () => void;
  onHide?: () => void;
}) {
  const [initiallyLoaded, setInitiallyLoaded] = useState(false);
  const [isHovered, setIsHovered] = useState(false);
  const [isHiding, setIsHiding] = useState(false);
  const queryClient = useQueryClient();
  const removeDm = useWebSocketStore((state) => state.removeDm);

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
    return dm.participants.map((p) => p.display_name || p.email || 'Unknown').join(', ');
  };

  const handleHideClick = async (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsHiding(true);
    try {
      await apiClient.hideDm(dm.id);
      // Remove from WebSocket store immediately
      removeDm(dm.id);
      // Invalidate queries
      queryClient.invalidateQueries({ queryKey: ['dms'] });
      onHide?.();
    } catch (error) {
      console.error('Failed to hide DM:', error);
      alert((error as Error).message || 'Failed to close conversation');
    } finally {
      setIsHiding(false);
    }
  };

  return (
    <div
      className={`group relative flex items-center rounded transition-colors ${
        isActive
          ? 'bg-blue-600 text-white'
          : 'text-gray-300 hover:bg-gray-800'
      }`}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <button
        onClick={onSelect}
        className="flex-1 px-2 py-1.5 text-left text-sm"
      >
        <div className="flex items-center justify-between">
          <div className="flex items-center min-w-0 flex-1">
            <span className="mr-1.5">💬</span>
            <span className={`truncate ${hasUnread && !isActive ? 'font-bold' : ''}`}>
              {getDmName(dm)}
            </span>
          </div>
          {hasUnread && !isActive && !isHovered && (
            <span className="ml-2 flex-shrink-0 rounded-full bg-red-500 px-2 py-0.5 text-xs font-semibold text-white">
              {unreadCount > 99 ? '99+' : unreadCount}
            </span>
          )}
        </div>
      </button>
      {isHovered && (
        <button
          onClick={handleHideClick}
          className={`mr-1 flex-shrink-0 rounded p-1 transition-colors ${
            isActive
              ? 'text-white/70 hover:bg-blue-700 hover:text-white'
              : 'text-gray-500 hover:bg-gray-700 hover:text-red-400'
          }`}
          title="Close conversation"
          disabled={isHiding}
        >
          <svg className="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      )}
    </div>
  );
}

export default function DirectMessageList({
  dms,
  activeDm,
  onSelectDm,
  onHideDm,
}: DirectMessageListProps) {
  return (
    <div className="space-y-1">
      {dms.map((dm) => (
        <DirectMessageItem
          key={dm.id}
          dm={dm}
          isActive={activeDm?.id === dm.id}
          onSelect={() => onSelectDm(dm)}
          onHide={() => onHideDm?.(dm.id)}
        />
      ))}
      {dms.length === 0 && (
        <p className="px-2 py-2 text-xs text-gray-500">No direct messages yet</p>
      )}
    </div>
  );
}
