import type {
  Channel,
  ChannelMember,
  CreateChannelRequest,
  UpdateChannelRequest,
  AddMemberRequest,
  DirectMessage,
  CreateDmRequest,
  Message,
  SendMessageRequest,
  UpdateMessageRequest,
  Reaction,
  ReactionCount,
  AddReactionRequest,
  User,
  UserStatus,
  UpdateUserRequest,
  UpdateUserStatusRequest,
  ThreadResponse,
  UnreadCountResponse,
  MarkAsReadRequest,
  Attachment,
  AttachmentUploadResponse,
  PinnedMessage,
  Bookmark,
  LinkPreview,
} from './types';

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

class ApiClient {
  private baseUrl: string;
  private token: string | null = null;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  setToken(token: string) {
    this.token = token;
    if (typeof window !== 'undefined') {
      localStorage.setItem('openchat_token', token);
    }
  }

  getToken(): string | null {
    if (!this.token && typeof window !== 'undefined') {
      this.token = localStorage.getItem('openchat_token');
    }
    return this.token;
  }

  clearToken() {
    this.token = null;
    if (typeof window !== 'undefined') {
      localStorage.removeItem('openchat_token');
    }
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const token = this.getToken();
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    if (options.headers) {
      Object.entries(options.headers).forEach(([key, value]) => {
        if (typeof value === 'string') {
          headers[key] = value;
        }
      });
    }

    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      // Handle 401 Unauthorized - clear token and trigger re-authentication
      if (response.status === 401) {
        console.warn('401 Unauthorized - clearing token and redirecting to login');
        this.clearToken();

        // Only redirect if we're not already on the SSO callback page
        if (typeof window !== 'undefined' && !window.location.pathname.includes('/sso/callback')) {
          // Trigger re-authentication by reloading the page
          // The auth.initialize() will detect no valid token and start OAuth flow
          window.location.href = '/';
        }

        throw new Error('Authentication required');
      }

      const error = await response.json().catch(() => ({
        message: `HTTP ${response.status}: ${response.statusText}`,
      }));
      throw new Error(error.message || 'API request failed');
    }

    // Handle empty responses (204 No Content, etc.)
    const contentType = response.headers.get('content-type');
    if (response.status === 204 || !contentType?.includes('application/json')) {
      return undefined as T;
    }

    return response.json();
  }

  // Channel endpoints
  async listChannels(): Promise<Channel[]> {
    return this.request<Channel[]>('/api/channels');
  }

  async createChannel(data: CreateChannelRequest): Promise<Channel> {
    return this.request<Channel>('/api/channels', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getChannel(id: string): Promise<Channel> {
    return this.request<Channel>(`/api/channels/${id}`);
  }

  async updateChannel(id: string, data: UpdateChannelRequest): Promise<Channel> {
    return this.request<Channel>(`/api/channels/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async deleteChannel(id: string): Promise<void> {
    return this.request<void>(`/api/channels/${id}`, {
      method: 'DELETE',
    });
  }

  async listChannelMembers(channelId: string): Promise<ChannelMember[]> {
    return this.request<ChannelMember[]>(`/api/channels/${channelId}/members`);
  }

  async addChannelMember(channelId: string, data: AddMemberRequest): Promise<void> {
    return this.request<void>(`/api/channels/${channelId}/members`, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async removeChannelMember(channelId: string, userId: string): Promise<void> {
    return this.request<void>(`/api/channels/${channelId}/members/${userId}`, {
      method: 'DELETE',
    });
  }

  async listChannelMessages(channelId: string, limit = 50, before?: string): Promise<Message[]> {
    const params = new URLSearchParams({ limit: limit.toString() });
    if (before) params.append('before', before);
    const response = await this.request<{ messages: Message[]; has_more: boolean; next_cursor?: string }>(`/api/channels/${channelId}/messages?${params}`);
    return response.messages;
  }

  async markChannelAsRead(channelId: string, lastMessageId?: string): Promise<void> {
    const body: MarkAsReadRequest = lastMessageId ? { last_message_id: lastMessageId } : {};
    return this.request<void>(`/api/channels/${channelId}/read`, {
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  async getChannelUnreadCount(channelId: string): Promise<number> {
    const response = await this.request<UnreadCountResponse>(`/api/channels/${channelId}/unread`);
    return response.unread_count;
  }

  // Direct Message endpoints
  async listDms(): Promise<DirectMessage[]> {
    return this.request<DirectMessage[]>('/api/dms');
  }

  async createDm(data: CreateDmRequest): Promise<DirectMessage> {
    return this.request<DirectMessage>('/api/dms', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getDm(id: string): Promise<DirectMessage> {
    return this.request<DirectMessage>(`/api/dms/${id}`);
  }

  async listDmMessages(dmId: string, limit = 50, before?: string): Promise<Message[]> {
    const params = new URLSearchParams({ limit: limit.toString() });
    if (before) params.append('before', before);
    const response = await this.request<{ messages: Message[]; has_more: boolean; next_cursor?: string }>(`/api/dms/${dmId}/messages?${params}`);
    return response.messages;
  }

  async markDmAsRead(dmId: string, lastMessageId?: string): Promise<void> {
    const body: MarkAsReadRequest = lastMessageId ? { last_message_id: lastMessageId } : {};
    return this.request<void>(`/api/dms/${dmId}/read`, {
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  async getDmUnreadCount(dmId: string): Promise<number> {
    const response = await this.request<UnreadCountResponse>(`/api/dms/${dmId}/unread`);
    return response.unread_count;
  }

  // Message endpoints
  async sendMessage(data: SendMessageRequest): Promise<Message> {
    return this.request<Message>('/api/messages', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateMessage(id: string, data: UpdateMessageRequest): Promise<Message> {
    return this.request<Message>(`/api/messages/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async deleteMessage(id: string): Promise<void> {
    return this.request<void>(`/api/messages/${id}`, {
      method: 'DELETE',
    });
  }

  async getMessageThread(id: string): Promise<ThreadResponse> {
    return this.request<ThreadResponse>(`/api/messages/${id}/thread`);
  }

  // Reaction endpoints
  async listReactions(messageId: string): Promise<Reaction[]> {
    return this.request<Reaction[]>(`/api/messages/${messageId}/reactions`);
  }

  async getReactionCounts(messageId: string): Promise<ReactionCount[]> {
    return this.request<ReactionCount[]>(`/api/messages/${messageId}/reactions/counts`);
  }

  async addReaction(messageId: string, data: AddReactionRequest): Promise<void> {
    return this.request<void>(`/api/messages/${messageId}/reactions`, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async removeReaction(messageId: string, emoji: string): Promise<void> {
    return this.request<void>(`/api/messages/${messageId}/reactions/${encodeURIComponent(emoji)}`, {
      method: 'DELETE',
    });
  }

  // User endpoints
  async listUsers(): Promise<User[]> {
    return this.request<User[]>('/api/users');
  }

  async getUser(id: string): Promise<User> {
    return this.request<User>(`/api/users/${id}`);
  }

  async updateUser(id: string, data: UpdateUserRequest): Promise<User> {
    return this.request<User>(`/api/users/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async updateUserStatus(id: string, data: UpdateUserStatusRequest): Promise<void> {
    return this.request<void>(`/api/users/${id}/status`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async getUserStatus(userId: string): Promise<UserStatus> {
    return this.request<UserStatus>(`/api/users/${userId}/status`);
  }

  async updateMyStatus(data: UpdateUserStatusRequest): Promise<UserStatus> {
    return this.request<UserStatus>('/api/users/me/status', {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  // Attachment endpoints
  async uploadAttachment(messageId: string, file: File): Promise<AttachmentUploadResponse[]> {
    const token = this.getToken();
    const formData = new FormData();
    formData.append('message_id', messageId);
    formData.append('file', file);

    const response = await fetch(`${this.baseUrl}/api/attachments/upload`, {
      method: 'POST',
      headers: {
        'Authorization': token ? `Bearer ${token}` : '',
      },
      body: formData,
    });

    if (!response.ok) {
      if (response.status === 401) {
        this.clearToken();
        if (typeof window !== 'undefined' && !window.location.pathname.includes('/sso/callback')) {
          window.location.href = '/';
        }
        throw new Error('Authentication required');
      }
      const error = await response.json().catch(() => ({
        message: `HTTP ${response.status}: ${response.statusText}`,
      }));
      throw new Error(error.message || 'Upload failed');
    }

    return response.json();
  }

  async downloadAttachment(attachmentId: string): Promise<Blob> {
    const token = this.getToken();
    const response = await fetch(`${this.baseUrl}/api/attachments/${attachmentId}/download`, {
      method: 'GET',
      headers: {
        'Authorization': token ? `Bearer ${token}` : '',
      },
    });

    if (!response.ok) {
      if (response.status === 401) {
        this.clearToken();
        if (typeof window !== 'undefined' && !window.location.pathname.includes('/sso/callback')) {
          window.location.href = '/';
        }
        throw new Error('Authentication required');
      }
      throw new Error('Download failed');
    }

    return response.blob();
  }

  async deleteAttachment(attachmentId: string): Promise<void> {
    return this.request<void>(`/api/attachments/${attachmentId}`, {
      method: 'DELETE',
    });
  }

  async getMessageAttachments(messageId: string): Promise<Attachment[]> {
    return this.request<Attachment[]>(`/api/messages/${messageId}/attachments`);
  }

  // SSO endpoints
  async exchangeSSOCode(code: string): Promise<{
    access_token: string;
    token_type: string;
    expires_in: number;
    refresh_token?: string;
    id_token?: string;
    user_claims?: {
      sub: string;
      email?: string;
      name?: string;
      org_id?: string;
      org_name?: string;
    };
  }> {
    return this.request<{
      access_token: string;
      token_type: string;
      expires_in: number;
      refresh_token?: string;
      id_token?: string;
      user_claims?: {
        sub: string;
        email?: string;
        name?: string;
        org_id?: string;
        org_name?: string;
      };
    }>('/api/sso/exchange', {
      method: 'POST',
      body: JSON.stringify({ code }),
    });
  }

  async getUserInfo(token: string): Promise<{
    sub: string;
    email: string;
    name?: string;
    org_id?: string;
    roles?: string[];
  }> {
    const response = await fetch(`${this.baseUrl}/api/sso/userinfo`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      throw new Error('Failed to get user info');
    }

    return response.json();
  }

  // Search endpoints
  async searchMessages(query: string, scope?: string, channelId?: string, dmId?: string, limit?: number): Promise<{
    messages: Message[];
    total_count: number;
  }> {
    const params = new URLSearchParams({ q: query });
    if (scope) params.append('scope', scope);
    if (channelId) params.append('channel_id', channelId);
    if (dmId) params.append('dm_id', dmId);
    if (limit) params.append('limit', limit.toString());

    return this.request<{ messages: Message[]; total_count: number }>(`/api/search/messages?${params}`);
  }

  // Notification endpoints
  async listNotifications(limit?: number, offset?: number, unreadOnly?: boolean): Promise<{
    notifications: Array<{
      id: string;
      user_id: string;
      notification_type: string;
      message_id?: string;
      channel_id?: string;
      dm_id?: string;
      read: boolean;
      created_at: string;
    }>;
    total: number;
  }> {
    const params = new URLSearchParams();
    if (limit) params.append('limit', limit.toString());
    if (offset) params.append('offset', offset.toString());
    if (unreadOnly !== undefined) params.append('unread_only', unreadOnly.toString());

    return this.request(`/api/notifications?${params}`);
  }

  async getUnreadNotificationCount(): Promise<{ count: number }> {
    return this.request<{ count: number }>('/api/notifications/unread-count');
  }

  async markNotificationAsRead(notificationId: string): Promise<void> {
    return this.request<void>(`/api/notifications/${notificationId}/read`, {
      method: 'POST',
    });
  }

  async markAllNotificationsAsRead(): Promise<{ success: boolean; count: number }> {
    return this.request<{ success: boolean; count: number }>('/api/notifications/read-all', {
      method: 'POST',
    });
  }

  // Pin endpoints
  async pinMessage(messageId: string): Promise<PinnedMessage> {
    return this.request<PinnedMessage>(`/api/messages/${messageId}/pin`, {
      method: 'POST',
    });
  }

  async unpinMessage(messageId: string): Promise<void> {
    return this.request<void>(`/api/messages/${messageId}/pin`, {
      method: 'DELETE',
    });
  }

  async getChannelPins(channelId: string): Promise<PinnedMessage[]> {
    return this.request<PinnedMessage[]>(`/api/channels/${channelId}/pins`);
  }

  // Bookmark endpoints
  async bookmarkMessage(messageId: string): Promise<Bookmark> {
    return this.request<Bookmark>('/api/bookmarks', {
      method: 'POST',
      body: JSON.stringify({ message_id: messageId }),
    });
  }

  async unbookmarkMessage(messageId: string): Promise<void> {
    return this.request<void>(`/api/bookmarks/${messageId}`, {
      method: 'DELETE',
    });
  }

  async getUserBookmarks(): Promise<Bookmark[]> {
    return this.request<Bookmark[]>('/api/bookmarks');
  }

  // Link Preview endpoints
  async getLinkPreview(url: string): Promise<LinkPreview> {
    const encodedUrl = encodeURIComponent(url);
    return this.request<LinkPreview>(`/api/links/preview?url=${encodedUrl}`);
  }
}

export const apiClient = new ApiClient(API_URL);
