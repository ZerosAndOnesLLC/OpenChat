/**
 * Message Drafts Manager
 * Stores drafts in IndexedDB per channel/DM
 */

const DB_NAME = 'openchat_drafts';
const STORE_NAME = 'drafts';
const DB_VERSION = 1;

export interface Draft {
  key: string; // channelId or dmId
  content: string;
  updatedAt: number;
}

class DraftsManager {
  private db: IDBDatabase | null = null;
  private initPromise: Promise<void> | null = null;

  /**
   * Initialize IndexedDB
   */
  async init(): Promise<void> {
    if (this.db) return;
    if (this.initPromise) return this.initPromise;

    this.initPromise = new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onerror = () => {
        reject(new Error('Failed to open IndexedDB'));
      };

      request.onsuccess = () => {
        this.db = request.result;
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;

        // Create object store if it doesn't exist
        if (!db.objectStoreNames.contains(STORE_NAME)) {
          db.createObjectStore(STORE_NAME, { keyPath: 'key' });
        }
      };
    });

    return this.initPromise;
  }

  /**
   * Save draft to IndexedDB
   */
  async saveDraft(key: string, content: string): Promise<void> {
    await this.init();

    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readwrite');
      const store = transaction.objectStore(STORE_NAME);

      const draft: Draft = {
        key,
        content,
        updatedAt: Date.now(),
      };

      const request = store.put(draft);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(new Error('Failed to save draft'));
    });
  }

  /**
   * Get draft from IndexedDB
   */
  async getDraft(key: string): Promise<string | null> {
    await this.init();

    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readonly');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.get(key);

      request.onsuccess = () => {
        const draft = request.result as Draft | undefined;
        resolve(draft?.content || null);
      };

      request.onerror = () => reject(new Error('Failed to get draft'));
    });
  }

  /**
   * Delete draft from IndexedDB
   */
  async deleteDraft(key: string): Promise<void> {
    await this.init();

    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readwrite');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.delete(key);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(new Error('Failed to delete draft'));
    });
  }

  /**
   * Get all drafts
   */
  async getAllDrafts(): Promise<Draft[]> {
    await this.init();

    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readonly');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.getAll();

      request.onsuccess = () => resolve(request.result as Draft[]);
      request.onerror = () => reject(new Error('Failed to get drafts'));
    });
  }

  /**
   * Clear all drafts
   */
  async clearAllDrafts(): Promise<void> {
    await this.init();

    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readwrite');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.clear();

      request.onsuccess = () => resolve();
      request.onerror = () => reject(new Error('Failed to clear drafts'));
    });
  }
}

// Export singleton instance
export const draftsManager = new DraftsManager();
