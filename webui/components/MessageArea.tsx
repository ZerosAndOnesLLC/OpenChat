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
import ForwardMessageModal from './ForwardMessageModal';
import Toast from './Toast';

interface MessageAreaProps {
  channel: Channel | null;
  dm: DirectMessage | null;
  onLeaveChannel?: () => void;
}

export default function MessageArea({ channel, dm, onLeaveChannel }: MessageAreaProps) {
  const { user } = useAuth();
  const { messages, channelData, setMessages, subscribeChannel, unsubscribeChannel, subscribeDm, unsubscribeDm, typing, setLastReadMessageId, lastReadMessageIds, unreadCounts, setActiveChannel, markAsRead, pinMessage, unpinMessage, addBookmark, removeBookmark, notificationPrefs, setNotificationPref } =
    useWebSocketStore();
  const prevChannelRef = useRef<string | null>(null);
  const prevDmRef = useRef<string | null>(null);
  const [replyTo, setReplyTo] = useState<Message | undefined>(undefined);
  const [openThread, setOpenThread] = useState<string | null>(null);
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' | 'info' } | null>(null);
  const [showChannelMenu, setShowChannelMenu] = useState(false);
  const [showNotifMenu, setShowNotifMenu] = useState(false);
  const [showMuteSubmenu, setShowMuteSubmenu] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [showAddMembersModal, setShowAddMembersModal] = useState(false);
  const [forwardMessage, setForwardMessage] = useState<Message | null>(null);
  const queryClient = useQueryClient();
  const channelMenuRef = useRef<HTMLDivElement>(null);
  const notifMenuRef = useRef<HTMLDivElement>(null);

  // Check if current user is the channel creator
  const isChannelCreator = channel && user?.id === channel.created_by;

  // Current notification pref for this channel/DM
  const currentKey = channel?.id || dm?.id || '';
  const currentNotifPref = currentKey ? notificationPrefs[currentKey] : undefined;
  const currentPreference = currentNotifPref?.preference || 'all';
  const isMuted = currentPreference === 'nothing' || (currentNotifPref?.mute_until && new Date(currentNotifPref.mute_until) > new Date());

  // Close menus when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (channelMenuRef.current && !channelMenuRef.current.contains(event.target as Node)) {
        setShowChannelMenu(false);
      }
      if (notifMenuRef.current && !notifMenuRef.current.contains(event.target as Node)) {
        setShowNotifMenu(false);
        setShowMuteSubmenu(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Handle setting notification preference
  const handleSetNotifPref = async (preference: 'all' | 'mentions' | 'nothing', muteUntil?: string | null) => {
    const key = channel?.id || dm?.id;
    if (!key) return;

    const data = { preference, mute_until: muteUntil || null };
    try {
      if (channel) {
        await apiClient.setChannelNotificationPref(channel.id, data);
      } else if (dm) {
        await apiClient.setDmNotificationPref(dm.id, data);
      }
      setNotificationPref(key, data);
      setShowNotifMenu(false);
      setShowMuteSubmenu(false);
    } catch {
      setToast({ message: 'Failed to update notification preference', type: 'error' });
    }
  };

  const handleMuteDuration = (hours: number | null) => {
    if (hours === null) {
      // "Until I turn it back on" — no expiry
      handleSetNotifPref('nothing', null);
    } else {
      const muteUntil = new Date(Date.now() + hours * 60 * 60 * 1000).toISOString();
      handleSetNotifPref('nothing', muteUntil);
    }
  };

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
  const { data: fetchedMessages, isError, error } = useQuery({
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
    const timer = setTimeout(() => {
      const lastMessage = localMessages[localMessages.length - 1];
      if (lastMessage) {
        // Use WebSocket to mark as read (more efficient than HTTP)
        markAsRead(channel?.id, dm?.id, lastMessage.id);
      }
    }, 1000); // Wait 1 second before marking as read

    return () => clearTimeout(timer);
  }, [currentKey, localMessages, channel, dm, markAsRead]);

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

  // Handle forwarding a message
  const handleForward = (message: Message) => {
    setForwardMessage(message);
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

  // Handle pin/unpin (via WebSocket)
  const handlePin = (message: Message) => {
    if (!channel) return;

    const isPinned = pinnedMessageIds.has(message.id);
    if (isPinned) {
      unpinMessage(message.id);
      setToast({ message: 'Message unpinned', type: 'success' });
    } else {
      pinMessage(message.id);
      setToast({ message: 'Message pinned', type: 'success' });
    }
    // Note: WebSocket broadcast will update the UI automatically
  };

  // Handle bookmark/unbookmark (via WebSocket)
  const handleBookmark = (message: Message) => {
    const isBookmarked = bookmarkedMessageIds.has(message.id);
    if (isBookmarked) {
      removeBookmark(message.id);
      setToast({ message: 'Bookmark removed', type: 'success' });
    } else {
      addBookmark(message.id);
      setToast({ message: 'Message bookmarked', type: 'success' });
    }
    // Note: WebSocket broadcast will update the UI automatically
    // Still need to invalidate the bookmarks query since it's fetched via HTTP
    queryClient.invalidateQueries({ queryKey: ['bookmarks'] });
  };

  // Handle unpin from panel (via WebSocket)
  const handleUnpinFromPanel = (messageId: string) => {
    if (!channel) return;

    unpinMessage(messageId);
    setToast({ message: 'Message unpinned', type: 'success' });
    // Note: WebSocket broadcast will update the UI automatically
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
            <div className="flex items-center gap-1">
              {/* Notification preference bell icon */}
              {(channel || dm) && (
                <div className="relative" ref={notifMenuRef}>
                  <button
                    onClick={() => { setShowNotifMenu(!showNotifMenu); setShowMuteSubmenu(false); }}
                    className="rounded p-2 text-gray-400 hover:bg-gray-800 hover:text-white"
                    title="Notification preferences"
                  >
                    {isMuted ? (
                      <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2" />
                      </svg>
                    ) : currentPreference === 'mentions' ? (
                      <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                        <circle cx="18" cy="8" r="3" fill="currentColor" />
                      </svg>
                    ) : (
                      <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                      </svg>
                    )}
                  </button>
                  {showNotifMenu && (
                    <div className="absolute right-0 top-full z-50 mt-1 w-52 rounded-md bg-gray-800 py-1 shadow-lg ring-1 ring-black ring-opacity-5">
                      <button
                        onClick={() => handleSetNotifPref('all')}
                        className={`flex w-full items-center px-4 py-2 text-sm hover:bg-gray-700 ${currentPreference === 'all' && !isMuted ? 'text-blue-400' : 'text-gray-300'}`}
                      >
                        <svg className="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                        </svg>
                        All messages
                        {currentPreference === 'all' && !isMuted && <span className="ml-auto">✓</span>}
                      </button>
                      <button
                        onClick={() => handleSetNotifPref('mentions')}
                        className={`flex w-full items-center px-4 py-2 text-sm hover:bg-gray-700 ${currentPreference === 'mentions' ? 'text-blue-400' : 'text-gray-300'}`}
                      >
                        <svg className="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 12a4 4 0 10-8 0 4 4 0 008 0zm0 0v1.5a2.5 2.5 0 005 0V12a9 9 0 10-9 9m4.5-1.206a8.959 8.959 0 01-4.5 1.207" />
                        </svg>
                        Mentions only
                        {currentPreference === 'mentions' && <span className="ml-auto">✓</span>}
                      </button>
                      <div className="my-1 border-t border-gray-700" />
                      <div className="relative">
                        <button
                          onClick={() => setShowMuteSubmenu(!showMuteSubmenu)}
                          className={`flex w-full items-center px-4 py-2 text-sm hover:bg-gray-700 ${isMuted ? 'text-blue-400' : 'text-gray-300'}`}
                        >
                          <svg className="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2" />
                          </svg>
                          {isMuted ? 'Muted' : 'Mute channel'}
                          <svg className="ml-auto h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                          </svg>
                        </button>
                        {showMuteSubmenu && (
                          <div className="absolute left-full top-0 z-50 ml-1 w-48 rounded-md bg-gray-800 py-1 shadow-lg ring-1 ring-black ring-opacity-5">
                            {isMuted && (
                              <>
                                <button
                                  onClick={() => handleSetNotifPref('all')}
                                  className="flex w-full items-center px-4 py-2 text-sm text-green-400 hover:bg-gray-700"
                                >
                                  Unmute
                                </button>
                                <div className="my-1 border-t border-gray-700" />
                              </>
                            )}
                            <button
                              onClick={() => handleMuteDuration(1)}
                              className="flex w-full items-center px-4 py-2 text-sm text-gray-300 hover:bg-gray-700"
                            >
                              For 1 hour
                            </button>
                            <button
                              onClick={() => handleMuteDuration(8)}
                              className="flex w-full items-center px-4 py-2 text-sm text-gray-300 hover:bg-gray-700"
                            >
                              For 8 hours
                            </button>
                            <button
                              onClick={() => handleMuteDuration(24)}
                              className="flex w-full items-center px-4 py-2 text-sm text-gray-300 hover:bg-gray-700"
                            >
                              For 24 hours
                            </button>
                            <button
                              onClick={() => handleMuteDuration(168)}
                              className="flex w-full items-center px-4 py-2 text-sm text-gray-300 hover:bg-gray-700"
                            >
                              For 1 week
                            </button>
                            <button
                              onClick={() => handleMuteDuration(null)}
                              className="flex w-full items-center px-4 py-2 text-sm text-gray-300 hover:bg-gray-700"
                            >
                              Until I turn it back on
                            </button>
                          </div>
                        )}
                      </div>
                    </div>
                  )}
                </div>
              )}
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
              onForward={handleForward}
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

      {/* Forward Message Modal */}
      {forwardMessage && (
        <ForwardMessageModal
          message={forwardMessage}
          isOpen={!!forwardMessage}
          onClose={() => setForwardMessage(null)}
          onSuccess={() => {
            setToast({ message: 'Message forwarded', type: 'success' });
          }}
        />
      )}
    </>
  );
}
