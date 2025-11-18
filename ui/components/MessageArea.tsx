'use client';

import { useEffect, useRef, useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import { useWebSocketStore } from '@/lib/websocket';
import type { Channel, DirectMessage } from '@/lib/types';
import MessageList from './MessageList';
import MessageInput from './MessageInput';
import TypingIndicator from './TypingIndicator';

interface MessageAreaProps {
  channel: Channel | null;
  dm: DirectMessage | null;
}

export default function MessageArea({ channel, dm }: MessageAreaProps) {
  const { messages, setMessages, subscribeChannel, unsubscribeChannel, typing } =
    useWebSocketStore();
  const prevChannelRef = useRef<string | null>(null);

  const currentKey = channel?.id || dm?.id || '';

  // Fetch messages when channel/dm changes
  const { data: fetchedMessages, isError, error, isLoading } = useQuery({
    queryKey: ['messages', currentKey],
    queryFn: async () => {
      console.log('Fetching messages for:', currentKey, { channel, dm });
      if (channel) {
        const messages = await apiClient.listChannelMessages(channel.id);
        console.log('Fetched channel messages:', messages);
        return messages;
      } else if (dm) {
        const messages = await apiClient.listDmMessages(dm.id);
        console.log('Fetched DM messages:', messages);
        return messages;
      }
      return [];
    },
    enabled: !!currentKey,
  });

  // Log errors
  if (isError) {
    console.error('Error fetching messages:', error);
  }

  // Subscribe/unsubscribe to channel when it changes
  useEffect(() => {
    if (channel && channel.id !== prevChannelRef.current) {
      if (prevChannelRef.current) {
        unsubscribeChannel(prevChannelRef.current);
      }
      subscribeChannel(channel.id);
      prevChannelRef.current = channel.id;
    }

    return () => {
      if (prevChannelRef.current) {
        unsubscribeChannel(prevChannelRef.current);
      }
    };
  }, [channel, subscribeChannel, unsubscribeChannel]);

  // Set fetched messages to store when they arrive
  useEffect(() => {
    console.log('Setting messages to store:', { currentKey, fetchedMessages, isArray: Array.isArray(fetchedMessages) });
    if (currentKey && Array.isArray(fetchedMessages)) {
      // Replace store messages with fetched messages (clears any old WebSocket-only messages)
      setMessages(currentKey, fetchedMessages);
      console.log('Messages set to store for key:', currentKey, 'count:', fetchedMessages.length);
    }
  }, [currentKey, fetchedMessages, setMessages]);

  // Get messages from store (includes both fetched and new WebSocket messages)
  const localMessages = useMemo(() => {
    console.log('Computing localMessages for:', currentKey);
    if (!currentKey) return [];

    const storeMessages = messages[currentKey];
    console.log('Store messages for', currentKey, ':', storeMessages);

    // Ensure storeMessages is an array
    if (!Array.isArray(storeMessages)) {
      console.log('Store messages is not an array, returning empty');
      return [];
    }

    // Sort by created_at
    const sorted = [...storeMessages].sort(
      (a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
    );

    console.log('Computed localMessages count:', sorted.length);
    return sorted;
  }, [currentKey, messages]);

  // Get typing indicators for current channel/dm
  const currentTyping = typing.filter(
    (t) =>
      (channel && t.channelId === channel.id) || (dm && t.dmId === dm.id)
  );

  if (!channel && !dm) {
    return (
      <div className="flex flex-1 items-center justify-center bg-black">
        <div className="text-center">
          <div className="mb-4 text-6xl">💬</div>
          <h2 className="mb-2 text-2xl font-bold text-white">Welcome to OpenChat</h2>
          <p className="text-gray-400">Select a channel or start a direct message to begin</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-1 flex-col bg-black">
      <div className="flex h-14 items-center border-b border-gray-800 px-6">
        <div className="flex items-center">
          <span className="mr-2 text-xl">
            {channel
              ? channel.channel_type === 'private'
                ? '🔒'
                : '#'
              : '💬'}
          </span>
          <h2 className="text-lg font-semibold text-white">
            {channel?.name || (dm && 'Direct Message')}
          </h2>
        </div>
        {channel?.description && (
          <p className="ml-4 text-sm text-gray-400">{channel.description}</p>
        )}
      </div>

      <div className="flex flex-1 flex-col overflow-hidden">
        <MessageList messages={localMessages} />
        {currentTyping.length > 0 && (
          <TypingIndicator users={currentTyping.map((t) => t.userName)} />
        )}
      </div>

      <MessageInput channelId={channel?.id} dmId={dm?.id} />
    </div>
  );
}
