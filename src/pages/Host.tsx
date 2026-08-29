import {
  FolderOpen,
  Play,
  Plus,
  Square,
  Terminal,
  Trash2,
} from "lucide-react";
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { clsx } from "clsx";
import { api } from "../lib/api";
import { promptDialog } from "../lib/dialog";
import { useApp } from "../stores/appStore";
import type { LocalServerInfo, LocalSoftware } from "../types";

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

  const active = servers.find((s) => s.id === selected) ?? servers[0] ?? null;

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
        setLog((l) => l + e.payload.line + "\n");
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

  async function createServer() {
    const name = await promptDialog("Nazwa serwera:", {
      title: "Nowy serwer lokalny",
      defaultValue: "Mój serwer",
      confirmLabel: "Utwórz",
    });
    if (!name) return;
    const inst = instances[0];
    setBusy(true);
    try {
      const s = await api.createLocalServer({
        name,
        gameVersion: inst?.gameVersion ?? "1.21.1",
        software: "paper" as LocalSoftware,
        onlineMode: false,
        eulaAccepted: true,
        sourceInstanceId: inst?.id ?? null,
      });
      setSelected(s.id);
      await refresh();
      showOk("Serwer utworzony.");
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
        if (!confirm(`Usunąć serwer „${active.name}"?`)) return;
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
      <header className="flex items-center justify-between border-b border-line px-6 py-4">
        <div>
          <h1 className="text-xl font-extrabold">Host serwera</h1>
          <p className="mt-0.5 text-xs text-mute">Lokalny serwer Minecraft — Paper / Vanilla / Fabric.</p>
        </div>
        <button
          disabled={busy}
          onClick={() => void createServer()}
          className="inline-flex items-center gap-2 rounded-full bg-accent px-4 py-2 text-sm font-semibold text-bg-on-accent"
        >
          <Plus size={16} />
          Nowy serwer
        </button>
      </header>

      <div className="flex min-h-0 flex-1">
        <aside className="w-56 shrink-0 overflow-auto border-r border-line p-3">
          {servers.length === 0 ? (
            <p className="px-2 py-6 text-xs text-mute">Brak serwerów — utwórz pierwszy.</p>
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
                    {s.status === "running" ? "Działa" : s.status}
                  </span>
                </div>
              </button>
            ))
          )}
        </aside>

        {active ? (
          <section className="flex min-w-0 flex-1 flex-col">
            <div className="flex flex-wrap items-center gap-2 border-b border-line px-4 py-3">
              <button
                disabled={busy || active.jarReady}
                onClick={() => void run("install")}
                className="rounded-full border border-line px-3 py-1 text-xs hover:bg-white/5 disabled:opacity-40"
              >
                {active.jarReady ? "Zainstalowany" : "Instaluj JAR"}
              </button>
              {active.status !== "running" ? (
                <button
                  disabled={busy || !active.jarReady}
                  onClick={() => void run("start")}
                  className="inline-flex items-center gap-1 rounded-full bg-good px-3 py-1 text-xs font-semibold text-black disabled:opacity-40"
                >
                  <Play size={12} />
                  Start
                </button>
              ) : (
                <button
                  disabled={busy}
                  onClick={() => void run("stop")}
                  className="inline-flex items-center gap-1 rounded-full bg-danger px-3 py-1 text-xs font-semibold text-white"
                >
                  <Square size={12} />
                  Stop
                </button>
              )}
              <button
                onClick={() => api.openLocalServerFolder(active.id).catch((e) => showError(String(e)))}
                className="inline-flex items-center gap-1 rounded-full border border-line px-3 py-1 text-xs"
              >
                <FolderOpen size={12} />
                Folder
              </button>
              <button
                disabled={busy}
                onClick={() => void run("delete")}
                className="ml-auto inline-flex items-center gap-1 rounded-full border border-danger/40 px-3 py-1 text-xs text-danger"
              >
                <Trash2 size={12} />
                Usuń
              </button>
              <span className="w-full text-[10px] text-mute">
                Port {active.port} · {active.address}
                {active.lanIp ? ` · LAN ${active.lanIp}` : ""}
              </span>
            </div>

            <div className="min-h-0 flex-1 overflow-auto bg-black/40 p-3 font-mono text-[11px] leading-relaxed text-mute">
              {log || "Log serwera pojawi się tutaj…"}
            </div>

            <form
              className="flex items-center gap-2 border-t border-line p-3"
              onSubmit={(e) => {
                e.preventDefault();
                if (!cmd.trim()) return;
                api.sendLocalServerCommand(active.id, cmd).catch((e) => showError(String(e)));
                setCmd("");
              }}
            >
              <Terminal size={14} className="text-mute" />
              <input
                value={cmd}
                onChange={(e) => setCmd(e.target.value)}
                placeholder="Komenda serwera…"
                className="flex-1 rounded-xl bg-raised px-3 py-2 text-sm outline-none ring-1 ring-line"
                disabled={active.status !== "running"}
              />
            </form>
          </section>
        ) : (
          <div className="grid flex-1 place-items-center text-sm text-mute">
            Wybierz lub utwórz serwer lokalny.
          </div>
        )}
      </div>
    </div>
  );
}
