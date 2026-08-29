import { Copy, Loader2, Plus, RefreshCw, Trash2 } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { api } from "../lib/api";
import { confirmDialog, promptDialog } from "../lib/dialog";
import { pl } from "../locales/pl";
import { useApp } from "../stores/appStore";
import { useOctra } from "../stores/octraStore";
import { SectionHeader } from "../components/ui/SectionHeader";
import type { LocalServerInfo, ServerEntry, ServerPingResult } from "../types";

type ServerRow = ServerEntry & {
  ping?: ServerPingResult | null;
  local?: boolean;
};

export function ServersPage() {
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);
  const selectedId = useOctra((s) => s.selectedId);
  const [servers, setServers] = useState<ServerRow[]>([]);
  const [localServers, setLocalServers] = useState<LocalServerInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [pinging, setPinging] = useState(false);

  const loadList = useCallback(async () => {
    setLoading(true);
    try {
      const [list, local] = await Promise.all([
        api.listServers(),
        api.listLocalServers().catch(() => [] as LocalServerInfo[]),
      ]);
      const localAddrs = new Set<string>();
      for (const s of local) {
        localAddrs.add(s.address.trim().toLowerCase());
        if (s.lanIp) localAddrs.add(`${s.lanIp}:${s.port}`.toLowerCase());
        localAddrs.add(`127.0.0.1:${s.port}`.toLowerCase());
        localAddrs.add(`localhost:${s.port}`.toLowerCase());
      }
      setLocalServers(local);
      setServers(
        list.map((s) => ({
          ...s,
          local: localAddrs.has(s.address.trim().toLowerCase()),
        })),
      );
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [showError]);

  useEffect(() => {
    void loadList();
  }, [loadList]);

  async function refreshPings(rows: ServerRow[]) {
    setPinging(true);
    try {
      const updated = await Promise.all(
        rows.map(async (s) => {
          try {
            const ping = await api.pingServer(s.address);
            return { ...s, ping };
          } catch {
            return { ...s, ping: { online: false } as ServerPingResult };
          }
        }),
      );
      setServers(updated);
    } finally {
      setPinging(false);
    }
  }

  async function saveAll(next: ServerEntry[]) {
    const saved = await api.saveServers(next);
    const localAddrs = new Set<string>();
    for (const s of localServers) {
      localAddrs.add(s.address.trim().toLowerCase());
      if (s.lanIp) localAddrs.add(`${s.lanIp}:${s.port}`.toLowerCase());
      localAddrs.add(`127.0.0.1:${s.port}`.toLowerCase());
      localAddrs.add(`localhost:${s.port}`.toLowerCase());
    }
    setServers(
      saved.map((s) => ({
        ...s,
        local: localAddrs.has(s.address.trim().toLowerCase()),
      })),
    );
  }

  async function addServer() {
    const name = await promptDialog(pl.servers.namePrompt, { title: pl.servers.add });
    if (!name?.trim()) return;
    const address = await promptDialog(pl.servers.addressPrompt, { title: pl.servers.add });
    if (!address?.trim()) return;
    await saveAll([...servers, { name: name.trim(), address: address.trim() }]);
  }

  async function editServer(index: number) {
    const current = servers[index];
    if (!current) return;
    const name = await promptDialog(pl.servers.namePrompt, {
      title: pl.servers.edit,
      defaultValue: current.name,
    });
    if (!name?.trim()) return;
    const address = await promptDialog(pl.servers.addressPrompt, {
      title: pl.servers.edit,
      defaultValue: current.address,
    });
    if (!address?.trim()) return;
    const next = servers.map((s, i) =>
      i === index ? { name: name.trim(), address: address.trim() } : s,
    );
    await saveAll(next);
  }

  async function deleteServer(index: number) {
    const target = servers[index];
    if (!target) return;
    const ok = await confirmDialog(
      pl.servers.deleteConfirm.replace("{name}", target.name),
      { title: pl.servers.delete, danger: true },
    );
    if (!ok) return;
    await saveAll(servers.filter((_, i) => i !== index));
  }

  async function copyAddress(address: string) {
    try {
      await navigator.clipboard.writeText(address);
      showOk(pl.servers.copied);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    }
  }

  async function syncToProfile() {
    if (!selectedId) return;
    try {
      const n = await api.syncServersToInstance(selectedId);
      showOk(pl.servers.syncDone.replace("{n}", String(n)));
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="border-b border-line px-5 py-4">
        <SectionHeader
          title={pl.servers.title}
          action={
            <div className="flex flex-wrap gap-2">
              <button
                type="button"
                onClick={() => void refreshPings(servers)}
                disabled={pinging || servers.length === 0}
                className="inline-flex items-center gap-1 rounded-lg border border-line px-3 py-1.5 text-xs text-mute hover:text-ink disabled:opacity-50"
              >
                {pinging ? <Loader2 size={14} className="animate-spin" /> : <RefreshCw size={14} />}
                {pl.servers.refresh}
              </button>
              {selectedId ? (
                <button
                  type="button"
                  onClick={() => void syncToProfile()}
                  className="inline-flex items-center gap-1 rounded-lg border border-line px-3 py-1.5 text-xs text-mute hover:text-ink"
                >
                  {pl.servers.sync}
                </button>
              ) : null}
              <button
                type="button"
                onClick={() => void addServer()}
                className="inline-flex items-center gap-1 rounded-lg bg-accent px-3 py-1.5 text-xs font-semibold text-bg-on-accent"
              >
                <Plus size={14} />
                {pl.servers.add}
              </button>
            </div>
          }
        />
        <p className="mt-1 text-[13px] text-mute">{pl.servers.subtitle}</p>
      </div>

      <div className="min-h-0 flex-1 overflow-auto p-5">
        {loading ? (
          <div className="flex items-center justify-center gap-2 py-16 text-sm text-mute">
            <Loader2 size={18} className="animate-spin" />
            Ładowanie…
          </div>
        ) : servers.length === 0 ? (
          <p className="mt-16 text-center text-sm text-mute">{pl.servers.empty}</p>
        ) : (
          <ul className="mx-auto max-w-3xl space-y-2">
            {servers.map((s, i) => (
              <li
                key={`${s.address}-${i}`}
                className="flex flex-wrap items-center gap-3 rounded-2xl border border-line bg-raised2 px-4 py-3"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <p className="font-semibold text-ink">{s.name}</p>
                    {s.local ? (
                      <span className="rounded-full bg-good/15 px-2 py-0.5 text-[10px] font-bold text-good">
                        {pl.servers.local}
                      </span>
                    ) : null}
                    <span
                      className={`rounded-full px-2 py-0.5 text-[10px] font-bold ${
                        s.ping?.online
                          ? "bg-good/15 text-good"
                          : s.ping
                            ? "bg-danger/15 text-danger"
                            : "bg-white/8 text-mute"
                      }`}
                    >
                      {s.ping?.online
                        ? pl.servers.online
                        : s.ping
                          ? pl.servers.offline
                          : "—"}
                    </span>
                  </div>
                  <p className="mt-0.5 font-mono text-xs text-mute">{s.address}</p>
                  <div className="mt-1 flex flex-wrap gap-3 text-[11px] text-mute">
                    {s.ping?.latencyMs != null ? (
                      <span>{pl.servers.pingMs.replace("{ms}", String(s.ping.latencyMs))}</span>
                    ) : null}
                    {s.ping?.players != null && s.ping.maxPlayers != null ? (
                      <span>
                        {pl.servers.players
                          .replace("{online}", String(s.ping.players))
                          .replace("{max}", String(s.ping.maxPlayers))}
                      </span>
                    ) : null}
                    {s.ping?.version ? <span>{s.ping.version}</span> : null}
                  </div>
                </div>
                <div className="flex gap-1">
                  <button
                    type="button"
                    title={pl.servers.connect}
                    onClick={() => void copyAddress(s.address)}
                    className="grid h-9 w-9 place-items-center rounded-lg text-mute hover:bg-white/6 hover:text-ink"
                  >
                    <Copy size={16} />
                  </button>
                  <button
                    type="button"
                    title={pl.servers.edit}
                    onClick={() => void editServer(i)}
                    className="rounded-lg px-3 py-1.5 text-xs text-mute hover:bg-white/6 hover:text-ink"
                  >
                    {pl.servers.edit}
                  </button>
                  <button
                    type="button"
                    title={pl.servers.delete}
                    onClick={() => void deleteServer(i)}
                    className="grid h-9 w-9 place-items-center rounded-lg text-mute hover:bg-danger/12 hover:text-danger"
                  >
                    <Trash2 size={16} />
                  </button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
