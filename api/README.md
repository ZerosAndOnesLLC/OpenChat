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

### Authentication (Phase 3+)
- `POST /api/auth/verify` - Verify TitaniumVault JWT token

### Channels (Phase 5+)
- `GET /api/channels` - List channels
- `POST /api/channels` - Create channel
- `GET /api/channels/:id` - Get channel details
- `PUT /api/channels/:id` - Update channel
- `DELETE /api/channels/:id` - Delete channel

### Messages (Phase 6+)
- `POST /api/messages` - Send message
- `PUT /api/messages/:id` - Edit message
- `DELETE /api/messages/:id` - Delete message

### WebSocket (Phase 9+)
- `WS /api/ws?token=<jwt>` - WebSocket connection for real-time messaging

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | Yes |
| `REDIS_URL` | Redis connection string | Yes |
| `TV_API_URL` | TitaniumVault API URL | Yes |
| `JWT_SECRET` | Secret for JWT signing | Yes |
| `PORT` | Server port (default: 8080) | No |
| `HOST` | Server host (default: 0.0.0.0) | No |
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

**Next**: Phase 3 - Authentication & Middleware

## Version History

### v0.2.0 (Phase 2)
- Database schema with 9 tables (organizations, users, channels, messages, etc.)
- Row Level Security policies for multi-tenant isolation
- Database connection pool with SQLx
- Migration system for schema management

### v0.1.0 (Phase 1)
- Initial project setup
- Basic Actix-web server with health check endpoint
- Configuration and error handling infrastructure
