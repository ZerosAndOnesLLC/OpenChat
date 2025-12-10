# OpenChat UI

Modern real-time team chat application built with Next.js 16.

## Tech Stack

- **Next.js 16** - React framework with App Router
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
- ✅ Threaded conversations with inline previews
- ✅ Thread side panel with breadcrumb navigation
- ✅ **Unread message tracking with visual indicators**
  - ✅ Unread count badges on channels and DMs
  - ✅ Bold text for channels/DMs with unread messages
  - ✅ "New messages" separator in message list
  - ✅ Auto-scroll to first unread message
  - ✅ Auto-mark as read after viewing
- ✅ Typing indicators
- ✅ User presence (online/offline/away)
- ✅ Message reactions (emoji)
- ✅ Custom emojis per organization
  - ✅ Admin upload dialog for custom emojis
  - ✅ Emoji picker with custom and standard emojis
  - ✅ Emoji autocomplete in message input
  - ✅ Render custom emojis in messages
- ✅ Message editing and deletion
- ✅ Message drafts (auto-save to IndexedDB)
- ✅ Rich text formatting (Markdown)
- ✅ Keyboard shortcuts (Cmd+K, Cmd+/, Esc, Cmd+Enter)
- ✅ Quick channel/DM switcher (Cmd+K)
- ✅ **Audit Logging** (Admin)
  - ✅ Comprehensive audit log viewer at /admin/audit-logs
  - ✅ Filter by user, action, resource type, resource ID, date range
  - ✅ CSV export for compliance reporting
  - ✅ Pagination and search functionality
  - ✅ Detailed metadata inspection per log entry
- ✅ TitaniumVault SSO integration
- ✅ Resizable sidebar (drag to resize, persists to localStorage)
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
│   ├── MessageItem.tsx    # Individual message with reactions and thread preview
│   ├── MessageInput.tsx   # Message input with typing indicators and drafts
│   ├── ThreadPanel.tsx    # Thread side panel for replies
│   ├── TypingIndicator.tsx # Typing indicator UI
│   ├── KeyboardShortcutsHelp.tsx # Keyboard shortcuts help modal
│   └── QuickSwitcher.tsx  # Quick channel/DM switcher modal
├── lib/                   # Core libraries
│   ├── api.ts            # API client
│   ├── websocket.ts      # WebSocket client and store
│   ├── auth.tsx          # Authentication hooks and store
│   ├── drafts.ts         # Message drafts manager (IndexedDB)
│   ├── keyboard-shortcuts.ts # Keyboard shortcuts manager
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

Current version: 0.26.0

Increment version before commits:
- **Patch** (0.1.x): Bug fixes, small tweaks
- **Minor** (0.x.0): New features, backward-compatible
- **Major** (x.0.0): Breaking changes

### Recent Changes

**v0.26.0** - Redesigned Settings with Tree Navigation
- Complete redesign of the settings page with a modern tree-structure navigation
- Left sidebar with collapsible categories: Account, Devices, Integrations
- Right panel displays detailed settings content for selected item
- Categories: Profile, Privacy, Active Sessions, Desktop App, Webhooks, About
- Responsive design: side-by-side on desktop, full-screen navigation on mobile
- URL state management for direct linking to specific settings sections
- Auto-save for toggle settings (no save button needed)
- Consistent dark theme styling across all settings panels
- New components: SettingsLayout, ProfileSettings, PrivacySettings, AboutSettings, DesktopAppSettings
- Refactored DeviceManagement and WebhookManagement with improved styling

**v0.25.0** - Resizable sidebar
- Added draggable resize handle between sidebar and main content area
- Sidebar width persists to localStorage across sessions
- Min width 180px, max width 400px, default 256px
- Visual feedback with blue highlight when resizing

**v0.24.5** - Refactor leave/hide to use direct async handlers
- Replaced react-query mutations with direct async/await handlers for channel leave and DM hide
- This ensures removeChannel/removeDm is called synchronously after API success
- Both ChannelList and DirectMessageList now use the same reliable pattern

**v0.24.4** - Fix channel leave not updating sidebar
- Fixed leaving a public channel not removing it from sidebar immediately
- Moved removeChannel call inside mutationFn to ensure it runs reliably
- Added public-channels query invalidation on leave

**v0.24.3** - Fix channel rejoin not showing in sidebar
- Fixed channels not appearing in sidebar after leaving and rejoining
- BrowseChannelsModal now adds channel to WebSocket store immediately when joining

**v0.24.2** - Fix sidebar real-time updates for channels and DMs
- Fixed new DMs not appearing in sidebar (condition now always uses WebSocket store when loaded)
- Fixed new channels not appearing in sidebar after creation
- Added `addChannel`, `removeChannel`, `updateChannel` methods to WebSocket store
- Channel rename now updates sidebar immediately via WebSocket store
- Channel leave now removes from sidebar immediately via WebSocket store
- Sidebar now properly syncs with WebSocket store for all channel/DM operations

**v0.24.1** - Fix DM and attachment display issues
- Fixed file attachments not displaying until app reload (now refreshes messages after upload)
- Fixed users showing as "Unknown" in DM list when they have no display name (falls back to email)
- Fixed new DMs not appearing in sidebar without closing window (updates WebSocket store)
- Fixed close DM button not working (removes DM from WebSocket store immediately)
- Added `addDm` and `removeDm` methods to WebSocket store for real-time DM list updates

**v0.23.2** - Fix unread indicator flashing for active channel
- Fixed issue where "New messages" indicator briefly flashed when sending or receiving messages in active channel
- Added active channel/DM tracking to WebSocket store
- Messages received for the active channel are immediately marked as read (no 1-second delay)
- Eliminates visual flicker of unread badges and markers for channels being actively viewed

**v0.22.3** - Fix Turbopack build configuration
- Added turbopack.root to next.config.ts to fix workspace root inference
- Fixed package.json import path in settings page for Turbopack compatibility

**v0.18.0** - Data Retention Policies UI (Phase 5.4 Complete)
- Added retention policies management page at /admin/retention
- Separate message and file retention policy configuration
- Configurable retention period in days with enable/disable toggles
- Legal hold information panel explaining freeze functionality
- Warning messages about data permanence and compliance requirements
- Real-time policy updates with success/error feedback
- Admin menu enhanced with Retention Policies link
- User roles field added to User type (from SSO userinfo)
- Admin navigation menu now checks for openchat-admin role
- Settings menu item added for all users
- Admin section (Storage, Audit Logs, Retention) only visible to admins
- UserProfile component enhanced with router navigation
- Background job enforcement deferred to Phase 10 (Background Workers)

**v0.17.0** - Audit Logging UI (Phase 5.3 Complete)
- Added comprehensive audit log viewer at /admin/audit-logs
- Advanced filtering: user ID, action, resource type, resource ID, date range
- Pagination support with configurable page sizes (25/50/100)
- CSV export functionality for compliance reporting
- Expandable row details showing full metadata, IP address, user agent
- Filter dropdowns populated from backend (actions and resource types)
- Real-time data fetching with loading states
- Responsive table design with proper overflow handling
- Error handling and user feedback
- Protected route requiring admin permissions (org.view_audit_logs)

**v0.9.0** - Unread Message Tracking UI (Phase 1.1 Complete)
- Implemented unread count badges on channels in sidebar (red pill badges)
- Implemented unread count badges on DMs in sidebar (red pill badges)
- Added bold text styling for channels/DMs with unread messages
- Implemented "New messages" separator line in message list
- Auto-scroll to first unread message when opening a channel/DM
- Smart scroll behavior: stays at unread marker or maintains scroll position
- Auto-mark messages as read after 2 seconds of viewing
- Real-time unread count updates (30-second polling + WebSocket support)
- Unread badges hide when channel/DM is active
- Badges show "99+" for counts over 99
- Enhanced ChannelList and DirectMessageList with per-item unread tracking
- Updated MessageList to display visual unread separator
- Integrated unread count fetching in MessageArea

**v0.8.0** - Keyboard Shortcuts (Phase 2.7)
- Implemented global keyboard shortcuts manager
- Cmd/Ctrl+K: Quick switcher for channels and DMs with search
- Cmd/Ctrl+/: Show keyboard shortcuts help modal
- Cmd/Ctrl+Enter: Send message from textarea
- Escape: Close modals and panels
- QuickSwitcher component with fuzzy search and keyboard navigation
- KeyboardShortcutsHelp modal showing all available shortcuts
- Category-based organization of shortcuts (Navigation, Messaging, General)
- Platform-aware shortcut display (⌘ for Mac, Ctrl for others)

**v0.7.0** - Message Drafts (Phase 2.6)
- Implemented IndexedDB-based draft storage per channel/DM
- Auto-save draft every 2 seconds while typing
- Automatic draft restore when switching channels/DMs
- Draft cleared automatically on message send
- Drafts persist across browser sessions
- Clean draft management with automatic cleanup of empty drafts

**v0.6.0** - Read Receipts & Message History (Phase 2.4-2.5)
- API integration for read receipts and message edit history
- Backend support for tracking message views
- Foundation for future UI implementation

**v0.5.0** - Rich Text Formatting (Phase 2.2)
- Full Markdown support with live preview toggle
- Markdown toolbar with formatting shortcuts
- Syntax highlighting for code blocks
- Sanitized HTML output to prevent XSS attacks
- Support for bold, italic, code, lists, links, quotes, and headings

**v0.4.0** - Thread Display UI (Phase 1.2)
- Added `first_reply` field to Message type for inline thread previews
- Enhanced MessageItem to display inline preview of first reply with author name
- Improved thread indicator button with better styling and hover effects
- Added breadcrumb navigation to ThreadPanel showing author and reply count
- Reduced thread polling interval from 5s to 2s for near-realtime updates
- Thread panel now refetches on window focus for better sync
- Enhanced ThreadPanel header layout with multi-line support for breadcrumbs
- API client updated to support first_reply in message responses

**v0.3.0** - Unread message tracking (API support) - SUPERSEDED BY v0.9.0
- Added UnreadCountResponse and MarkAsReadRequest types
- Implemented markChannelAsRead() and getChannelUnreadCount() API methods
- Implemented markDmAsRead() and getDmUnreadCount() API methods
- Added WebSocket message type for unread count updates
- API foundation for UI implementation (completed in v0.9.0)

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
