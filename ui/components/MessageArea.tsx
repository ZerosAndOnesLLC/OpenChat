'use client';

import { useEffect, useRef, useMemo, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import { useWebSocketStore } from '@/lib/websocket';
import type { Channel, DirectMessage, Message } from '@/lib/types';
import MessageList from './MessageList';
import MessageInput from './MessageInput';
import TypingIndicator from './TypingIndicator';
import ThreadPanel from './ThreadPanel';
import PinnedMessagesPanel from './PinnedMessagesPanel';
import Toast from './Toast';

interface MessageAreaProps {
  channel: Channel | null;
  dm: DirectMessage | null;
}

export default function MessageArea({ channel, dm }: MessageAreaProps) {
  const { messages, setMessages, subscribeChannel, unsubscribeChannel, typing } =
    useWebSocketStore();
  const prevChannelRef = useRef<string | null>(null);
  const [replyTo, setReplyTo] = useState<Message | undefined>(undefined);
  const [openThread, setOpenThread] = useState<string | null>(null);
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' | 'info' } | null>(null);
  const queryClient = useQueryClient();

  const currentKey = channel?.id || dm?.id || '';

  // Fetch unread count before loading messages
  const { data: unreadCount = 0 } = useQuery({
    queryKey: ['unread-count', currentKey],
    queryFn: async () => {
      if (channel) {
        return await apiClient.getChannelUnreadCount(channel.id);
      } else if (dm) {
        return await apiClient.getDmUnreadCount(dm.id);
      }
      return 0;
    },
    enabled: !!currentKey,
  });

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

  // Set fetched messages to store when they arrive and mark as read
  useEffect(() => {
    console.log('Setting messages to store:', { currentKey, fetchedMessages, isArray: Array.isArray(fetchedMessages) });
    if (currentKey && Array.isArray(fetchedMessages)) {
      // Replace store messages with fetched messages (clears any old WebSocket-only messages)
      setMessages(currentKey, fetchedMessages);
      console.log('Messages set to store for key:', currentKey, 'count:', fetchedMessages.length);

      // Mark as read after a short delay (to give user time to see unread indicator)
      const timer = setTimeout(async () => {
        try {
          const lastMessage = fetchedMessages[fetchedMessages.length - 1];
          if (channel && lastMessage) {
            await apiClient.markChannelAsRead(channel.id, lastMessage.id);
          } else if (dm && lastMessage) {
            await apiClient.markDmAsRead(dm.id, lastMessage.id);
          }
        } catch (error) {
          console.error('Failed to mark as read:', error);
        }
      }, 2000); // Wait 2 seconds before marking as read

      return () => clearTimeout(timer);
    }
  }, [currentKey, fetchedMessages, setMessages, channel, dm]);

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

  // Handle replying to a message
  const handleReply = (message: Message) => {
    setReplyTo(message);
  };

  const handleClearReply = () => {
    setReplyTo(undefined);
  };

  // Handle opening a thread
  const handleOpenThread = (message: Message) => {
    setOpenThread(message.id);
  };

  const handleCloseThread = () => {
    setOpenThread(null);
  };

  // Fetch pinned messages for current channel
  const { data: pinnedMessages = [] } = useQuery({
    queryKey: ['pinned-messages', channel?.id],
    queryFn: () => channel ? apiClient.getChannelPins(channel.id) : Promise.resolve([]),
    enabled: !!channel,
  });

  // Fetch user bookmarks
  const { data: bookmarks = [] } = useQuery({
    queryKey: ['bookmarks'],
    queryFn: () => apiClient.getUserBookmarks(),
  });

  // Create sets for fast lookup
  const pinnedMessageIds = useMemo(() => {
    return new Set(pinnedMessages.map(p => p.message_id));
  }, [pinnedMessages]);

  const bookmarkedMessageIds = useMemo(() => {
    return new Set(bookmarks.map(b => b.message_id));
  }, [bookmarks]);

  // Handle pin/unpin
  const handlePin = async (message: Message) => {
    if (!channel) return;

    try {
      const isPinned = pinnedMessageIds.has(message.id);
      if (isPinned) {
        await apiClient.unpinMessage(message.id);
        setToast({ message: 'Message unpinned', type: 'success' });
      } else {
        await apiClient.pinMessage(message.id);
        setToast({ message: 'Message pinned', type: 'success' });
      }
      // Refresh pinned messages
      queryClient.invalidateQueries({ queryKey: ['pinned-messages', channel.id] });
    } catch (error) {
      console.error('Failed to pin/unpin message:', error);
      setToast({ message: 'Failed to update pin', type: 'error' });
    }
  };

  // Handle bookmark/unbookmark
  const handleBookmark = async (message: Message) => {
    try {
      const isBookmarked = bookmarkedMessageIds.has(message.id);
      if (isBookmarked) {
        await apiClient.unbookmarkMessage(message.id);
        setToast({ message: 'Bookmark removed', type: 'success' });
      } else {
        await apiClient.bookmarkMessage(message.id);
        setToast({ message: 'Message bookmarked', type: 'success' });
      }
      // Refresh bookmarks
      queryClient.invalidateQueries({ queryKey: ['bookmarks'] });
    } catch (error) {
      console.error('Failed to bookmark/unbookmark message:', error);
      setToast({ message: 'Failed to update bookmark', type: 'error' });
    }
  };

  // Handle unpin from panel
  const handleUnpinFromPanel = async (messageId: string) => {
    if (!channel) return;

    try {
      await apiClient.unpinMessage(messageId);
      setToast({ message: 'Message unpinned', type: 'success' });
      queryClient.invalidateQueries({ queryKey: ['pinned-messages', channel.id] });
    } catch (error) {
      console.error('Failed to unpin message:', error);
      setToast({ message: 'Failed to unpin message', type: 'error' });
    }
  };

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
    <>
      <div className="flex flex-1 overflow-hidden">
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

          {/* Pinned messages panel - only show for channels */}
          {channel && <PinnedMessagesPanel channelId={channel.id} onUnpin={handleUnpinFromPanel} />}

          <div className="flex flex-1 flex-col overflow-hidden">
            <MessageList
              messages={localMessages}
              unreadCount={unreadCount}
              onReply={handleReply}
              onOpenThread={handleOpenThread}
              onPin={handlePin}
              onBookmark={handleBookmark}
              pinnedMessageIds={pinnedMessageIds}
              bookmarkedMessageIds={bookmarkedMessageIds}
            />
            {currentTyping.length > 0 && (
              <TypingIndicator users={currentTyping.map((t) => t.userName)} />
            )}
          </div>

          <MessageInput
            channelId={channel?.id}
            dmId={dm?.id}
            replyTo={replyTo}
            onClearReply={handleClearReply}
          />
        </div>

        {/* Thread panel */}
        {openThread && (
          <ThreadPanel
            messageId={openThread}
            onClose={handleCloseThread}
          />
        )}
      </div>

      {/* Toast notification */}
      {toast && (
        <Toast
          message={toast.message}
          type={toast.type}
          onClose={() => setToast(null)}
        />
      )}
    </>
  );
}
