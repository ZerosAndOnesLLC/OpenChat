// User types
export interface User {
  id: string;
  org_id: string;
  tv_user_id: string;
  email: string;
  display_name: string;
  avatar_url?: string;
  status: 'online' | 'offline' | 'away' | 'dnd';
  created_at: string;
  updated_at: string;
  user_status?: UserStatus;
  disable_read_receipts?: boolean;
  roles?: string[];
}

export interface UserStatus {
  user_id: string;
  status: 'online' | 'offline' | 'away' | 'dnd';
  custom_message?: string;
  emoji?: string;
  clear_at?: string;
  updated_at: string;
}

// Channel types
export interface Channel {
  id: string;
  org_id: string;
  name: string;
  description?: string;
  channel_type: 'public' | 'private';
  created_by: string;
  created_at: string;
  updated_at: string;
}

export interface ChannelMember {
  id: string;
  channel_id: string;
  user_id: string;
  role: 'admin' | 'member';
  joined_at: string;
  user?: User;
}

// Message types
export interface Message {
  id: string;
  channel_id?: string;
  dm_id?: string;
  user_id: string;
  content: string;
  parent_message_id?: string;
  created_at: string;
  edited_at?: string;
  deleted_at?: string;
  user?: User;
  reactions?: Reaction[];
  reply_count?: number;
  first_reply?: Message;
  attachments?: Attachment[];
}

// Attachment types
export interface Attachment {
  id: string;
  message_id: string;
  file_name: string;
  file_url: string;
  file_type?: string;
  file_size?: number;
  storage_type: string;
  storage_path: string;
  created_at: string;
}

export interface AttachmentUploadResponse {
  id: string;
  file_name: string;
  file_url: string;
  file_type?: string;
  file_size: number;
  storage_type: string;
}

// Custom Emoji types
export interface CustomEmoji {
  id: string;
  org_id: string;
  name: string;
  image_url?: string;
  storage_type: string;
  storage_path: string;
  created_by: string;
  created_at: string;
}

export interface EmojiUploadResponse {
  id: string;
  name: string;
  image_url: string;
  storage_type: string;
  created_at: string;
}

// Reaction types
export interface Reaction {
  id: string;
  message_id: string;
  user_id: string;
  emoji: string;
  created_at: string;
}

export interface ReactionCount {
  emoji: string;
  count: number;
  user_ids: string[];
}

// Direct Message types
export interface DirectMessage {
  id: string;
  org_id: string;
  created_by: string;
  created_at: string;
  participants?: User[];
}

export interface DmParticipant {
  id: string;
  dm_id: string;
  user_id: string;
  joined_at: string;
}

// WebSocket Initial State types
export interface ChannelMetadata {
  id: string;
  name: string;
  description?: string;
  channel_type: 'public' | 'private';
  unread_count: number;
  last_message_preview?: string;
  last_message_at?: string;
}

export interface DmMetadata {
  id: string;
  other_user_id: string;
  other_user_name: string;
  unread_count: number;
  last_message_preview?: string;
  last_message_at?: string;
}

// Channel data types for WebSocket subscription
export interface MessageWithDetails {
  id: string;
  channel_id?: string;
  dm_id?: string;
  user_id: string;
  user_name: string;
  content: string;
  parent_message_id?: string;
  created_at: string;
  edited_at?: string;
  reply_count: number;
}

export interface PinnedMessageInfo {
  id: string;
  message_id: string;
  pinned_by: string;
  pinned_at: string;
}

export interface ChannelMemberInfo {
  id: string;
  user_id: string;
  user_name: string;
  role: string;
  joined_at: string;
}

export interface UnreadInfo {
  count: number;
  last_read_message_id?: string;
  mentions: number;
}

// WebSocket message types
export type WSClientMessage =
  | { type: 'send_message'; channel_id?: string; dm_id?: string; content: string; parent_message_id?: string }
  | { type: 'typing'; channel_id?: string; dm_id?: string }
  | { type: 'subscribe_channel'; channel_id: string }
  | { type: 'unsubscribe_channel'; channel_id: string }
  | { type: 'update_status'; status: 'online' | 'offline' | 'away' };

export type WSServerMessage =
  | { type: 'initial_state'; user_id: string; channels: ChannelMetadata[]; dms: DmMetadata[] }
  | { type: 'channel_data'; channel_id: string; messages: MessageWithDetails[]; pins: PinnedMessageInfo[]; members: ChannelMemberInfo[]; unread_info: UnreadInfo }
  | { type: 'new_message'; id: string; channel_id?: string; dm_id?: string; user_id: string; user_name: string; content: string; parent_message_id?: string; created_at: string }
  | { type: 'message_edited'; message_id: string; content: string; edited_at: string }
  | { type: 'message_deleted'; message_id: string }
  | { type: 'user_typing'; user_id: string; channel_id?: string; dm_id?: string; user_name: string }
  | { type: 'user_status'; user_id: string; status: 'online' | 'offline' | 'away' }
  | { type: 'status_update'; user_id: string; status: 'online' | 'offline' | 'away' | 'dnd'; custom_message?: string; emoji?: string }
  | { type: 'reaction_added'; message_id: string; user_id: string; emoji: string }
  | { type: 'reaction_removed'; message_id: string; user_id: string; emoji: string }
  | { type: 'unread_count_updated'; channel_id?: string; dm_id?: string; unread_count: number; last_read_message_id?: string }
  | { type: 'notification_count_updated'; unread_count: number }
  | { type: 'new_notification'; notification_id: string; notification_type: string; message_id?: string; channel_id?: string; dm_id?: string; created_at: string }
  | { type: 'message_pinned'; channel_id: string; message_id: string; pinned_by: string; pinned_by_name: string; pinned_at: string }
  | { type: 'message_unpinned'; channel_id: string; message_id: string; unpinned_by: string; unpinned_by_name: string }
  | { type: 'bookmark_added'; message_id: string; bookmarked_at: string }
  | { type: 'bookmark_removed'; message_id: string }
  | { type: 'channel_updated'; channel_id: string; name?: string; description?: string; updated_by: string; updated_by_name: string }
  | { type: 'member_joined'; channel_id: string; user_id: string; user_name: string; role: string; joined_at: string }
  | { type: 'member_left'; channel_id: string; user_id: string; user_name: string }
  | { type: 'connected'; user_id: string }
  | { type: 'error'; message: string }
  | { type: 'pong' };

// API Request/Response types
export interface CreateChannelRequest {
  name: string;
  description?: string;
  channel_type: 'public' | 'private';
}

export interface UpdateChannelRequest {
  name?: string;
  description?: string;
}

export interface AddMemberRequest {
  user_id: string;
  role?: 'admin' | 'member';
}

export interface CreateDmRequest {
  participant_ids: string[];
}

export interface SendMessageRequest {
  channel_id?: string;
  dm_id?: string;
  content: string;
  parent_message_id?: string;
}

export interface UpdateMessageRequest {
  content: string;
}

export interface AddReactionRequest {
  emoji: string;
}

export interface UpdateUserRequest {
  display_name?: string;
  avatar_url?: string;
  disable_read_receipts?: boolean;
}

export interface UpdateUserStatusRequest {
  status: 'online' | 'offline' | 'away' | 'dnd';
  custom_message?: string;
  emoji?: string;
  clear_after_minutes?: number;
  back_at?: string; // ISO 8601 datetime for when user will be back
}

// Pagination
export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  page_size: number;
}

// Thread types
export interface ThreadResponse {
  parent: Message;
  replies: Message[];
}

// Read status types
export interface UnreadCountResponse {
  unread_count: number;
  last_read_message_id?: string;
}

export interface MarkAsReadRequest {
  last_message_id?: string;
}

// Pinned message types
export interface PinnedMessage {
  id: string;
  channel_id: string;
  message_id: string;
  pinned_by: string;
  pinned_at: string;
  message?: Message;
}

// Bookmark types
export interface Bookmark {
  id: string;
  user_id: string;
  message_id: string;
  bookmarked_at: string;
  message?: Message;
}

// Link Preview types
export interface LinkPreview {
  url: string;
  title?: string;
  description?: string;
  image?: string;
  site_name?: string;
}

// Read Receipt types
export interface ReadReceipt {
  id: string;
  message_id: string;
  user_id: string;
  read_at: string;
}

export interface ReadReceiptWithUser {
  id: string;
  message_id: string;
  user_id: string;
  read_at: string;
  display_name: string;
  avatar_url?: string;
}

// Message Edit History types
export interface MessageEdit {
  id: string;
  message_id: string;
  old_content: string;
  edited_by: string;
  edited_at: string;
}

export interface MessageEditWithUser {
  id: string;
  message_id: string;
  old_content: string;
  edited_by: string;
  edited_at: string;
  editor_name: string;
}

// Device Session types
export interface DeviceSession {
  id: string;
  user_id: string;
  org_id: string;
  device_type: 'desktop' | 'mobile' | 'web';
  device_name?: string;
  device_fingerprint?: string;
  last_active_at: string;
  created_at: string;
}

export interface PairingCodeResponse {
  code: string;
  expires_in: number;
  api_url: string;
  full_code: string;
}

export interface VerifyPairingCodeRequest {
  code: string;
  device_name?: string;
}

export interface VerifyPairingCodeResponse {
  access_token: string;
  user: User;
  device_id: string;
}

// Incoming Webhook types
export interface IncomingWebhook {
  id: string;
  org_id: string;
  channel_id: string;
  display_name: string;
  description?: string;
  icon_url?: string;
  username?: string;
  enabled: boolean;
  created_by: string;
  created_at: string;
  updated_at: string;
  url?: string;
}

export interface CreateWebhookRequest {
  channel_id: string;
  display_name: string;
  description?: string;
  icon_url?: string;
  username?: string;
}

export interface UpdateWebhookRequest {
  channel_id?: string;
  display_name?: string;
  description?: string;
  icon_url?: string;
  username?: string;
  enabled?: boolean;
}
