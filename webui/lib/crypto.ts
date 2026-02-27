import { cryptoStore, type StoredIdentity, type StoredSession } from './crypto-store';

export interface EncryptedPayload {
  encrypted_content: string; // base64
  encryption_metadata: {
    algorithm: string;
    sender_device_id: string;
    session_id: string;
    iv: string; // base64
  };
}

export interface DeviceKeys {
  device_id: string;
  identity_key: string; // base64 public key
  signing_key: string;  // base64 public key
}

let identity: StoredIdentity | null = null;

function arrayBufferToBase64(buffer: ArrayBuffer | Uint8Array): string {
  const bytes = buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

function base64ToArrayBuffer(base64: string): ArrayBuffer {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

async function exportKeyToBase64(key: CryptoKey): Promise<string> {
  const exported = await crypto.subtle.exportKey('jwk', key);
  return btoa(JSON.stringify(exported));
}

async function generateDeviceId(): Promise<string> {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

export async function initCrypto(userId: string, existingDeviceId?: string): Promise<DeviceKeys | null> {
  // Check if we already have an identity in IndexedDB
  const stored = await cryptoStore.getIdentity();
  if (stored && stored.userId === userId) {
    identity = stored;
    return {
      device_id: stored.deviceId,
      identity_key: btoa(JSON.stringify(stored.identityKeyPair.publicKey)),
      signing_key: btoa(JSON.stringify(stored.signingKeyPair.publicKey)),
    };
  }

  // Generate new identity
  const deviceId = existingDeviceId || await generateDeviceId();

  // Generate ECDH key pair for key exchange
  const identityKeyPair = await crypto.subtle.generateKey(
    { name: 'ECDH', namedCurve: 'P-256' },
    true,
    ['deriveBits']
  );

  // Generate ECDSA key pair for signing
  const signingKeyPair = await crypto.subtle.generateKey(
    { name: 'ECDSA', namedCurve: 'P-256' },
    true,
    ['sign', 'verify']
  );

  const idPub = await crypto.subtle.exportKey('jwk', identityKeyPair.publicKey);
  const idPriv = await crypto.subtle.exportKey('jwk', identityKeyPair.privateKey);
  const sigPub = await crypto.subtle.exportKey('jwk', signingKeyPair.publicKey);
  const sigPriv = await crypto.subtle.exportKey('jwk', signingKeyPair.privateKey);

  identity = {
    userId,
    deviceId,
    identityKeyPair: { publicKey: idPub, privateKey: idPriv },
    signingKeyPair: { publicKey: sigPub, privateKey: sigPriv },
  };

  await cryptoStore.setIdentity(identity);

  return {
    device_id: deviceId,
    identity_key: btoa(JSON.stringify(idPub)),
    signing_key: btoa(JSON.stringify(sigPub)),
  };
}

async function getOrCreateSessionKey(sessionId: string): Promise<{ key: CryptoKey; session: StoredSession }> {
  const existing = await cryptoStore.getSession(sessionId);
  if (existing) {
    const key = await crypto.subtle.importKey(
      'jwk',
      existing.sessionKey,
      { name: 'AES-GCM', length: 256 },
      true,
      ['encrypt', 'decrypt']
    );
    return { key, session: existing };
  }

  // Generate new AES-256-GCM session key
  const key = await crypto.subtle.generateKey(
    { name: 'AES-GCM', length: 256 },
    true,
    ['encrypt', 'decrypt']
  );

  const exported = await crypto.subtle.exportKey('jwk', key);
  const session: StoredSession = {
    id: sessionId,
    sessionKey: exported,
    createdAt: Date.now(),
    messageCount: 0,
  };

  await cryptoStore.setSession(session);
  return { key, session };
}

export async function encryptMessage(content: string, channelId?: string, dmId?: string): Promise<EncryptedPayload | null> {
  if (!identity) return null;

  const sessionId = channelId || dmId;
  if (!sessionId) return null;

  const { key, session } = await getOrCreateSessionKey(sessionId);

  // Generate IV
  const iv = crypto.getRandomValues(new Uint8Array(12));

  // Encrypt
  const encoded = new TextEncoder().encode(content);
  const ciphertext = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv },
    key,
    encoded
  );

  // Increment message count
  session.messageCount++;
  await cryptoStore.setSession(session);

  return {
    encrypted_content: arrayBufferToBase64(ciphertext),
    encryption_metadata: {
      algorithm: 'aes-256-gcm',
      sender_device_id: identity.deviceId,
      session_id: sessionId,
      iv: arrayBufferToBase64(iv),
    },
  };
}

export async function decryptMessage(
  encryptedContent: string,
  metadata: { algorithm: string; session_id: string; iv: string }
): Promise<string> {
  if (!identity) throw new Error('Crypto not initialized');

  const sessionId = metadata.session_id;
  const existing = await cryptoStore.getSession(sessionId);
  if (!existing) {
    throw new Error('No session key available for decryption');
  }

  const key = await crypto.subtle.importKey(
    'jwk',
    existing.sessionKey,
    { name: 'AES-GCM', length: 256 },
    false,
    ['decrypt']
  );

  const iv = base64ToArrayBuffer(metadata.iv);
  const ciphertext = base64ToArrayBuffer(encryptedContent);

  const plaintext = await crypto.subtle.decrypt(
    { name: 'AES-GCM', iv: new Uint8Array(iv) },
    key,
    ciphertext
  );

  return new TextDecoder().decode(plaintext);
}

export function generateOneTimeKeys(count: number): Record<string, string> {
  const keys: Record<string, string> = {};
  for (let i = 0; i < count; i++) {
    const keyBytes = crypto.getRandomValues(new Uint8Array(32));
    const keyId = `otk_${i}_${Date.now()}`;
    keys[keyId] = arrayBufferToBase64(keyBytes);
  }
  return keys;
}

export async function exportKeys(passphrase: string): Promise<string> {
  if (!identity) throw new Error('Crypto not initialized');

  const data = JSON.stringify(identity);
  const encoded = new TextEncoder().encode(data);

  // Derive key from passphrase
  const salt = crypto.getRandomValues(new Uint8Array(16));
  const keyMaterial = await crypto.subtle.importKey(
    'raw',
    new TextEncoder().encode(passphrase),
    'PBKDF2',
    false,
    ['deriveBits', 'deriveKey']
  );
  const key = await crypto.subtle.deriveKey(
    { name: 'PBKDF2', salt, iterations: 100000, hash: 'SHA-256' },
    keyMaterial,
    { name: 'AES-GCM', length: 256 },
    false,
    ['encrypt']
  );

  const iv = crypto.getRandomValues(new Uint8Array(12));
  const ciphertext = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv },
    key,
    encoded
  );

  return JSON.stringify({
    salt: arrayBufferToBase64(salt),
    iv: arrayBufferToBase64(iv),
    data: arrayBufferToBase64(ciphertext),
  });
}

export async function importKeys(backup: string, passphrase: string): Promise<void> {
  const { salt, iv, data } = JSON.parse(backup);

  const keyMaterial = await crypto.subtle.importKey(
    'raw',
    new TextEncoder().encode(passphrase),
    'PBKDF2',
    false,
    ['deriveBits', 'deriveKey']
  );
  const key = await crypto.subtle.deriveKey(
    { name: 'PBKDF2', salt: base64ToArrayBuffer(salt), iterations: 100000, hash: 'SHA-256' },
    keyMaterial,
    { name: 'AES-GCM', length: 256 },
    false,
    ['decrypt']
  );

  const plaintext = await crypto.subtle.decrypt(
    { name: 'AES-GCM', iv: new Uint8Array(base64ToArrayBuffer(iv)) },
    key,
    base64ToArrayBuffer(data)
  );

  const decoded = new TextDecoder().decode(plaintext);
  const restored: StoredIdentity = JSON.parse(decoded);

  identity = restored;
  await cryptoStore.setIdentity(restored);
}

export function getDeviceId(): string | null {
  return identity?.deviceId ?? null;
}

export function isInitialized(): boolean {
  return identity !== null;
}
