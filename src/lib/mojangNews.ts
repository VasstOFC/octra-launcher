import { useEffect, useState } from "react";
import { api } from "./api";
import type { MojangNewsItem } from "../types";

const CACHE_KEY = "octra-mojang-news";
const CACHE_TTL_MS = 1000 * 60 * 30;

type CachePayload = { at: number; items: MojangNewsItem[] };

function readCache(): MojangNewsItem[] | null {
  try {
    const raw = localStorage.getItem(CACHE_KEY);
    if (!raw) return null;
    const data = JSON.parse(raw) as CachePayload;
    if (Date.now() - data.at > CACHE_TTL_MS) return null;
    return data.items;
  } catch {
    return null;
  }
}

function writeCache(items: MojangNewsItem[]) {
  localStorage.setItem(CACHE_KEY, JSON.stringify({ at: Date.now(), items }));
}

export function useMojangNews() {
  const [items, setItems] = useState<MojangNewsItem[]>(() => readCache() ?? []);
  const [loading, setLoading] = useState(!readCache());
  const [offline, setOffline] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      const cached = readCache();
      if (cached?.length) {
        setItems(cached);
        setLoading(false);
      }
      try {
        const parsed = await api.fetchMojangNews();
        if (!cancelled && parsed.length) {
          setItems(parsed);
          writeCache(parsed);
          setOffline(false);
        }
      } catch {
        if (!cancelled) {
          setOffline(!readCache());
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    void load();
    return () => {
      cancelled = true;
    };
  }, []);

  return { items, loading, offline };
}
