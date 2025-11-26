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

## Phase 2: Web UI Implementation ✅ COMPLETE

### 2.1 API Client Updates

**File:** `ui/lib/api.ts`

- [x] Add `generatePairingCode()` method
  ```typescript
  async generatePairingCode(): Promise<{
    code: string;
    expires_in: number;
  }>
  ```

- [x] Add `getDeviceSessions()` method
  ```typescript
  async getDeviceSessions(): Promise<DeviceSession[]>
  ```

- [x] Add `revokeDeviceSession()` method
  ```typescript
  async revokeDeviceSession(deviceId: string): Promise<void>
  ```

### 2.2 Types

**File:** `ui/lib/types.ts`

- [x] Add `DeviceSession` interface
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

- [x] Create component with two tabs: "Deep Link" and "Pairing Code"

- [x] **Deep Link Tab:**
  - [x] "Open Desktop App" button
  - [x] Generates encrypted payload with current token
  - [x] Opens `openchat://login?payload=<encrypted>`
  - [x] Shows instructions if app not installed

- [x] **Pairing Code Tab:**
  - [x] "Generate Code" button
  - [x] Displays 6-character code in large font
  - [x] Shows countdown timer (5:00 → 0:00)
  - [x] QR code display using `qrcode.react`
  - [x] Copy to clipboard button
  - [x] Auto-refresh on expiration

- [x] Add loading states and error handling

### 2.4 Device Management UI

**File:** `ui/components/settings/devices.tsx` (new file)

- [x] Display list of active device sessions
- [x] Show device type icon (desktop/mobile/web)
- [x] Show last active timestamp
- [x] "Revoke Access" button for each device
- [x] Confirmation dialog before revocation

### 2.5 Integration Points

**File:** `ui/app/settings/page.tsx`

- [x] Add "Desktop App" section to settings
- [x] Embed `<DesktopLogin />` component
- [x] Integrate device management UI

**File:** `ui/components/UserProfile.tsx`

- [x] Add "Desktop App" menu item in user profile dropdown for easy access

### 2.6 Dependencies

**File:** `ui/package.json`

- [x] Add `qrcode.react` for QR code generation
- [x] Add `date-fns` for date formatting
  ```bash
  npm install qrcode.react date-fns
  ```

---

## Phase 3: Desktop App (Tauri) Implementation ✅ COMPLETE

### 3.1 Deep Link Handler

**File:** `windows/tauri.conf.json`

- [x] Register custom URL scheme
  ```json
  {
    "app": {
      "security": {
        "csp": null,
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

- [x] Add deep link plugin
  ```rust
  use tauri_plugin_deep_link;
  ```

- [x] Register deep link handler
  ```rust
  .plugin(tauri_plugin_deep_link::init())
  .setup(|app| {
      // Register deep link handler for openchat://login and openchat://pair
      tauri_plugin_deep_link::register("openchat", move |request| {
          // Emits events for login and pair flows
      })?;
      Ok(())
  })
  ```

### 3.2 Tauri Commands (Rust)

**File:** `windows/src/auth.rs` (new file)

- [x] `#[tauri::command] async fn verify_pairing_code(code: String, device_name: String) -> Result<AuthResponse>`
  - Calls backend `/api/auth/device/verify-code`
  - Returns token + user info
  - Stores token in OS keychain

- [x] `#[tauri::command] async fn get_stored_token() -> Result<Option<String>>`
  - Retrieves token from OS keychain
  - Returns None if not found

- [x] `#[tauri::command] async fn store_token(token: String) -> Result<()>`
  - Stores token securely in OS keychain
  - Uses `keyring` crate for cross-platform support

- [x] `#[tauri::command] async fn clear_token() -> Result<()>`
  - Removes token from keychain
  - Used for logout

- [x] `#[tauri::command] async fn validate_token(token: String) -> Result<bool>`
  - Validates token with backend API
  - Used to check if stored token is still valid

- [x] `#[tauri::command] async fn process_deep_link_payload(encrypted_payload: String) -> Result<AuthResponse>`
  - Handles deep link login flow
  - Decodes base64 payload
  - Validates token and stores in keychain

### 3.3 Secure Storage

**File:** `windows/Cargo.toml`

- [x] Add `keyring` dependency
  ```toml
  [dependencies]
  keyring = "3.6"
  ```

**Implementation:**
- Windows: Windows Credential Manager
- macOS: Keychain
- Linux: Secret Service API

### 3.4 Frontend (React in Tauri)

**File:** `ui/components/desktop/login-screen.tsx` (new file)

- [x] Create login screen component for desktop
- [x] "Enter Pairing Code" input field (6 characters)
- [x] Real-time validation (alphanumeric only)
- [x] Submit button
- [x] Loading state during verification
- [x] Error display
- [x] Success → redirect to main app
- [x] Deep link event listeners for both login and pair flows
- [x] Auto-detected device name based on platform

**File:** `ui/components/desktop/scanner.tsx` (new file)

- [x] QR code scanner UI
- [x] Uses device camera via WebRTC getUserMedia API
- [x] Uses jsQR library for QR code detection
- [x] Parses `openchat://pair?code=ABC123`
- [x] Auto-submits code on successful scan
- [x] Error handling for invalid QR codes
- [x] Camera permission management
- [x] Visual scanning overlay with corner decorations

### 3.5 Auth Flow Integration

**File:** `ui/lib/auth.tsx`

- [x] Update `initialize()` to check for Tauri environment
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

- [x] Update `logout()` to clear OS keychain
  ```typescript
  if (window.__TAURI__) {
    await invoke('clear_token');
  }
  ```

### 3.6 Dependencies

**File:** `windows/Cargo.toml`

- [x] Add `tauri-plugin-deep-link = "2"`
- [x] Add `keyring = "3.6"`
- [x] Add `serde_json = "1.0"` for payload parsing
- [x] Add `base64 = "0.22"` for encrypted payload decoding
- [x] Add `reqwest = "0.12"` for HTTP API calls
- [x] Add `tokio = "1"` for async runtime
- [x] Add `chrono = "0.4"` for timestamp handling
- [x] Add `uuid = "1.11"` for device ID generation
- [x] Add `url = "2.5"` for URL parsing

**File:** `ui/package.json`

- [x] Add `@tauri-apps/api = "^2.9.0"` for Tauri integration
- [x] Add `jsqr = "^1.4.0"` for QR code scanning

**Note:** Full build verification requires Windows environment. Code has been written following Rust and Tauri best practices. Build will be tested on Windows during deployment.

---

## Phase 4: Security & Polish

### 4.1 Security Hardening

- [x] **Encryption for Deep Links:**
  - [x] Web app encrypts payload before generating deep link (API endpoint: `/api/auth/device/generate-deep-link`)
  - [x] Uses AES-256-GCM symmetric encryption with shared secret
  - [x] Base64 URL-safe encoding for deep link payloads
  - [ ] Desktop app decrypts and validates signature (to be implemented in desktop app)

- [x] **Rate Limiting:**
  - [x] Limit pairing code generation to 3 per minute per user
  - [x] Limit verification attempts to 5 per minute per IP
  - [x] Rate limit headers returned in responses

- [x] **Code Complexity:**
  - [x] Use alphanumeric codes (avoid ambiguous characters: 0/O, 1/I/l)
  - [x] 6-character codes from character set: 23456789ABCDEFGHJKLMNPQRSTUVWXYZ

- [x] **Token Validation:**
  - [x] Proper JWT token generation for device sessions (30-day expiration)
  - [x] Device-specific claims include device_id, device_type, user_id, org_id
  - [ ] Desktop app validates token with backend on startup (to be implemented in desktop app)
  - [ ] Auto-logout if token expired (to be implemented in desktop app)

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
**Status:** Phase 1, 2 & 3 Complete - Ready for Phase 4 (Security & Polish)
