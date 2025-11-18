# OpenChat UI

Modern real-time team chat application built with Next.js 15.

## Tech Stack

- **Next.js 15** - React framework with App Router
- **React 19** - Latest React with concurrent features
- **TypeScript** - Type safety
- **Tailwind CSS** - Utility-first styling
- **TanStack Query** (React Query v5) - Data fetching and caching
- **Zustand** - Lightweight state management
- **WebSocket API** - Real-time messaging

## Features

- ✅ Real-time messaging via WebSockets
- ✅ Public and private channels
- ✅ Direct messages (1-on-1 and group)
- ✅ Unread message tracking (API support)
- ✅ Typing indicators
- ✅ User presence (online/offline/away)
- ✅ Message reactions (emoji)
- ✅ Message editing and deletion
- ✅ TitaniumVault SSO integration
- ✅ Beautiful, responsive UI

## Project Structure

```
openchat/ui/
├── app/                    # Next.js App Router
│   ├── layout.tsx         # Root layout with providers
│   ├── page.tsx           # Main chat page
│   └── globals.css        # Global styles
├── components/            # React components
│   ├── ChatLayout.tsx     # Main chat layout
│   ├── Sidebar.tsx        # Channel/DM sidebar
│   ├── ChannelList.tsx    # Channel list
│   ├── DirectMessageList.tsx # DM list
│   ├── UserProfile.tsx    # User profile dropdown
│   ├── MessageArea.tsx    # Message display area
│   ├── MessageList.tsx    # Message list with scrolling
│   ├── MessageItem.tsx    # Individual message with reactions
│   ├── MessageInput.tsx   # Message input with typing indicators
│   └── TypingIndicator.tsx # Typing indicator UI
├── lib/                   # Core libraries
│   ├── api.ts            # API client
│   ├── websocket.ts      # WebSocket client and store
│   ├── auth.tsx          # Authentication hooks and store
│   ├── types.ts          # TypeScript types
│   └── providers.tsx     # React Query provider
└── .env.local            # Environment variables
```

## Environment Variables

Create a `.env.local` file:

```env
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_WS_URL=ws://localhost:8080/api/ws
NEXT_PUBLIC_TV_API_URL=https://api.titanium-vault.com
```

## Getting Started

### Install Dependencies

```bash
npm install
```

### Development

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

### Build for Production

```bash
npm run build
```

### Run Production Build

```bash
npm start
```

## Authentication Flow

1. User visits the app
2. If not authenticated, redirect to TitaniumVault login
3. TV-API returns JWT token with user info
4. OpenChat verifies token and creates/updates user
5. WebSocket connection established with token
6. User can now chat in real-time

## Real-time Features

### WebSocket Connection

The app automatically connects to the WebSocket server when authenticated. The connection:

- Auto-reconnects on disconnect
- Handles token authentication
- Manages subscriptions to channels
- Broadcasts typing indicators
- Updates user presence status

### Message Flow

1. User types message in input
2. Typing indicator sent via WebSocket
3. User sends message
4. Message sent via WebSocket
5. Server broadcasts to all subscribed users
6. UI updates in real-time

### Reactions (Mattermost-style)

- Hover over a message to see the "+" button inline with reactions
- Click "+" to open comprehensive emoji picker
- Click existing reaction to toggle (add/remove)
- Reactions display grouped by emoji with counts
- Your reactions are highlighted in blue
- Full emoji library with categories and search

## State Management

### Zustand Stores

**Auth Store** (`useAuthStore`)
- User authentication state
- Token management
- Login/logout flows

**WebSocket Store** (`useWebSocketStore`)
- WebSocket connection state
- Real-time message state
- Typing indicators
- User presence

### React Query

Used for:
- Fetching channels, DMs, users
- Caching server data
- Optimistic updates
- Automatic refetching

## Deployment

### S3 + CloudFront (Production)

1. Build the application:
```bash
npm run build
```

2. Export static files:
```bash
npm run export
```

3. Upload to S3 bucket configured for static hosting

4. CloudFront distribution serves the app

**Note**: S3 static hosting requires trailing slashes for routes.

## Performance Optimizations

- Message list virtualization (infinite scroll ready)
- Debounced typing indicators (3 second timeout)
- React Query caching (1 minute stale time)
- WebSocket message deduplication
- Optimistic UI updates

## Troubleshooting

### WebSocket Won't Connect

- Check `NEXT_PUBLIC_WS_URL` is correct
- Ensure API server is running
- Verify token is valid

### Messages Not Appearing

- Check browser console for errors
- Verify WebSocket connection is active
- Check API server logs

### Build Errors

```bash
# Clear Next.js cache
rm -rf .next

# Reinstall dependencies
rm -rf node_modules package-lock.json
npm install

# Try building again
npm run build
```

## Development Guidelines

- Fix all TypeScript errors properly (no `@ts-ignore`)
- Fix all ESLint warnings (no `eslint-disable` unless necessary)
- Use Tailwind CSS for styling
- Follow existing component patterns
- Keep components small and focused
- Use proper TypeScript types
- Add loading states for async operations

## Version

Current version: 0.3.0

Increment version before commits:
- **Patch** (0.1.x): Bug fixes, small tweaks
- **Minor** (0.x.0): New features, backward-compatible
- **Major** (x.0.0): Breaking changes

### Recent Changes

**v0.3.0** - Unread message tracking (API support)
- Added UnreadCountResponse and MarkAsReadRequest types
- Implemented markChannelAsRead() and getChannelUnreadCount() API methods
- Implemented markDmAsRead() and getDmUnreadCount() API methods
- Added WebSocket message type for unread count updates
- API foundation ready for UI implementation of unread badges
- Prepared for future unread count display in sidebar

**v0.2.2** - Fix reaction removal (toggle functionality)
- Fixed API client to handle empty responses (204 No Content)
- Clicking an existing reaction now properly removes it
- Added proper content-type checking before JSON parsing
- Toggle reaction on/off now works correctly

**v0.2.1** - Fix reactions not appearing immediately when clicked
- Added optimistic UI updates for reactions
- Reactions now appear instantly when clicked (no WebSocket delay)
- Automatic rollback if API call fails
- Improved reaction responsiveness and user experience

**v0.2.0** - Mattermost-style emoji reactions with comprehensive picker
- Integrated emoji-picker-react library with full emoji support
- Moved "+" button to display inline with reactions (Mattermost UX)
- Added comprehensive emoji picker with categories and search
- Emoji picker appears on hover, positioned below reactions
- Click outside picker to close
- Removed emoji button from top-right hover menu
- Improved reaction UI consistency

**v0.1.8** - Fix WebSocket message parsing errors
- Added defensive checks for all WebSocket message handlers
- Prevent crashes when receiving malformed or incomplete messages
- Improved error logging to help debug WebSocket issues
- Added default case handler for unknown message types

**v0.1.7** - Dark theme update for messages and channels
- Updated message area background to black
- Updated all text colors for better visibility on dark background
- Updated message input, reactions, and action buttons to match dark theme
- Improved contrast and readability

## Related Projects

- **openchat/api** - Rust/Actix backend API
- **TitaniumVault** - SSO authentication provider
