'use client';

import { useEffect, useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import { useWebSocketStore } from '@/lib/websocket';
import type { Channel } from '@/lib/types';

const useRemoveChannel = () => useWebSocketStore((state) => state.removeChannel);

interface ChannelListProps {
  channels: Channel[];
  activeChannel: Channel | null;
  onSelectChannel: (channel: Channel) => void;
  onLeaveChannel?: (channelId: string) => void;
}

function ChannelItem({
  channel,
  isActive,
  onSelect,
  onLeave,
}: {
  channel: Channel;
  isActive: boolean;
  onSelect: () => void;
  onLeave?: () => void;
}) {
  const [initiallyLoaded, setInitiallyLoaded] = useState(false);
  const [isHovered, setIsHovered] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const queryClient = useQueryClient();
  const removeChannel = useRemoveChannel();

  // Get unread count and initial state status from WebSocket store
  const wsUnreadCount = useWebSocketStore((state) => state.unreadCounts[channel.id]);
  const initialStateLoaded = useWebSocketStore((state) => state.initialStateLoaded);
  const unreadCount = wsUnreadCount ?? 0;

  const leaveMutation = useMutation({
    mutationFn: () => apiClient.leaveChannel(channel.id),
    onSuccess: () => {
      // Remove from WebSocket store so it disappears immediately
      removeChannel(channel.id);
      queryClient.invalidateQueries({ queryKey: ['channels'] });
      onLeave?.();
    },
    onError: (error: Error) => {
      alert(error.message || 'Failed to leave channel');
    },
  });

  // Load initial unread count only if WebSocket initial state hasn't loaded
  useEffect(() => {
    if (!initiallyLoaded && !initialStateLoaded) {
      apiClient.getChannelUnreadCount(channel.id).then((data) => {
        useWebSocketStore.setState((state) => ({
          unreadCounts: {
            ...state.unreadCounts,
            [channel.id]: data.unread_count,
          },
        }));
        setInitiallyLoaded(true);
      }).catch((error) => {
        console.error('Failed to load initial unread count:', error);
      });
    } else if (initialStateLoaded) {
      setInitiallyLoaded(true);
    }
  }, [channel.id, initiallyLoaded, initialStateLoaded]);

  const hasUnread = unreadCount > 0;

  const handleLeaveClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (channel.channel_type === 'private') {
      setShowConfirm(true);
    } else {
      leaveMutation.mutate();
    }
  };

  const handleConfirmLeave = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowConfirm(false);
    leaveMutation.mutate();
  };

  const handleCancelLeave = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowConfirm(false);
  };

  if (showConfirm) {
    return (
      <div className="rounded bg-gray-800 p-2 text-xs">
        <p className="mb-2 text-gray-300">
          Leave this private channel? It will be archived if you're the last member.
        </p>
        <div className="flex gap-2">
          <button
            onClick={handleConfirmLeave}
            className="flex-1 rounded bg-red-600 px-2 py-1 text-white hover:bg-red-700"
            disabled={leaveMutation.isPending}
          >
            {leaveMutation.isPending ? 'Leaving...' : 'Leave'}
          </button>
          <button
            onClick={handleCancelLeave}
            className="flex-1 rounded bg-gray-700 px-2 py-1 text-gray-300 hover:bg-gray-600"
          >
            Cancel
          </button>
        </div>
      </div>
    );
  }

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
            <span className="mr-1.5">
              {channel.channel_type === 'private' ? '🔒' : '#'}
            </span>
            <span className={`truncate ${hasUnread && !isActive ? 'font-bold' : ''}`}>
              {channel.name}
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
          onClick={handleLeaveClick}
          className={`mr-1 flex-shrink-0 rounded p-1 transition-colors ${
            isActive
              ? 'text-white/70 hover:bg-blue-700 hover:text-white'
              : 'text-gray-500 hover:bg-gray-700 hover:text-red-400'
          }`}
          title="Leave channel"
          disabled={leaveMutation.isPending}
        >
          <svg className="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      )}
    </div>
  );
}

export default function ChannelList({
  channels,
  activeChannel,
  onSelectChannel,
  onLeaveChannel,
}: ChannelListProps) {
  return (
    <div className="space-y-1">
      {channels.map((channel) => (
        <ChannelItem
          key={channel.id}
          channel={channel}
          isActive={activeChannel?.id === channel.id}
          onSelect={() => onSelectChannel(channel)}
          onLeave={() => onLeaveChannel?.(channel.id)}
        />
      ))}
      {channels.length === 0 && (
        <p className="px-2 py-2 text-xs text-gray-500">No channels yet</p>
      )}
    </div>
  );
}
