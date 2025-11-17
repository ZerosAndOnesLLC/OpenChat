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
- `POST /api/messages` - Send message to channel or DM
- `GET /api/channels/:id/messages` - List channel messages (paginated)
- `PUT /api/messages/:id` - Edit message
- `DELETE /api/messages/:id` - Soft delete message
- `POST /api/messages/:id/reactions` - Add reaction to message
- `GET /api/messages/:id/reactions` - List reactions on message
- `GET /api/messages/:id/reactions/counts` - Get reaction counts
- `DELETE /api/messages/:id/reactions/:emoji` - Remove reaction

### Direct Messages (Phase 7+)
- `GET /api/dms` - List user's DMs
- `POST /api/dms` - Create DM (1-on-1 or group)
- `GET /api/dms/:id` - Get DM details
- `GET /api/dms/:id/messages` - List DM messages (paginated)

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

**Next**: Phase 12 - Frontend Setup

## Version History

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
