const DB_NAME = "bili-music-cache";
const DB_VERSION = 1;
const STORE_NAME = "audio";
const MAX_CACHE_SIZE = 500 * 1024 * 1024;

interface CacheEntry {
  bvid: string;
  data: ArrayBuffer;
  timestamp: number;
  size: number;
}

class AudioCacheManager {
  private db: IDBDatabase | null = null;
  private _currentSize = 0;

  get currentSize() {
    return this._currentSize;
  }

  async init(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);
      request.onupgradeneeded = () => {
        const db = request.result;
        if (!db.objectStoreNames.contains(STORE_NAME)) {
          db.createObjectStore(STORE_NAME, { keyPath: "bvid" });
        }
      };
      request.onsuccess = () => {
        this.db = request.result;
        this.updateSize();
        resolve();
      };
      request.onerror = () => reject(request.error);
    });
  }

  private async updateSize() {
    if (!this.db) return;
    const entries = await this.getAll();
    this._currentSize = entries.reduce((sum, e) => sum + e.size, 0);
  }

  async get(bvid: string): Promise<ArrayBuffer | null> {
    if (!this.db) return null;
    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(STORE_NAME, "readonly");
      const store = tx.objectStore(STORE_NAME);
      const request = store.get(bvid);
      request.onsuccess = () => {
        const entry = request.result as CacheEntry | undefined;
        if (entry) {
          this.touch(bvid);
          resolve(entry.data);
        } else {
          resolve(null);
        }
      };
      request.onerror = () => reject(request.error);
    });
  }

  async set(bvid: string, data: ArrayBuffer): Promise<void> {
    if (!this.db) return;
    const size = data.byteLength;
    await this.evictIfNeeded(size);
    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(STORE_NAME, "readwrite");
      const store = tx.objectStore(STORE_NAME);
      store.put({ bvid, data, timestamp: Date.now(), size });
      tx.oncomplete = () => {
        this._currentSize += size;
        resolve();
      };
      tx.onerror = () => reject(tx.error);
    });
  }

  async clear(): Promise<void> {
    if (!this.db) return;
    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(STORE_NAME, "readwrite");
      tx.objectStore(STORE_NAME).clear();
      tx.oncomplete = () => {
        this._currentSize = 0;
        resolve();
      };
      tx.onerror = () => reject(tx.error);
    });
  }

  private touch(bvid: string) {
    if (!this.db) return;
    const tx = this.db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);
    const getReq = store.get(bvid);
    getReq.onsuccess = () => {
      const entry = getReq.result as CacheEntry | undefined;
      if (entry) {
        entry.timestamp = Date.now();
        store.put(entry);
      }
    };
  }

  private async getAll(): Promise<CacheEntry[]> {
    if (!this.db) return [];
    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(STORE_NAME, "readonly");
      const store = tx.objectStore(STORE_NAME);
      const request = store.getAll();
      request.onsuccess = () => resolve(request.result || []);
      request.onerror = () => reject(request.error);
    });
  }

  private async evictIfNeeded(needed: number) {
    while (this._currentSize + needed > MAX_CACHE_SIZE) {
      const entries = await this.getAll();
      if (entries.length === 0) break;
      entries.sort((a, b) => a.timestamp - b.timestamp);
      await this.deleteEntry(entries[0].bvid);
    }
  }

  private async deleteEntry(bvid: string) {
    if (!this.db) return;
    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(STORE_NAME, "readwrite");
      const store = tx.objectStore(STORE_NAME);
      const getReq = store.get(bvid);
      getReq.onsuccess = () => {
        const entry = getReq.result as CacheEntry | undefined;
        if (entry) this._currentSize -= entry.size;
      };
      store.delete(bvid);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
  }
}

export const audioCache = new AudioCacheManager();

export function useAudioCache() {
  return { audioCache };
}
