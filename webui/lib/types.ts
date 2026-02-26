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
  archived?: boolean;
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
  poll?: Poll;
  forwarded_from_message_id?: string;
  forwarded_from_channel_id?: string;
  forwarded_from_channel_name?: string;
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
  forwarded_from_message_id?: string;
  forwarded_from_channel_id?: string;
  forwarded_from_channel_name?: string;
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
  | { type: 'subscribe_dm'; dm_id: string }
  | { type: 'unsubscribe_dm'; dm_id: string }
  | { type: 'update_status'; status: 'online' | 'offline' | 'away' }
  | { type: 'mark_as_read'; channel_id?: string; dm_id?: string; last_message_id?: string }
  | { type: 'add_reaction'; message_id: string; emoji: string }
  | { type: 'remove_reaction'; message_id: string; emoji: string }
  | { type: 'pin_message'; message_id: string }
  | { type: 'unpin_message'; message_id: string }
  | { type: 'add_bookmark'; message_id: string }
  | { type: 'remove_bookmark'; message_id: string }
  | { type: 'edit_message'; message_id: string; content: string }
  | { type: 'delete_message'; message_id: string }
  | { type: 'subscribe_thread'; message_id: string }
  | { type: 'unsubscribe_thread'; message_id: string };

export type WSServerMessage =
  | { type: 'initial_state'; user_id: string; channels: ChannelMetadata[]; dms: DmMetadata[]; notification_preferences: { channel_id?: string; dm_id?: string; preference: string; mute_until?: string | null }[] }
  | { type: 'channel_data'; channel_id: string; messages: MessageWithDetails[]; pins: PinnedMessageInfo[]; members: ChannelMemberInfo[]; unread_info: UnreadInfo }
  | { type: 'dm_data'; dm_id: string; messages: MessageWithDetails[]; unread_info: UnreadInfo }
  | { type: 'new_message'; id: string; channel_id?: string; dm_id?: string; user_id: string; user_name: string; content: string; parent_message_id?: string; created_at: string; forwarded_from_message_id?: string; forwarded_from_channel_id?: string; forwarded_from_channel_name?: string }
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
  | { type: 'reminder_triggered'; reminder_id: string; message_id: string; channel_id?: string; dm_id?: string; message_preview: string; created_at: string }
  | { type: 'ephemeral_message'; content: string; channel_id?: string; dm_id?: string }
  | { type: 'poll_vote_updated'; poll_id: string; message_id: string; channel_id?: string; dm_id?: string; options: { index: number; text: string; votes: number }[]; total_votes: number; user_votes: number[] }
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

// Scheduled Message types
export interface ScheduledMessage {
  id: string;
  org_id: string;
  user_id: string;
  channel_id?: string;
  dm_id?: string;
  content: string;
  parent_message_id?: string;
  scheduled_at: string;
  sent: boolean;
  created_at: string;
}

// Channel Section types
export interface ChannelSection {
  id: string;
  user_id: string;
  org_id: string;
  name: string;
  position: number;
  collapsed: boolean;
  created_at: string;
  channel_ids: string[];
}

export interface CreateChannelSectionRequest {
  name: string;
}

export interface UpdateChannelSectionRequest {
  name?: string;
  collapsed?: boolean;
}

export interface ReorderSectionRequest {
  order: { id: string; position: number }[];
}

export interface ReorderSectionItemsRequest {
  order: { channel_id: string; position: number }[];
}

// User Group types
export interface UserGroup {
  id: string;
  org_id: string;
  name: string;
  handle: string;
  description?: string;
  created_by: string;
  created_at: string;
  updated_at: string;
}

export interface UserGroupMember {
  id: string;
  group_id: string;
  user_id: string;
  added_at: string;
}

export interface CreateUserGroupRequest {
  name: string;
  handle: string;
  description?: string;
}

export interface UpdateUserGroupRequest {
  name?: string;
  handle?: string;
  description?: string;
}

// Reminder types
export interface Reminder {
  id: string;
  user_id: string;
  org_id: string;
  message_id: string;
  channel_id?: string;
  dm_id?: string;
  remind_at: string;
  message_preview: string;
  completed: boolean;
  created_at: string;
}

export interface NotificationPref {
  preference: 'all' | 'mentions' | 'nothing';
  mute_until?: string | null;
}

// Slash Command types
export interface SlashCommand {
  name: string;
  description: string;
  usage_hint?: string;
  handler_type: string;
  id?: string;
}

export interface SlashCommandFull {
  id: string;
  org_id: string;
  command_name: string;
  description: string;
  usage_hint?: string;
  handler_type: string;
  webhook_url?: string;
  response_type: string;
  created_by: string;
  enabled: boolean;
  created_at: string;
}

export interface ExecuteCommandRequest {
  command: string;
  text: string;
  channel_id?: string;
  dm_id?: string;
}

export interface ExecuteCommandResponse {
  response_type: 'ephemeral' | 'in_channel';
  content: string;
  message_id?: string;
}

// Poll types
export interface PollOption {
  index: number;
  text: string;
  votes: number;
}

export interface Poll {
  id: string;
  message_id?: string;
  question: string;
  options: PollOption[];
  poll_type: 'single' | 'multiple';
  anonymous: boolean;
  total_votes: number;
  user_votes: number[];
  closed: boolean;
  expires_at?: string;
  created_by: string;
  created_at: string;
}

export interface ForwardMessageRequest {
  channel_id?: string;
  dm_id?: string;
  comment?: string;
}

export interface CreatePollRequest {
  channel_id?: string;
  dm_id?: string;
  question: string;
  options: string[];
  poll_type?: 'single' | 'multiple';
  anonymous?: boolean;
  expires_at?: string;
}
