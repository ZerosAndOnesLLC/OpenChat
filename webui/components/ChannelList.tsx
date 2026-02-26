'use client';

import { useEffect, useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import { useWebSocketStore } from '@/lib/websocket';
import type { Channel } from '@/lib/types';

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
  const [isLeaving, setIsLeaving] = useState(false);
  const queryClient = useQueryClient();
  const removeChannel = useWebSocketStore((state) => state.removeChannel);

  // Get unread count, initial state status, and notification prefs from WebSocket store
  const wsUnreadCount = useWebSocketStore((state) => state.unreadCounts[channel.id]);
  const initialStateLoaded = useWebSocketStore((state) => state.initialStateLoaded);
  const notifPref = useWebSocketStore((state) => state.notificationPrefs[channel.id]);
  const unreadCount = wsUnreadCount ?? 0;

  const preference = notifPref?.preference || 'all';
  const isMuted = preference === 'nothing' || (notifPref?.mute_until ? new Date(notifPref.mute_until) > new Date() : false);

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

  const doLeaveChannel = async () => {
    setIsLeaving(true);
    try {
      await apiClient.leaveChannel(channel.id);
      // Remove from WebSocket store immediately
      removeChannel(channel.id);
      // Invalidate queries
      queryClient.invalidateQueries({ queryKey: ['channels'] });
      queryClient.invalidateQueries({ queryKey: ['public-channels'] });
      onLeave?.();
    } catch (error) {
      console.error('Failed to leave channel:', error);
      alert((error as Error).message || 'Failed to leave channel');
    } finally {
      setIsLeaving(false);
    }
  };

  const handleLeaveClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (channel.channel_type === 'private') {
      setShowConfirm(true);
    } else {
      doLeaveChannel();
    }
  };

  const handleConfirmLeave = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowConfirm(false);
    doLeaveChannel();
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
            disabled={isLeaving}
          >
            {isLeaving ? 'Leaving...' : 'Leave'}
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
      className={`group relative flex items-center rounded transition-colors duration-150 ${
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
            <span className={`mr-1.5 ${isMuted && !isActive ? 'opacity-50' : ''}`}>
              {channel.channel_type === 'private' ? '🔒' : '#'}
            </span>
            <span className={`truncate ${hasUnread && !isActive ? 'font-bold' : ''} ${isMuted && !isActive ? 'text-gray-500' : ''}`}>
              {channel.name}
            </span>
            {isMuted && !isActive && (
              <svg className="ml-1 h-3 w-3 flex-shrink-0 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2" />
              </svg>
            )}
            {preference === 'mentions' && !isMuted && !isActive && (
              <svg className="ml-1 h-3 w-3 flex-shrink-0 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 12a4 4 0 10-8 0 4 4 0 008 0zm0 0v1.5a2.5 2.5 0 005 0V12a9 9 0 10-9 9m4.5-1.206a8.959 8.959 0 01-4.5 1.207" />
              </svg>
            )}
          </div>
          {hasUnread && !isActive && !isHovered && (
            <span className={`ml-2 flex-shrink-0 rounded-full px-2 py-0.5 text-xs font-semibold text-white ${isMuted ? 'bg-gray-600' : 'bg-red-500'}`}>
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
          disabled={isLeaving}
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
