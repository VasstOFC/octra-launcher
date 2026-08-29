import {
  FolderOpen,
  Play,
  Plus,
  Square,
  Terminal,
  Trash2,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { clsx } from "clsx";
import { api } from "../lib/api";
import { confirmDialog } from "../lib/dialog";
import { pl } from "../locales/pl";
import { LocalServerCreateDialog, type CreateServerDraft } from "../components/LocalServerCreateDialog";
import { LocalServerSettings } from "../components/LocalServerSettings";
import { useApp } from "../stores/appStore";
import type { LocalServerInfo } from "../types";

export function HostPage() {
  const instances = useApp((s) => s.instances);
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);
  const markServerStatus = useApp((s) => s.markServerStatus);
  const [servers, setServers] = useState<LocalServerInfo[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [log, setLog] = useState("");
  const [cmd, setCmd] = useState("");
  const [busy, setBusy] = useState(false);
  const [createOpen, setCreateOpen] = useState(false);
  const logEndRef = useRef<HTMLDivElement>(null);

  const active = servers.find((s) => s.id === selected) ?? servers[0] ?? null;
  const defaultVersion = instances[0]?.gameVersion ?? "1.21.1";

  async function refresh() {
    try {
      const list = await api.listLocalServers();
      setServers(list);
      if (!selected && list[0]) setSelected(list[0].id);
    } catch (e) {
      showError(String(e));
    }
  }

  useEffect(() => {
    void refresh();
    const unsubs: Array<() => void> = [];
    void listen<{ serverId: string; line: string }>("local-server-log", (e) => {
      if (e.payload.serverId === selected) {
        setLog((l) => {
          const next = l + e.payload.line + "\n";
          return next.length > 200_000 ? next.slice(-200_000) : next;
        });
      }
    }).then((u) => unsubs.push(u));
    void listen<{ serverId: string; status: string; name: string }>(
      "local-server-status",
      (e) => {
        markServerStatus(e.payload.serverId, e.payload.name, e.payload.status);
        void refresh();
      },
    ).then((u) => unsubs.push(u));
    return () => unsubs.forEach((u) => u());
  }, [selected]);

  useEffect(() => {
    if (!active) return;
    api.readLocalServerLog(active.id).then(setLog).catch(() => setLog(""));
  }, [active?.id]);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [log]);

  async function createServer(draft: CreateServerDraft) {
    const inst = instances[0];
    setBusy(true);
    try {
      const s = await api.createLocalServer({
        name: draft.name,
        gameVersion: draft.gameVersion,
        software: draft.software,
        onlineMode: draft.onlineMode,
        eulaAccepted: true,
        sourceInstanceId: inst?.id ?? null,
        motd: draft.motd,
        port: draft.port,
        maxPlayers: draft.maxPlayers,
        difficulty: draft.difficulty,
        viewDistance: draft.viewDistance,
        memoryMb: draft.memoryMb,
      });
      setSelected(s.id);
      setCreateOpen(false);
      await refresh();
      showOk(pl.host.created);
    } catch (e) {
      showError(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function run(action: "install" | "start" | "stop" | "delete") {
    if (!active) return;
    setBusy(true);
    try {
      if (action === "install") await api.installLocalServer(active.id);
      else if (action === "start") await api.startLocalServer(active.id);
      else if (action === "stop") await api.stopLocalServer(active.id);
      else if (action === "delete") {
        const ok = await confirmDialog(
          pl.host.deleteConfirm.replace("{name}", active.name),
          { title: pl.host.delete, confirmLabel: pl.host.delete },
        );
        if (!ok) return;
        await api.deleteLocalServer(active.id);
        setSelected(null);
      }
      await refresh();
    } catch (e) {
      showError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <LocalServerCreateDialog
        open={createOpen}
        defaultVersion={defaultVersion}
        busy={busy}
        onClose={() => setCreateOpen(false)}
        onSubmit={(draft) => void createServer(draft)}
      />

      <header className="flex items-center justify-between border-b border-line px-6 py-4">
        <div>
          <h1 className="text-xl font-extrabold">{pl.host.title}</h1>
          <p className="mt-0.5 text-xs text-mute">{pl.host.subtitle}</p>
        </div>
        <button
          disabled={busy}
          onClick={() => setCreateOpen(true)}
          className="inline-flex items-center gap-2 rounded-full bg-accent px-4 py-2 text-sm font-semibold text-bg-on-accent"
        >
          <Plus size={16} />
          {pl.host.newServer}
        </button>
      </header>

      <div className="flex min-h-0 flex-1">
        <aside className="w-56 shrink-0 overflow-auto border-r border-line p-3">
          {servers.length === 0 ? (
            <p className="px-2 py-6 text-xs text-mute">{pl.host.empty}</p>
          ) : (
            servers.map((s) => (
              <button
                key={s.id}
                onClick={() => setSelected(s.id)}
                className={clsx(
                  "mb-1 w-full rounded-xl px-3 py-2 text-left text-xs",
                  selected === s.id ? "bg-accent/20" : "hover:bg-white/5",
                )}
              >
                <div className="font-semibold">{s.name}</div>
                <div className="text-[10px] text-mute">
                  {s.software} · {s.gameVersion} ·{" "}
                  <span className={s.status === "running" ? "text-good" : "text-mute"}>
                    {s.status === "running" ? pl.host.running : s.status}
                  </span>
                </div>
              </button>
            ))
          )}
        </aside>

        {active ? (
          <section className="flex min-w-0 flex-1">
            <div className="flex min-w-0 flex-1 flex-col">
              <div className="flex flex-wrap items-center gap-2 border-b border-line px-4 py-3">
                <button
                  disabled={busy || active.jarReady}
                  onClick={() => void run("install")}
                  className="rounded-full border border-line px-3 py-1 text-xs hover:bg-white/5 disabled:opacity-40"
                >
                  {active.jarReady ? pl.host.installed : pl.host.install}
                </button>
                {active.status !== "running" ? (
                  <button
                    disabled={busy || !active.jarReady}
                    onClick={() => void run("start")}
                    className="inline-flex items-center gap-1 rounded-full bg-good px-3 py-1 text-xs font-semibold text-black disabled:opacity-40"
                  >
                    <Play size={12} />
                    {pl.host.start}
                  </button>
                ) : (
                  <button
                    disabled={busy}
                    onClick={() => void run("stop")}
                    className="inline-flex items-center gap-1 rounded-full bg-danger px-3 py-1 text-xs font-semibold text-white"
                  >
                    <Square size={12} />
                    {pl.host.stop}
                  </button>
                )}
                <button
                  onClick={() => api.openLocalServerFolder(active.id).catch((e) => showError(String(e)))}
                  className="inline-flex items-center gap-1 rounded-full border border-line px-3 py-1 text-xs"
                >
                  <FolderOpen size={12} />
                  {pl.host.folder}
                </button>
                <button
                  disabled={busy}
                  onClick={() => void run("delete")}
                  className="ml-auto inline-flex items-center gap-1 rounded-full border border-danger/40 px-3 py-1 text-xs text-danger"
                >
                  <Trash2 size={12} />
                  {pl.host.delete}
                </button>
                <span className="w-full text-[10px] text-mute">
                  {pl.host.port} {active.port} · {active.address}
                  {active.lanIp ? ` · LAN ${active.lanIp}` : ""}
                </span>
              </div>

              <pre className="min-h-0 flex-1 overflow-auto whitespace-pre-wrap break-words bg-black/40 p-3 font-mono text-[11px] leading-relaxed text-mute">
                {log || pl.host.logEmpty}
                <div ref={logEndRef} />
              </pre>

              <form
                className="flex items-center gap-2 border-t border-line p-3"
                onSubmit={(e) => {
                  e.preventDefault();
                  if (!cmd.trim()) return;
                  api.sendLocalServerCommand(active.id, cmd).catch((e) => showError(String(e)));
                  setCmd("");
                }}
              >
                <Terminal size={14} className="shrink-0 text-mute" />
                <input
                  value={cmd}
                  onChange={(e) => setCmd(e.target.value)}
                  placeholder={pl.host.commandPlaceholder}
                  className="flex-1 rounded-xl bg-raised px-3 py-2 text-sm outline-none ring-1 ring-line"
                  disabled={active.status !== "running"}
                />
              </form>
            </div>

            <aside className="w-72 shrink-0 border-l border-line bg-raised/30">
              <LocalServerSettings
                server={active}
                disabled={busy}
                onSaved={() => void refresh()}
              />
            </aside>
          </section>
        ) : (
          <div className="grid flex-1 place-items-center text-sm text-mute">{pl.host.pickServer}</div>
        )}
      </div>
    </div>
  );
}
