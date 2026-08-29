import { FolderPlus } from "lucide-react";

import { useEffect, useState } from "react";

import { api } from "../lib/api";

import { contentTabLabel } from "../lib/contentUi";

import { formatBytes, formatRam } from "../lib/format";

import { InstanceContentPane } from "../components/InstanceContentPane";

import { useApp } from "../stores/appStore";

import { useOctra, type ContentTab } from "../stores/octraStore";

import type { WorldEntry } from "../types";



const TABS: ContentTab[] = ["loader", "mods", "shaders", "worlds", "resources", "advanced"];



export function ContentOverlay() {

  const overlay = useOctra((s) => s.overlay);

  const close = useOctra((s) => s.closeOverlay);

  const selectedId = useOctra((s) => s.selectedId);

  const instances = useApp((s) => s.instances);

  const inst = instances.find((i) => i.id === selectedId) ?? instances[0];

  if (!overlay || overlay.kind !== "content" || !inst) return null;

  return (

    <div className="absolute inset-0 z-30 flex bg-black/60 p-8">

      <div className="flex min-h-0 w-full overflow-hidden rounded-3xl border border-line bg-raised shadow-2xl">

        <nav className="flex w-48 shrink-0 flex-col gap-1 border-r border-line p-4">

          <p className="mb-2 text-[10px] font-semibold uppercase tracking-wider text-mute">

            Wersja {inst.gameVersion}

          </p>

          {TABS.map((t) => (

            <button

              key={t}

              className={`rounded-xl px-3 py-2 text-left text-sm ${

                overlay.tab === t ? "bg-accent/25 text-ink" : "text-mute hover:bg-white/5"

              } ${t === "advanced" ? "mt-auto ring-1 ring-danger/40 text-danger" : ""}`}

              onClick={() => useOctra.getState().openContent(t)}

            >

              {contentTabLabel(t)}

            </button>

          ))}

        </nav>

        <div className="relative min-w-0 flex-1">

          <button

            className="absolute right-4 top-4 z-10 text-mute hover:text-ink"

            onClick={close}

          >

            ✕

          </button>

          {overlay.tab === "advanced" ? (

            <AdvancedPane id={inst.id} />

          ) : overlay.tab === "worlds" ? (

            <WorldsPane id={inst.id} />

          ) : overlay.tab === "loader" ? (

            <div className="p-6 text-sm text-mute">

              Loader: {inst.loader} {inst.loaderVersion ?? ""} · Minecraft {inst.gameVersion}

            </div>

          ) : (

            <InstanceContentPane inst={inst} tab={overlay.tab} />

          )}

        </div>

      </div>

    </div>

  );

}



function WorldsPane({ id }: { id: string }) {

  const [worlds, setWorlds] = useState<WorldEntry[]>([]);

  const [q, setQ] = useState("");

  const showError = useApp((s) => s.showError);

  useEffect(() => {

    api.listWorlds(id).then(setWorlds).catch((e) => showError(String(e)));

  }, [id, showError]);

  const list = worlds.filter((w) => w.name.toLowerCase().includes(q.toLowerCase()));

  return (

    <div className="flex h-full flex-col p-6">

      <div className="flex items-center gap-2">

        <input

          value={q}

          onChange={(e) => setQ(e.target.value)}

          placeholder="Szukaj świata…"

          className="flex-1 rounded-xl bg-bg px-3 py-2 text-sm ring-1 ring-line"

        />

        <button

          className="grid h-9 w-9 place-items-center rounded-xl bg-white/5"

          onClick={() => api.openInstanceSubdir(id, "saves").catch((e) => showError(String(e)))}

          title="Otwórz folder światów"

        >

          <FolderPlus size={16} />

        </button>

      </div>

      <p className="mt-2 text-xs text-mute">Światy · {list.length}</p>

      <button

        className="mt-3 rounded-2xl border border-dashed border-line px-4 py-8 text-sm text-mute"

        onClick={() => api.openInstanceSubdir(id, "saves").catch((e) => showError(String(e)))}

      >

        + Przeciągnij światy do folderu saves

      </button>

      <div className="mt-3 overflow-auto">

        {list.map((w) => (

          <div key={w.folder} className="flex items-center justify-between border-b border-line py-2">

            <div>

              <div className="text-sm font-medium">{w.name}</div>

              <div className="text-[11px] text-mute">{formatBytes(w.size)}</div>

            </div>

            <button

              className="text-xs text-mute"

              onClick={() => api.openWorldFolder(id, w.folder).catch((e) => showError(String(e)))}

            >

              Folder

            </button>

          </div>

        ))}

      </div>

    </div>

  );

}



function AdvancedPane({ id }: { id: string }) {

  const instances = useApp((s) => s.instances);

  const refresh = useApp((s) => s.refreshInstances);

  const inst = instances.find((i) => i.id === id);

  if (!inst) return null;

  const mem = inst.customMemory ? inst.memoryMaxMb : useApp.getState().settings?.memoryMaxMb ?? 4096;

  return (

    <div className="h-full overflow-auto p-6">

      <div className="flex items-center gap-2">

        <h2 className="text-lg font-semibold">Ustawienia zaawansowane</h2>

        <span className="text-xs text-danger">Ostrożnie.</span>

      </div>

      <Row title="Pamięć RAM" hint="Nadpisuje globalny RAM dla tego profilu.">

        <input

          type="range"

          min={1024}

          max={32768}

          step={256}

          value={mem}

          onChange={async (e) => {

            const v = Number(e.target.value);

            await api.updateInstance({

              ...inst,

              customMemory: true,

              memoryMaxMb: v,

              memoryMinMb: Math.min(inst.memoryMinMb || 512, v),

            });

            await refresh();

          }}

          className="w-full"

        />

        <p className="text-xs text-mute">{formatRam(mem)}</p>

      </Row>

      <Row title="Rozdzielczość">

        <div className="flex gap-2">

          <input

            className="w-24 rounded-lg bg-bg px-2 py-1 ring-1 ring-line"

            type="number"

            value={inst.windowWidth ?? 1920}

            onChange={async (e) => {

              await api.updateInstance({

                ...inst,

                customWindow: true,

                windowWidth: Number(e.target.value),

              });

              await refresh();

            }}

          />

          <span className="text-mute">×</span>

          <input

            className="w-24 rounded-lg bg-bg px-2 py-1 ring-1 ring-line"

            type="number"

            value={inst.windowHeight ?? 1080}

            onChange={async (e) => {

              await api.updateInstance({

                ...inst,

                customWindow: true,

                windowHeight: Number(e.target.value),

              });

              await refresh();

            }}

          />

        </div>

        <label className="mt-2 flex items-center gap-2 text-xs">

          <input

            type="checkbox"

            checked={Boolean(inst.fullscreen)}

            onChange={async (e) => {

              await api.updateInstance({ ...inst, customWindow: true, fullscreen: e.target.checked });

              await refresh();

            }}

          />

          Pełny ekran

        </label>

      </Row>

      <Row title="Argumenty JVM" hint="Własne flagi Javy.">

        <textarea

          className="h-20 w-full rounded-xl bg-bg p-2 text-xs ring-1 ring-line"

          value={inst.javaArgs}

          onChange={async (e) => {

            await api.updateInstance({ ...inst, customJavaArgs: true, javaArgs: e.target.value });

            await refresh();

          }}

        />

      </Row>

    </div>

  );

}



function Row({

  title,

  hint,

  children,

}: {

  title: string;

  hint?: string;

  children: React.ReactNode;

}) {

  return (

    <section className="mt-6 rounded-2xl border border-line p-4">

      <h3 className="text-sm font-semibold">{title}</h3>

      {hint && <p className="mt-1 text-xs text-mute">{hint}</p>}

      <div className="mt-3">{children}</div>

    </section>

  );

}


