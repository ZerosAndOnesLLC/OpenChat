# OpenChat Windows Desktop Client

A native Windows Forms desktop client for OpenChat.

## Features

- **Device Authentication**: Secure pairing code-based authentication
- **Real-time Messaging**: WebSocket-based real-time message delivery
- **Channels**: View and participate in public and private channels
  - Browse and search public channels
  - Join public channels directly from the app
  - Click the + button next to "Channels" to browse
- **Direct Messages**: 1-on-1 and group conversations
  - Start new DM conversations with any user in your org
  - Click the + button next to "Direct Messages" to start a new conversation
  - Search users by name or email
- **User Status**: Set and display your presence status
  - Click on your username to change status
  - Status options: Online, Away, Do Not Disturb, Invisible
  - Set custom status messages with emoji
  - Auto-clear status after a set time period
- **Emoji Support**: Full emoji integration with:
  - WebView2-based emoji picker matching the web UI exactly
  - Categories: Smileys, Gestures, People, Animals, Food, Activities, Travel, Objects, Symbols
  - Frequently used emojis section for quick access
  - Custom organization emojis synced from the server
  - Emoji search/filter functionality
  - Inline custom emoji rendering in messages
  - Local emoji image caching for performance
- **Message Reactions**: Slack-style reactions on messages
  - Click + button on any message to add a reaction
  - Quick reaction picker with common emojis
  - Full emoji picker for more options
  - Click existing reaction to toggle (add/remove)
  - Reaction counts with visual indicator for your reactions
  - Highlighted reactions you've added
- **Modern Dark UI**: Slack/Mattermost-inspired dark theme with:
  - Custom-drawn sidebar with rounded selection highlights
  - Unread badges with notification counts
  - Online status indicators with real-time updates
  - Smooth anti-aliased graphics
  - Rounded input fields and buttons
  - Beautiful emoji picker matching the web UI
- **Message History**: Load and view message history
- **Typing Indicators**: See when others are typing (backend support)
- **Unread Counts**: Track unread messages per channel/DM

## Requirements

- Windows 10 or later
- .NET 10.0 Runtime
- Microsoft Edge WebView2 Runtime (for emoji picker)
- Internet connection to connect to OpenChat API

## Building from Source

1. Open the solution in Visual Studio 2022 or later
2. Restore NuGet packages:
   ```bash
   dotnet restore
   ```
3. Build the project:
   ```bash
   cd OpenChat
   dotnet build
   ```
4. Run the application:
   ```bash
   dotnet run
   ```

## Usage

### First-Time Setup

1. Launch the OpenChat Windows application
2. You'll see a beautiful login screen with step-by-step instructions
3. Click the web app link (or manually go to https://openchat.zerosandones.us)
4. Log in with your TitaniumVault account
5. Click on your profile and select "Pair Desktop App" (or similar option)
6. A pairing code will be displayed (e.g., "ABC12345")
7. Enter this code in the Windows app and click "Connect"
8. The app will authenticate and open the main chat interface

### Using the Application

**Channels Tab:**
- View all channels you're a member of
- Unread counts shown in parentheses
- Click a channel to view its messages
- Public channels are prefixed with `#`

**Direct Messages Tab:**
- View all your direct message conversations
- Unread counts shown in parentheses
- Click a DM to view messages with that user

**Sending Messages:**
- Type your message in the text box at the bottom
- Press Enter or click "Send" to send
- Supports Shift+Enter for new lines

**Using Emojis:**
- Click the 😀 button next to the Send button to open the emoji picker
- **Standard Tab**: Browse Unicode emojis by category or use the search bar
- **Custom Tab**: View and use your organization's custom emojis
- Click any emoji to insert it into your message
- Custom emojis use the `:emoji_name:` format (e.g., `:thumbsup:`)
- Admins can upload custom emojis by clicking "+ Upload" in the Custom tab

## Architecture

The application is structured as follows:

```
OpenChat/
├── Models/              # Data models matching API responses
│   ├── User.cs          # User and UserStatus models
│   ├── Channel.cs
│   ├── Message.cs
│   ├── DirectMessage.cs
│   ├── CustomEmoji.cs   # Custom emoji models
│   ├── Reaction.cs      # Message reaction models
│   └── AuthResponse.cs
├── Controls/            # Custom UI controls
│   └── MessagePanel.cs  # Virtualized message display with reactions
├── Services/            # Business logic and API communication
│   ├── ApiClient.cs     # REST API client (channels, DMs, status, emojis, reactions)
│   ├── WebSocketClient.cs  # WebSocket client for real-time updates
│   ├── EmojiCache.cs    # Local emoji caching service
│   └── CredentialManager.cs  # Secure credential storage using DPAPI
├── Theme.cs             # Dark theme color palette and fonts
├── LoginForm.cs         # Device authentication UI
├── LoginForm.Designer.cs
├── MainForm.cs          # Main chat interface
├── MainForm.Designer.cs
├── WebEmojiPickerForm.cs # WebView2-based emoji picker (matches web UI)
├── EmojiPickerForm.cs   # Legacy virtualized emoji picker (fallback)
├── ReactionPickerForm.cs # Quick reaction picker popup
├── EmojiUploadDialog.cs # Custom emoji upload dialog
├── StatusPickerForm.cs  # User status picker (online/away/dnd/offline)
├── BrowseChannelsForm.cs # Public channel browser and join
├── NewDmForm.cs         # New direct message dialog
└── Program.cs           # Application entry point
```

## Configuration

The application connects to:
- **API Base URL**: `https://openchat-api.zerosandones.us:9876`
- **WebSocket URL**: `wss://openchat-api.zerosandones.us:9876/api/ws`

To change these URLs, edit the respective files:
- `LoginForm.cs` line 24 (API URL)
- `MainForm.cs` line 31 (API URL) and line 205 (WebSocket URL)

## Security

- Access tokens are encrypted and stored locally using Windows DPAPI
- Credentials are stored in `%LOCALAPPDATA%\OpenChat\credentials.dat`
- Credentials remain valid for 365 days before requiring re-pairing
- Device pairing uses secure 8-character codes
- All communication uses HTTPS/WSS encryption
- Tokens expire after 30 days (server-side)
- Device sessions can be revoked from the web interface

## Troubleshooting

**Login Failed:**
- Ensure the pairing code is entered correctly (case-insensitive)
- Pairing codes expire after 5 minutes
- Check your internet connection

**WebSocket Connection Failed:**
- Check firewall settings
- Ensure you're not behind a proxy that blocks WebSocket connections
- Try restarting the application

**Messages Not Appearing:**
- Check that you're a member of the channel
- Try refreshing by selecting another channel and back
- Verify your internet connection

## Known Limitations

- No support for file attachments yet
- No support for threaded conversations yet
- No notifications for new messages when app is in background

## Future Enhancements

- [ ] File upload/download support
- [x] Message reactions
- [ ] Threaded conversations
- [ ] Desktop notifications
- [ ] System tray integration
- [ ] Message search
- [x] User presence/status indicators
- [x] Status picker (online/away/dnd/offline)
- [ ] Typing indicators display
- [ ] Message editing
- [ ] Message deletion
- [ ] Emoji autocomplete while typing
- [x] Browse and join public channels
- [x] Create new DM conversations

## License

Same as the main OpenChat project - Server Side Public License (SSPL) v1.

## Contributing

Contributions are welcome! Please follow the main OpenChat contribution guidelines.
