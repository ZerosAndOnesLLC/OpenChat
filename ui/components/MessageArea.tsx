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
  const { data: fetchedMessages, isError, error } = useQuery({
    queryKey: ['messages', currentKey],
    queryFn: async () => {
      if (channel) {
        return apiClient.listChannelMessages(channel.id);
      } else if (dm) {
        return apiClient.listDmMessages(dm.id);
      }
      return [];
    },
    enabled: !!currentKey,
  });

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

  // Compute local messages using useMemo to avoid cascading renders
  const localMessages = useMemo(() => {
    if (!currentKey) return [];

    const wsMessages = messages[currentKey] || [];
    // Safely handle fetchedMessages being undefined
    const fetchedArray = Array.isArray(fetchedMessages) ? fetchedMessages : [];
    const allMessages = [...fetchedArray, ...wsMessages];

    // Deduplicate by ID
    const uniqueMessages = Array.from(
      new Map(allMessages.map((msg) => [msg.id, msg])).values()
    );

    // Sort by created_at
    uniqueMessages.sort(
      (a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
    );

    return uniqueMessages;
  }, [currentKey, fetchedMessages, messages]);

  // Set fetched messages to store
  useEffect(() => {
    if (currentKey && fetchedMessages && fetchedMessages.length > 0) {
      setMessages(currentKey, []);
    }
  }, [currentKey, fetchedMessages, setMessages]);

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
