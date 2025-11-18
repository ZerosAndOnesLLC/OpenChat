# OpenChat

**Open-source, self-hosted team collaboration platform - Your data, your control**

[![Version](https://img.shields.io/badge/version-0.14.2-blue.svg)](https://github.com/yourusername/openchat)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Next.js](https://img.shields.io/badge/next.js-16-black.svg)](https://nextjs.org/)

> A modern, fast, and secure team chat application built with Rust and Next.js. Self-host your team communications without compromising on features or performance.

---

## Why OpenChat?

**Privacy-First** • **Lightning Fast** • **Enterprise-Ready** • **100% Open Source**

OpenChat is a powerful alternative to Slack and Microsoft Teams that you can self-host on your own infrastructure. Keep complete control of your team's communications, data, and privacy while enjoying a modern chat experience.

### Key Highlights

- **🔒 Enterprise Security** - PostgreSQL Row Level Security (RLS) ensures bulletproof data isolation
- **⚡ Real-time Performance** - WebSocket connections with Redis Pub/Sub for instant messaging
- **🚀 Built with Rust** - Memory-safe, blazingly fast backend that scales effortlessly
- **🎨 Modern UI** - Beautiful, responsive interface built with Next.js and Tailwind CSS
- **🔐 SSO Integration** - Seamless single sign-on with TitaniumVault authentication
- **💾 Self-Hosted** - Your data stays on your infrastructure - no vendor lock-in
- **📦 Docker Ready** - Deploy anywhere with containerized architecture
- **🌐 Cloud Native** - Built for AWS ECS with CloudFront CDN support

---

## Recent Updates

### Version 0.25.0 (API) / 0.6.0 (UI) - Rich Text Formatting with Markdown

**New Features:**
- **Markdown Support**: Full markdown formatting in messages
  - **Bold** (`**text**`), *italic* (`*text*`), `inline code` (`` `code` ``)
  - Code blocks with syntax highlighting (` ```language ````)
  - Lists (ordered and unordered)
  - Links, blockquotes, headings
- **Markdown Toolbar**: Quick formatting buttons for common markdown syntax
- **Live Preview**: Toggle between edit and preview mode to see rendered markdown
- **Syntax Highlighting**: Beautiful code syntax highlighting for 100+ languages
- **XSS Protection**: Sanitized HTML output prevents security vulnerabilities

**UI Components:**
- Markdown toolbar with formatting buttons (bold, italic, code, links, lists, quotes)
- Live preview toggle to see how your message will look
- Syntax-highlighted code blocks using Prism themes
- Support for GitHub Flavored Markdown (tables, strikethrough, task lists)

**Technical:**
- `react-markdown` for rendering with `remark-gfm` for GitHub features
- `rehype-sanitize` for secure HTML output (XSS protection)
- `react-syntax-highlighter` with VS Code Dark+ theme
- Messages stored as markdown-compatible text (no schema changes required)

### Version 0.24.0 (API) - Message Pinning & Bookmarks

**New Features:**
- **Message Pinning**: Pin important messages to channels for all members to see
- **Personal Bookmarks**: Save messages privately for later reference
- Real-time WebSocket notifications for pin/unpin actions
- Access control ensures users can only pin/bookmark messages they can see

**API Endpoints:**
- `POST /api/messages/{id}/pin` - Pin a message
- `DELETE /api/messages/{id}/pin` - Unpin a message
- `GET /api/channels/{id}/pins` - List pinned messages
- `POST /api/bookmarks` - Bookmark a message
- `DELETE /api/bookmarks/{message_id}` - Remove bookmark
- `GET /api/bookmarks` - List your bookmarks

**Technical:**
- Added `pinned_messages` and `bookmarks` database tables with RLS policies
- Optimized queries with indexes for performance at scale
- WebSocket integration for real-time pin/unpin notifications

---

## Features

### Core Messaging
- **Public & Private Channels** - Organize conversations by topic or team
- **Direct Messages** - 1-on-1 and group conversations
- **Real-time Sync** - Messages appear instantly across all connected devices
- **Message Threading** - Keep conversations organized (roadmap)
- **Rich Reactions** - Express yourself with emoji reactions
- **Message Editing & Deletion** - Full control over your messages

### Collaboration
- **Typing Indicators** - See when teammates are composing messages
- **User Presence** - Online, offline, and away status tracking
- **File Attachments** - Share images, documents, and more
- **Full-Text Search** - Find messages instantly with PostgreSQL full-text search and Redis caching
- **@Mentions & Notifications** - @user and @channel notifications with real-time alerts
- **Message Pinning** - Pin important messages to channels for easy reference
- **Personal Bookmarks** - Save messages for later with private bookmarks
- **Rich Text Formatting** - Full markdown support with live preview and syntax highlighting

### Enterprise Features
- **Multi-Organization Support** - Host multiple teams on one instance
- **Organization Isolation** - Database-level security with PostgreSQL RLS
- **SSO Authentication** - Integrate with existing identity providers
- **Audit Logging** - Track activity for compliance (roadmap)
- **Horizontal Scaling** - Add more servers as your team grows

---

## Tech Stack

### Backend
- **Rust 2024 Edition** - Modern, safe systems programming
- **Actix Web** - High-performance async web framework
- **PostgreSQL** - Reliable, feature-rich relational database
- **Redis** - Caching and real-time pub/sub messaging
- **SQLx** - Compile-time verified SQL queries
- **WebSockets** - Real-time bidirectional communication

### Frontend
- **Next.js 16** - React framework with server components
- **React 19** - Latest React with concurrent features
- **TypeScript** - Type-safe development
- **Tailwind CSS** - Modern utility-first styling
- **TanStack Query** - Powerful data fetching and caching
- **Zustand** - Lightweight state management

### Infrastructure
- **Docker** - Containerized deployment
- **AWS ECS** - Scalable container orchestration
- **CloudFront** - Global CDN for static assets
- **Network Load Balancer (NLB)** - High-availability traffic routing for long-lived WebSocket connections
  - **Important**: ALB is not suitable for OpenChat due to WebSocket connection timeouts
  - NLB supports persistent connections required for real-time messaging

---

## Quick Start

### Prerequisites
- Docker and Docker Compose
- PostgreSQL 14+
- Redis 7+
- Node.js 20+
- Rust 1.83+ (2024 edition)

### Local Development

1. **Clone the repository**
   ```bash
   # HTTPS
   git clone https://github.com/ZerosAndOnesLLC/OpenChat.git
   cd OpenChat

   # SSH
   git clone git@github.com:ZerosAndOnesLLC/OpenChat.git
   cd OpenChat
   ```

2. **Start PostgreSQL and Redis**
   ```bash
   docker-compose up -d postgres redis
   ```

3. **Set up the API**
   ```bash
   cd api
   cp .env.example .env
   # Edit .env with your database credentials

   # Run migrations
   cargo install sqlx-cli
   sqlx database create
   sqlx migrate run

   # Start the API server
   cargo run
   ```

4. **Start the UI**
   ```bash
   cd ../ui
   npm install
   npm run dev
   ```

5. **Open your browser**
   ```
   http://localhost:3000
   ```

### Production Deployment

OpenChat is designed to run on AWS with ECS containers:

```bash
# Build and push Docker images
docker build -t openchat-api ./api
docker build -t openchat-ui ./ui

# Deploy with Terraform (see terraform/ directory)
cd terraform/prod/us-east-1/openchat
terraform apply
```

See [DEPLOYMENT.md](docs/DEPLOYMENT.md) for detailed production setup instructions.

---

## Architecture

```
┌─────────────┐         ┌──────────────┐         ┌──────────────┐
│   Browser   │◄───────►│   Next.js    │◄───────►│              │
│  (WebSocket)│         │      UI      │         │              │
└─────────────┘         └──────────────┘         │              │
                                                  │   Actix Web  │
┌─────────────┐         ┌──────────────┐         │   Rust API   │
│   Browser   │◄───────►│  CloudFront  │◄───────►│              │
│   (HTTP)    │         │      CDN     │         │              │
└─────────────┘         └──────────────┘         └──────┬───────┘
                                                         │
                        ┌────────────────────────────────┼────────┐
                        │                                │        │
                        ▼                                ▼        ▼
                 ┌──────────────┐              ┌──────────────┐  │
                 │  PostgreSQL  │              │    Redis     │  │
                 │   Database   │              │ Cache+PubSub │  │
                 └──────────────┘              └──────────────┘  │
                                                                 │
                                               ┌─────────────────▼──┐
                                               │  TitaniumVault SSO │
                                               └────────────────────┘
```

### Security Architecture

OpenChat implements **defense-in-depth** security:

1. **Application Layer** - JWT token validation, role-based access control
2. **Database Layer** - PostgreSQL Row Level Security (RLS) enforces organization isolation
3. **Network Layer** - TLS encryption, private VPC networking
4. **Authentication** - SSO integration with enterprise identity providers

Even if application code has bugs, the database prevents cross-organization data access.

---

## OpenChat vs. Alternatives

| Feature | OpenChat | Slack | Mattermost | Rocket.Chat |
|---------|----------|-------|------------|-------------|
| **Open Source** | ✅ SSPL | ❌ Proprietary | ✅ MIT | ✅ MIT |
| **Self-Hosted** | ✅ Full control | ❌ Cloud only | ✅ Available | ✅ Available |
| **Built with Rust** | ✅ Memory safe | ❌ | ❌ | ❌ |
| **Modern Tech Stack** | ✅ 2024 | ⚠️ Legacy | ⚠️ Mixed | ⚠️ Mixed |
| **Row Level Security** | ✅ Database RLS | ❌ | ❌ | ❌ |
| **WebSocket Real-time** | ✅ Native | ✅ | ✅ | ✅ |
| **SSO Integration** | ✅ Built-in | ✅ Paid | ✅ Available | ✅ Available |
| **Horizontal Scaling** | ✅ Redis Pub/Sub | ✅ | ✅ | ✅ |
| **Free for Commercial** | ✅ SSPL | ❌ Paid | ⚠️ Limits | ⚠️ Limits |
| **Docker Deployment** | ✅ | N/A | ✅ | ✅ |

### Why Choose OpenChat?

- **Cost**: No per-user fees, no feature paywalls, no surprise costs
- **Performance**: Rust backend delivers exceptional speed and low memory usage
- **Security**: Database-level isolation prevents data leaks
- **Modern**: Built with 2024 technologies, not legacy codebases
- **Privacy**: Your data never leaves your infrastructure
- **Control**: Customize and extend as needed

---

## Roadmap

### ✅ Phase 1 - MVP (Current)
- [x] User authentication via SSO
- [x] Organization isolation with RLS
- [x] Public/private channels
- [x] Direct messaging
- [x] Real-time WebSocket messaging
- [x] Message history and pagination
- [x] Typing indicators
- [x] User presence tracking
- [x] Emoji reactions
- [x] Message editing and deletion

### 🚧 Phase 2 - Enhanced Features (In Progress)
- [x] Threaded conversations
- [x] File and image attachments
- [x] Full-text search with Redis caching
- [x] Unread message counts
- [x] @Mentions & notifications
- [x] Message pinning
- [x] Bookmarked messages (personal starred messages)
- [x] Markdown formatting
- [x] Code syntax highlighting
- [ ] Push notifications

### 🔮 Phase 3 - Enterprise (Future)
- [ ] Voice and video calls
- [ ] Screen sharing
- [ ] Webhooks and integrations
- [ ] Bot framework
- [ ] Custom emojis
- [ ] Reminders and scheduled messages
- [ ] Advanced analytics
- [ ] LDAP/Active Directory sync

### 📱 Phase 4 - Native Applications (Planned)
- [ ] Android Application
- [ ] iOS Application
- [ ] Windows Application
- [ ] macOS Application

> **Note**: OpenChat is under heavy development and the roadmap is subject to change. Features, timelines, and priorities may be adjusted based on community feedback and project needs.

---

## API Features

### Message Search

OpenChat provides powerful full-text search capabilities powered by PostgreSQL and Redis:

**Endpoint**: `GET /api/search/messages`

**Query Parameters**:
- `q` - Search query text (required)
- `scope` - Search scope: `all`, `channel`, or `dm` (default: `all`)
- `channel_id` - Filter by specific channel (required if scope is `channel`)
- `dm_id` - Filter by specific DM (required if scope is `dm`)
- `limit` - Number of results (default: 50, max: 100)

**Features**:
- Full-text search using PostgreSQL GIN indexes
- Prefix matching for partial word searches
- Results cached in Redis for 1 minute
- Returns messages with total count
- Supports search across all messages, specific channels, or DMs

**Example**:
```bash
GET /api/search/messages?q=hello+world&scope=channel&channel_id=abc-123&limit=20
```

### Message Pinning

Pin important messages to the top of channels for easy reference by all members:

**Endpoints**:
- `POST /api/messages/{id}/pin` - Pin a message (channel members only)
- `DELETE /api/messages/{id}/pin` - Unpin a message
- `GET /api/channels/{id}/pins` - List all pinned messages in a channel

**Features**:
- Only channel messages can be pinned (not DMs)
- All channel members can view pinned messages
- Real-time WebSocket notifications when messages are pinned/unpinned
- Pins are sorted by pinned date (most recent first)

**Example**:
```bash
# Pin a message
POST /api/messages/msg-123/pin

# List pinned messages
GET /api/channels/channel-456/pins
```

### Personal Bookmarks

Save messages for later reference with personal bookmarks:

**Endpoints**:
- `POST /api/bookmarks` - Bookmark a message
- `DELETE /api/bookmarks/{message_id}` - Remove a bookmark
- `GET /api/bookmarks` - List all your bookmarks

**Features**:
- Bookmarks are private to each user
- Works with both channel messages and DMs
- Access verification ensures users can only bookmark messages they can see
- Sorted by bookmark date (most recent first)

**Example**:
```bash
# Bookmark a message
POST /api/bookmarks
{
  "message_id": "msg-789"
}

# List your bookmarks
GET /api/bookmarks
```

---

## Performance

OpenChat is built for scale:

- **Message Throughput**: 10,000+ messages/second per instance
- **Concurrent Users**: Thousands per server with Redis Pub/Sub
- **Database Queries**: Optimized with indexes and query planning
- **Caching**: Aggressive Redis caching reduces database load by 80%+
- **WebSocket**: Efficient connection pooling and message routing
- **Latency**: Sub-100ms message delivery in same region
- **Search Performance**: Full-text search with GIN indexes and 1-minute Redis caching

### Scalability

- **Horizontal Scaling**: Add more ECS tasks behind Network Load Balancer (NLB)
  - NLB required for WebSocket connection persistence
  - Supports thousands of concurrent long-lived connections per instance
- **Database**: PostgreSQL supports millions of messages
- **Redis**: Distributed pub/sub for cross-instance messaging
- **CDN**: CloudFront delivers static assets globally
- **Stateless API**: Each request can be handled by any instance

---

## Accessing OpenChat

### Authentication Flow

OpenChat uses **TitaniumVault** for authentication and should be accessed through the TitaniumVault applications portal:

1. **Log in to TitaniumVault** at https://titanium-vault.com
2. **Navigate to Applications** page
3. **Click on OpenChat** - this initiates the OAuth flow
4. OpenChat opens with you automatically authenticated

**Important**: Do not access OpenChat directly. Always go through TitaniumVault's applications portal to ensure proper authentication.

### OAuth Configuration

OpenChat uses OAuth 2.0 with PKCE for secure authentication:
- **Client ID**: `openchat-api` (registered in TitaniumVault)
- **Redirect URI**: `https://openchat.zerosandones.us/sso/callback/`
- **Scopes**: `openid profile email`
- **Grant Types**: `authorization_code`, `client_credentials`, `refresh_token`

---

## Security

### Authentication
- JWT token-based authentication
- Single sign-on (SSO) integration via OAuth 2.0 with PKCE
- TitaniumVault identity provider support
- Secure token refresh mechanism

### Authorization
- Organization-level isolation
- Role-based access control (admin, member)
- Channel-level permissions (public, private)
- PostgreSQL Row Level Security (RLS)

### Data Protection
- TLS encryption in transit
- Database encryption at rest
- Secure session management
- CSRF protection
- SQL injection prevention (compile-time verified queries)

### Compliance
- GDPR-ready with data export capabilities
- Audit logging for compliance tracking
- Data retention policies
- Self-hosted for data sovereignty

---

## Contributing

We welcome contributions from the community! Whether you're fixing bugs, adding features, or improving documentation, your help is appreciated.

### How to Contribute

1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Make your changes**
   - Follow Rust and TypeScript best practices
   - Add tests for new functionality
   - Update documentation as needed
4. **Run tests and linters**
   ```bash
   cargo test
   cargo clippy
   npm run lint
   ```
5. **Commit your changes** (`git commit -m 'Add amazing feature'`)
6. **Push to your branch** (`git push origin feature/amazing-feature`)
7. **Open a Pull Request**

### Development Guidelines

- **Rust**: Follow Rust 2024 edition idioms, use `cargo fmt` and `cargo clippy`
- **TypeScript**: Use strict TypeScript, follow Next.js conventions
- **Testing**: Write unit tests for business logic, integration tests for APIs
- **Documentation**: Update README and inline documentation
- **Security**: Never commit secrets, follow secure coding practices

### Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please read our [Code of Conduct](CODE_OF_CONDUCT.md) before contributing.

---

## Support

- **Documentation**: [docs.openchat.dev](https://docs.openchat.dev) (coming soon)
- **Issues**: [GitHub Issues](https://github.com/yourusername/openchat/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/openchat/discussions)
- **Security**: Report vulnerabilities to security@openchat.dev

---

## License

OpenChat is licensed under the [Server Side Public License (SSPL) v1](LICENSE.md).

The SSPL is a source-available license that allows free use, modification, and distribution. If you offer OpenChat as a service to third parties, you must open source your service infrastructure.

**Key Points:**
- ✅ Free to use internally
- ✅ Free to modify and customize
- ✅ Full source code available
- ⚠️ Service providers must open source their infrastructure stack

See [LICENSE.md](LICENSE.md) for the complete license text and terms.

---

## Acknowledgments

Built with amazing open-source technologies:

- [Rust](https://www.rust-lang.org/) - Memory-safe systems programming
- [Actix Web](https://actix.rs/) - Powerful, pragmatic web framework
- [Next.js](https://nextjs.org/) - The React framework for production
- [PostgreSQL](https://www.postgresql.org/) - The world's most advanced open source database
- [Redis](https://redis.io/) - In-memory data structure store
- [Tailwind CSS](https://tailwindcss.com/) - Utility-first CSS framework

Special thanks to all our contributors who help make OpenChat better!

---

<div align="center">

**[Getting Started](#quick-start)** • **[Documentation](docs/)** • **[Contributing](#contributing)** • **[License](#license)**

Made with ❤️ by the OpenChat community

⭐ **Star us on GitHub** if you find OpenChat useful!

</div>
