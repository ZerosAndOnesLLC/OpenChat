# OpenChat: Slack-Killer Implementation Plan

## Context

OpenChat already has a solid foundation: channels, DMs, threads, reactions, mentions, pins, bookmarks, drafts, file attachments, WebSocket-first architecture, RBAC, audit logging, retention/legal hold, custom emojis, incoming webhooks, and a Tauri desktop app. However, it's missing the features that make Slack indispensable: voice/video calling, a workflow builder, mobile apps, E2E encryption, channel organization, and many quality-of-life features. This plan closes every gap and then some.

Each phase = its own branch + PR. No branch reuse.

---

## Phase 1: Background Workers Infrastructure
**Branch:** `feature/background-workers`
**Status:** In Progress

Foundation for scheduled messages, reminders, webhook delivery, and retention enforcement.

### 1.1 Worker Binary
- [x] Create separate binary target in `api/Cargo.toml`: `[[bin]] name = "openchat-worker"`
- [x] Create `api/src/bin/worker.rs` entry point
- [x] Shared PgPool and Redis pool with API
- [x] Redis Streams job queue (`XADD`/`XREADGROUP` consumer group pattern)
- [x] Job types enum: `RetentionEnforcement`, `WebhookDelivery`, `ScheduledMessage`, `Reminder`, `EmailNotification`
- [x] Create migration: `job_queue` table (id, job_type, payload JSONB, status, attempts, max_attempts, scheduled_at, started_at, completed_at, error_message, created_at)
- [x] Graceful shutdown (tokio signal + drain in-flight jobs)
- [x] Exponential backoff retry (1s, 5s, 30s, 5m)
- [x] Structured logging with tracing

### 1.2 Retention Enforcement Worker
- [x] Scheduled daily job (configurable time)
- [x] Query `retention_policies` for active policies per org
- [x] Skip channels with active `legal_holds`
- [x] Batch delete messages in transactions (100 per batch)
- [x] Delete associated files from storage via `StorageFactory`
- [x] Audit log all deletions

### 1.3 Webhook Delivery Worker
- [x] Process webhook jobs from queue
- [x] POST with HMAC-SHA256 signature header
- [x] 3 retries with exponential backoff, 10s timeout
- [x] Create migration: `webhook_deliveries` table (webhook_id, event_type, payload JSONB, status, response_code, delivered_at)
- [x] Dead letter queue for permanently failed deliveries

**Key files:** `api/src/bin/worker.rs` (new), `api/src/tasks/job_queue.rs` (new), `api/src/tasks/retention_worker.rs` (new), `api/src/tasks/webhook_worker.rs` (new)

### Verification
- [x] `cargo check` both binary targets (api + worker)
- [ ] Run worker locally, enqueue a test job via Redis, verify processing
- [ ] Test retention enforcement against dev DB with test retention policy

---

## Phase 2: Scheduled Messages & Reminders
**Branch:** `feature/scheduled-messages-reminders`
**Status:** Complete

### 2.1 Scheduled Messages
- [x] Create migration: `scheduled_messages` table (id, org_id, user_id, channel_id, dm_id, content, parent_message_id, scheduled_at, sent, created_at)
- [x] Index on (scheduled_at, sent)
- [x] Model: `api/src/models/scheduled_message.rs`
- [x] Handler: `api/src/handlers/scheduled_messages.rs`
- [x] API: POST `/api/messages/scheduled`, GET `/api/messages/scheduled`, PUT `/api/messages/scheduled/{id}`, DELETE `/api/messages/scheduled/{id}`
- [x] Worker: poll every 30s for `scheduled_at <= now() AND sent = false`, create message via `Message::create()`
- [x] UI: "Schedule send" clock icon in `MessageInput.tsx` next to send button
- [x] UI: DateTime picker for scheduling (ScheduleSendModal)
- [x] UI: Scheduled messages list view (accessible from sidebar)
- [x] UI: Edit/cancel scheduled messages

### 2.2 Message Reminders
- [x] Create migration: `reminders` table (id, user_id, org_id, message_id, channel_id, dm_id, remind_at, message_preview, completed, created_at)
- [x] Index on (user_id, remind_at, completed)
- [x] Model: `api/src/models/reminder.rs`
- [x] Handler: `api/src/handlers/reminders.rs`
- [x] API: POST `/api/reminders`, GET `/api/reminders`, DELETE `/api/reminders/{id}`
- [x] Worker: poll every 30s, send WebSocket `ReminderTriggered` + create notification
- [x] UI: "Remind me" in `MessageItem.tsx` hover actions
- [x] UI: Quick time options: 30 min, 1 hour, 3 hours, tomorrow 9am, next week, custom
- [x] UI: Reminders section in `NotificationsPanel.tsx`

**Key files:** `api/src/handlers/scheduled_messages.rs` (new), `api/src/handlers/reminders.rs` (new), `api/src/models/scheduled_message.rs` (new), `api/src/models/reminder.rs` (new), `api/src/websocket/messages.rs` (add `ReminderTriggered` variant), `webui/components/MessageInput.tsx`, `webui/components/MessageItem.tsx`

### Verification
- [x] Create scheduled message, verify it sends at the right time
- [x] Create reminder, verify WebSocket notification fires
- [x] Test cancel/edit of scheduled messages
- [x] `cargo check`, `next build`

---

## Phase 3: Channel Sections & User Groups
**Branch:** `feature/channel-sections-user-groups`
**Status:** Complete

### 3.1 Channel Sections/Folders
- [x] Create migration: `channel_sections` table (id, user_id, org_id, name, position INTEGER, collapsed BOOLEAN DEFAULT false, created_at)
- [x] Create migration: `channel_section_items` table (id, section_id, channel_id, position INTEGER)
- [x] Unique index on (section_id, channel_id), (user_id, org_id, name)
- [x] Model: `api/src/models/channel_section.rs`
- [x] Handler: `api/src/handlers/channel_sections.rs`
- [x] API: CRUD for sections + add/remove channels + bulk reorder
- [x] Cache user sections in Redis (5 min TTL)
- [x] UI: Refactor `Sidebar.tsx` to group channels by section with collapsible headers
- [x] UI: Drag-and-drop reordering of channels between sections
- [x] UI: Right-click context menu on section headers (rename, delete)
- [x] UI: Default "Starred" and "Channels" sections lazy-init on first load

### 3.2 User Groups
- [x] Create migration: `user_groups` table (id, org_id, name, handle VARCHAR(50), description, created_by, created_at, updated_at)
- [x] Create migration: `user_group_members` table (id, group_id, user_id, added_at)
- [x] Unique index on (org_id, handle), (group_id, user_id)
- [x] Model: `api/src/models/user_group.rs`
- [x] Handler: `api/src/handlers/user_groups.rs`
- [x] API: CRUD for groups + add/remove members
- [x] Update `api/src/services/mention_parser.rs`: recognize @group-handle, expand to all group members for notifications
- [x] Cache group members in Redis (5 min TTL)
- [x] UI: Update `MentionAutocomplete.tsx` to show groups with group icon
- [x] UI: Admin page for user group management (create, edit members)
- [x] UI: Distinct styling for group mentions in messages (purple highlight)

**Key files:** `api/src/handlers/channel_sections.rs` (new), `api/src/handlers/user_groups.rs` (new), `api/src/models/channel_section.rs` (new), `api/src/models/user_group.rs` (new), `api/src/services/mention_parser.rs`, `webui/components/Sidebar.tsx`, `webui/components/MentionAutocomplete.tsx`, `webui/components/ChannelSectionList.tsx` (new), `webui/components/ContextMenu.tsx` (new), `webui/app/admin/user-groups/page.tsx` (new)

### Verification
- [x] Create sections, drag channels between them, verify persistence on reload
- [x] Create user group, @mention it, verify all members get notifications
- [x] `cargo check`, `npm run build`

---

## Phase 4: Per-Channel Notification Preferences
**Branch:** `feature/notification-preferences`
**Status:** Complete

### 4.1 API
- [x] Create migration: `notification_preferences` table (id, user_id, channel_id, dm_id, preference VARCHAR(20) -- 'all'/'mentions'/'nothing', mute_until TIMESTAMPTZ, created_at, updated_at)
- [x] Unique index on (user_id, channel_id), (user_id, dm_id)
- [x] Model: `api/src/models/notification_pref.rs`
- [x] Handler: `api/src/handlers/notification_prefs.rs`
- [x] API: PUT/GET `/api/channels/{id}/notifications`, PUT/GET `/api/dms/{id}/notifications`
- [x] Cache prefs per user in Redis (5 min TTL)

### 4.2 Notification Filtering
- [x] Modify `api/src/handlers/messages.rs`: check preferences before creating notifications
- [x] "mentions" = only notify on direct @mention or @group mention
- [x] "nothing" = suppress all notifications
- [x] Respect mute_until expiry
- [x] Include preferences in WebSocket `InitialState` so UI renders correctly on load

### 4.3 UI
- [x] Bell icon dropdown in channel header area of `MessageArea.tsx`
- [x] Options: "All messages", "Mentions only", "Mute" (with duration picker: 1h, 8h, 24h, 1 week, forever)
- [x] Muted channel visual: dimmed text + muted bell icon in `ChannelList.tsx`
- [x] Muted channels show gray unread count instead of red badge

**Key files:** `api/src/handlers/notification_prefs.rs` (new), `api/src/models/notification_pref.rs` (new), `api/src/handlers/messages.rs`, `webui/components/MessageArea.tsx`, `webui/components/ChannelList.tsx`

### Verification
- [x] Set channel to "mentions only", send regular message — no notification. Send @mention — notification appears.
- [x] Mute a channel, verify visual indicator and no notifications
- [x] Test mute_until expiry
- [x] `cargo check`, `eslint src/`

---

## Phase 5: Slash Commands & Polls
**Branch:** `feature/slash-commands-polls`
**Status:** Complete

### 5.1 Slash Commands
- [x] Create migration: `slash_commands` table (id, org_id, command_name, description, usage_hint, handler_type -- 'builtin'/'webhook', webhook_url, response_type -- 'ephemeral'/'in_channel', created_by, enabled, created_at)
- [x] Unique index on (org_id, command_name)
- [x] Model: `api/src/models/slash_command.rs`
- [x] Handler: `api/src/handlers/commands.rs`
- [x] API: POST `/api/commands/execute`, GET `/api/commands`, POST/PUT/DELETE CRUD (admin)
- [x] Command parser: detect `/command args` at message start
- [x] Built-in commands: `/shrug`, `/tableflip`, `/me` (action message), `/mute`, `/unmute`
- [x] Webhook commands: POST payload to external URL, display response
- [x] Ephemeral messages: new `ServerMessage::EphemeralMessage` variant (only sender sees it)
- [x] UI: `SlashCommandAutocomplete.tsx` (new) — triggers on `/` at start of input
- [x] UI: Command help panel
- [x] UI: Admin command management page

### 5.2 Polls
- [x] Create migration: `polls` table (id, message_id, org_id, question, options JSONB, poll_type -- 'single'/'multiple', anonymous BOOLEAN, expires_at, created_by, created_at)
- [x] Create migration: `poll_votes` table (id, poll_id, user_id, option_index INTEGER, voted_at)
- [x] Unique constraint for single-vote polls: (poll_id, user_id)
- [x] Model: `api/src/models/poll.rs`
- [x] Handler: `api/src/handlers/polls.rs`
- [x] API: POST `/api/polls`, POST `/api/polls/{id}/vote`, DELETE `/api/polls/{id}/vote`, GET `/api/polls/{id}/results`
- [x] WebSocket: `PollVoteUpdated` server message for real-time count updates
- [x] Slash command: `/poll "Question?" "Opt1" "Opt2" ...`
- [x] UI: `PollCreator.tsx` (new) — triggered by `/poll` or toolbar button
- [x] UI: `PollDisplay.tsx` (new) — embedded in `MessageItem.tsx`, progress bars, vote buttons
- [x] UI: Anonymous poll support (hide voter names)
- [x] UI: Real-time vote count updates

**Key files:** `api/src/handlers/commands.rs` (new), `api/src/handlers/polls.rs` (new), `api/src/models/slash_command.rs` (new), `api/src/models/poll.rs` (new), `webui/components/SlashCommandAutocomplete.tsx` (new), `webui/components/PollCreator.tsx` (new), `webui/components/PollDisplay.tsx` (new), `webui/components/MessageItem.tsx`

### Verification
- [x] Type `/shrug` — verify appended to message
- [x] Create custom webhook command, verify external POST + response display
- [x] Create poll, vote, verify real-time updates across multiple users
- [x] Test anonymous poll — voter names hidden
- [x] `cargo check`, `eslint src/`

---

## Phase 6: Message Forwarding & UX Polish
**Branch:** `feature/forwarding-ux-polish`
**Status:** Complete

### 6.1 Message Forwarding
- [x] Create migration: add `forwarded_from_message_id UUID`, `forwarded_from_channel_id UUID` to messages
- [x] Add forward endpoint in `api/src/handlers/messages.rs`: POST `/api/messages/{id}/forward` (body: channel_id/dm_id + optional comment)
- [x] Forward creates new message with forwarded metadata
- [x] UI: "Forward" button in `MessageItem.tsx` hover actions
- [x] UI: `ForwardMessageModal.tsx` (new) — channel/DM picker with search + optional comment
- [x] UI: Forwarded messages render with "Forwarded from #channel" attribution bar

### 6.2 Animations & Transitions
- [x] CSS transitions on sidebar items (hover, selection)
- [x] Message appear animation (subtle slide-up + fade-in)
- [x] Modal open/close animation (scale + fade)
- [x] Sidebar collapse/expand animation
- [x] Toast slide-in/out in `ToastProvider.tsx`
- [x] Typing indicator bouncing dots animation
- [x] Thread panel slide-in animation
- [x] Respect `prefers-reduced-motion` media query

### 6.3 Notification Sounds & Browser Integration
- [x] Multiple notification sound options in settings
- [x] Different sounds for mentions vs regular messages
- [x] Browser tab title badge: "(3) OpenChat"
- [x] Dynamic favicon with unread count
- [x] Browser Notification API for background tab alerts
- [x] Sound picker component in `webui/components/settings/`

### 6.4 Accessibility
- [x] ARIA labels on all interactive elements
- [x] Keyboard navigation in message list (arrow keys)
- [x] Screen reader announcements for new messages (aria-live region)
- [x] Focus trap in all modals
- [x] High contrast CSS custom properties

**Key files:** `api/src/handlers/messages.rs`, `api/src/models/message.rs`, `webui/components/ForwardMessageModal.tsx` (new), `webui/components/MessageItem.tsx`, `webui/app/globals.css`, `webui/hooks/useNotificationSound.ts`, `webui/components/settings/`

### Verification
- [x] Forward message to another channel, verify attribution renders
- [x] Test all animations (enable/disable reduced motion)
- [x] Verify browser notifications work when tab is background
- [x] Screen reader test on message list
- [x] `cargo check`, `eslint src/`

---

## Phase 7: Voice/Video Calling (LiveKit)
**Branch:** `feature/voice-video-livekit`
**Status:** Not Started

### 7.1 Infrastructure
- [ ] Add LiveKit server to infra (Docker/ECS — open-source Go binary)
- [ ] Terraform: ECS service for LiveKit, ALB listener, security groups
- [ ] Env vars: `LIVEKIT_URL`, `LIVEKIT_API_KEY`, `LIVEKIT_API_SECRET`
- [ ] Add `livekit-api` crate to `api/Cargo.toml`

### 7.2 Calling API
- [ ] Create migration: `calls` table (id, org_id, channel_id, dm_id, call_type -- 'audio'/'video', status -- 'ringing'/'active'/'ended', started_by, started_at, ended_at, livekit_room_name)
- [ ] Create migration: `call_participants` table (id, call_id, user_id, joined_at, left_at, muted, video_off)
- [ ] Model: `api/src/models/call.rs`
- [ ] Handler: `api/src/handlers/calls.rs`
- [ ] Service: `api/src/services/livekit.rs` — room creation, token generation
- [ ] API: POST `/api/calls/start` (creates LiveKit room, returns join token)
- [ ] API: POST `/api/calls/{id}/join` (returns LiveKit token)
- [ ] API: POST `/api/calls/{id}/leave`, POST `/api/calls/{id}/end`
- [ ] API: GET `/api/calls/active` (active calls in user's channels)
- [ ] WebSocket events: `CallStarted`, `CallEnded`, `CallParticipantJoined`, `CallParticipantLeft`, `CallRinging`

### 7.3 Call UI
- [ ] Add `livekit-client` npm package to webui
- [ ] Call buttons (phone + video) in channel/DM header in `MessageArea.tsx`
- [ ] `IncomingCallBanner.tsx` (new) — accept/decline with ringtone
- [ ] `CallOverlay.tsx` (new) — participant tiles, controls (mute, video toggle, screen share, end)
- [ ] Active call indicator in sidebar (green phone icon on channel)
- [ ] Picture-in-picture when navigating away from call channel
- [ ] Call duration timer
- [ ] Screen sharing via LiveKit track publishing

### 7.4 Huddles
- [ ] API: POST `/api/channels/{id}/huddle/start`, `/join`, `/leave`
- [ ] Huddles = persistent audio-only rooms, no ringing, join/leave at will
- [ ] UI: Headphone icon in channel header
- [ ] `HuddleBar.tsx` (new) — bottom bar showing active participants, click to join/leave

**Key files:** `api/src/handlers/calls.rs` (new), `api/src/models/call.rs` (new), `api/src/services/livekit.rs` (new), `api/src/websocket/messages.rs`, `webui/components/CallOverlay.tsx` (new), `webui/components/IncomingCallBanner.tsx` (new), `webui/components/HuddleBar.tsx` (new), `webui/lib/livekit.ts` (new)

### Verification
- [ ] Start audio call in a channel, second user joins — verify audio works
- [ ] Start video call in DM, verify video tiles render
- [ ] Screen share — verify other participants see shared screen
- [ ] Huddle: join/leave without ringing, verify persistent room
- [ ] Test with 5+ participants
- [ ] `cargo check`, `eslint src/`

---

## Phase 8: Workflow Builder
**Branch:** `feature/workflow-builder`
**Status:** Not Started

### 8.1 Workflow Engine (API)
- [ ] Create migration: `workflows` table (id, org_id, name, description, trigger_type, trigger_config JSONB, enabled, created_by, created_at, updated_at)
- [ ] Create migration: `workflow_steps` table (id, workflow_id, step_order INTEGER, action_type, action_config JSONB, created_at)
- [ ] Create migration: `workflow_executions` table (id, workflow_id, trigger_data JSONB, status, started_at, completed_at, error_message)
- [ ] Create migration: `workflow_execution_steps` table (id, execution_id, step_id, status, input_data JSONB, output_data JSONB, started_at, completed_at)
- [ ] Trigger types: `message_posted`, `reaction_added`, `channel_join`, `scheduled`, `webhook`, `slash_command`
- [ ] Action types: `send_message`, `create_form`, `call_webhook`, `add_reaction`, `create_channel`, `invite_to_channel`, `update_channel_topic`, `delay`
- [ ] Model: `api/src/models/workflow.rs`
- [ ] Handler: `api/src/handlers/workflows.rs`
- [ ] Engine: `api/src/services/workflow_engine.rs` — sequential step execution with variable interpolation
- [ ] API: Full CRUD for workflows + steps + enable/disable + execution history
- [ ] Hook into `handlers/messages.rs`: check `message_posted` triggers after message creation
- [ ] Hook into `handlers/reactions.rs`: check `reaction_added` triggers
- [ ] Hook into `handlers/channels.rs`: check `channel_join` triggers
- [ ] Scheduled triggers via background worker
- [ ] Webhook trigger endpoint: POST `/api/workflows/webhook/{workflow_id}`

### 8.2 Workflow Builder UI
- [ ] Admin page: `webui/app/admin/workflows/page.tsx`
- [ ] `WorkflowBuilder.tsx` (new) — visual step builder, drag-and-drop step reordering
- [ ] `WorkflowStepEditor.tsx` (new) — action configurator per step (channel picker, message template, webhook URL)
- [ ] Trigger configurator (type selector + config form per type)
- [ ] Variable interpolation: `{{user.name}}`, `{{message.content}}`, `{{channel.name}}`
- [ ] Execution history viewer with step-by-step status
- [ ] Test workflow button (dry run)
- [ ] Enable/disable toggle

### 8.3 Workflow Forms
- [ ] Create migration: `workflow_forms` table (id, workflow_id, step_id, title, fields JSONB, submitted_by, submitted_data JSONB, created_at)
- [ ] Field types: text, textarea, select, multi_select, date, user_picker, channel_picker
- [ ] API: POST `/api/forms/{id}/submit`, GET `/api/forms/{id}`
- [ ] WebSocket: `FormRequested` sent to target user
- [ ] `FormModal.tsx` (new) — dynamic form renderer
- [ ] Form submission triggers next workflow step

**Key files:** `api/src/handlers/workflows.rs` (new), `api/src/models/workflow.rs` (new), `api/src/services/workflow_engine.rs` (new), `webui/app/admin/workflows/page.tsx` (new), `webui/components/WorkflowBuilder.tsx` (new), `webui/components/WorkflowStepEditor.tsx` (new), `webui/components/FormModal.tsx` (new)

### Verification
- [ ] Create workflow: "when message posted in #general containing 'help' → send DM to poster with help info"
- [ ] Create workflow: "when reaction :white_check_mark: added → send webhook to external service"
- [ ] Create workflow with form: trigger → collect form → post summary to channel
- [ ] Test scheduled trigger fires at correct time
- [ ] View execution history, verify step statuses
- [ ] `cargo check`, `eslint src/`

---

## Phase 9: E2E Encryption
**Branch:** `feature/e2e-encryption`
**Status:** Not Started

### 9.1 Crypto Infrastructure (API)
- [ ] Add `vodozemac` crate (open-source Rust, Matrix Olm/Megolm)
- [ ] Create migration: `user_devices` table (id, user_id, device_name, identity_key, signing_key, one_time_keys JSONB, last_seen, created_at)
- [ ] Create migration: `encrypted_channels` table (channel_id, encryption_enabled BOOLEAN, algorithm, created_at)
- [ ] Create migration: add `encrypted_content BYTEA`, `encryption_metadata JSONB` to messages
- [ ] Create migration: `encryption_sessions` table (session_id, channel_id, dm_id, algorithm, created_at, rotated_at)
- [ ] Create module: `api/src/crypto/` (mod.rs, device.rs, keys.rs, session.rs)
- [ ] Handler: `api/src/handlers/crypto.rs`
- [ ] API: device registration, key upload, key claim, session info endpoints

### 9.2 Encrypted Message Handling
- [ ] Accept `encrypted_content` + `encryption_metadata` in message creation (server stores blob, never decrypts)
- [ ] Validation of encryption metadata format
- [ ] Full-text search disabled for encrypted messages (by design)
- [ ] WebSocket forwards encrypted messages as-is

### 9.3 Frontend Crypto
- [ ] Add `@matrix-org/olm` (or vodozemac WASM bindings)
- [ ] `webui/lib/crypto.ts` (new) — crypto store, encrypt/decrypt, key management
- [ ] IndexedDB store for device keys, session keys
- [ ] Device registration on first login
- [ ] Olm sessions for DMs, Megolm sessions for channels
- [ ] Encrypt before send / decrypt on receive
- [ ] Key backup/recovery (passphrase-based)

### 9.4 Encryption UX
- [ ] Lock icon on encrypted channels in sidebar
- [ ] "Enable E2E Encryption" toggle in channel settings (admin, irreversible)
- [ ] Device list in user settings (verify/remove)
- [ ] Device verification flow (safety numbers)
- [ ] Key backup setup wizard
- [ ] Warning for unverified devices
- [ ] "Encryption enabled" banner in encrypted channels

**Key files:** `api/src/crypto/` (new module), `api/src/handlers/crypto.rs` (new), `api/src/handlers/messages.rs`, `api/src/models/message.rs`, `webui/lib/crypto.ts` (new), `webui/components/EncryptionSettings.tsx` (new), `webui/components/DeviceVerification.tsx` (new)

### Verification
- [ ] Enable encryption on channel, send message — verify stored as encrypted blob in DB
- [ ] Second user decrypts message successfully
- [ ] Multi-device: register 2 devices, verify both can decrypt
- [ ] Key backup: backup keys, restore on new device, verify decryption
- [ ] Verify non-encrypted channels unaffected
- [ ] `cargo check`, `eslint src/`

---

## Phase 10: Flutter Mobile & Desktop App
**Branch:** `feature/flutter-app`
**Status:** Not Started

### 10.1 Project Setup
- [ ] Initialize Flutter project at `openchat/flutter/`
- [ ] Structure: `lib/core/`, `lib/features/`, `lib/shared/`
- [ ] Dependencies: dio, web_socket_channel, flutter_secure_storage, riverpod, go_router, freezed
- [ ] Platform configs: Android (min SDK 24), iOS (min 15), macOS, Windows
- [ ] CI/CD: GitHub Actions for APK, IPA, macOS, Windows builds

### 10.2 Core Infrastructure
- [ ] API client (Dio) matching webui's `ApiClient`
- [ ] WebSocket client with auto-reconnect
- [ ] Riverpod state management (mirror Zustand store)
- [ ] OAuth 2.0 PKCE flow for TitaniumVault SSO
- [ ] Secure token storage (flutter_secure_storage)
- [ ] Deep linking for OAuth callback
- [ ] Offline message queue (drift/sqflite)
- [ ] Push notifications: FCM (Android) + APNs (iOS)
- [ ] New API endpoint: POST `/api/devices/push-token` for push token registration

### 10.3 Feature Screens
- [ ] Login/SSO screen
- [ ] Main layout: sidebar (drawer on mobile, fixed on desktop) + content
- [ ] Channel list with sections and unread badges
- [ ] DM list with online indicators
- [ ] Message list: text, attachments, polls, forwarded, encrypted
- [ ] Thread view (slide-over panel)
- [ ] Message input: attachments, mentions, emoji, slash commands
- [ ] Search, notifications, user profile, settings screens
- [ ] Channel/DM creation
- [ ] Call UI (livekit_client Flutter package)
- [ ] Encryption support (vodozemac Dart bindings or FFI)

### 10.4 Platform-Specific
- [ ] Android: share extension, notification channels
- [ ] iOS: share extension, notification badges
- [ ] macOS: system tray, native notifications, menu bar
- [ ] Windows: system tray, native notifications, auto-start

### 10.5 Testing & Release
- [ ] Unit tests for API client, state, crypto
- [ ] Widget tests for major screens
- [ ] Integration tests (E2E flow)
- [ ] Android: signed APK/AAB for Play Store
- [ ] iOS: TestFlight + App Store
- [ ] macOS: DMG, Windows: MSIX

**Key files:** `openchat/flutter/` (entire new project), `api/src/handlers/push_tokens.rs` (new)

### Verification
- [ ] Login on Android/iOS, verify WebSocket connects and messages flow
- [ ] Send message from Flutter, receive on web — and vice versa
- [ ] Join call from Flutter app
- [ ] Push notification when app is backgrounded
- [ ] Desktop: system tray icon works on macOS and Windows

---

## Phase 11: Performance, Testing & Monitoring
**Branch:** `feature/perf-testing-monitoring`
**Status:** Not Started

### 11.1 API Test Coverage
- [ ] Unit tests for all new models (`sqlx::test`)
- [ ] Integration tests for all handler endpoints
- [ ] WebSocket integration tests (connect, subscribe, message flow)
- [ ] Load test: k6 scripts — 10,000 concurrent WebSocket connections
- [ ] Load test: 1,000 messages/second throughput
- [ ] Encryption round-trip tests
- [ ] Workflow execution tests

### 11.2 UI Test Coverage
- [ ] Component tests (React Testing Library) for major components
- [ ] WebSocket store tests (mock WebSocket)
- [ ] E2E tests with Playwright (login → send message → receive → call flow)

### 11.3 Monitoring
- [ ] Add `prometheus` crate for metrics export
- [ ] Metrics: message throughput, WS connections, API latency (p50/p95/p99), error rates, cache hit rates, active calls
- [ ] CloudWatch integration
- [ ] Request ID tracing across WebSocket + HTTP
- [ ] Alerting: high error rate, high latency, connection spike

### 11.4 Performance Optimization
- [ ] Flamegraph profiling of API hot paths
- [ ] `EXPLAIN ANALYZE` for all new queries
- [ ] Connection pool sizing review (PgPool + Redis)
- [ ] WebSocket memory profiling under load

### Verification
- [ ] All tests pass
- [ ] Load test: 10k WS connections sustained without OOM
- [ ] Metrics endpoint returns valid Prometheus format
- [ ] Version bump to 1.0.0

---

## Phase Dependency Order

```
Phase 1 (Workers) → Phase 2 (Scheduled/Reminders)
Phase 3 (Sections/Groups) → independent
Phase 4 (Notif Prefs) → independent (benefits from Phase 3)
Phase 5 (Commands/Polls) → Phase 8 (Workflows, uses slash commands as trigger)
Phase 6 (Forward/Polish) → independent
Phase 7 (Voice/Video) → independent
Phase 9 (E2E Encryption) → after all message features (Phases 2-6)
Phase 10 (Flutter) → after all API endpoints stable (Phases 1-9)
Phase 11 (Testing) → after all features (Phases 1-10)
```

Recommended execution order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11
