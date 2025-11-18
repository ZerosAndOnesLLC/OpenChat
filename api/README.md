# OpenChat API

Open-source team chat application backend - similar to Slack/Mattermost.

## Tech Stack

- **Language**: Rust 2024 Edition
- **Framework**: Actix-web 4.x
- **Database**: PostgreSQL with SQLx
- **Cache/Pub-Sub**: Redis
- **Authentication**: TitaniumVault SSO integration
- **Real-time**: WebSockets (actix-ws) with Redis Pub/Sub

## Features

- Multi-tenant architecture with org-level isolation
- Row Level Security (RLS) for defense-in-depth
- Role-based access control (openchat and openchat-admin roles)
- Real-time messaging via WebSockets
- Public and private channels
- Direct messages (1-on-1 and groups)
- Message threading (replies to messages)
- Message reactions and editing
- User presence and typing indicators
- Horizontally scalable WebSocket support

## Getting Started

### Prerequisites

- Rust 1.91.0 or later
- PostgreSQL 14+
- Redis 6+

### Setup

1. Clone the repository and navigate to the API directory:
   ```bash
   cd openchat/api
   ```

2. Copy `.env` and configure:
   ```bash
   cp .env.example .env
   ```

3. Set up the database:
   ```bash
   # Install sqlx-cli if not already installed
   cargo install sqlx-cli --no-default-features --features postgres

   # Create database and run migrations
   sqlx database create
   sqlx migrate run
   ```

4. Run the server:
   ```bash
   cargo run
   ```

The server will start on `http://0.0.0.0:8080` (configurable via `PORT` env var).

## Development

### Running Checks

```bash
# Type checking
cargo check

# Run tests
cargo test

# Build for production
cargo build --release
```

### Project Structure

```
src/
├── main.rs          # Application entry point
├── config.rs        # Configuration management
├── errors.rs        # Custom error types
├── routes/          # API route definitions (Phase 5+)
├── models/          # Database models (Phase 4+)
├── handlers/        # Request handlers (Phase 4+)
├── middleware/      # Auth and other middleware (Phase 3+)
├── services/        # Business logic (Phase 3+)
├── websocket/       # WebSocket handling (Phase 9+)
└── cache/           # Redis caching layer (Phase 4+)
```

## API Endpoints

### Health Check
- `GET /health` - Returns server health status

### SSO Authentication
- `POST /api/sso/exchange` - Exchange OAuth authorization code for access token
- `POST /api/sso/userinfo` - Get user info from TitaniumVault (proxy endpoint)

**Note**: All API endpoints except SSO routes require authentication with the "openchat" role.

### Channels (Phase 5+)
- `GET /api/channels` - List channels
- `POST /api/channels` - Create channel
- `GET /api/channels/:id` - Get channel details
- `PUT /api/channels/:id` - Update channel
- `DELETE /api/channels/:id` - Delete channel

### Messages (Phase 6+)
- `POST /api/messages` - Send message to channel or DM (supports `parent_message_id` for threading)
- `GET /api/channels/:id/messages` - List channel messages (paginated, includes `reply_count`)
- `GET /api/messages/:id/thread` - Get thread messages (parent message + all replies)
- `PUT /api/messages/:id` - Edit message
- `DELETE /api/messages/:id` - Soft delete message
- `POST /api/messages/:id/reactions` - Add reaction to message
- `GET /api/messages/:id/reactions` - List reactions on message
- `GET /api/messages/:id/reactions/counts` - Get reaction counts
- `DELETE /api/messages/:id/reactions/:emoji` - Remove reaction

### File Attachments
- `POST /api/attachments/upload` - Upload file attachment (multipart/form-data with message_id and file)
- `GET /api/attachments/:id/download` - Download attachment
- `DELETE /api/attachments/:id` - Delete attachment (only message owner)
- `GET /api/messages/:id/attachments` - List message attachments

**Features**:
- Configurable storage backend (Local filesystem or S3)
- Per-organization storage settings in `storage_settings` table
- File type validation (images, documents, videos, audio)
- File size limits (configurable, default: 25MB)
- Support for S3-compatible storage providers
- Automatic file path generation with UUID and timestamp
- Storage metadata tracked in `attachments` table

**Storage Configuration**:
- **Local Storage**: Files stored in `LOCAL_STORAGE_PATH` (default: `/var/openchat/uploads`)
- **S3 Storage**: Configure per-org S3 bucket, region, and credentials in `storage_settings` table
- Default storage type: Local (no additional configuration required)

### Direct Messages (Phase 7+)
- `GET /api/dms` - List user's DMs
- `POST /api/dms` - Create DM (1-on-1 or group)
- `GET /api/dms/:id` - Get DM details
- `GET /api/dms/:id/messages` - List DM messages (paginated)

### Unread Message Tracking
- `POST /api/channels/:id/read` - Mark channel as read
- `GET /api/channels/:id/unread` - Get unread message count for channel
- `POST /api/dms/:id/read` - Mark DM as read
- `GET /api/dms/:id/unread` - Get unread message count for DM

**Features**:
- Tracks last read message and timestamp per user per channel/DM
- Calculates unread count by comparing last read timestamp with message timestamps
- Redis caching for unread counts (60-second TTL)
- Cache invalidation on mark-as-read operations
- WebSocket support for unread count updates

### Message Search
- `GET /api/search/messages?q={query}&scope={channel|dm|all}&channel_id={id}` - Full-text search messages

**Features**:
- PostgreSQL full-text search with GIN index
- Automatic tsvector updates via database trigger
- Search scopes: channel-specific, DM-specific, or all messages
- Redis caching for search results (1-minute TTL)

### Mentions & Notifications
- `GET /api/mentions` - List user's mentions (paginated)
- `GET /api/mentions/unread-count` - Get count of unread mentions
- `GET /api/notifications?unread_only={bool}` - List notifications (paginated)
- `GET /api/notifications/unread-count` - Get unread notification count
- `POST /api/notifications/:id/read` - Mark notification as read
- `POST /api/notifications/read-all` - Mark all notifications as read

**Features**:
- Automatic mention detection: @username, @channel, @here, @everyone
- Case-insensitive user lookup by display name
- Thread reply notifications
- Mention notifications for user and channel-wide mentions
- Self-notification prevention
- Unread mention and notification counters

### Message Pinning & Bookmarks
- `POST /api/messages/:id/pin` - Pin message to channel (requires permission)
- `DELETE /api/messages/:id/pin` - Unpin message from channel
- `GET /api/channels/:id/pins` - List pinned messages in channel
- `POST /api/bookmarks` - Bookmark a message for later
- `DELETE /api/bookmarks/:message_id` - Remove bookmark
- `GET /api/bookmarks` - List user's bookmarked messages

**Features**:
- Channel-level message pinning (admin/moderator only)
- Personal message bookmarks (private to user)
- WebSocket events for pin/unpin actions
- Bookmarks persist across devices

### User Status & Presence
- `PUT /api/users/me/status` - Update status with custom message and emoji
- `POST /api/users/me/status/online` - Quick set status to online
- `POST /api/users/me/status/away` - Quick set status to away
- `POST /api/users/me/status/offline` - Quick set status to offline
- `GET /api/users/:id/status` - Get user's current status
- `GET /api/users/status/active` - Get all active users in organization

**Features**:
- Status types: online, away, dnd (do not disturb), offline
- Custom status messages with emoji support
- Auto-clear status after specified time
- Auto-away after 15 minutes of inactivity
- Redis caching for status data (5-minute TTL)
- WebSocket broadcasting of status changes
- Activity tracking via WebSocket heartbeat
- Background task for auto-away and expired status cleanup

### Read Receipts
- `POST /api/messages/:id/read` - Record read receipt for a message
- `GET /api/messages/:id/receipts` - Get all read receipts for a message (includes user details)
- `POST /api/read-receipts/batch` - Record read receipts for multiple messages at once

**Features**:
- Track who has read each message (Slack-style "seen by" feature)
- Privacy setting: users can disable sending read receipts (`disable_read_receipts` field in users table)
- Batch read receipt recording for marking multiple messages as read efficiently
- Read receipts include user details (display name, avatar) for UI display
- WebSocket broadcasting of read receipts to message senders in real-time
- Row Level Security (RLS) ensures users can only see receipts for messages they have access to
- Unique constraint prevents duplicate receipts (automatically updates timestamp on conflict)
- Efficient database indexes for querying receipts by message or user
- Supports both channel messages and direct messages

**Privacy**:
- Users who disable read receipts (`disable_read_receipts = true`) will not send receipts
- Read receipt API calls for users with disabled receipts return 204 No Content
- Other users can still see read receipts from users who haven't disabled the feature

### Message Editing History
- `GET /api/messages/:id/history` - Get edit history for a message

**Features**:
- Tracks complete edit history for all messages
- Stores previous content, editor, and timestamp for each edit
- Automatic history recording when messages are updated via `PUT /api/messages/:id`
- Edit history is preserved in the `message_edits` table
- Efficient database indexes for querying edit history
- Row Level Security (RLS) ensures users can only view edit history for messages they have access to
- Messages display `edited_at` timestamp when they've been modified

**Database Schema**:
- `message_edits` table:
  - `id` (UUID): Unique identifier for the edit record
  - `message_id` (UUID): Reference to the edited message
  - `old_content` (TEXT): The content before the edit
  - `edited_by` (UUID): User who made the edit
  - `edited_at` (TIMESTAMPTZ): When the edit occurred
  - Index on `(message_id, edited_at DESC)` for efficient history retrieval

**Implementation Details**:
- Message updates use database transactions to ensure atomicity
- Old content is saved to `message_edits` before updating the message
- Edit history is ordered by `edited_at` DESC (newest first)
- Permission checks ensure users can only view history for messages in their channels/DMs

### WebSocket (Phase 9+)
- `WS /api/ws?token=<jwt>` - WebSocket connection for real-time messaging
  - Real-time message delivery
  - Typing indicators
  - User presence (online/offline/away)
  - Channel subscriptions
  - Heartbeat/ping-pong

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | Yes |
| `REDIS_URL` | Redis connection string | Yes |
| `TV_API_URL` | TitaniumVault API URL | Yes |
| `OAUTH_CLIENT_ID` | OAuth client ID for SSO | Yes |
| `OAUTH_CLIENT_SECRET` | OAuth client secret for SSO | Yes |
| `OAUTH_REDIRECT_URI` | OAuth redirect URI for SSO callback | Yes |
| `PORT` | Server port (default: 8080) | No |
| `HOST` | Server host (default: 0.0.0.0) | No |
| `ENABLE_TLS` | Enable TLS (default: false) | No |
| `TLS_CERT_PATH` | Path to TLS certificate | No |
| `TLS_KEY_PATH` | Path to TLS private key | No |
| `RUST_LOG` | Logging level | No |
| `LOCAL_STORAGE_PATH` | Local file storage path (default: /var/openchat/uploads) | No |
| `MAX_FILE_SIZE` | Maximum file upload size in bytes (default: 26214400 / 25MB) | No |
| `ALLOWED_FILE_TYPES` | Comma-separated allowed file types (default: images,documents,videos,audio) | No |

## Deployment

This service is designed to run in AWS ECS containers:

- **Container Registry**: Amazon ECR
- **Container Orchestration**: ECS Fargate
- **Load Balancer**: Application Load Balancer
- **Logs**: CloudWatch Logs at `/ecs/0n1-us-east-1/openchat-service`

Infrastructure is managed via Terraform in `~/dev/terraform/prod/us-east-1/openchat/`.

## Current Status

**Phase 1 Complete** - Project setup and foundation
- ✅ Rust project initialized with 2024 edition
- ✅ All dependencies configured
- ✅ Basic Actix-web server with health check
- ✅ Configuration management
- ✅ Error handling framework

**Phase 2 Complete** - Database schema and Row Level Security
- ✅ PostgreSQL schema with all core tables
- ✅ Database migrations using SQLx
- ✅ Row Level Security (RLS) policies for org isolation
- ✅ Database connection pool setup
- ✅ RLS context helper for middleware

**Phase 3 Complete** - Authentication & Middleware
- ✅ TitaniumVault API integration for token verification
- ✅ JWT token extraction from Authorization header
- ✅ Authentication middleware with org_id extraction
- ✅ Role-based authorization (openchat and openchat-admin roles)
- ✅ Automatic RLS context setting per request
- ✅ User model with database operations
- ✅ User upsert on authentication
- ✅ Middleware applied to all protected API routes

**Phase 4 Complete** - User & Organization Management
- ✅ Organization model with CRUD operations
- ✅ User management endpoints (list, get, update)
- ✅ User profile update endpoint
- ✅ User status update endpoint (online/offline/away)
- ✅ Redis caching infrastructure for user data
- ✅ Cache invalidation on updates
- ✅ RESTful API routes wired up

**Phase 5 Complete** - Channel Management
- ✅ Channel model with full CRUD operations
- ✅ Channel member management model
- ✅ Create, read, update, delete channels
- ✅ Add/remove members from channels
- ✅ Public and private channel support
- ✅ Redis caching for channels and members
- ✅ Authorization (only creators can manage channels)

**Phase 6 Complete** - Messaging (REST)
- ✅ Message model with CRUD operations
- ✅ Send messages to channels and DMs
- ✅ Cursor-based pagination for message history
- ✅ Edit and soft delete messages
- ✅ Authorization (users can only edit/delete their own messages)
- ✅ Redis caching infrastructure for messages
- ✅ Member verification for channel messages
- ✅ RESTful API endpoints for messaging

**Phase 7 Complete** - Direct Messages (REST)
- ✅ DirectMessage and DmParticipant models
- ✅ Create DMs (1-on-1 and group DMs)
- ✅ Automatic DM deduplication (reuses existing DMs with same participants)
- ✅ List user's DMs with participant information
- ✅ Get DM details with authorization
- ✅ List DM messages with pagination
- ✅ Participant verification for all DM operations
- ✅ Redis caching infrastructure for DMs
- ✅ Cross-org DM prevention

**Phase 8 Complete** - Reactions
- ✅ Reaction model with emoji reactions
- ✅ Add reaction to messages (with duplicate handling)
- ✅ Remove reaction from messages
- ✅ List all reactions on a message
- ✅ Get aggregated reaction counts by emoji
- ✅ Authorization checks (channel member or DM participant)
- ✅ Support for multiple users reacting with same emoji
- ✅ UNIQUE constraint prevents duplicate reactions

**Phase 9 Complete** - WebSocket Basic (Single Instance)
- ✅ WebSocket session management with actix-ws
- ✅ In-memory connection registry (user → sessions mapping)
- ✅ JWT authentication via query parameter
- ✅ Real-time message broadcasting
- ✅ Typing indicators
- ✅ User presence tracking (online/offline)
- ✅ Channel subscription system
- ✅ Heartbeat/ping-pong for connection health
- ✅ Multi-device support (multiple sessions per user)
- ✅ Automatic cleanup on disconnect

**Phase 10 Complete** - WebSocket Advanced Features
- ✅ Real-time typing indicators with broadcast
- ✅ Channel subscription/unsubscription
- ✅ User status updates (online/offline/away)
- ✅ Status broadcast to organization members
- ✅ Message broadcasting infrastructure
- ✅ WebSocket message type system (Client ↔ Server)
- ✅ Error handling and validation
- ✅ Connection health monitoring

**Phase 11 Complete** - WebSocket Scaling (Redis Pub/Sub)
- ✅ Redis Pub/Sub integration for cross-instance messaging
- ✅ PubSubEvent system for serializable events
- ✅ Pattern-based subscription (openchat:*)
- ✅ Automatic reconnection on Redis failure
- ✅ Message broadcasting across ECS instances
- ✅ Typing indicator distribution
- ✅ Status update synchronization
- ✅ Reaction event broadcasting
- ✅ Graceful degradation (works without Redis)
- ✅ Horizontal scaling support for production

**Phase 1.1 Complete** - Unread Message Tracking
- ✅ Database migrations for channel_read_status and dm_read_status tables
- ✅ ReadStatus models with mark_as_read and get_unread_count methods
- ✅ Redis caching for unread counts with 60-second TTL
- ✅ API endpoints: POST /channels/:id/read, GET /channels/:id/unread
- ✅ API endpoints: POST /dms/:id/read, GET /dms/:id/unread
- ✅ Cache invalidation on mark-as-read operations
- ✅ WebSocket message type for unread count updates
- ✅ UI API client methods for unread endpoints

**Phase 1.2 Complete** - Thread Display UI
- ✅ Inline thread preview showing first reply with user name
- ✅ Thread side panel with all replies
- ✅ Thread breadcrumb navigation showing author and reply count
- ✅ "Reply in thread" button in message hover actions
- ✅ Real-time thread updates via polling (2-second interval)
- ✅ Batch fetching of first replies for performance
- ✅ Thread count badge on parent messages

**Phase 1.3 Complete** - File Attachments with Configurable Storage
- ✅ Database migrations for attachments and storage_settings tables
- ✅ FileStorage trait with Local and S3 implementations
- ✅ Local storage handler (configurable path)
- ✅ S3 storage handler using aws-sdk-s3
- ✅ Storage factory based on org settings (default: local)
- ✅ API endpoints: POST /attachments/upload, GET /attachments/:id/download, DELETE /attachments/:id
- ✅ API endpoint: GET /messages/:id/attachments
- ✅ File type validation (images, documents, videos, audio)
- ✅ File size limits (configurable, default: 25MB)
- ✅ Environment variables for storage configuration
- ✅ UI API client methods for upload/download

**Phase 1.4 Complete** - Message Search (Full-Text)
- ✅ Database migration adding GIN index on messages.content
- ✅ Full-text search using PostgreSQL tsvector
- ✅ Auto-updating tsvector column with trigger
- ✅ API endpoint: GET /api/search/messages?q={query}&scope={channel|dm|all}&channel_id={id}
- ✅ Redis caching for search results (1-minute TTL)

**Phase 1.5 Complete** - @Mentions & Notifications
- ✅ Database migrations for mentions and notifications tables
- ✅ Mention and Notification models with CRUD operations
- ✅ Mention parser for @username, @channel, @here, @everyone
- ✅ User lookup by display name (case-insensitive)
- ✅ Automatic mention parsing on message creation
- ✅ Notification creation for user mentions
- ✅ Notification creation for @channel/@here/@everyone mentions
- ✅ Thread reply notifications
- ✅ API endpoints: GET /api/mentions, GET /api/mentions/unread-count
- ✅ API endpoints: GET /api/notifications, GET /api/notifications/unread-count
- ✅ API endpoints: POST /api/notifications/:id/read, POST /api/notifications/read-all
- ✅ Regex-based mention detection in message content
- ✅ Prevention of self-notification

**Phase 2.1 Complete** - Message Pinning & Bookmarks
- ✅ Database migrations for pinned_messages and bookmarks tables
- ✅ PinnedMessage and Bookmark models with CRUD operations
- ✅ API endpoints: POST /api/messages/:id/pin, DELETE /api/messages/:id/pin
- ✅ API endpoint: GET /api/channels/:id/pins - List pinned messages
- ✅ API endpoints: POST /api/bookmarks, DELETE /api/bookmarks/:message_id
- ✅ API endpoint: GET /api/bookmarks - List user's bookmarks
- ✅ WebSocket broadcasting for pin/unpin events

**Phase 2.2 Complete** - Rich Text Formatting (Markdown)
- ✅ Messages stored as text/markdown in database
- ✅ UI: Markdown preview toggle in message input
- ✅ UI: Markdown toolbar (bold, italic, code, list, link, quote)
- ✅ UI: Markdown rendering with react-markdown
- ✅ UI: Syntax highlighting with react-syntax-highlighter
- ✅ UI: Support for bold, italic, code, lists, links, quotes, headings
- ✅ Security: HTML sanitization with rehype-sanitize

**Phase 2.3 Complete** - Advanced Status & Presence
- ✅ Database migration for user_status table
- ✅ UserStatus model with status types (online/away/dnd/offline)
- ✅ API endpoints: PUT /api/users/me/status - Update status with custom message/emoji
- ✅ API endpoints: Quick status setters (POST /api/users/me/status/{online|away|offline})
- ✅ API endpoint: GET /api/users/:id/status - Get user status
- ✅ API endpoint: GET /api/users/status/active - Get active users in org
- ✅ Auto-away logic after 15 minutes of inactivity
- ✅ WebSocket heartbeat for activity tracking
- ✅ WebSocket broadcasting of status changes
- ✅ Redis caching for user status (5-minute TTL)
- ✅ Background tasks for auto-away and expired status cleanup
- ✅ Custom status messages with emoji and auto-clear time

**Phase 2.4 Complete** - Read Receipts
- ✅ Database migration for message_read_receipts table with indexes
- ✅ Database migration: Add disable_read_receipts to users table
- ✅ MessageReadReceipt model with CRUD operations
- ✅ API endpoint: POST /api/messages/:id/read - Record read receipt
- ✅ API endpoint: GET /api/messages/:id/receipts - Get receipts with user details
- ✅ API endpoint: POST /api/read-receipts/batch - Batch read receipt recording
- ✅ Privacy setting: users can disable sending read receipts
- ✅ WebSocket broadcasting of read receipts to message senders
- ✅ Row Level Security (RLS) for read receipts
- ✅ Efficient database indexes for message and user queries
- ✅ Support for both channel messages and direct messages
- ✅ Unique constraint with automatic timestamp updates on conflicts

**Next**: Phase 2.5 - Message Editing History

## Version History

### v0.27.0 (Phase 2.4 - Read Receipts)
- Added database migration for message_read_receipts table
- Added database migration for disable_read_receipts column in users table
- Implemented MessageReadReceipt model with record, batch, and query operations
- API endpoint: POST /api/messages/:id/read - Record read receipt for a message
- API endpoint: GET /api/messages/:id/receipts - Get all receipts with user details
- API endpoint: POST /api/read-receipts/batch - Batch record multiple read receipts
- Privacy feature: users can disable sending read receipts via disable_read_receipts field
- WebSocket support: ReadReceipt message type for real-time receipt broadcasting
- Row Level Security (RLS) policies for read receipts (users can only see receipts for accessible messages)
- Efficient database indexes: message_id, user_id, and composite indexes for performance
- Unique constraint on (message_id, user_id) with automatic timestamp updates on conflict
- Read receipts work for both channel messages and direct messages
- Handler includes authorization checks for channel membership and DM participation
- Read receipts include user details (display_name, avatar_url) for UI rendering

### v0.26.0 (Phase 2.3 - Advanced Status & Presence)
- Added database migration for user_status table
- Implemented UserStatus model with status types (online/away/dnd/offline)
- API endpoints for updating and retrieving user status
- Quick status setter endpoints for online/away/offline
- Custom status messages with emoji and auto-clear functionality
- Auto-away logic: sets users to 'away' after 15 minutes of inactivity
- Background tasks module for auto-away and expired status cleanup
- WebSocket activity tracking via message handling
- WebSocket broadcasting of status changes to organization members
- Redis caching for user status with 5-minute TTL
- Touch activity tracking in database updated_at field
- Tasks run every 5 minutes (auto-away) and 10 minutes (clear expired)

### v0.25.0 (Phase 2.2 - Rich Text Formatting / Markdown)
- UI: Markdown toolbar for message composition
- UI: Markdown preview toggle in message input
- UI: Full markdown rendering with react-markdown
- UI: Syntax highlighting for code blocks using react-syntax-highlighter
- Security: HTML sanitization with rehype-sanitize
- Support for bold, italic, inline code, code blocks, lists, links, quotes, headings
- Dependencies added: react-markdown, react-syntax-highlighter, rehype-sanitize, remark-gfm

### v0.24.0 (Phase 2.1 - Message Pinning & Bookmarks)
- Added database migrations for pinned_messages and bookmarks tables
- Implemented PinnedMessage model with pin/unpin operations
- Implemented Bookmark model with create/delete operations
- API endpoints: POST/DELETE /api/messages/:id/pin for pinning
- API endpoint: GET /api/channels/:id/pins for listing pinned messages
- API endpoints: POST /api/bookmarks, DELETE /api/bookmarks/:message_id
- API endpoint: GET /api/bookmarks for listing user bookmarks
- WebSocket PubSubEvent for pin/unpin broadcasts
- Personal bookmarks are private to each user

### v0.23.0 (Phase 1.5 - @Mentions & Notifications)
- Added database migrations for mentions and notifications tables
- Implemented Mention model with MentionType (user/channel/here/everyone)
- Implemented Notification model with NotificationType (mention/dm/thread_reply/channel_invite)
- Created mention parser service with regex-based detection
- Added user lookup by display name (case-insensitive)
- Integrated mention parsing into message creation flow
- Automatic notification creation for mentions and thread replies
- API endpoints: GET /api/mentions, GET /api/mentions/unread-count
- API endpoints: GET /api/notifications, GET /api/notifications/unread-count
- API endpoints: POST /api/notifications/:id/read, POST /api/notifications/read-all
- Support for @username, @channel, @here, @everyone mentions
- Self-notification prevention logic
- Unread mention counter with join query on read status
- Crate added: regex

### v0.22.0 (Phase 1.4 - Message Search Full-Text)
- Added database migration with GIN index for full-text search
- Implemented tsvector column with auto-update trigger
- Created search handler with query parsing
- API endpoint: GET /api/search/messages
- Redis caching for search results (1-minute TTL)
- Support for search scopes: channel, dm, all
- Case-insensitive search with PostgreSQL full-text search

### v0.21.0 (Phase 1.3 - File Attachments with Configurable Storage)
- Added database migrations for attachments and storage_settings tables
- Implemented FileStorage trait with Local and S3 implementations
- Added storage factory with per-org configuration support
- Created attachment upload, download, and delete endpoints
- Added multipart/form-data file upload support
- Implemented file type and size validation
- Added support for local filesystem storage (default)
- Added support for S3 and S3-compatible object storage
- Environment variables: LOCAL_STORAGE_PATH, MAX_FILE_SIZE, ALLOWED_FILE_TYPES
- UI API client methods for attachment operations
- Crates added: aws-sdk-s3, aws-config, mime, mime_guess, tokio-util, bytes, actix-multipart, async-trait

### v0.20.0 (Phase 1.2 - Thread Display UI)
- Added `get_first_replies_batch` method to Message model for efficient first reply fetching
- Extended MessageResponse to include `first_reply` field
- Enhanced list_channel_messages endpoint to include first reply for messages with threads
- Batch fetching of first reply users for optimal performance
- Removed unused functions: count_replies, get_for_channel, get_all_for_user (ChannelReadStatus), get_for_dm, get_all_for_user (DmReadStatus)
- UI: Inline thread preview displaying first reply content and author
- UI: Thread panel with breadcrumb navigation showing author and reply count
- UI: Improved thread side panel with 2-second polling for near-realtime updates
- UI: Enhanced thread indicator button styling with hover effects

### v0.19.0 (Phase 1.1 - Unread Message Tracking)
- Added channel_read_status and dm_read_status database tables
- Implemented ChannelReadStatus and DmReadStatus models
- Added mark_as_read and get_unread_count methods for channels and DMs
- Redis caching for unread counts with 60-second TTL
- API endpoints for marking channels/DMs as read
- API endpoints for getting unread message counts
- Cache invalidation on mark-as-read operations
- WebSocket message type for unread count updates (UnreadCountUpdated)
- UI API client methods for unread functionality

### v0.17.0 (Feature Release)
- Added reactions to message responses when listing channel and DM messages
- Messages now include `reactions` array with full reaction details
- Efficient bulk fetching of reactions for all messages in a single query
- ReactionResponse includes id, message_id, user_id, emoji, and created_at
- Reactions persist across page refreshes and are included in all message endpoints

### v0.15.5 (Bug Fix Release)
- Fixed `SET LOCAL` to `SET` for RLS context (SET LOCAL requires transaction block)
- Added organization upsert before user upsert to satisfy foreign key constraint
- Restored Organization model with upsert functionality
- Added org_name to TokenClaims for proper organization management
- Fixed foreign key constraint violations on user creation

### v0.15.4 (Bug Fix Release)
- Fixed RLS context setting SQL syntax error
- Removed `SET LOCAL` parameterized query (not supported by PostgreSQL)
- Changed to use string formatting with validated UUID type
- Removed unused `with_admin_role` function
- Removed unused `scope` field from AccessTokenClaims

### v0.15.1 (Bug Fix Release)
- Fixed SSO redirect loop where users were redirected back to /applications/ after login
- Implemented role-based access control with "openchat" and "openchat-admin" roles
- Applied authentication middleware to all protected API routes
- Middleware now automatically creates/upserts users in database on first authenticated request
- Users must have "openchat" role to access the application
- Fixed SSO callback to use user_claims from token exchange response
- Updated TypeScript types to include user_claims in SSO responses

### v0.15.0 (Maintenance Release)
- Migrated from deprecated `actix-web-actors` to modern `actix-ws` crate
- Updated WebSocket implementation to use async task-based approach
- Removed dependency on deprecated crate for better long-term maintainability
- No functional changes - WebSocket functionality remains identical
- Improved code quality and future-proofing

### v0.11.0 (Phase 11)
- Redis Pub/Sub for horizontal WebSocket scaling
- Cross-instance message broadcasting
- PubSubEvent system with comprehensive event types
- Pattern-based Redis channel subscription (openchat:*)
- Automatic Redis reconnection with 5s backoff
- Message, typing, status, and reaction event distribution
- Graceful degradation when Redis unavailable
- Multi-ECS instance support for production deployment
- Organization and channel-specific event routing
- Serializable event protocol for Redis transport

### v0.10.0 (Phase 10)
- Advanced WebSocket features (built on Phase 9 foundation)
- Real-time typing indicators with automatic broadcast
- Channel subscription and unsubscription handlers
- User status management (online/offline/away)
- Organization-wide status broadcasts
- Comprehensive WebSocket message type system
- Client-to-server and server-to-client message protocols
- Error handling and message validation
- WebSocket connection health monitoring
- Foundation for future REST-to-WebSocket integration

### v0.9.0 (Phase 9)
- WebSocket support with actix-ws
- Real-time messaging (single server instance)
- WebSocket session management with heartbeat
- In-memory connection registry (user/org/channel mappings)
- JWT authentication via query parameter
- Message broadcasting to channels and DMs
- Typing indicator support with auto-timeout
- User presence tracking (online/offline status)
- Channel subscription/unsubscribe system
- Automatic presence updates on connect/disconnect
- Multi-device support (multiple sessions per user)
- Ping/pong heartbeat for connection health
- Graceful disconnect handling with cleanup

### v0.8.0 (Phase 8)
- Reaction model with full CRUD operations
- Add reaction endpoint with emoji support
- Remove reaction endpoint (users can only remove own reactions)
- List reactions endpoint with authorization
- Aggregated reaction counts endpoint (grouped by emoji)
- UNIQUE constraint (message_id, user_id, emoji) prevents duplicates
- ON CONFLICT handling for duplicate reactions
- Authorization checks (channel members and DM participants only)
- Support for multiple users reacting with same emoji
- User ID list in reaction count aggregation

### v0.7.0 (Phase 7)
- DirectMessage and DmParticipant models with full CRUD
- Create DM functionality (1-on-1 and group DMs)
- Automatic DM deduplication (finds and reuses existing DMs)
- List user's DMs endpoint with participant data
- Get DM details endpoint with authorization checks
- List DM messages endpoint with cursor-based pagination
- Participant verification for all DM operations
- Cross-organization DM prevention
- Redis caching infrastructure for DM data
- Transaction-based DM creation with participant management

### v0.6.0 (Phase 6)
- Message model with full CRUD operations
- Send messages to channels and DMs
- Cursor-based pagination for efficient message history loading
- Edit message functionality with edited_at timestamp
- Soft delete for messages (deleted_at timestamp)
- Message endpoints (send, list, update, delete)
- Authorization checks (users can only edit/delete their own messages)
- Member verification for channel messages
- Redis caching infrastructure for message data
- Support for threaded messages (parent_message_id)

### v0.5.0 (Phase 5)
- Channel model with CRUD operations
- Channel member management (add/remove)
- Public/private channel types
- Channel endpoints (list, create, get, update, delete)
- Member endpoints (list, add, remove)
- Redis caching for channel data
- Creator-based authorization for channel management

### v0.4.0 (Phase 4)
- Organization model for multi-tenant support
- User management REST API endpoints
- User profile and status update functionality
- Redis caching layer for user data
- Cache invalidation strategies
- Authorization checks (users can only update their own data)

### v0.3.0 (Phase 3)
- TitaniumVault SSO integration via API client
- Authentication middleware for protected routes
- JWT token verification and claims extraction
- Automatic user creation/update on authentication
- User model with CRUD operations
- Integrated RLS context setting in auth flow

### v0.2.0 (Phase 2)
- Database schema with 9 tables (organizations, users, channels, messages, etc.)
- Row Level Security policies for multi-tenant isolation
- Database connection pool with SQLx
- Migration system for schema management

### v0.1.0 (Phase 1)
- Initial project setup
- Basic Actix-web server with health check endpoint
- Configuration and error handling infrastructure
