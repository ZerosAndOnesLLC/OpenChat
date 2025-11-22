# OpenChat Desktop Quick Login - Implementation Plan

## Overview

Implement a seamless desktop authentication flow that allows users to log into the Tauri desktop app without re-entering credentials. This will use a hybrid approach with **Deep Link OAuth** as primary and **Device Pairing Codes** as fallback.

**Goals:**
- Enable one-click login from web app to desktop app
- Provide pairing code fallback for manual entry
- Optional QR code scanning for camera-enabled devices
- Secure token storage using OS keychain instead of localStorage
- Track desktop devices as separate sessions

---

## Architecture

```
┌─────────────┐                    ┌──────────────┐                    ┌─────────────┐
│   Web App   │                    │  Backend API │                    │ Desktop App │
│  (Next.js)  │                    │    (Rust)    │                    │   (Tauri)   │
└─────────────┘                    └──────────────┘                    └─────────────┘
       │                                   │                                    │
       │  1. Generate pairing code         │                                    │
       ├──────────────────────────────────>│                                    │
       │  Returns: { code: "ABC123" }      │                                    │
       │<──────────────────────────────────┤                                    │
       │                                   │                                    │
       │  2. Display code + QR             │                                    │
       │  "Enter ABC123 in desktop app"    │                                    │
       │                                   │                                    │
       │                                   │  3. User enters code               │
       │                                   │<───────────────────────────────────┤
       │                                   │                                    │
       │                                   │  4. Verify code, return JWT        │
       │                                   ├───────────────────────────────────>│
       │                                   │  Returns: { token, user }          │
       │                                   │                                    │
       │                                   │                                    │
       │  Alternative: Deep Link           │                                    │
       ├───────────────────────────────────────────────────────────────────────>│
       │  openchat://login?payload=<encrypted>                                  │
```

---

## Phase 1: Backend Infrastructure ✅ COMPLETE

### 1.1 Database Schema

**File:** `api/migrations/20251122185935_add_device_pairing.sql`

- [x] Create `device_pairing_codes` table
  ```sql
  CREATE TABLE device_pairing_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(6) NOT NULL UNIQUE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    org_id UUID NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    used BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
  );
  CREATE INDEX idx_device_pairing_codes_code ON device_pairing_codes(code);
  CREATE INDEX idx_device_pairing_codes_expires ON device_pairing_codes(expires_at);
  ```

- [x] Create `device_sessions` table for tracking desktop devices
  ```sql
  CREATE TABLE device_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    org_id UUID NOT NULL,
    device_type VARCHAR(50) NOT NULL, -- 'desktop', 'mobile', 'web'
    device_name VARCHAR(255),
    device_fingerprint TEXT,
    last_active_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
  );
  CREATE INDEX idx_device_sessions_user ON device_sessions(user_id);
  ```

### 1.2 Rust Models

**File:** `api/src/models/device.rs` (new file)

- [x] Create `DevicePairingCode` model
  ```rust
  pub struct DevicePairingCode {
      pub id: Uuid,
      pub code: String,
      pub user_id: Uuid,
      pub org_id: Uuid,
      pub expires_at: DateTime<Utc>,
      pub used: bool,
      pub created_at: DateTime<Utc>,
  }
  ```

- [x] Create `DeviceSession` model
  ```rust
  pub struct DeviceSession {
      pub id: Uuid,
      pub user_id: Uuid,
      pub org_id: Uuid,
      pub device_type: String,
      pub device_name: Option<String>,
      pub device_fingerprint: Option<String>,
      pub last_active_at: DateTime<Utc>,
      pub created_at: DateTime<Utc>,
  }
  ```

- [x] Add module declaration in `api/src/models/mod.rs`

### 1.3 Backend Services

**File:** `api/src/services/device_pairing.rs` (new file)

- [x] Implement `generate_pairing_code()` function
  - Generate random 6-character alphanumeric code
  - Check uniqueness in database
  - Set expiration (5 minutes)
  - Return code to web app

- [x] Implement `verify_pairing_code()` function
  - Validate code exists and not expired
  - Check not already used
  - Mark as used
  - Generate JWT token for desktop app
  - Create device session record
  - Return token + user info

- [x] Implement `cleanup_expired_codes()` background job
  - Delete codes older than 10 minutes
  - Run every 5 minutes via cron or tokio task

### 1.4 API Routes

**File:** `api/src/handlers/device_auth.rs` (new file)

- [x] `POST /api/auth/device/generate-code`
  - **Auth:** Requires valid JWT (from web user)
  - **Request:** `{}`
  - **Response:** `{ code: "ABC123", expires_in: 300 }`

- [x] `POST /api/auth/device/verify-code`
  - **Auth:** None (public endpoint)
  - **Request:** `{ code: "ABC123", device_name: "My Windows PC" }`
  - **Response:** `{ access_token: "jwt...", user: {...}, device_id: "uuid" }`

- [x] `GET /api/auth/device/sessions`
  - **Auth:** Requires valid JWT
  - **Response:** List of active device sessions for user

- [x] `DELETE /api/auth/device/sessions/:id`
  - **Auth:** Requires valid JWT
  - **Purpose:** Revoke device session / logout from device

- [x] Register routes in `api/src/main.rs`

### 1.5 Redis Caching (Optional Performance)

**File:** `api/src/services/device_pairing.rs`

- [ ] Cache pairing codes in Redis instead of DB (deferred - using PostgreSQL for Phase 1)
  - Key: `pairing_code:{CODE}`
  - Value: `{ user_id, org_id, expires_at }`
  - TTL: 5 minutes (automatic expiration)

---

## Phase 2: Web UI Implementation

### 2.1 API Client Updates

**File:** `ui/lib/api.ts`

- [ ] Add `generatePairingCode()` method
  ```typescript
  async generatePairingCode(): Promise<{
    code: string;
    expires_in: number;
  }>
  ```

- [ ] Add `getDeviceSessions()` method
  ```typescript
  async getDeviceSessions(): Promise<DeviceSession[]>
  ```

- [ ] Add `revokeDeviceSession()` method
  ```typescript
  async revokeDeviceSession(deviceId: string): Promise<void>
  ```

### 2.2 Types

**File:** `ui/lib/types.ts`

- [ ] Add `DeviceSession` interface
  ```typescript
  export interface DeviceSession {
    id: string;
    device_type: 'desktop' | 'mobile' | 'web';
    device_name?: string;
    last_active_at: string;
    created_at: string;
  }
  ```

### 2.3 Desktop Login Component

**File:** `ui/components/desktop-login.tsx` (new file)

- [ ] Create component with two tabs: "Deep Link" and "Pairing Code"

- [ ] **Deep Link Tab:**
  - [ ] "Open Desktop App" button
  - [ ] Generates encrypted payload with current token
  - [ ] Opens `openchat://login?payload=<encrypted>`
  - [ ] Shows instructions if app not installed

- [ ] **Pairing Code Tab:**
  - [ ] "Generate Code" button
  - [ ] Displays 6-character code in large font
  - [ ] Shows countdown timer (5:00 → 0:00)
  - [ ] QR code display using `qrcode.react`
  - [ ] Copy to clipboard button
  - [ ] Auto-refresh on expiration

- [ ] Add loading states and error handling

### 2.4 Device Management UI

**File:** `ui/components/settings/devices.tsx` (new file)

- [ ] Display list of active device sessions
- [ ] Show device type icon (desktop/mobile/web)
- [ ] Show last active timestamp
- [ ] "Revoke Access" button for each device
- [ ] Confirmation dialog before revocation

### 2.5 Integration Points

**File:** `ui/app/settings/page.tsx` or similar

- [ ] Add "Desktop App" section to settings
- [ ] Embed `<DesktopLogin />` component
- [ ] Add "Manage Devices" link to device management page

**File:** `ui/app/page.tsx`

- [ ] Show prominent "Login Desktop App" button if user is authenticated on web

### 2.6 Dependencies

**File:** `ui/package.json`

- [ ] Add `qrcode.react` for QR code generation
  ```bash
  npm install qrcode.react
  npm install --save-dev @types/qrcode.react
  ```

---

## Phase 3: Desktop App (Tauri) Implementation

### 3.1 Deep Link Handler

**File:** `windows/tauri.conf.json`

- [ ] Register custom URL scheme
  ```json
  {
    "app": {
      "security": {
        "dangerousRemoteDomainIpcAccess": [],
        "capabilities": ["deep-link"]
      }
    },
    "bundle": {
      "deeplink": {
        "protocol": "openchat",
        "schemes": ["openchat"]
      }
    }
  }
  ```

**File:** `windows/src/lib.rs`

- [ ] Add deep link plugin
  ```rust
  use tauri_plugin_deep_link;
  ```

- [ ] Register deep link handler
  ```rust
  .plugin(tauri_plugin_deep_link::init())
  .setup(|app| {
      app.handle().plugin(
          tauri_plugin_deep_link::Builder::new()
              .on_open_url(|url| {
                  // Handle openchat://login?payload=...
                  // Handle openchat://pair?code=...
              })
              .build(),
      )?;
      Ok(())
  })
  ```

### 3.2 Tauri Commands (Rust)

**File:** `windows/src/auth.rs` (new file)

- [ ] `#[tauri::command] async fn verify_pairing_code(code: String, device_name: String) -> Result<AuthResponse>`
  - Calls backend `/api/auth/device/verify-code`
  - Returns token + user info
  - Stores token in OS keychain

- [ ] `#[tauri::command] async fn get_stored_token() -> Result<Option<String>>`
  - Retrieves token from OS keychain
  - Returns None if not found

- [ ] `#[tauri::command] async fn store_token(token: String) -> Result<()>`
  - Stores token securely in OS keychain
  - Uses `keyring` crate for cross-platform support

- [ ] `#[tauri::command] async fn clear_token() -> Result<()>`
  - Removes token from keychain
  - Used for logout

### 3.3 Secure Storage

**File:** `windows/Cargo.toml`

- [ ] Add `keyring` dependency
  ```toml
  [dependencies]
  keyring = "2.3"
  ```

**Implementation:**
- Windows: Windows Credential Manager
- macOS: Keychain
- Linux: Secret Service API

### 3.4 Frontend (React in Tauri)

**File:** `ui/components/desktop/login-screen.tsx` (new file)

- [ ] Create login screen component for desktop
- [ ] "Enter Pairing Code" input field (6 characters)
- [ ] Real-time validation (alphanumeric only)
- [ ] Submit button
- [ ] Loading state during verification
- [ ] Error display
- [ ] Success → redirect to main app

**File:** `ui/components/desktop/scanner.tsx` (new file - optional)

- [ ] QR code scanner UI
- [ ] Uses device camera via Tauri plugin
- [ ] Parses `openchat://pair?code=ABC123`
- [ ] Auto-submits code

### 3.5 Auth Flow Integration

**File:** `ui/lib/auth.tsx`

- [ ] Update `initialize()` to check for Tauri environment
  ```typescript
  if (window.__TAURI__) {
    // Desktop flow: check OS keychain first
    const token = await invoke('get_stored_token');
    if (token) {
      // Validate token with backend
      // Set auth state
    } else {
      // Show pairing code screen
    }
  } else {
    // Web flow: existing OAuth
  }
  ```

- [ ] Update `logout()` to clear OS keychain
  ```typescript
  if (window.__TAURI__) {
    await invoke('clear_token');
  }
  ```

### 3.6 Dependencies

**File:** `windows/Cargo.toml`

- [ ] Add `tauri-plugin-deep-link` (if not bundled)
- [ ] Add `keyring = "2.3"`
- [ ] Add `serde_json` for payload parsing
- [ ] Add `base64` for encrypted payload decoding

**File:** `ui/package.json`

- [ ] Ensure `@tauri-apps/api` is installed and up-to-date

---

## Phase 4: Security & Polish

### 4.1 Security Hardening

- [ ] **Encryption for Deep Links:**
  - [ ] Web app encrypts payload before generating deep link
  - [ ] Desktop app decrypts and validates signature
  - [ ] Use symmetric encryption (AES-256-GCM) with shared secret

- [ ] **Rate Limiting:**
  - [ ] Limit pairing code generation to 3 per minute per user
  - [ ] Limit verification attempts to 5 per minute per IP

- [ ] **Code Complexity:**
  - [ ] Use alphanumeric codes (avoid ambiguous characters: 0/O, 1/I/l)
  - [ ] Consider using words instead of random chars for accessibility

- [ ] **Token Validation:**
  - [ ] Desktop app validates token with backend on startup
  - [ ] Auto-logout if token expired

### 4.2 UX Improvements

- [ ] **Auto-detection:**
  - [ ] Web app detects if desktop app is installed
  - [ ] Show appropriate CTA based on detection

- [ ] **Notifications:**
  - [ ] Desktop app shows system notification on successful login
  - [ ] Web app shows toast when desktop device connects

- [ ] **Accessibility:**
  - [ ] High contrast mode for pairing code display
  - [ ] Screen reader support for all flows
  - [ ] Keyboard navigation

### 4.3 Error Handling

- [ ] **Web App:**
  - [ ] Handle code generation failures
  - [ ] Show retry button on errors
  - [ ] Graceful degradation if backend unavailable

- [ ] **Desktop App:**
  - [ ] Handle invalid codes with clear error messages
  - [ ] Network error handling (offline mode)
  - [ ] Corrupted keychain data recovery

### 4.4 Testing

- [ ] **Backend Unit Tests:**
  - [ ] Test pairing code generation uniqueness
  - [ ] Test code expiration logic
  - [ ] Test verification with invalid/expired codes

- [ ] **Integration Tests:**
  - [ ] End-to-end pairing flow
  - [ ] Deep link handling
  - [ ] Token storage/retrieval from keychain

- [ ] **Manual Testing:**
  - [ ] Test on Windows 10/11
  - [ ] Test on macOS (if supporting)
  - [ ] Test on Linux (if supporting)
  - [ ] Test with slow network
  - [ ] Test with backend down

---

## Phase 5: Documentation & Deployment

### 5.1 User Documentation

**File:** `windows/README.md`

- [ ] Add "Quick Login" section
- [ ] Screenshot of pairing code flow
- [ ] Troubleshooting guide

**File:** `docs/desktop-login.md` (new file)

- [ ] Step-by-step user guide
- [ ] FAQ section
- [ ] Known issues

### 5.2 Developer Documentation

**File:** `docs/architecture/desktop-auth.md` (new file)

- [ ] Architecture diagram
- [ ] API endpoint documentation
- [ ] Security considerations
- [ ] Future improvements

### 5.3 Deployment Checklist

- [ ] Run database migrations on production
- [ ] Deploy updated backend API
- [ ] Deploy updated web UI
- [ ] Build and sign desktop app installers
- [ ] Test deep links on clean install
- [ ] Monitor error rates in first 24 hours

### 5.4 Monitoring

- [ ] Add metrics for:
  - [ ] Pairing code generation rate
  - [ ] Successful vs failed verifications
  - [ ] Desktop device session counts
  - [ ] Token validation errors

- [ ] Set up alerts for:
  - [ ] High verification failure rate (>10%)
  - [ ] Unusual pairing code generation spikes
  - [ ] Keychain access errors

---

## Success Criteria

- [ ] User can generate pairing code from web app in <2 seconds
- [ ] Desktop app accepts valid code and logs in in <3 seconds
- [ ] Tokens are stored securely in OS keychain (verified with security audit)
- [ ] Deep link flow works on Windows 10/11
- [ ] User can manage and revoke device sessions from web UI
- [ ] No more than 0.1% error rate in production after 1 week
- [ ] Desktop app remembers login across restarts
- [ ] Zero plain-text token storage in desktop app

---

## Future Enhancements

- [ ] Biometric authentication (Windows Hello, Touch ID)
- [ ] Multi-factor authentication for sensitive actions
- [ ] Device location tracking (with user consent)
- [ ] Suspicious device login alerts
- [ ] Token refresh rotation
- [ ] Cross-device notification sync
- [ ] Backup codes for keychain loss
- [ ] Web-based device approval flow (similar to GitHub)

---

## Notes

- **Priority:** Focus on pairing code flow first (Phase 1-3), then add deep links
- **Security:** Never log tokens or codes in production logs
- **Compatibility:** Target Windows 10/11 initially, expand to macOS/Linux later
- **Performance:** Code verification should be <100ms at p99
- **Fallback:** If keychain unavailable, gracefully degrade to session-only tokens

---

## Timeline Estimate

- **Phase 1 (Backend):** 2-3 days
- **Phase 2 (Web UI):** 2-3 days
- **Phase 3 (Desktop):** 3-4 days
- **Phase 4 (Security & Polish):** 1-2 days
- **Phase 5 (Documentation):** 1 day

**Total:** ~10-14 days for complete implementation

---

**Last Updated:** 2025-11-22
**Author:** Claude Code
**Status:** Planning Phase
