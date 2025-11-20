import { create } from 'zustand';
import type { Message, WSClientMessage, WSServerMessage, ChannelMetadata, DmMetadata } from './types';

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080/api/ws';

interface TypingIndicator {
  userId: string;
  userName: string;
  channelId?: string;
  dmId?: string;
  timestamp: number;
}

interface WebSocketStore {
  ws: WebSocket | null;
  connected: boolean;
  initialStateLoaded: boolean;
  channels: ChannelMetadata[]; // Channels with metadata from initial state
  dms: DmMetadata[]; // DMs with metadata from initial state
  messages: Record<string, Message[]>; // channelId/dmId -> messages
  typing: TypingIndicator[];
  userStatuses: Record<string, 'online' | 'offline' | 'away'>;
  unreadCounts: Record<string, number>; // channelId/dmId -> unread count
  lastReadMessageIds: Record<string, string | undefined>; // channelId/dmId -> last read message ID
  notificationCount: number;

  connect: (token: string) => void;
  disconnect: () => void;
  sendMessage: (channelId: string | undefined, dmId: string | undefined, content: string, parentMessageId?: string) => void;
  sendTyping: (channelId: string | undefined, dmId: string | undefined) => void;
  subscribeChannel: (channelId: string) => void;
  unsubscribeChannel: (channelId: string) => void;
  updateStatus: (status: 'online' | 'offline' | 'away') => void;
  addMessage: (key: string, message: Message) => void;
  updateMessage: (messageId: string, content: string, editedAt: string) => void;
  deleteMessage: (messageId: string) => void;
  addReaction: (messageId: string, userId: string, emoji: string) => void;
  removeReaction: (messageId: string, userId: string, emoji: string) => void;
  setMessages: (key: string, messages: Message[]) => void;
  clearMessages: (key: string) => void;
  setLastReadMessageId: (key: string, messageId: string | undefined) => void;
}

export const useWebSocketStore = create<WebSocketStore>((set, get) => ({
  ws: null,
  connected: false,
  initialStateLoaded: false,
  channels: [],
  dms: [],
  messages: {},
  typing: [],
  userStatuses: {},
  unreadCounts: {},
  lastReadMessageIds: {},
  notificationCount: 0,

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

            set({
              channels: message.channels,
              dms: message.dms,
              unreadCounts: newUnreadCounts,
              initialStateLoaded: true,
            });
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
}));
