import { Plus, Settings2, SquarePen } from "lucide-react";
import { useMemo, useState } from "react";
import { clsx } from "clsx";
import { api } from "../lib/api";
import { confirmDialog, promptDialog } from "../lib/dialog";
import { formatDate, LOADER_LABEL, loaderChipClass } from "../lib/format";
import { pl } from "../locales/pl";
import { InstanceCreator } from "../components/InstanceCreator";
import { ProfileContextMenu } from "../components/ProfileContextMenu";
import { useApp } from "../stores/appStore";
import { useOctra } from "../stores/octraStore";
import { SectionHeader } from "../components/ui/SectionHeader";
import type { Instance, Loader } from "../types";

const LOADER_FILTERS: { id: Loader | "all"; label: string }[] = [
  { id: "all", label: pl.versions.filterAll },
  { id: "vanilla", label: "Vanilla" },
  { id: "fabric", label: "Fabric" },
  { id: "forge", label: "Forge" },
  { id: "quilt", label: "Quilt" },
  { id: "neoforge", label: "NeoForge" },
];

export function VersionsPage() {
  const instances = useApp((s) => s.instances);
  const refresh = useApp((s) => s.refreshInstances);
  const play = useApp((s) => s.playInstance);
  const selectedId = useOctra((s) => s.selectedId);
  const select = useOctra((s) => s.selectInstance);
  const openContent = useOctra((s) => s.openContent);
  const creatorOpen = useOctra((s) => s.instanceCreatorOpen);
  const openCreator = useOctra((s) => s.openInstanceCreator);
  const closeCreator = useOctra((s) => s.closeInstanceCreator);
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);
  const [filter, setFilter] = useState<Loader | "all">("all");
  const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number; inst: Instance } | null>(null);

  const filtered = useMemo(() => {
    const list = instances
      .slice()
      .sort((a, b) => (b.lastPlayed ?? "").localeCompare(a.lastPlayed ?? ""));
    if (filter === "all") return list;
    return list.filter((i) => i.loader === filter);
  }, [instances, filter]);

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-auto p-5">
      <InstanceCreator
        open={creatorOpen}
        onClose={closeCreator}
        onCreated={async (id) => {
          await refresh();
          select(id);
        }}
      />
      <SectionHeader
        title={pl.versions.title}
        action={
          <button
            className="inline-flex items-center gap-2 rounded-lg bg-accent px-3 py-1.5 text-sm font-semibold text-white"
            onClick={openCreator}
          >
            <Plus size={16} />
            {pl.versions.newProfile}
          </button>
        }
      />

      <div className="mt-3 flex flex-wrap gap-1.5">
        {LOADER_FILTERS.map((f) => (
          <button
            key={f.id}
            type="button"
            onClick={() => setFilter(f.id)}
            className={clsx(
              "rounded-full px-3 py-1 text-[11px] font-semibold transition",
              filter === f.id
                ? "bg-accent/25 text-ink ring-1 ring-accent/50"
                : "bg-raised2 text-mute hover:text-ink",
            )}
          >
            {f.label}
          </button>
        ))}
      </div>

      <div className="mt-3 flex flex-col gap-1.5">
        {filtered.map((inst) => {
          const active = selectedId === inst.id;
          return (
            <article
              key={inst.id}
              className={clsx(
                "flex items-center gap-3 rounded-xl border bg-raised2/80 px-3 py-2",
                active ? "border-accent/50" : "border-line",
              )}
              onContextMenu={(e) => {
                e.preventDefault();
                setCtxMenu({ x: e.clientX, y: e.clientY, inst });
              }}
            >
              <button
                type="button"
                className="min-w-0 flex-1 text-left"
                onClick={() => select(inst.id)}
              >
                <div className="flex flex-wrap items-center gap-2">
                  <span className="truncate text-sm font-semibold">{inst.name}</span>
                  <span
                    className={clsx(
                      "rounded-full px-2 py-0.5 text-[10px] font-semibold",
                      loaderChipClass(inst.loader),
                    )}
                  >
                    {LOADER_LABEL[inst.loader] ?? inst.loader} {inst.gameVersion}
                  </span>
                </div>
                <p className="mt-0.5 text-[10px] text-mute">
                  {pl.versions.lastPlayed}: {formatDate(inst.lastPlayed)}
                </p>
              </button>
              <div className="flex shrink-0 items-center gap-1">
                <button
                  className="grid h-8 w-8 place-items-center rounded-lg text-mute hover:bg-white/6 hover:text-ink"
                  onClick={() => {
                    select(inst.id);
                    openContent("mods");
                  }}
                  title={pl.versions.mods}
                >
                  <SquarePen size={15} />
                </button>
                <button
                  className="grid h-8 w-8 place-items-center rounded-lg text-mute hover:bg-white/6 hover:text-ink"
                  onClick={() => {
                    select(inst.id);
                    openContent("advanced");
                  }}
                  title={pl.versions.advanced}
                >
                  <Settings2 size={15} />
                </button>
                <button
                  className={clsx(
                    "rounded-full px-3 py-1 text-[11px] font-bold",
                    active ? "bg-launch text-white" : "bg-white/10 text-ink",
                  )}
                  onClick={() => {
                    select(inst.id);
                    void play(inst.id);
                  }}
                >
                  {pl.versions.play}
                </button>
              </div>
            </article>
          );
        })}
        {filtered.length === 0 && (
          <button
            onClick={openCreator}
            className="grid min-h-[120px] place-items-center rounded-xl border border-dashed border-line text-sm text-mute"
          >
            {instances.length === 0 ? pl.versions.noProfiles : "Brak profili dla tego filtra"}
          </button>
        )}
      </div>

      <ImportSection refresh={refresh} select={select} />

      {ctxMenu && (
        <ProfileContextMenu
          x={ctxMenu.x}
          y={ctxMenu.y}
          inst={ctxMenu.inst}
          onClose={() => setCtxMenu(null)}
          onDelete={() => {
            void (async () => {
              const target = ctxMenu.inst;
              const ok = await confirmDialog(
                pl.versions.deleteConfirm.replace("{name}", target.name),
                {
                  title: pl.versions.deleteProfile,
                  confirmLabel: pl.versions.deleteProfile,
                  danger: true,
                },
              );
              setCtxMenu(null);
              if (!ok) return;
              try {
                await api.deleteInstance(target.id);
                await refresh();
                if (selectedId === target.id) {
                  const rest = instances.filter((i) => i.id !== target.id);
                  select(rest[0]?.id ?? null);
                }
                showOk(pl.versions.deleted);
              } catch (e) {
                showError(e instanceof Error ? e.message : String(e));
              }
            })();
          }}
        />
      )}
    </div>
  );
}

function ImportSection({
  refresh,
  select,
}: {
  refresh: () => Promise<void>;
  select: (id: string) => void;
}) {
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);
  const [busy, setBusy] = useState(false);

  async function importMrpack() {
    setBusy(true);
    try {
      const path = await api.pickMrpackFile(null);
      if (!path) return;
      const inst = await api.importMrpack(path);
      await refresh();
      select(inst.id);
      showOk("Zaimportowano paczkę.");
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function importModrinth() {
    const slug = await promptDialog("Slug paczki Modrinth:", {
      title: pl.versions.importModrinth,
      confirmLabel: "Importuj",
    });
    if (!slug) return;
    setBusy(true);
    try {
      const inst = await api.importModrinthPack(slug.trim());
      await refresh();
      select(inst.id);
      showOk("Zaimportowano z Modrinth.");
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function importCurseforge() {
    setBusy(true);
    try {
      const hits = await api.scanCurseforgeInstances(null);
      if (hits.length === 0) {
        showError("Nie znaleziono instancji CurseForge App.");
        return;
      }
      const pick = hits[0]!;
      const inst = await api.importCurseforgeInstance(pick.path);
      await refresh();
      select(inst.id);
      showOk(`Zaimportowano „${pick.name}".`);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function importPrism() {
    setBusy(true);
    try {
      const hits = await api.scanPrismInstances();
      if (hits.length === 0) {
        showError("Nie znaleziono profili Prism Launcher.");
        return;
      }
      const pick = hits[0]!;
      const inst = await api.importLauncherInstance(pick.path, pick.source);
      await refresh();
      select(inst.id);
      showOk(`Zaimportowano „${pick.name}".`);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function importMultimc() {
    setBusy(true);
    try {
      const hits = await api.scanMultimcInstances(null);
      if (hits.length === 0) {
        showError("Nie znaleziono profili MultiMC.");
        return;
      }
      const pick = hits[0]!;
      const inst = await api.importLauncherInstance(pick.path, pick.source);
      await refresh();
      select(inst.id);
      showOk(`Zaimportowano „${pick.name}".`);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  const btn =
    "rounded-full border border-line px-3 py-1.5 text-[11px] font-semibold text-mute hover:border-accent/40 hover:text-ink disabled:opacity-40";

  return (
    <section className="mt-6 rounded-xl border border-line bg-raised/50 p-4">
      <h2 className="text-sm font-bold">{pl.versions.importTitle}</h2>
      <div className="mt-3 flex flex-wrap gap-2">
        <button className={btn} disabled={busy} onClick={() => void importModrinth()}>
          {pl.versions.importModrinth}
        </button>
        <button className={btn} disabled={busy} onClick={() => void importMrpack()}>
          {pl.versions.importZip}
        </button>
        <button className={btn} disabled={busy} onClick={() => void importCurseforge()}>
          {pl.versions.importCurseforge}
        </button>
        <button className={btn} disabled={busy} onClick={() => void importPrism()}>
          {pl.versions.importPrism}
        </button>
        <button className={btn} disabled={busy} onClick={() => void importMultimc()}>
          {pl.versions.importMultimc}
        </button>
      </div>
    </section>
  );
}
