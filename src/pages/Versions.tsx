import { Plus } from "lucide-react";
import { useMemo, useState } from "react";
import { clsx } from "clsx";
import { api } from "../lib/api";
import { confirmDialog, promptDialog } from "../lib/dialog";
import { pl } from "../locales/pl";
import { InstanceCreator } from "../components/InstanceCreator";
import { ProfileCard } from "../components/ProfileCard";
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

      <p className="mt-1 text-sm text-mute">{pl.versions.libraryHint}</p>

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

      <div
        className="mt-4 grid grid-cols-1 gap-4 lg:grid-cols-2"
        onContextMenu={(e) => {
          const card = (e.target as HTMLElement).closest("[data-profile-id]");
          if (!card) return;
          const id = card.getAttribute("data-profile-id");
          const inst = filtered.find((i) => i.id === id);
          if (!inst) return;
          e.preventDefault();
          setCtxMenu({ x: e.clientX, y: e.clientY, inst });
        }}
      >
        {filtered.map((inst) => (
          <div key={inst.id} data-profile-id={inst.id}>
            <ProfileCard
              inst={inst}
              variant="library"
              active={selectedId === inst.id}
              showPlaytime
              onClick={() => select(inst.id)}
              onPlay={() => {
                select(inst.id);
                void play(inst.id);
              }}
              onEdit={() => {
                select(inst.id);
                openContent("mods");
              }}
              onSettings={() => {
                select(inst.id);
                openContent("appearance");
              }}
            />
          </div>
        ))}
        {filtered.length === 0 && (
          <button
            onClick={openCreator}
            className="col-span-full grid min-h-[180px] place-items-center rounded-2xl border border-dashed border-line bg-raised2/30 text-sm text-mute hover:border-accent/30 hover:text-ink"
          >
            {instances.length === 0 ? pl.versions.noProfiles : pl.versions.noFilterMatch}
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
          onPlay={() => {
            select(ctxMenu.inst.id);
            setCtxMenu(null);
            void play(ctxMenu.inst.id);
          }}
          onMods={() => {
            select(ctxMenu.inst.id);
            setCtxMenu(null);
            openContent("mods");
          }}
          onAppearance={() => {
            select(ctxMenu.inst.id);
            setCtxMenu(null);
            openContent("appearance");
          }}
          onAdvanced={() => {
            select(ctxMenu.inst.id);
            setCtxMenu(null);
            openContent("advanced");
          }}
          onOpenFolder={() => {
            setCtxMenu(null);
            void api.openInstanceFolder(ctxMenu.inst.id).catch((e) =>
              showError(e instanceof Error ? e.message : String(e)),
            );
          }}
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
    <section className="mt-8 rounded-xl border border-line bg-raised/50 p-4">
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
