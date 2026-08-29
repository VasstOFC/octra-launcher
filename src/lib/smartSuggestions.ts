import type { FeaturedPackInfo, Instance, PackUpdateInfo, ServerEntry, ServerPingResult } from "../types";

export type SmartSuggestionKind =
  | "pack-update"
  | "last-played"
  | "featured"
  | "server-online";

export type SmartSuggestion = {
  id: string;
  kind: SmartSuggestionKind;
  title: string;
  hint: string;
  instanceId?: string;
  serverAddress?: string;
  featuredSlug?: string;
};

const PING_THRESHOLD_MS = 250;

export function buildSmartSuggestions(opts: {
  instances: Instance[];
  selectedId: string | null;
  packUpdates: Map<string, PackUpdateInfo>;
  featured: FeaturedPackInfo | null;
  servers: ServerEntry[];
  serverPings: Map<string, ServerPingResult>;
  max?: number;
}): SmartSuggestion[] {
  const out: SmartSuggestion[] = [];
  const max = opts.max ?? 3;

  for (const inst of opts.instances) {
    if (!inst.packLocked || !inst.linkedPack) continue;
    const update = opts.packUpdates.get(inst.id);
    if (update?.hasUpdate) {
      out.push({
        id: `pack-${inst.id}`,
        kind: "pack-update",
        title: inst.name,
        hint: update.latestVersion,
        instanceId: inst.id,
      });
      break;
    }
  }

  const sorted = opts.instances
    .slice()
    .sort((a, b) => (b.lastPlayed ?? "").localeCompare(a.lastPlayed ?? ""));
  const last = sorted[0];
  if (last && last.id !== opts.selectedId) {
    out.push({
      id: `last-${last.id}`,
      kind: "last-played",
      title: last.name,
      hint: last.gameVersion,
      instanceId: last.id,
    });
  }

  if (
    opts.featured?.enabled &&
    opts.featured.slug &&
    !opts.instances.some((i) => i.linkedPack === opts.featured!.slug)
  ) {
    out.push({
      id: "featured",
      kind: "featured",
      title: opts.featured.title,
      hint: opts.featured.blurb || opts.featured.slug,
      featuredSlug: opts.featured.slug,
    });
  }

  for (const server of opts.servers) {
    const ping = opts.serverPings.get(server.address);
    if (ping?.online && ping.latencyMs != null && ping.latencyMs <= PING_THRESHOLD_MS) {
      out.push({
        id: `server-${server.address}`,
        kind: "server-online",
        title: server.name,
        hint: `${ping.latencyMs} ms`,
        serverAddress: server.address,
      });
      break;
    }
  }

  return out.slice(0, max);
}
