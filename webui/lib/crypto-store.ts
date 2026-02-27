const DB_NAME = 'openchat-crypto';
const DB_VERSION = 1;

const STORES = {
  identity: 'identity',
  sessions: 'sessions',
  key_backup: 'key_backup',
} as const;

function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(STORES.identity)) {
        db.createObjectStore(STORES.identity, { keyPath: 'key' });
      }
      if (!db.objectStoreNames.contains(STORES.sessions)) {
        db.createObjectStore(STORES.sessions, { keyPath: 'id' });
      }
      if (!db.objectStoreNames.contains(STORES.key_backup)) {
        db.createObjectStore(STORES.key_backup, { keyPath: 'id' });
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

async function get<T>(store: string, key: string): Promise<T | undefined> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(store, 'readonly');
    const req = tx.objectStore(store).get(key);
    req.onsuccess = () => resolve(req.result?.value as T | undefined);
    req.onerror = () => reject(req.error);
  });
}

async function set(store: string, key: string, value: unknown): Promise<void> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(store, 'readwrite');
    tx.objectStore(store).put({ key, value });
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

async function remove(store: string, key: string): Promise<void> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(store, 'readwrite');
    tx.objectStore(store).delete(key);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

export interface StoredIdentity {
  userId: string;
  deviceId: string;
  identityKeyPair: { publicKey: JsonWebKey; privateKey: JsonWebKey };
  signingKeyPair: { publicKey: JsonWebKey; privateKey: JsonWebKey };
}

export interface StoredSession {
  id: string; // channelId or dmId
  sessionKey: JsonWebKey;
  createdAt: number;
  messageCount: number;
}

export const cryptoStore = {
  async getIdentity(): Promise<StoredIdentity | undefined> {
    return get<StoredIdentity>(STORES.identity, 'current');
  },

  async setIdentity(identity: StoredIdentity): Promise<void> {
    await set(STORES.identity, 'current', identity);
  },

  async clearIdentity(): Promise<void> {
    await remove(STORES.identity, 'current');
  },

  async getSession(id: string): Promise<StoredSession | undefined> {
    return get<StoredSession>(STORES.sessions, id);
  },

  async setSession(session: StoredSession): Promise<void> {
    const db = await openDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORES.sessions, 'readwrite');
      tx.objectStore(STORES.sessions).put(session);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
  },

  async clearSessions(): Promise<void> {
    const db = await openDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORES.sessions, 'readwrite');
      tx.objectStore(STORES.sessions).clear();
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
  },

  async setKeyBackup(id: string, data: string): Promise<void> {
    const db = await openDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORES.key_backup, 'readwrite');
      tx.objectStore(STORES.key_backup).put({ id, data });
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
  },

  async getKeyBackup(id: string): Promise<string | undefined> {
    const db = await openDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORES.key_backup, 'readonly');
      const req = tx.objectStore(STORES.key_backup).get(id);
      req.onsuccess = () => resolve(req.result?.data);
      req.onerror = () => reject(req.error);
    });
  },

  async clearAll(): Promise<void> {
    const db = await openDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction([STORES.identity, STORES.sessions, STORES.key_backup], 'readwrite');
      tx.objectStore(STORES.identity).clear();
      tx.objectStore(STORES.sessions).clear();
      tx.objectStore(STORES.key_backup).clear();
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
  },
};
