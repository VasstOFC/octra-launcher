import { ArrowRight, Download, Play, Server, Sparkles } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { api } from "../lib/api";
import { buildSmartSuggestions, type SmartSuggestion } from "../lib/smartSuggestions";
import { notifyPackUpdate } from "../stores/notifications";
import { pl } from "../locales/pl";
import { useApp } from "../stores/appStore";
import { useOctra } from "../stores/octraStore";
import { usePackUpdate } from "../stores/packUpdateStore";
import type { FeaturedPackInfo, PackUpdateInfo, ServerPingResult } from "../types";

const KIND_ICON = {
  "pack-update": Download,
  "last-played": Play,
  featured: Sparkles,
  "server-online": Server,
} as const;

export function SmartSuggestionsStrip() {
  const instances = useApp((s) => s.instances);
  const selectedId = useOctra((s) => s.selectedId);
  const select = useOctra((s) => s.selectInstance);
  const setView = useOctra((s) => s.setView);
  const playInstance = useApp((s) => s.playInstance);
  const showOk = useApp((s) => s.showOk);
  const showError = useApp((s) => s.showError);
  const openPackUpdate = usePackUpdate((s) => s.openFor);
  const [featured, setFeatured] = useState<FeaturedPackInfo | null>(null);
  const [packUpdates, setPackUpdates] = useState<Map<string, PackUpdateInfo>>(new Map());
  const [serverPings, setServerPings] = useState<Map<string, ServerPingResult>>(new Map());
  const [servers, setServers] = useState<{ name: string; address: string }[]>([]);

  useEffect(() => {
    void api.getFeaturedPack().then(setFeatured).catch(() => setFeatured(null));
    void api.listServers().then(setServers).catch(() => setServers([]));
  }, []);

  useEffect(() => {
    let cancelled = false;
    const locked = instances.filter((i) => i.packLocked && i.linkedPack);
    if (!locked.length) {
      setPackUpdates(new Map());
      return;
    }
    void Promise.all(
      locked.map(async (inst) => {
        try {
          const info = await api.checkPackUpdate(inst.id);
          return [inst.id, info] as const;
        } catch {
          return null;
        }
      }),
    ).then((rows) => {
      if (cancelled) return;
      const map = new Map<string, PackUpdateInfo>();
      for (const row of rows) {
        if (row) map.set(row[0], row[1]);
      }
      setPackUpdates(map);
      for (const row of rows) {
        if (!row || !row[1].hasUpdate) continue;
        const inst = locked.find((i) => i.id === row[0]);
        if (!inst) continue;
        const key = `octra-pack-notify-${inst.id}-${row[1].latestVersionId}`;
        if (sessionStorage.getItem(key)) continue;
        sessionStorage.setItem(key, "1");
        notifyPackUpdate(inst.id, inst.name);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [instances]);

  useEffect(() => {
    if (!servers.length) return;
    let cancelled = false;
    void Promise.all(
      servers.slice(0, 5).map(async (s) => {
        try {
          const ping = await api.pingServer(s.address);
          return [s.address, ping] as const;
        } catch {
          return null;
        }
      }),
    ).then((rows) => {
      if (cancelled) return;
      const map = new Map<string, ServerPingResult>();
      for (const row of rows) {
        if (row) map.set(row[0], row[1]);
      }
      setServerPings(map);
    });
    return () => {
      cancelled = true;
    };
  }, [servers]);

  const suggestions = useMemo(
    () =>
      buildSmartSuggestions({
        instances,
        selectedId,
        packUpdates,
        featured,
        servers,
        serverPings,
      }),
    [instances, selectedId, packUpdates, featured, servers, serverPings],
  );

  if (!suggestions.length) return null;

  async function onAction(s: SmartSuggestion) {
    switch (s.kind) {
      case "pack-update":
        if (s.instanceId) openPackUpdate(s.instanceId);
        break;
      case "last-played":
        if (s.instanceId) {
          select(s.instanceId);
          await playInstance(s.instanceId);
        }
        break;
      case "featured":
        setView("store");
        break;
      case "server-online":
        if (s.serverAddress) {
          try {
            await navigator.clipboard.writeText(s.serverAddress);
            showOk(pl.servers.copied);
          } catch (e) {
            showError(e instanceof Error ? e.message : String(e));
          }
        }
        break;
    }
  }

  function label(s: SmartSuggestion): string {
    switch (s.kind) {
      case "pack-update":
        return pl.smartStart.packUpdate;
      case "last-played":
        return pl.smartStart.lastPlayed;
      case "featured":
        return pl.smartStart.featured;
      case "server-online":
        return pl.smartStart.serverOnline;
    }
  }

  function hint(s: SmartSuggestion): string {
    switch (s.kind) {
      case "pack-update":
        return pl.smartStart.packUpdateHint.replace("{name}", s.title);
      case "last-played":
        return pl.smartStart.lastPlayedHint.replace("{name}", s.title);
      case "featured":
        return pl.smartStart.featuredHint.replace("{title}", s.title);
      case "server-online":
        return pl.smartStart.serverHint
          .replace("{name}", s.title)
          .replace("{ms}", s.hint);
    }
  }

  return (
    <section className="mt-4">
      <h2 className="text-xs font-bold uppercase tracking-wider text-mute">
        {pl.smartStart.title}
      </h2>
      <div className="mt-2 grid grid-cols-1 gap-2 sm:grid-cols-3">
        {suggestions.map((s) => {
          const Icon = KIND_ICON[s.kind];
          return (
            <button
              key={s.id}
              type="button"
              onClick={() => void onAction(s)}
              className="flex items-center gap-3 rounded-xl border border-line bg-raised2/60 px-4 py-3 text-left transition hover:border-accent/35 hover:bg-raised2"
            >
              <div className="grid h-9 w-9 shrink-0 place-items-center rounded-lg bg-accent/15 text-accent">
                <Icon size={16} />
              </div>
              <div className="min-w-0 flex-1">
                <p className="text-[10px] font-bold uppercase text-accent">{label(s)}</p>
                <p className="truncate text-sm font-semibold text-ink">{s.title}</p>
                <p className="truncate text-[11px] text-mute">{hint(s)}</p>
              </div>
              <ArrowRight size={14} className="shrink-0 text-mute" />
            </button>
          );
        })}
      </div>
    </section>
  );
}
