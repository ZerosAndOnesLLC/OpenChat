import { create } from 'zustand';
import type { Message, WSClientMessage, WSServerMessage, ChannelMetadata, DmMetadata, PinnedMessageInfo, ChannelMemberInfo, MessageWithDetails, NotificationPref } from './types';
import { apiClient } from './api';

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080/api/ws';

interface TypingIndicator {
  userId: string;
  userName: string;
  channelId?: string;
  dmId?: string;
  timestamp: number;
}

interface ChannelDataState {
  messages: Message[];
  pins: PinnedMessageInfo[];
  members: ChannelMemberInfo[];
  loaded: boolean;
}

interface DmDataState {
  messages: Message[];
  loaded: boolean;
}

interface UserStatusInfo {
  status: 'online' | 'offline' | 'away' | 'dnd';
  custom_message?: string;
  emoji?: string;
}

interface WebSocketStore {
  ws: WebSocket | null;
  connected: boolean;
  currentUserId: string | null;
  initialStateLoaded: boolean;
  channels: ChannelMetadata[]; // Channels with metadata from initial state
  dms: DmMetadata[]; // DMs with metadata from initial state
  messages: Record<string, Message[]>; // channelId/dmId -> messages
  channelData: Record<string, ChannelDataState>; // channelId -> channel data (messages, pins, members)
  dmData: Record<string, DmDataState>; // dmId -> dm data (messages)
  typing: TypingIndicator[];
  userStatuses: Record<string, 'online' | 'offline' | 'away' | 'dnd'>;
  userStatusDetails: Record<string, UserStatusInfo>; // Full status info with custom message/emoji
  unreadCounts: Record<string, number>; // channelId/dmId -> unread count
  lastReadMessageIds: Record<string, string | undefined>; // channelId/dmId -> last read message ID
  notificationCount: number;
  notificationPrefs: Record<string, NotificationPref>; // channelId/dmId -> pref
  activeChannelId: string | null; // Currently active/viewed channel
  activeDmId: string | null; // Currently active/viewed DM

  connect: (token: string) => void;
  disconnect: () => void;
  sendMessage: (channelId: string | undefined, dmId: string | undefined, content: string, parentMessageId?: string) => void;
  sendTyping: (channelId: string | undefined, dmId: string | undefined) => void;
  subscribeChannel: (channelId: string) => void;
  unsubscribeChannel: (channelId: string) => void;
  subscribeDm: (dmId: string) => void;
  unsubscribeDm: (dmId: string) => void;
  updateStatus: (status: 'online' | 'offline' | 'away') => void;
  addMessage: (key: string, message: Message) => void;
  updateMessage: (messageId: string, content: string, editedAt: string) => void;
  deleteMessage: (messageId: string) => void;
  addReaction: (messageId: string, userId: string, emoji: string) => void;
  removeReaction: (messageId: string, userId: string, emoji: string) => void;
  setMessages: (key: string, messages: Message[]) => void;
  clearMessages: (key: string) => void;
  setLastReadMessageId: (key: string, messageId: string | undefined) => void;
  setUserStatusDetails: (userId: string, status: 'online' | 'offline' | 'away' | 'dnd', customMessage?: string, emoji?: string) => void;
  setActiveChannel: (channelId: string | null, dmId: string | null) => void;
  addDm: (dm: DmMetadata) => void;
  removeDm: (dmId: string) => void;
  addChannel: (channel: ChannelMetadata) => void;
  removeChannel: (channelId: string) => void;
  updateChannel: (channelId: string, updates: Partial<ChannelMetadata>) => void;
  // New WebSocket-based operations (replacing HTTP calls)
  markAsRead: (channelId: string | undefined, dmId: string | undefined, lastMessageId?: string) => void;
  wsAddReaction: (messageId: string, emoji: string) => void;
  wsRemoveReaction: (messageId: string, emoji: string) => void;
  pinMessage: (messageId: string) => void;
  unpinMessage: (messageId: string) => void;
  addBookmark: (messageId: string) => void;
  removeBookmark: (messageId: string) => void;
  wsEditMessage: (messageId: string, content: string) => void;
  wsDeleteMessage: (messageId: string) => void;
  subscribeThread: (messageId: string) => void;
  unsubscribeThread: (messageId: string) => void;
  setNotificationPref: (key: string, pref: NotificationPref) => void;
}

export const useWebSocketStore = create<WebSocketStore>((set, get) => ({
  ws: null,
  connected: false,
  currentUserId: null,
  initialStateLoaded: false,
  channels: [],
  dms: [],
  messages: {},
  channelData: {},
  dmData: {},
  typing: [],
  userStatuses: {},
  userStatusDetails: {},
  unreadCounts: {},
  lastReadMessageIds: {},
  notificationCount: 0,
  notificationPrefs: {},
  activeChannelId: null,
  activeDmId: null,

  connect: (token: string) => {
    const ws = new WebSocket(`${WS_URL}?token=${token}`);

    ws.onopen = () => {
      console.log('WebSocket connected');
      set({ connected: true });
    };

    ws.onclose = () => {
      console.log('WebSocket disconnected');
      set({ connected: false, ws: null });

      // Auto-reconnect after 3 seconds
      setTimeout(() => {
        const currentWs = get().ws;
        if (!currentWs || currentWs.readyState === WebSocket.CLOSED) {
          get().connect(token);
        }
      }, 3000);
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    ws.onmessage = (event) => {
      try {
        const message: WSServerMessage = JSON.parse(event.data);

        switch (message.type) {
          case 'initial_state': {
            console.log('Received initial state with', message.channels.length, 'channels and', message.dms.length, 'DMs');

            // Initialize unread counts from channels and DMs
            const newUnreadCounts: Record<string, number> = {};
            message.channels.forEach(channel => {
              newUnreadCounts[channel.id] = channel.unread_count;
            });
            message.dms.forEach(dm => {
              newUnreadCounts[dm.id] = dm.unread_count;
            });

            // Parse notification preferences
            const newNotificationPrefs: Record<string, NotificationPref> = {};
            if (message.notification_preferences) {
              for (const pref of message.notification_preferences) {
                const key = pref.channel_id || pref.dm_id;
                if (key) {
                  newNotificationPrefs[key] = {
                    preference: pref.preference as NotificationPref['preference'],
                    mute_until: pref.mute_until || null,
                  };
                }
              }
            }

            set({
              channels: message.channels,
              dms: message.dms,
              unreadCounts: newUnreadCounts,
              notificationPrefs: newNotificationPrefs,
              initialStateLoaded: true,
            });
            break;
          }

          case 'channel_data': {
            console.log('Received channel data for channel', message.channel_id, 'with', message.messages.length, 'messages,', message.pins.length, 'pins,', message.members.length, 'members');

            // Convert MessageWithDetails to Message format
            const messages: Message[] = message.messages.map((msg: MessageWithDetails) => ({
              id: msg.id,
              channel_id: msg.channel_id,
              dm_id: msg.dm_id,
              user_id: msg.user_id,
              content: msg.content,
              parent_message_id: msg.parent_message_id,
              created_at: msg.created_at,
              edited_at: msg.edited_at,
              reply_count: msg.reply_count,
              user: {
                id: msg.user_id,
                display_name: msg.user_name,
                email: '',
                org_id: '',
                tv_user_id: '',
                status: 'online',
                created_at: '',
                updated_at: '',
              },
            }));

            // Update channel data and unread counts
            set((state) => ({
              channelData: {
                ...state.channelData,
                [message.channel_id]: {
                  messages,
                  pins: message.pins,
                  members: message.members,
                  loaded: true,
                },
              },
              messages: {
                ...state.messages,
                [message.channel_id]: messages,
              },
              unreadCounts: {
                ...state.unreadCounts,
                [message.channel_id]: message.unread_info.count,
              },
              lastReadMessageIds: {
                ...state.lastReadMessageIds,
                [message.channel_id]: message.unread_info.last_read_message_id,
              },
            }));
            break;
          }

          case 'dm_data': {
            console.log('Received DM data for dm', message.dm_id, 'with', message.messages.length, 'messages');

            // Convert MessageWithDetails to Message format
            const dmMessages: Message[] = message.messages.map((msg: MessageWithDetails) => ({
              id: msg.id,
              channel_id: msg.channel_id,
              dm_id: msg.dm_id,
              user_id: msg.user_id,
              content: msg.content,
              parent_message_id: msg.parent_message_id,
              created_at: msg.created_at,
              edited_at: msg.edited_at,
              reply_count: msg.reply_count,
              user: {
                id: msg.user_id,
                display_name: msg.user_name,
                email: '',
                org_id: '',
                tv_user_id: '',
                status: 'online',
                created_at: '',
                updated_at: '',
              },
            }));

            // Update DM data and unread counts
            set((state) => ({
              dmData: {
                ...state.dmData,
                [message.dm_id]: {
                  messages: dmMessages,
                  loaded: true,
                },
              },
              messages: {
                ...state.messages,
                [message.dm_id]: dmMessages,
              },
              unreadCounts: {
                ...state.unreadCounts,
                [message.dm_id]: message.unread_info.count,
              },
              lastReadMessageIds: {
                ...state.lastReadMessageIds,
                [message.dm_id]: message.unread_info.last_read_message_id,
              },
            }));
            break;
          }

          case 'new_message': {
            // Message fields come directly on the message object now
            const newMessage: Message = {
              id: message.id,
              channel_id: message.channel_id,
              dm_id: message.dm_id,
              user_id: message.user_id,
              content: message.content,
              parent_message_id: message.parent_message_id,
              created_at: message.created_at,
              user: {
                id: message.user_id,
                display_name: message.user_name,
                email: '',
                org_id: '',
                tv_user_id: '',
                status: 'online',
                created_at: '',
                updated_at: '',
              },
            };
            const key = newMessage.channel_id || newMessage.dm_id || '';
            get().addMessage(key, newMessage);

            // If this message is for the currently active channel/DM, immediately mark it as read
            // This prevents the "unread" indicator from briefly flashing
            const { activeChannelId, activeDmId } = get();
            const isActiveChannel = activeChannelId && message.channel_id === activeChannelId;
            const isActiveDm = activeDmId && message.dm_id === activeDmId;

            if (isActiveChannel || isActiveDm) {
              set((state) => ({
                lastReadMessageIds: {
                  ...state.lastReadMessageIds,
                  [key]: message.id,
                },
                unreadCounts: {
                  ...state.unreadCounts,
                  [key]: 0,
                },
              }));
            }
            break;
          }

          case 'message_edited': {
            if (!message.message_id || !message.content || !message.edited_at) {
              console.error('Received message_edited with missing data:', message);
              break;
            }
            get().updateMessage(message.message_id, message.content, message.edited_at);
            break;
          }

          case 'message_deleted': {
            if (!message.message_id) {
              console.error('Received message_deleted without message_id:', message);
              break;
            }
            get().deleteMessage(message.message_id);
            break;
          }

          case 'user_typing': {
            if (!message.user_id || !message.user_name) {
              console.error('Received user_typing with missing data:', message);
              break;
            }
            const { typing } = get();

            // Remove old typing indicator for this user in this channel/dm
            const filtered = typing.filter(
              (t) =>
                !(
                  t.userId === message.user_id &&
                  ((message.channel_id && t.channelId === message.channel_id) ||
                    (message.dm_id && t.dmId === message.dm_id))
                )
            );

            // Add new typing indicator
            filtered.push({
              userId: message.user_id,
              userName: message.user_name,
              channelId: message.channel_id,
              dmId: message.dm_id,
              timestamp: Date.now(),
            });

            set({ typing: filtered });

            // Remove typing indicator after 3 seconds
            setTimeout(() => {
              set((state) => ({
                typing: state.typing.filter(
                  (t) => !(t.userId === message.user_id && (
                    (message.channel_id && t.channelId === message.channel_id) ||
                    (message.dm_id && t.dmId === message.dm_id)
                  ))
                ),
              }));
            }, 3000);
            break;
          }

          case 'user_status': {
            if (!message.user_id || !message.status) {
              console.error('Received user_status with missing data:', message);
              break;
            }
            set((state) => ({
              userStatuses: {
                ...state.userStatuses,
                [message.user_id]: message.status,
              },
              userStatusDetails: {
                ...state.userStatusDetails,
                [message.user_id]: {
                  status: message.status,
                },
              },
            }));
            break;
          }

          case 'status_update': {
            if (!message.user_id || !message.status) {
              console.error('Received status_update with missing data:', message);
              break;
            }
            set((state) => ({
              userStatuses: {
                ...state.userStatuses,
                [message.user_id]: message.status,
              },
              userStatusDetails: {
                ...state.userStatusDetails,
                [message.user_id]: {
                  status: message.status,
                  custom_message: message.custom_message,
                  emoji: message.emoji,
                },
              },
            }));
            break;
          }

          case 'reaction_added': {
            if (!message.message_id || !message.user_id || !message.emoji) {
              console.error('Received reaction_added with missing data:', message);
              break;
            }
            get().addReaction(message.message_id, message.user_id, message.emoji);
            break;
          }

          case 'reaction_removed': {
            if (!message.message_id || !message.user_id || !message.emoji) {
              console.error('Received reaction_removed with missing data:', message);
              break;
            }
            get().removeReaction(message.message_id, message.user_id, message.emoji);
            break;
          }

          case 'connected': {
            console.log('WebSocket connection confirmed for user:', message.user_id);
            set({ currentUserId: message.user_id });
            break;
          }

          case 'error': {
            console.error('WebSocket error from server:', message.message);
            break;
          }

          case 'pong': {
            // Response to ping, can be used for latency monitoring
            break;
          }

          case 'unread_count_updated': {
            if (message.unread_count === undefined) {
              console.error('Received unread_count_updated without unread_count:', message);
              break;
            }
            const key = message.channel_id || message.dm_id || '';
            set((state) => ({
              unreadCounts: {
                ...state.unreadCounts,
                [key]: message.unread_count,
              },
              lastReadMessageIds: {
                ...state.lastReadMessageIds,
                [key]: message.last_read_message_id,
              },
            }));
            break;
          }

          case 'notification_count_updated': {
            if (message.unread_count === undefined) {
              console.error('Received notification_count_updated without unread_count:', message);
              break;
            }
            set({ notificationCount: message.unread_count });
            break;
          }

          case 'new_notification': {
            // Notification details are received, could be used to show a toast
            console.log('New notification received:', message);
            break;
          }

          case 'message_pinned': {
            console.log('Message pinned:', message.message_id, 'in channel:', message.channel_id);
            // Refresh pins list for this channel
            set((state) => {
              const channelData = state.channelData[message.channel_id];
              if (channelData) {
                return {
                  channelData: {
                    ...state.channelData,
                    [message.channel_id]: {
                      ...channelData,
                      pins: [
                        ...channelData.pins,
                        {
                          id: crypto.randomUUID(),
                          message_id: message.message_id,
                          pinned_by: message.pinned_by,
                          pinned_at: message.pinned_at,
                        },
                      ],
                    },
                  },
                };
              }
              return state;
            });
            break;
          }

          case 'message_unpinned': {
            console.log('Message unpinned:', message.message_id, 'in channel:', message.channel_id);
            // Remove from pins list for this channel
            set((state) => {
              const channelData = state.channelData[message.channel_id];
              if (channelData) {
                return {
                  channelData: {
                    ...state.channelData,
                    [message.channel_id]: {
                      ...channelData,
                      pins: channelData.pins.filter(pin => pin.message_id !== message.message_id),
                    },
                  },
                };
              }
              return state;
            });
            break;
          }

          case 'bookmark_added': {
            console.log('Bookmark added for message:', message.message_id);
            // Note: Bookmarks are user-specific, handled in components via refetch
            break;
          }

          case 'bookmark_removed': {
            console.log('Bookmark removed for message:', message.message_id);
            // Note: Bookmarks are user-specific, handled in components via refetch
            break;
          }

          case 'channel_updated': {
            console.log('Channel updated:', message.channel_id, message.name, message.description);
            // Update channel metadata in channels list
            set((state) => ({
              channels: state.channels.map(ch =>
                ch.id === message.channel_id
                  ? { ...ch, name: message.name || ch.name, description: message.description ?? ch.description }
                  : ch
              ),
            }));
            break;
          }

          case 'member_joined': {
            console.log('Member joined:', message.user_name, 'in channel:', message.channel_id);

            // If the current user is being added to the channel, add it to their sidebar
            const currentUserId = get().currentUserId;
            if (currentUserId && message.user_id === currentUserId) {
              // Check if channel is already in the list
              const existingChannel = get().channels.find(ch => ch.id === message.channel_id);
              if (!existingChannel) {
                // Fetch channel details and add to sidebar
                apiClient.getChannel(message.channel_id).then(channel => {
                  const channelMetadata: ChannelMetadata = {
                    id: channel.id,
                    name: channel.name,
                    description: channel.description,
                    channel_type: channel.channel_type,
                    unread_count: 0,
                  };
                  get().addChannel(channelMetadata);
                  console.log('Added channel to sidebar:', channel.name);
                }).catch(err => {
                  console.error('Failed to fetch channel for sidebar:', err);
                });
              }
            }

            // Add to members list for this channel (for other viewers)
            set((state) => {
              const channelData = state.channelData[message.channel_id];
              if (channelData) {
                return {
                  channelData: {
                    ...state.channelData,
                    [message.channel_id]: {
                      ...channelData,
                      members: [
                        ...channelData.members,
                        {
                          id: crypto.randomUUID(),
                          user_id: message.user_id,
                          user_name: message.user_name,
                          role: message.role,
                          joined_at: message.joined_at,
                        },
                      ],
                    },
                  },
                };
              }
              return state;
            });
            break;
          }

          case 'reminder_triggered': {
            console.log('Reminder triggered:', message.reminder_id);
            // Show toast notification for reminder
            if (typeof window !== 'undefined' && (window as any).showToast) {
              (window as any).showToast(`Reminder: ${message.message_preview}`, 'info');
            }
            break;
          }

          case 'member_left': {
            console.log('Member left:', message.user_name, 'from channel:', message.channel_id);
            // Remove from members list for this channel
            set((state) => {
              const channelData = state.channelData[message.channel_id];
              if (channelData) {
                return {
                  channelData: {
                    ...state.channelData,
                    [message.channel_id]: {
                      ...channelData,
                      members: channelData.members.filter(member => member.user_id !== message.user_id),
                    },
                  },
                };
              }
              return state;
            });
            break;
          }

          default: {
            console.warn('Received unknown WebSocket message type:', message);
            break;
          }
        }
      } catch (error) {
        console.error('Error parsing WebSocket message:', error, 'Raw data:', event.data);
      }
    };

    set({ ws });
  },

  disconnect: () => {
    const { ws } = get();
    if (ws) {
      ws.close();
      set({ ws: null, connected: false });
    }
  },

  sendMessage: (channelId, dmId, content, parentMessageId) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'send_message',
        content,
        ...(channelId ? { channel_id: channelId } : {}),
        ...(dmId ? { dm_id: dmId } : {}),
        ...(parentMessageId ? { parent_message_id: parentMessageId } : {}),
      };
      ws.send(JSON.stringify(message));
    }
  },

  sendTyping: (channelId, dmId) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'typing',
        ...(channelId ? { channel_id: channelId } : {}),
        ...(dmId ? { dm_id: dmId } : {}),
      };
      ws.send(JSON.stringify(message));
    }
  },

  subscribeChannel: (channelId) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'subscribe_channel',
        channel_id: channelId,
      };
      ws.send(JSON.stringify(message));
    }
  },

  unsubscribeChannel: (channelId) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'unsubscribe_channel',
        channel_id: channelId,
      };
      ws.send(JSON.stringify(message));
    }
  },

  subscribeDm: (dmId) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'subscribe_dm',
        dm_id: dmId,
      };
      ws.send(JSON.stringify(message));
    }
  },

  unsubscribeDm: (dmId) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'unsubscribe_dm',
        dm_id: dmId,
      };
      ws.send(JSON.stringify(message));
    }
  },

  updateStatus: (status) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'update_status',
        status,
      };
      ws.send(JSON.stringify(message));
    }
  },

  addMessage: (key, message) => {
    set((state) => ({
      messages: {
        ...state.messages,
        [key]: [...(state.messages[key] || []), message],
      },
    }));
  },

  updateMessage: (messageId, content, editedAt) => {
    set((state) => {
      const newMessages = { ...state.messages };
      Object.keys(newMessages).forEach((key) => {
        newMessages[key] = newMessages[key].map((msg) =>
          msg.id === messageId ? { ...msg, content, edited_at: editedAt } : msg
        );
      });
      return { messages: newMessages };
    });
  },

  deleteMessage: (messageId) => {
    set((state) => {
      const newMessages = { ...state.messages };
      Object.keys(newMessages).forEach((key) => {
        newMessages[key] = newMessages[key].filter((msg) => msg.id !== messageId);
      });
      return { messages: newMessages };
    });
  },

  addReaction: (messageId, userId, emoji) => {
    set((state) => {
      const newMessages = { ...state.messages };
      Object.keys(newMessages).forEach((key) => {
        newMessages[key] = newMessages[key].map((msg) => {
          if (msg.id === messageId) {
            const reactions = msg.reactions || [];
            // Check if reaction already exists
            const exists = reactions.some(
              (r) => r.user_id === userId && r.emoji === emoji
            );
            if (!exists) {
              return {
                ...msg,
                reactions: [
                  ...reactions,
                  {
                    id: `${messageId}-${userId}-${emoji}`,
                    message_id: messageId,
                    user_id: userId,
                    emoji,
                    created_at: new Date().toISOString(),
                  },
                ],
              };
            }
          }
          return msg;
        });
      });
      return { messages: newMessages };
    });
  },

  removeReaction: (messageId, userId, emoji) => {
    set((state) => {
      const newMessages = { ...state.messages };
      Object.keys(newMessages).forEach((key) => {
        newMessages[key] = newMessages[key].map((msg) => {
          if (msg.id === messageId) {
            const reactions = msg.reactions || [];
            return {
              ...msg,
              reactions: reactions.filter(
                (r) => !(r.user_id === userId && r.emoji === emoji)
              ),
            };
          }
          return msg;
        });
      });
      return { messages: newMessages };
    });
  },

  setMessages: (key, messages) => {
    set((state) => ({
      messages: {
        ...state.messages,
        [key]: messages,
      },
    }));
  },

  clearMessages: (key) => {
    set((state) => {
      const newMessages = { ...state.messages };
      delete newMessages[key];
      return { messages: newMessages };
    });
  },

  setLastReadMessageId: (key, messageId) => {
    set((state) => ({
      lastReadMessageIds: {
        ...state.lastReadMessageIds,
        [key]: messageId,
      },
    }));
  },

  setUserStatusDetails: (userId, status, customMessage, emoji) => {
    set((state) => ({
      userStatuses: {
        ...state.userStatuses,
        [userId]: status,
      },
      userStatusDetails: {
        ...state.userStatusDetails,
        [userId]: {
          status,
          custom_message: customMessage,
          emoji,
        },
      },
    }));
  },

  setActiveChannel: (channelId, dmId) => {
    set({ activeChannelId: channelId, activeDmId: dmId });
  },

  addDm: (dm) => {
    set((state) => {
      // Check if DM already exists
      const exists = state.dms.some((d) => d.id === dm.id);
      if (exists) {
        return state;
      }
      return {
        dms: [...state.dms, dm],
        unreadCounts: {
          ...state.unreadCounts,
          [dm.id]: dm.unread_count,
        },
      };
    });
  },

  removeDm: (dmId) => {
    set((state) => ({
      dms: state.dms.filter((dm) => dm.id !== dmId),
    }));
  },

  addChannel: (channel) => {
    set((state) => {
      // Check if channel already exists
      const exists = state.channels.some((c) => c.id === channel.id);
      if (exists) {
        return state;
      }
      return {
        channels: [...state.channels, channel],
        unreadCounts: {
          ...state.unreadCounts,
          [channel.id]: channel.unread_count,
        },
      };
    });
  },

  removeChannel: (channelId) => {
    set((state) => ({
      channels: state.channels.filter((ch) => ch.id !== channelId),
    }));
  },

  updateChannel: (channelId, updates) => {
    set((state) => ({
      channels: state.channels.map((ch) =>
        ch.id === channelId ? { ...ch, ...updates } : ch
      ),
    }));
  },

  // New WebSocket-based operations (replacing HTTP calls)
  markAsRead: (channelId, dmId, lastMessageId) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'mark_as_read',
        ...(channelId ? { channel_id: channelId } : {}),
        ...(dmId ? { dm_id: dmId } : {}),
        ...(lastMessageId ? { last_message_id: lastMessageId } : {}),
      };
      ws.send(JSON.stringify(message));
    }
  },

  wsAddReaction: (messageId, emoji) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'add_reaction',
        message_id: messageId,
        emoji,
      };
      ws.send(JSON.stringify(message));
    }
  },

  wsRemoveReaction: (messageId, emoji) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'remove_reaction',
        message_id: messageId,
        emoji,
      };
      ws.send(JSON.stringify(message));
    }
  },

  pinMessage: (messageId) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'pin_message',
        message_id: messageId,
      };
      ws.send(JSON.stringify(message));
    }
  },

  unpinMessage: (messageId) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'unpin_message',
        message_id: messageId,
      };
      ws.send(JSON.stringify(message));
    }
  },

  addBookmark: (messageId) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'add_bookmark',
        message_id: messageId,
      };
      ws.send(JSON.stringify(message));
    }
  },

  removeBookmark: (messageId) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'remove_bookmark',
        message_id: messageId,
      };
      ws.send(JSON.stringify(message));
    }
  },

  wsEditMessage: (messageId, content) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'edit_message',
        message_id: messageId,
        content,
      };
      ws.send(JSON.stringify(message));
    }
  },

  wsDeleteMessage: (messageId) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'delete_message',
        message_id: messageId,
      };
      ws.send(JSON.stringify(message));
    }
  },

  subscribeThread: (messageId) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'subscribe_thread',
        message_id: messageId,
      };
      ws.send(JSON.stringify(message));
    }
  },

  unsubscribeThread: (messageId) => {
    const { ws } = get();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const message: WSClientMessage = {
        type: 'unsubscribe_thread',
        message_id: messageId,
      };
      ws.send(JSON.stringify(message));
    }
  },

  setNotificationPref: (key, pref) => {
    set((state) => ({
      notificationPrefs: {
        ...state.notificationPrefs,
        [key]: pref,
      },
    }));
  },
}));
