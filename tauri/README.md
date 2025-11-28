# OpenChat Desktop

A cross-platform desktop application for OpenChat built with Tauri and Rust.

## Overview

OpenChat Desktop provides a native desktop experience for OpenChat, leveraging the existing Next.js web UI within a lightweight, secure Rust-based application framework.

## Technology Stack

- **Backend**: Rust with Tauri framework
- **Frontend**: Next.js (shared with web application)
- **Platforms**: Windows, macOS, Linux

## Features

- Native desktop application experience
- System tray integration
- Native notifications
- File system access
- Shell integration
- Lightweight (~3-10MB bundle size)
- Secure communication with native APIs
- Secure credential storage with 365-day expiry (OS keychain with file fallback for Windows)
- Deep link authentication support
- Robust token validation with graceful handling of network issues
- **Configurable Server URL**: Connect to any OpenChat server instance - no hardcoded URLs
- **Window State Persistence**: Remembers window position, size, and monitor across restarts (multi-monitor support)

## Quick Start

1. Launch the OpenChat Desktop app
2. On first launch, you'll see a login screen
3. Enter the **Server URL** (provided by your OpenChat instance)
4. Enter the **Pairing Code** (generate one from the web app: Settings → Desktop App)
5. Click Connect - you're logged in!

The app remembers your server and credentials, so subsequent launches go straight to chat.

## Prerequisites

### Windows
- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/) (v18 or later)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually pre-installed on Windows 10/11)
- Visual Studio Build Tools or Visual Studio with C++ development tools

### macOS
- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/) (v18 or later)
- Xcode Command Line Tools: `xcode-select --install`

### Linux
- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/) (v18 or later)
- System dependencies:
  ```bash
  # Debian/Ubuntu
  sudo apt update
  sudo apt install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

  # Fedora
  sudo dnf install webkit2gtk4.1-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel

  # Arch
  sudo pacman -S webkit2gtk-4.1 \
    base-devel \
    curl \
    wget \
    file \
    openssl \
    libappindicator-gtk3 \
    librsvg
  ```

## Development

### Setup

1. Install dependencies:
```bash
# Install UI dependencies (from openchat/ui directory)
cd ../ui
npm install

# Install Tauri CLI
cargo install tauri-cli
```

2. Run in development mode:
```bash
# From the src-tauri directory
cargo tauri dev
```

This will:
- Start the Next.js development server at `http://localhost:3000`
- Launch the Tauri application window
- Enable hot-reload for both frontend and backend changes

### Building

To create a production build:

```bash
# From the src-tauri directory
cargo tauri build
```

The built application will be in `src-tauri/target/release/bundle/`:
- **Windows**: `.msi` installer in `msi/` and `.exe` in `nsis/`
- **macOS**: `.dmg` and `.app` in `dmg/` and `macos/`
- **Linux**: `.deb`, `.AppImage`, or `.rpm` depending on your system

## Project Structure

```
src-tauri/
├── src/
│   ├── main.rs          # Application entry point
│   └── lib.rs           # Core application logic and plugin initialization
├── icons/               # Application icons for different platforms
├── capabilities/        # Tauri security capabilities configuration
├── Cargo.toml          # Rust dependencies and metadata
├── tauri.conf.json     # Tauri application configuration
└── build.rs            # Build script
```

## Configuration

### Application Settings

Edit `tauri.conf.json` to customize:
- Window size and behavior
- Application identifier
- Build targets
- Security policies
- Plugin permissions

### Rust Dependencies

Manage Rust dependencies in `Cargo.toml`. Current plugins:
- `tauri-plugin-shell`: Execute shell commands
- `tauri-plugin-dialog`: Native file dialogs
- `tauri-plugin-fs`: File system access
- `tauri-plugin-notification`: System notifications
- `tauri-plugin-log`: Logging functionality
- `tauri-plugin-deep-link`: Deep link / URL protocol handling
- `keyring`: Secure OS credential storage (Windows Credential Manager, macOS Keychain, Linux Secret Service)

## Security

Tauri follows a capabilities-based security model. All plugin permissions must be explicitly declared in the `capabilities/` directory. This ensures:
- Minimal attack surface
- Explicit permission grants
- Secure IPC communication between frontend and backend

## Performance

Tauri applications are significantly lighter than Electron:
- **Bundle Size**: 3-10MB (vs 50-100MB for Electron)
- **Memory Usage**: Uses OS native WebView
- **Startup Time**: Faster due to smaller binary size
- **Resource Usage**: Lower CPU and memory footprint

## Troubleshooting

### Build Errors on Linux

If you encounter WebKit2GTK errors:
```bash
# Make sure all system dependencies are installed
sudo apt install libwebkit2gtk-4.1-dev
```

### Windows Build Issues

If you encounter Visual Studio errors:
- Install Visual Studio Build Tools with C++ development tools
- Ensure WebView2 runtime is installed

### Code Signing (macOS/Windows)

For distribution, you'll need to code sign the application:
- **macOS**: Apple Developer Account and certificate
- **Windows**: Code signing certificate

## Contributing

When making changes:
1. Run `cargo check` to verify Rust code
2. Run `cargo tauri dev` to test changes
3. Ensure both UI and desktop functionality work
4. Update version in `Cargo.toml` before committing

## Version Management

Follow semantic versioning in `Cargo.toml`:
- **Major**: Breaking changes or big rewrites
- **Minor**: New features (backward-compatible)
- **Patch**: Bug fixes and small tweaks

## License

SSPL-1.0 (Server Side Public License)

Copyright (c) 2025 Zeros and Ones LLC
