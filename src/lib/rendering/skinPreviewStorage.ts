/**
 * IndexedDB cache dla miniaturek skinów — szkielet pod batch renderer Modrinth.
 * Źródło: apps/app-frontend/src/helpers/storage/skin-preview-storage.ts
 */

const DB_NAME = "octra-skin-previews";
const STORE = "previews";
const VERSION = 1;

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, VERSION);
    req.onerror = () => reject(req.error);
    req.onsuccess = () => resolve(req.result);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE)) {
        db.createObjectStore(STORE);
      }
    };
  });
}

export function previewCacheKey(skin: {
  textureKey: string;
  variant: string;
  capeId?: string;
}): string {
  return `v1:${skin.textureKey}:${skin.variant}:${skin.capeId ?? "no-cape"}`;
}

export async function getCachedPreview(key: string): Promise<Blob | null> {
  try {
    const db = await openDb();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORE, "readonly");
      const req = tx.objectStore(STORE).get(key);
      req.onsuccess = () => resolve((req.result as Blob) ?? null);
      req.onerror = () => reject(req.error);
    });
  } catch {
    return null;
  }
}

export async function setCachedPreview(key: string, blob: Blob): Promise<void> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, "readwrite");
    tx.objectStore(STORE).put(blob, key);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

/**
 * Placeholder — pełny batch renderer (Three.js) zostanie przeniesiony z Modrinth.
 */
export async function generateSkinPreviews(
  _skins: import("../skins").Skin[],
  _capes: import("../skins").Cape[],
): Promise<void> {
  // TODO: port batch-skin-renderer.ts
}
