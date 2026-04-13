const DB_NAME = "bili-music-cache";
const DB_VERSION = 1;
const STORE_NAME = "audio";
const MAX_CACHE_SIZE = 500 * 1024 * 1024;
const CACHE_TTL = 7 * 24 * 60 * 60 * 1000; // 7 天

interface CacheEntry {
  bvid: string;
  data: ArrayBuffer;
  timestamp: number;
  size: number;
}

class AudioCacheManager {
  private db: IDBDatabase | null = null;
  private _currentSize = 0;
  private cleanupHandle: ReturnType<typeof setInterval> | null = null;

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
        this.startCleanupTimer();
        resolve();
      };
      request.onerror = () => reject(request.error);
    });
  }

  destroy() {
    this.stopCleanupTimer();
  }

  private startCleanupTimer() {
    this.stopCleanupTimer();
    this.cleanupHandle = setInterval(() => this.cleanupExpired(), 30 * 60 * 1000);
  }

  private stopCleanupTimer() {
    if (this.cleanupHandle) {
      clearInterval(this.cleanupHandle);
      this.cleanupHandle = null;
    }
  }

  private async cleanupExpired() {
    const entries = await this.getAll();
    const cutoff = Date.now() - CACHE_TTL;
    const expired = entries.filter((e) => e.timestamp < cutoff);
    for (const e of expired) {
      await this.deleteEntry(e.bvid);
    }
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
    const entries = await this.getAll();
    entries.sort((a, b) => a.timestamp - b.timestamp);
    const cutoff = Date.now() - CACHE_TTL;
    // 先清理所有已过期条目
    for (const e of entries) {
      if (e.timestamp >= cutoff) break;
      await this.deleteEntry(e.bvid);
    }
    // 再按 LRU 淘汰直到容量足够
    while (this._currentSize + needed > MAX_CACHE_SIZE) {
      const remaining = await this.getAll();
      if (remaining.length === 0) break;
      remaining.sort((a, b) => a.timestamp - b.timestamp);
      await this.deleteEntry(remaining[0].bvid);
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
      tx.oncomplete = () => resolve(undefined);
      tx.onerror = () => reject(tx.error);
    });
  }

  async getEntryCount(): Promise<number> {
    if (!this.db) return 0;
    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(STORE_NAME, "readonly");
      const store = tx.objectStore(STORE_NAME);
      const request = store.count();
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }

  async refreshSize(): Promise<void> {
    await this.updateSize();
  }
}

export const audioCache = new AudioCacheManager();
export { MAX_CACHE_SIZE };

export function useAudioCache() {
  return { audioCache };
}
