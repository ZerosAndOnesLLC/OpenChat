'use client';

import { useEffect, useState } from 'react';
import { apiClient } from '@/lib/api';
import { useWebSocketStore } from '@/lib/websocket';
import type { Channel } from '@/lib/types';

interface ChannelListProps {
  channels: Channel[];
  activeChannel: Channel | null;
  onSelectChannel: (channel: Channel) => void;
}

function ChannelItem({
  channel,
  isActive,
  onSelect
}: {
  channel: Channel;
  isActive: boolean;
  onSelect: () => void;
}) {
  const [initiallyLoaded, setInitiallyLoaded] = useState(false);

  // Get unread count and initial state status from WebSocket store
  const wsUnreadCount = useWebSocketStore((state) => state.unreadCounts[channel.id]);
  const initialStateLoaded = useWebSocketStore((state) => state.initialStateLoaded);
  const unreadCount = wsUnreadCount ?? 0;

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
          <span className="mr-1.5">
            {channel.channel_type === 'private' ? '🔒' : '#'}
          </span>
          <span className={`truncate ${hasUnread && !isActive ? 'font-bold' : ''}`}>
            {channel.name}
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

export default function ChannelList({
  channels,
  activeChannel,
  onSelectChannel,
}: ChannelListProps) {
  return (
    <div className="space-y-1">
      {channels.map((channel) => (
        <ChannelItem
          key={channel.id}
          channel={channel}
          isActive={activeChannel?.id === channel.id}
          onSelect={() => onSelectChannel(channel)}
        />
      ))}
      {channels.length === 0 && (
        <p className="px-2 py-2 text-xs text-gray-500">No channels yet</p>
      )}
    </div>
  );
}
