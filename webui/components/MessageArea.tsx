'use client';

import { useEffect, useRef, useMemo, useState } from 'react';
import { useQuery, useQueryClient, useMutation } from '@tanstack/react-query';
import { apiClient } from '@/lib/api';
import { useAuth } from '@/lib/auth';
import { useWebSocketStore } from '@/lib/websocket';
import type { Channel, DirectMessage, Message } from '@/lib/types';
import MessageList from './MessageList';
import MessageInput from './MessageInput';
import TypingIndicator from './TypingIndicator';
import ThreadPanel from './ThreadPanel';
import PinnedMessagesPanel from './PinnedMessagesPanel';
import EditChannelModal from './EditChannelModal';
import AddMembersModal from './AddMembersModal';
import Toast from './Toast';

interface MessageAreaProps {
  channel: Channel | null;
  dm: DirectMessage | null;
  onLeaveChannel?: () => void;
}

export default function MessageArea({ channel, dm, onLeaveChannel }: MessageAreaProps) {
  const { user } = useAuth();
  const { messages, channelData, dmData, setMessages, subscribeChannel, unsubscribeChannel, subscribeDm, unsubscribeDm, typing, setLastReadMessageId, lastReadMessageIds, unreadCounts, setActiveChannel } =
    useWebSocketStore();
  const prevChannelRef = useRef<string | null>(null);
  const prevDmRef = useRef<string | null>(null);
  const [replyTo, setReplyTo] = useState<Message | undefined>(undefined);
  const [openThread, setOpenThread] = useState<string | null>(null);
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' | 'info' } | null>(null);
  const [showChannelMenu, setShowChannelMenu] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [showAddMembersModal, setShowAddMembersModal] = useState(false);
  const queryClient = useQueryClient();
  const channelMenuRef = useRef<HTMLDivElement>(null);

  // Check if current user is the channel creator
  const isChannelCreator = channel && user?.id === channel.created_by;

  // Close menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (channelMenuRef.current && !channelMenuRef.current.contains(event.target as Node)) {
        setShowChannelMenu(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Leave channel mutation
  const leaveChannelMutation = useMutation({
    mutationFn: (channelId: string) => apiClient.leaveChannel(channelId),
    onSuccess: () => {
      setToast({ message: 'You have left the channel', type: 'success' });
      queryClient.invalidateQueries({ queryKey: ['channels'] });
      onLeaveChannel?.();
    },
    onError: (error: Error) => {
      setToast({ message: error.message || 'Failed to leave channel', type: 'error' });
    },
  });

  const currentKey = channel?.id || dm?.id || '';

  // Use WebSocket data for both channels and DMs
  const useWebSocketData = !!(channel || dm);

  // Fetch unread count and last read message ID (only for DMs or as fallback)
  const { data: unreadData } = useQuery({
    queryKey: ['unread-count', currentKey],
    queryFn: async () => {
      if (channel) {
        return await apiClient.getChannelUnreadCount(channel.id);
      } else if (dm) {
        return await apiClient.getDmUnreadCount(dm.id);
      }
      return { unread_count: 0, last_read_message_id: undefined } as const;
    },
    enabled: !!currentKey && !useWebSocketData, // Only fetch for DMs
  });

  // Get unread count from WebSocket store for channels, or from query for DMs
  const unreadCount = useWebSocketData
    ? (unreadCounts[currentKey] ?? 0)
    : (unreadData?.unread_count ?? 0);

  // Store last read message ID when it's fetched (DMs only)
  useEffect(() => {
    if (currentKey && !useWebSocketData && unreadData?.last_read_message_id !== undefined) {
      setLastReadMessageId(currentKey, unreadData.last_read_message_id);
    }
  }, [currentKey, unreadData, setLastReadMessageId, useWebSocketData]);

  // Fetch messages when channel/dm changes (only for DMs now)
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
    enabled: !!currentKey && !useWebSocketData, // Only fetch for DMs
  });

  // Log errors
  if (isError) {
    console.error('Error fetching messages:', error);
  }

  // Track active channel/DM for immediate read status updates
  useEffect(() => {
    setActiveChannel(channel?.id || null, dm?.id || null);

    return () => {
      // Clear active channel on unmount
      setActiveChannel(null, null);
    };
  }, [channel?.id, dm?.id, setActiveChannel]);

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

  // Subscribe/unsubscribe to DM when it changes
  useEffect(() => {
    if (dm && dm.id !== prevDmRef.current) {
      if (prevDmRef.current) {
        unsubscribeDm(prevDmRef.current);
      }
      subscribeDm(dm.id);
      prevDmRef.current = dm.id;
    }

    return () => {
      if (prevDmRef.current) {
        unsubscribeDm(prevDmRef.current);
      }
    };
  }, [dm, subscribeDm, unsubscribeDm]);

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

  // Auto-mark messages as read when viewing the channel
  useEffect(() => {
    if (!currentKey || localMessages.length === 0) return;

    // Mark as read after a short delay when messages change
    const timer = setTimeout(async () => {
      try {
        const lastMessage = localMessages[localMessages.length - 1];
        if (channel && lastMessage) {
          await apiClient.markChannelAsRead(channel.id, lastMessage.id);
        } else if (dm && lastMessage) {
          await apiClient.markDmAsRead(dm.id, lastMessage.id);
        }
      } catch (error) {
        console.error('Failed to mark as read:', error);
      }
    }, 1000); // Wait 1 second before marking as read

    return () => clearTimeout(timer);
  }, [currentKey, localMessages, channel, dm]);

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

  // Get pinned messages from WebSocket data for channels, or fetch via HTTP for DMs
  const pinnedMessagesFromWs = channel && channelData[channel.id]?.pins ? channelData[channel.id].pins : [];

  const { data: pinnedMessagesFromHttp = [] } = useQuery({
    queryKey: ['pinned-messages', channel?.id],
    queryFn: () => channel ? apiClient.getChannelPins(channel.id) : Promise.resolve([]),
    enabled: !!channel && !useWebSocketData, // Only fetch if not using WebSocket data
  });

  const pinnedMessages = useWebSocketData ? pinnedMessagesFromWs : pinnedMessagesFromHttp;

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
          <div className="flex h-14 items-center justify-between border-b border-gray-800 px-6">
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
              {channel?.description && (
                <p className="ml-4 text-sm text-gray-400">{channel.description}</p>
              )}
            </div>
            {channel && (
              <div className="relative" ref={channelMenuRef}>
                <button
                  onClick={() => setShowChannelMenu(!showChannelMenu)}
                  className="rounded p-2 text-gray-400 hover:bg-gray-800 hover:text-white"
                  title="Channel options"
                >
                  <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
                  </svg>
                </button>
                {showChannelMenu && (
                  <div className="absolute right-0 top-full z-50 mt-1 w-48 rounded-md bg-gray-800 py-1 shadow-lg ring-1 ring-black ring-opacity-5">
                    {isChannelCreator && (
                      <button
                        onClick={() => {
                          setShowEditModal(true);
                          setShowChannelMenu(false);
                        }}
                        className="flex w-full items-center px-4 py-2 text-sm text-gray-300 hover:bg-gray-700"
                      >
                        <svg className="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                        </svg>
                        Edit channel
                      </button>
                    )}
                    {channel.channel_type === 'private' && (
                      <button
                        onClick={() => {
                          setShowAddMembersModal(true);
                          setShowChannelMenu(false);
                        }}
                        className="flex w-full items-center px-4 py-2 text-sm text-gray-300 hover:bg-gray-700"
                      >
                        <svg className="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z" />
                        </svg>
                        Add members
                      </button>
                    )}
                    <button
                      onClick={() => {
                        const message = channel.channel_type === 'private'
                          ? 'Are you sure you want to leave this private channel? It may be archived if you are the last member.'
                          : 'Are you sure you want to leave this channel?';
                        if (confirm(message)) {
                          leaveChannelMutation.mutate(channel.id);
                        }
                        setShowChannelMenu(false);
                      }}
                      className="flex w-full items-center px-4 py-2 text-sm text-red-400 hover:bg-gray-700"
                    >
                      <svg className="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                      </svg>
                      Leave channel
                    </button>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Pinned messages panel - only show for channels */}
          {channel && <PinnedMessagesPanel channelId={channel.id} onUnpin={handleUnpinFromPanel} />}

          <div className="flex flex-1 flex-col overflow-hidden">
            <MessageList
              key={currentKey}
              messages={localMessages}
              unreadCount={unreadCount}
              lastReadMessageId={lastReadMessageIds[currentKey]}
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

      {/* Edit Channel Modal */}
      {channel && (
        <EditChannelModal
          channel={channel}
          isOpen={showEditModal}
          onClose={() => setShowEditModal(false)}
          onSuccess={() => {
            setToast({ message: 'Channel updated successfully', type: 'success' });
          }}
        />
      )}

      {/* Add Members Modal */}
      {channel && channel.channel_type === 'private' && (
        <AddMembersModal
          channel={channel}
          isOpen={showAddMembersModal}
          onClose={() => setShowAddMembersModal(false)}
          onSuccess={() => {
            setToast({ message: 'Member added successfully', type: 'success' });
          }}
        />
      )}
    </>
  );
}
