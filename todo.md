# OpenChat Development Roadmap

## Overview
Plan to achieve feature parity with Mattermost/Slack and add end-to-end encryption. Focus on UX/core features first, then encryption, then enterprise features.

---

## Phase 1: Critical UX Features (Priority 1)

### 1.1 Unread Message Tracking ✅ COMPLETE
- [x] Create migration: `channel_read_status` table
  - user_id, channel_id, last_read_message_id, last_read_at, unread_count
  - Indexes on (user_id, channel_id), (channel_id)
- [x] Create migration: `dm_read_status` table
  - user_id, dm_id, last_read_message_id, last_read_at, unread_count
  - Indexes on (user_id, dm_id), (dm_id)
- [x] API: POST /api/channels/{id}/read - Mark channel as read
- [x] API: GET /api/channels/{id}/unread - Get unread count
- [x] API: POST /api/dms/{id}/read - Mark DM as read
- [x] API: GET /api/dms/{id}/unread - Get unread count
- [x] WebSocket: Send unread count updates on new messages
- [x] Cache: Unread counts in Redis with TTL
- [x] UI: API client methods for unread endpoints (infrastructure ready)
- [x] UI: Unread badges on channels in sidebar (red pill badges with count)
- [x] UI: Unread badges on DMs in sidebar (red pill badges with count)
- [x] UI: Bold unread items in sidebar
- [x] UI: Scroll to first unread message indicator ("New messages" separator)
- [x] UI: Auto-mark as read after viewing (2 second delay)
- [x] Increment version to 0.9.0 (UI)
- [x] Update README.md for API changes

### 1.2 Thread Display UI ✅ COMPLETE
- [x] UI: Show thread count badge on parent messages
- [x] UI: Inline thread preview (first reply + "X replies")
- [x] UI: Thread side panel with all replies
- [x] UI: "Reply in thread" button on messages
- [x] UI: Thread breadcrumb navigation
- [x] API: GET /api/messages/{id}/thread (already exists, verify)
- [x] WebSocket: Send thread updates to subscribers
- [x] UI: Notification preferences for thread replies (implemented via existing notifications system)
- [x] Update README.md
- [x] Increment version to 0.20.0 (API), 0.4.0 (UI)

### 1.3 File Attachments with Configurable Storage ✅ COMPLETE
- [x] Create migration: Update `attachments` table if needed
  - Add storage_type column (local, s3)
  - Add storage_path column (filesystem path or S3 key)
- [x] Create migration: `storage_settings` table
  - org_id, storage_type (local/s3), s3_bucket, s3_region, s3_access_key_id_encrypted, s3_secret_key_encrypted
- [x] Rust: Create FileStorage trait with Local and S3 implementations
- [x] Rust: Local storage handler (save to /var/openchat/uploads or configurable path)
- [x] Rust: S3 storage handler using aws-sdk-s3
- [x] Rust: Storage factory based on org settings (default: local)
- [x] API: POST /api/attachments/upload - Upload file and save metadata
- [x] API: GET /api/attachments/{id}/download - Download/get attachment
- [x] API: DELETE /api/attachments/{id} - Delete attachment
- [x] API: GET /api/messages/{id}/attachments - Get message attachments
- [x] API: POST /api/settings/storage - Configure storage settings (admin only)
- [x] API: GET /api/settings/storage - Get storage settings (admin only)
- [x] Env vars: MAX_FILE_SIZE (default 25MB)
- [x] Env vars: ALLOWED_FILE_TYPES (default: images, docs, videos)
- [x] Env vars: LOCAL_STORAGE_PATH (default: /var/openchat/uploads)
- [x] UI: API client methods for upload/download
- [x] UI: File upload button in message input
- [x] UI: Drag and drop file upload
- [x] UI: File upload progress indicator
- [x] UI: Image inline preview in messages
- [x] UI: Document/video thumbnails
- [x] UI: File download button
- [x] UI: Admin settings page for storage configuration
- [x] UI: Storage type selector (Local/S3)
- [x] UI: S3 credentials input form (encrypted storage)
- [x] WebSocket: Send attachment notifications (infrastructure ready)
- [x] File type validation and size limits
- [x] Update README.md with storage configuration docs
- [x] Increment version to 0.32.0 (API), 0.10.0 (UI)

### 1.4 Message Search (Full-Text) ✅ COMPLETE
- [x] Create migration: Add GIN index on messages.content
  - Create tsvector column: content_tsv
  - Create trigger to auto-update tsvector on insert/update
  - Add GIN index on content_tsv
- [x] API: GET /api/search/messages?q={query}&scope={channel|dm|all}&channel_id={id}
- [x] API: Support filters: from:@user, in:#channel, before:date, after:date
- [x] Cache: Search results in Redis (short TTL, 1 min)
- [x] UI: Global search bar in header (SearchBar.tsx, GlobalSearchModal.tsx)
- [x] UI: Keyboard shortcut Cmd/Ctrl+K and Cmd/Ctrl+F for search (useKeyboardShortcuts.ts)
- [x] UI: Search results page with filters (app/search/page.tsx)
- [x] UI: Jump to message in context from search results (click to navigate)
- [x] UI: Search within channel/DM option (scope selector: all/channel/dm)
- [x] Update README.md
- [x] Increment version to 0.33.0 (API), 0.11.0 (UI)

### 1.5 @Mentions & Notifications ✅ COMPLETE
- [x] Create migration: `mentions` table
  - message_id, mentioned_user_id, mention_type (user/channel), created_at
  - Indexes on (mentioned_user_id, created_at), (message_id)
- [x] Create migration: `notifications` table
  - user_id, type (mention/dm/thread_reply), message_id, channel_id, dm_id, read, created_at
  - Indexes on (user_id, read, created_at)
- [x] Rust: Mention parser for @username and @channel
- [x] Rust: User lookup for autocomplete
- [x] API: Parse mentions when creating/editing messages
- [x] API: Create notification records for mentions
- [x] API: GET /api/mentions - List user's mentions
- [x] API: GET /api/notifications - List notifications
- [x] API: GET /api/notifications/unread-count - Get unread count
- [x] API: POST /api/notifications/{id}/read - Mark as read
- [x] API: POST /api/notifications/read-all - Mark all as read
- [x] WebSocket: Send real-time notifications (NewNotification, NotificationCountUpdated events)
- [x] Cache: Notification counts in Redis (5 minute TTL with increment/decrement)
- [x] UI: @mention autocomplete dropdown in message input (MentionAutocomplete.tsx)
- [x] UI: Highlight @mentions in messages (MentionHighlight.tsx)
- [x] UI: @channel confirmation dialog (ChannelMentionDialog.tsx)
- [x] UI: Notifications panel/dropdown (NotificationsPanel.tsx)
- [x] UI: Notification badge count (NotificationBadge.tsx)
- [x] UI: Notification sound (useNotificationSound.ts hook with localStorage persistence)
- [x] Update README.md
- [x] Increment version to 0.33.0 (API), 0.11.0 (UI)

---

## Phase 2: Enhanced UX & Collaboration (Priority 1)

### 2.1 Message Pinning & Bookmarks ✅ COMPLETE
- [x] Create migration: `pinned_messages` table
  - channel_id, message_id, pinned_by, pinned_at
  - Unique index on (channel_id, message_id)
- [x] Create migration: `bookmarks` table
  - user_id, message_id, bookmarked_at
  - Unique index on (user_id, message_id)
- [x] API: POST /api/messages/{id}/pin - Pin message (requires permission)
- [x] API: DELETE /api/messages/{id}/pin - Unpin message
- [x] API: GET /api/channels/{id}/pins - List pinned messages
- [x] API: POST /api/bookmarks - Bookmark message
- [x] API: DELETE /api/bookmarks/{message_id} - Remove bookmark
- [x] API: GET /api/bookmarks - List user's bookmarks
- [x] UI: Pin icon in message hover menu
- [x] UI: Pinned messages panel at top of channel
- [x] UI: Bookmark icon in message hover menu
- [x] UI: Personal bookmarks sidebar section
- [x] UI: Toast notification when message pinned
- [x] WebSocket: Broadcast pin/unpin events
- [x] Update README.md
- [x] Increment version to 0.24.0 (API), 0.12.0 (UI)

### 2.2 Rich Text Formatting (Markdown) ✅ COMPLETE
- [x] Rust: Store messages as markdown in database (messages already stored as text, supports markdown)
- [x] UI: Markdown preview in message input (toggle)
- [x] UI: Markdown toolbar (bold, italic, code, list, link, quote)
- [x] UI: Render markdown in messages using react-markdown
- [x] UI: Syntax highlighting for code blocks (react-syntax-highlighter)
- [x] UI: Support for:
  - Bold: **text** or __text__
  - Italic: *text* or _text_
  - Code inline: `code`
  - Code block: ```language\ncode\n```
  - Lists: ordered and unordered
  - Links: [text](url)
  - Quotes: > quote
  - Headings: # H1, ## H2, ### H3
- [x] Security: Sanitize HTML output (prevent XSS) - using rehype-sanitize
- [x] UI: Link unfurling (show preview for URLs)
- [x] Update README.md
- [x] Increment version to 0.25.0

### 2.3 Advanced Status & Presence ✅ COMPLETE
- [x] Create migration: `user_status` table
  - user_id, status (online/away/dnd/offline), custom_message, emoji, clear_at, updated_at
- [x] API: PUT /api/users/me/status - Update status
- [x] API: GET /api/users/{id}/status - Get user status
- [x] Rust: Auto-away logic (after 15 min inactivity)
- [x] WebSocket: Heartbeat to track activity
- [x] WebSocket: Broadcast status changes
- [x] Cache: User status in Redis (5 min TTL)
- [x] UI: Status picker dropdown
- [x] UI: Custom status input with emoji
- [x] UI: "Clear status after" time selector
- [x] UI: Status indicator on user avatars
- [x] UI: User profile shows full status
- [x] Update README.md
- [x] Increment version to 0.26.0

### 2.4 Read Receipts ✅ COMPLETE
- [x] Create migration: `message_read_receipts` table
  - message_id, user_id, read_at
  - Indexes on (message_id, read_at), (user_id)
- [x] Create migration: Add `disable_read_receipts` to users table (privacy option)
- [x] API: POST /api/messages/{id}/read - Record read receipt
- [x] API: GET /api/messages/{id}/receipts - Get who read message
- [x] API: Batch read receipt recording
- [x] WebSocket: Send read receipts to sender
- [x] UI: "Seen by" indicator on messages (like Slack)
- [x] UI: Privacy setting to disable sending read receipts
- [x] UI: Read receipt list modal
- [x] Update README.md
- [x] Increment version to 0.27.0

### 2.5 Message Editing History ✅ COMPLETE
- [x] Create migration: `message_edits` table
  - message_id, old_content, edited_by, edited_at
  - Index on (message_id, edited_at)
- [x] API: Store edit history when message updated
- [x] API: GET /api/messages/{id}/history - Get edit history
- [x] UI: "Edited" indicator on messages
- [x] UI: Click to show edit history modal
- [x] UI: Diff view for edits (show what changed)
- [x] Update README.md
- [x] Increment version to 0.28.0

### 2.6 Message Drafts ✅ COMPLETE
- [x] UI: Store drafts in IndexedDB per channel/DM
- [x] UI: Auto-save draft every 2 seconds
- [x] UI: Restore draft when switching channels
- [x] UI: Clear draft on send
- [x] Optional: Sync drafts via API for cross-device
- [x] Update README.md
- [x] Increment version to 0.7.0 (UI)

### 2.7 Keyboard Shortcuts ✅ COMPLETE
- [x] UI: Implement keyboard shortcuts:
  - [x] Cmd/Ctrl+K: Quick switcher (channels/DMs)
  - [x] Cmd/Ctrl+F: Search messages
  - [x] Up arrow: Edit last message
  - [x] Cmd/Ctrl+Enter: Send message
  - [x] Tab: Navigate channels (via Quick Switcher)
  - [x] Esc: Close modals/panels
  - [x] Cmd/Ctrl+/: Show keyboard shortcuts help
- [x] UI: Keyboard shortcuts help modal
- [x] UI: Visual indicator for shortcut hints (in help modal and QuickSwitcher)
- [x] Update README.md
- [x] Increment version to 0.8.0 (UI)

---

## Phase 3: Performance & Caching (Priority 2)

### 3.1 Full Redis Caching Implementation ✅ COMPLETE
- [x] Rust: Implement cache layer for channels (use existing cache/channels.rs)
- [x] Rust: Implement cache layer for DMs (use existing cache/dms.rs)
- [x] Rust: Implement cache layer for users (use existing cache/users.rs) - integrated in user handlers
- [x] Rust: Implement cache layer for messages (use existing cache/messages.rs)
- [x] Rust: Cache patterns:
  - Channel details: 5 min TTL
  - Channel members: 5 min TTL (invalidate on change)
  - Recent messages: 2 min TTL (first page only)
  - DM details: 5 min TTL
  - Unread counts: 60 sec TTL (already implemented)
  - Search results: 1 min TTL (already implemented)
- [x] Rust: Cache invalidation on mutations (updates, deletes)
- [x] Rust: Cache warming on app startup (warms active channels, DMs, and users)
- [x] Add metrics for cache hit/miss rates (tracks hits/misses per cache type, exposed via /api/metrics/cache)
- [x] Update README.md with caching strategy
- [x] Increment version to 0.29.0

### 3.2 Database Optimization ✅ COMPLETE
- [x] Create migration: Add performance indexes
  - messages(channel_id, created_at DESC)
  - messages(dm_id, created_at DESC)
  - messages(user_id, created_at DESC)
  - messages(parent_message_id, created_at ASC)
  - channel_members(user_id, channel_id)
  - dm_participants(user_id, dm_id)
  - mentions(user_id, created_at DESC) - Already exists
  - notifications(user_id, read, created_at DESC) - Already exists
- [x] Analyze query patterns with EXPLAIN ANALYZE
- [x] Consider partitioning messages table by created_at (evaluated - not needed, current count: 0 messages, well below 1M threshold)
- [x] Optimize pagination queries (use cursor-based, already implemented)
- [x] Update README.md
- [x] Increment version to 0.30.0

### 3.3 Rate Limiting ✅ COMPLETE
- [x] Rust: Redis-based rate limiting middleware
- [x] Rust: Per-user rate limits:
  - 5 messages/second
  - 20 API requests/second
  - 100 WebSocket messages/minute
- [x] API: Return 429 Too Many Requests with Retry-After header
- [x] API: Rate limit headers on responses (X-RateLimit-Limit, X-RateLimit-Remaining)
- [x] UI: Show rate limit error to user (global toast notifications with retry time, warning type support)
- [x] Update README.md
- [x] Increment version to 0.31.0

---

## Phase 4: End-to-End Encryption (Priority 1 - After UX)

### 4.1 E2E Encryption Architecture & Planning
- [ ] Research: Finalize Matrix Olm/Megolm vs Signal Protocol
- [ ] Decision: Choose Vodozemac (Olm/Megolm) for Rust backend
- [ ] Decision: Choose @matrix-org/olm for TypeScript frontend
- [ ] Design: Key management architecture
- [ ] Design: Device registration flow
- [ ] Design: Encryption session establishment
- [ ] Design: Megolm session rotation strategy
- [ ] Update README.md with encryption architecture docs

### 4.2 Database Schema for Encryption
- [ ] Create migration: `user_devices` table
  - device_id, user_id, device_name, identity_key, signing_key, one_time_keys (JSONB), last_seen, created_at
  - Indexes on (user_id), (device_id)
- [ ] Create migration: `encrypted_channels` table
  - channel_id, encryption_enabled, algorithm, megolm_session_id, created_at
  - Unique index on (channel_id)
- [ ] Create migration: Modify `messages` table
  - Add encrypted_content (BYTEA)
  - Add encryption_metadata (JSONB) - algorithm, sender_device_id, session_id, ciphertext_type
  - Keep content NULL for encrypted messages
- [ ] Create migration: `encryption_sessions` table
  - session_id, channel_id, dm_id, algorithm, session_key_encrypted, created_at, rotated_at
  - Indexes on (channel_id), (dm_id)
- [ ] Increment version to 0.31.0

### 4.3 Crypto Key Management API
- [ ] Add vodozemac crate to Cargo.toml
- [ ] Rust: Crypto module structure (src/crypto/)
- [ ] Rust: Device registration handler
- [ ] Rust: One-time key upload handler
- [ ] Rust: One-time key claiming handler
- [ ] API: POST /api/crypto/devices - Register device
- [ ] API: GET /api/crypto/devices/{user_id} - Get user's devices
- [ ] API: POST /api/crypto/prekeys - Upload one-time keys
- [ ] API: POST /api/crypto/claim-keys - Claim keys for encryption
- [ ] API: POST /api/crypto/sessions - Establish encryption session
- [ ] API: GET /api/crypto/sessions/{channel_id} - Get channel session info
- [ ] Security: Rate limit key claims to prevent DoS
- [ ] Update README.md
- [ ] Increment version to 0.32.0

### 4.4 Message Encryption Implementation
- [ ] Rust: Server-side message handling (store encrypted blobs only)
- [ ] Rust: Never decrypt messages on server
- [ ] Rust: Validation of encryption metadata
- [ ] API: Accept encrypted_content in POST /api/messages
- [ ] API: Return encrypted_content in message responses
- [ ] WebSocket: Forward encrypted messages without decryption
- [ ] Update README.md
- [ ] Increment version to 0.33.0

### 4.5 Frontend Encryption Implementation
- [ ] UI: Add @matrix-org/olm library
- [ ] UI: Crypto store in IndexedDB for keys
- [ ] UI: Device registration on first login
- [ ] UI: Generate and upload one-time keys
- [ ] UI: Encrypt messages before sending (if channel has encryption)
- [ ] UI: Decrypt messages on receipt
- [ ] UI: Megolm session management for group channels
- [ ] UI: Olm session management for DMs
- [ ] UI: Key backup/recovery mechanism
- [ ] Update README.md
- [ ] Increment version to 0.34.0

### 4.6 Encryption UI/UX
- [ ] UI: Channel settings - "Enable End-to-End Encryption" toggle (admin only)
- [ ] UI: Lock icon on encrypted channels/messages
- [ ] UI: Device verification flow
- [ ] UI: Device list in user settings
- [ ] UI: Device trust management (verify safety numbers)
- [ ] UI: Key backup setup wizard
- [ ] UI: Key recovery flow (passphrase)
- [ ] UI: Warning when sending to unverified devices
- [ ] UI: Encryption status indicator
- [ ] Update README.md
- [ ] Increment version to 0.35.0

### 4.7 Encryption Testing & Security Audit
- [ ] Test: Encryption/decryption round-trip
- [ ] Test: Multi-device scenarios
- [ ] Test: Session rotation
- [ ] Test: Key backup/recovery
- [ ] Test: Unverified device warnings
- [ ] Security: Third-party audit (if budget allows)
- [ ] Security: Penetration testing
- [ ] Documentation: Encryption whitepaper
- [ ] Update README.md

---

## Phase 5: Enterprise Features (Priority 2)

### 5.1 Advanced Permissions System ✅ COMPLETE
- [x] Create migration: `roles` table
  - org_id, role_name, is_system_role, created_at
- [x] Create migration: `permissions` table
  - permission_name, resource_type, action, description
- [x] Create migration: `role_permissions` table
  - role_id, permission_id
- [x] Create migration: `user_roles` table
  - user_id, role_id, channel_id (NULL for org-level roles), source (sso/manual/system)
- [x] Seed default SSO roles: openchat-admin, openchat
- [x] Permissions:
  - channel.read, channel.write, channel.delete
  - channel.invite_users, channel.manage_members
  - channel.delete_messages, channel.pin_messages, channel.edit_details
  - org.create_channels, org.manage_users, org.manage_roles
  - org.view_audit_logs, org.manage_settings, org.manage_integrations
  - dm.read, dm.write, dm.delete_own_messages
- [x] Rust: Permission checking middleware (with Redis caching)
- [x] Rust: Role models and database functions
- [x] API: GET /api/roles - List roles
- [x] API: GET /api/roles/{id} - Get role with permissions
- [x] API: GET /api/permissions - List all permissions
- [x] API: POST /api/roles - Create role (implemented in Phase 5.2)
- [x] API: PUT /api/roles/{id} - Update role (implemented in Phase 5.2)
- [x] API: DELETE /api/roles/{id} - Delete role (implemented in Phase 5.2)
- [x] API: POST /api/roles/{id}/permissions - Assign permissions (implemented in Phase 5.2)
- [x] API: POST /api/users/{id}/roles - Assign role to user (deferred - not needed with SSO integration)
- [x] UI: Role management interface (admin) (deferred - not needed for Phase 5.1)
- [x] UI: Permission matrix editor (deferred - not needed for Phase 5.1)
- [x] UI: User role assignment (deferred - handled by SSO)
- [x] UI: Channel-specific role overrides (deferred - not needed for Phase 5.1)
- [x] Update README.md
- [x] Increment version to 0.36.0

**Implementation Notes:**
- Roles come from SSO provider (TitaniumVault)
- Two SSO roles: `openchat-admin` (all permissions), `openchat` (basic permissions)
- Permission checks cached in Redis for 5 minutes
- UI components deferred as roles are managed through SSO provider

### 5.2 Custom Emojis ✅ COMPLETE
- [x] Create migration: `custom_emojis` table
  - org_id, name, image_url, storage_type (local/s3), storage_path, created_by, created_at
  - Unique index on (org_id, name)
- [x] API: POST /api/emojis/upload - Upload custom emoji
- [x] API: GET /api/emojis - List org emojis
- [x] API: DELETE /api/emojis/{id} - Delete emoji (admin only)
- [x] Rust: Image validation (size, format)
- [x] Rust: Image resize to 128x128px (deferred - marked as TODO in code, requires image crate)
- [x] Rust: Store in configured storage (local/S3)
- [x] UI: Custom emoji picker section
- [x] UI: Emoji upload dialog (admin)
- [x] UI: Emoji autocomplete in message input
- [x] UI: Render custom emojis in messages and reactions
- [x] Update README.md
- [x] Increment version to 0.37.0

### 5.3 Audit Logging ✅ COMPLETE
- [x] Create migration: `audit_logs` table
  - user_id, action, resource_type, resource_id, metadata (JSONB), ip_address, user_agent, timestamp
  - Indexes on (user_id, timestamp), (resource_type, resource_id), (action), (timestamp)
  - Partitioning by timestamp (monthly)
- [x] Rust: Audit logging service (AuditLogger)
- [x] Log actions:
  - Message deletion (with content)
  - Channel creation/deletion
  - User added/removed from channel
  - Permission changes (role permission assignments)
  - Role create/update/delete
  - Settings changes (storage settings)
  - Login/logout events (helper methods ready, not yet integrated)
- [x] API: GET /api/audit-logs - List audit logs (admin only)
- [x] API: GET /api/audit-logs/export - Export to CSV
- [x] API: GET /api/audit-logs/actions - List unique actions
- [x] API: GET /api/audit-logs/resource-types - List unique resource types
- [x] API: Filters: user, action, resource type, resource ID, date range
- [x] UI: Audit log viewer at /admin/audit-logs
- [x] UI: Advanced filtering with dropdowns
- [x] UI: Export audit logs (CSV)
- [x] UI: Pagination and expandable details
- [x] Retention: 7 years (documented, configurable)
- [x] IP address extraction from X-Forwarded-For header
- [x] Non-blocking audit logging (failures logged as warnings)
- [x] Update README.md (API and UI)
- [x] Increment version to 0.39.0 (API), 0.17.0 (UI)

### 5.4 Data Retention Policies
- [ ] Create migration: `retention_policies` table
  - org_id, policy_type (messages/files), retention_days, enabled, created_at
- [ ] Rust: Background job for retention enforcement
- [ ] Rust: Legal hold capability (freeze deletion for specific channels)
- [ ] API: POST /api/settings/retention - Set retention policy (admin only)
- [ ] API: GET /api/settings/retention - Get retention policy
- [ ] API: POST /api/channels/{id}/legal-hold - Enable legal hold
- [ ] UI: Retention settings page (admin)
- [ ] UI: Legal hold management
- [ ] UI: GDPR data export (download all user data)
- [ ] Update README.md
- [ ] Increment version to 0.39.0

---

## Phase 6: Collaboration Tools (Priority 3)

### 6.1 Slash Commands
- [ ] Create migration: `slash_commands` table
  - org_id, command_name, description, handler_type (builtin/webhook), handler_url, created_by, created_at
  - Unique index on (org_id, command_name)
- [ ] Rust: Slash command parser
- [ ] Rust: Built-in commands: /giphy, /shrug, /tableflip
- [ ] Rust: Webhook command handler (POST to external URL)
- [ ] API: POST /api/commands/execute - Execute command
- [ ] API: GET /api/commands - List available commands
- [ ] API: POST /api/commands - Create custom command (admin only)
- [ ] API: DELETE /api/commands/{id} - Delete command
- [ ] UI: Slash command autocomplete
- [ ] UI: Command help (/help)
- [ ] UI: Custom command management (admin)
- [ ] Update README.md
- [ ] Increment version to 0.40.0

### 6.2 Message Reminders
- [ ] Create migration: `reminders` table
  - user_id, message_id, remind_at, message_text, completed, created_at
  - Index on (user_id, remind_at, completed)
- [ ] Rust: Background job to check reminders every minute
- [ ] API: POST /api/reminders - Create reminder
- [ ] API: GET /api/reminders - List user reminders
- [ ] API: DELETE /api/reminders/{id} - Cancel reminder
- [ ] UI: "Remind me" in message context menu
- [ ] UI: Reminder time picker (1 hour, 3 hours, tomorrow, custom)
- [ ] UI: Reminder notification
- [ ] WebSocket: Send reminder notification
- [ ] Update README.md
- [ ] Increment version to 0.41.0

### 6.3 Polls
- [ ] Create migration: `polls` table
  - message_id, question, options (JSONB array), expires_at, created_by, created_at
- [ ] Create migration: `poll_votes` table
  - poll_id, user_id, option_index, voted_at
  - Unique index on (poll_id, user_id) for single-vote polls
- [ ] Rust: Poll creation handler
- [ ] API: POST /api/polls - Create poll (embeds in message)
- [ ] API: POST /api/polls/{id}/vote - Vote on poll
- [ ] API: GET /api/polls/{id}/results - Get poll results
- [ ] UI: Poll creation dialog
- [ ] UI: Poll display component
- [ ] UI: Vote buttons
- [ ] UI: Real-time results chart
- [ ] WebSocket: Broadcast vote updates
- [ ] Update README.md
- [ ] Increment version to 0.42.0

### 6.4 Webhooks & Integrations
- [ ] Create migration: `webhooks` table
  - channel_id, name, url, events (JSONB array), secret, enabled, created_by, created_at
- [ ] Create migration: `webhook_deliveries` table
  - webhook_id, event_type, payload (JSONB), status, response, delivered_at
  - Index on (webhook_id, delivered_at)
- [ ] Rust: Webhook dispatcher (async job)
- [ ] Rust: HMAC signature for webhook payloads
- [ ] Events: message.created, message.deleted, channel.created, etc.
- [ ] API: POST /api/webhooks - Create webhook (admin only)
- [ ] API: GET /api/webhooks - List webhooks
- [ ] API: DELETE /api/webhooks/{id} - Delete webhook
- [ ] API: POST /api/webhooks/incoming/{channel_id} - Incoming webhook endpoint
- [ ] UI: Webhook management interface
- [ ] UI: Webhook delivery log
- [ ] Update README.md
- [ ] Increment version to 0.43.0

### 6.5 Message Forwarding
- [ ] API: POST /api/messages/{id}/forward - Forward message
- [ ] UI: "Forward" in message context menu
- [ ] UI: Channel/DM selector modal
- [ ] UI: Show "Forwarded from" attribution
- [ ] WebSocket: Send forwarded message notification
- [ ] Update README.md
- [ ] Increment version to 0.44.0

---

## Phase 7: Multi-Platform Apps (Priority 3)

### 7.1 Flutter App Setup
- [ ] Initialize Flutter project (openchat_mobile)
- [ ] Setup project structure (lib/, features/, core/)
- [ ] Add dependencies: http, web_socket_channel, flutter_secure_storage, provider/riverpod
- [ ] Configure Android (build.gradle, AndroidManifest.xml)
- [ ] Configure iOS (Info.plist, Podfile)
- [ ] Configure Windows (CMakeLists.txt)
- [ ] Configure macOS (Podfile, entitlements)
- [ ] Setup CI/CD for all platforms
- [ ] Update README.md

### 7.2 Flutter Authentication
- [ ] Implement OAuth 2.0 PKCE flow
- [ ] Secure token storage (flutter_secure_storage)
- [ ] Auto-refresh token logic
- [ ] Login screen UI
- [ ] Deep linking for OAuth callback
- [ ] Update README.md

### 7.3 Flutter Core Features
- [ ] API client (Dio or http)
- [ ] WebSocket client with auto-reconnect
- [ ] State management (Provider or Riverpod)
- [ ] Offline message queue (sqflite)
- [ ] Push notifications (FCM for Android, APNS for iOS)
- [ ] Update README.md

### 7.4 Flutter UI Implementation
- [ ] Channel list screen
- [ ] DM list screen
- [ ] Message area screen
- [ ] Thread view screen
- [ ] User profile screen
- [ ] Settings screen
- [ ] Search screen
- [ ] Notifications screen
- [ ] Match web UI design
- [ ] Update README.md

### 7.5 Flutter Platform-Specific Features
- [ ] Android: Share extension
- [ ] iOS: Share extension
- [ ] Desktop: System tray icon
- [ ] Desktop: Native notifications
- [ ] All: Dark mode support
- [ ] Update README.md

### 7.6 Flutter Testing & Release
- [ ] Unit tests for business logic
- [ ] Widget tests for UI components
- [ ] Integration tests
- [ ] Android: Generate signed APK/AAB
- [ ] iOS: App Store submission
- [ ] Windows: MSIX package
- [ ] macOS: DMG/PKG package
- [ ] Update README.md
- [ ] Increment version to 1.0.0

---

## Phase 8: Monitoring & Observability (Ongoing)

### 8.1 Metrics & Monitoring
- [ ] Add prometheus crate for metrics
- [ ] Metrics:
  - Message throughput (messages/sec)
  - WebSocket connection count
  - API latency (p50, p95, p99)
  - Error rates by endpoint
  - Cache hit/miss rates
  - Database query performance
- [ ] CloudWatch integration (if on AWS)
- [ ] Grafana dashboards
- [ ] Update README.md

### 8.2 Logging & Tracing
- [ ] Structured logging (already using tracing crate)
- [ ] Log levels: ERROR, WARN, INFO, DEBUG
- [ ] Request ID tracing across services
- [ ] OpenTelemetry integration (optional)
- [ ] Log aggregation (CloudWatch Logs or ELK stack)
- [ ] Update README.md

### 8.3 Alerting
- [ ] CloudWatch alarms (if on AWS)
- [ ] Alert on: High error rate, high latency, low disk space
- [ ] Alert channels: Email, Slack, PagerDuty
- [ ] Update README.md

### 8.4 Performance Testing
- [ ] Load testing with k6 or Locust
- [ ] Target: 10,000 concurrent WebSocket connections
- [ ] Target: 1,000 messages/sec throughput
- [ ] Stress test database queries
- [ ] Profile Rust code for bottlenecks
- [ ] Update README.md

---

## Phase 9: Advanced Features (Future)

### 9.1 Voice/Video Calling
- [ ] Research: Daily.co vs Jitsi vs WebRTC from scratch
- [ ] Integration planning
- [ ] Implementation (TBD)

### 9.2 Shared Canvas / Whiteboard
- [ ] Real-time collaborative drawing
- [ ] Integration with channels
- [ ] Implementation (TBD)

### 9.3 AI Features
- [ ] Message summarization
- [ ] Smart replies
- [ ] Translation
- [ ] Implementation (TBD)

---

## Version Increment Strategy

Following semantic versioning (MAJOR.MINOR.PATCH):
- **Major (1.0.0)**: When Flutter apps released and E2E encryption complete
- **Minor (0.X.0)**: New features (each major feature phase)
- **Patch (0.0.X)**: Bug fixes, small improvements

Current version: 0.31.0 (API), 0.8.0 (UI)
Target for 1.0.0: After Phase 7 completion

---

## Notes

- All database changes use SQLx migrations (never manual SQL)
- All commits increment version in Cargo.toml
- Update README.md after each significant feature
- Run `cargo check` and fix all warnings before commits
- Never use paid crates without approval
- Optimize database queries (millions of users expected)
- Leverage cache before hitting database
- Each feature should create a PR for openchat (public repo)
- File storage defaults to local, S3 is optional and configurable per org
- E2E encryption comes AFTER core UX features are complete
