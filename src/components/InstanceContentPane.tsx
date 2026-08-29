import {
  ArrowDownCircle,
  Download,
  FolderPlus,
  LayoutGrid,
  List,
  Loader2,
  RefreshCw,
  Search,
  Trash2,
} from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { clsx } from "clsx";
import { api } from "../lib/api";
import {
  contentTabLabel,
  loadContentDensity,
  modrinthLoader,
  modrinthProjectType,
  saveContentDensity,
  type ContentDensity,
} from "../lib/contentUi";
import { formatBytes, formatCount } from "../lib/format";
import { fetchModrinthProject, modrinthIconUrl } from "../lib/modrinthProject";
import { pl } from "../locales/pl";
import { useApp } from "../stores/appStore";
import type { ContentTab } from "../stores/octraStore";
import type {
  ContentFile,
  ContentKind,
  ContentUpdate,
  Instance,
  ModrinthPackHit,
} from "../types";

type PaneMode = "installed" | "modrinth";

function kindOf(tab: ContentTab): ContentKind | null {
  if (tab === "mods") return "mods";
  if (tab === "shaders") return "shaderpacks";
  if (tab === "resources") return "resourcepacks";
  return null;
}

export function InstanceContentPane({ inst, tab }: { inst: Instance; tab: ContentTab }) {
  const kind = kindOf(tab)!;
  const projectType = modrinthProjectType(tab)!;
  const [mode, setMode] = useState<PaneMode>("installed");
  const [density, setDensity] = useState<ContentDensity>(() => loadContentDensity());
  const [q, setQ] = useState("");
  const [files, setFiles] = useState<ContentFile[]>([]);
  const [loading, setLoading] = useState(true);
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);

  const reload = useCallback(async () => {
    setFiles(await api.listInstanceContent(inst.id));
  }, [inst.id]);

  useEffect(() => {
    setLoading(true);
    void reload()
      .catch((e) => showError(String(e)))
      .finally(() => setLoading(false));
  }, [inst.id, reload, showError]);

  const list = files.filter(
    (f) => f.kind === kind && f.displayName.toLowerCase().includes(q.toLowerCase()),
  );

  function setDensityAndSave(next: ContentDensity) {
    setDensity(next);
    saveContentDensity(next);
  }

  return (
    <div className="flex h-full min-h-0 flex-col p-6">
      <div className="flex flex-wrap items-center gap-2">
        <div className="flex min-w-[200px] flex-1 items-center gap-2 rounded-xl bg-bg px-3 py-2 ring-1 ring-line">
          <Search size={14} className="shrink-0 text-mute" />
          <input
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder={
              mode === "modrinth"
                ? pl.content.searchModrinth.replace("{type}", contentTabLabel(tab).toLowerCase())
                : pl.content.searchLocal.replace("{type}", contentTabLabel(tab).toLowerCase())
            }
            className="min-w-0 flex-1 bg-transparent text-sm outline-none"
          />
        </div>

        <div className="flex rounded-xl bg-raised2 p-0.5 ring-1 ring-line">
          <ModeBtn active={mode === "installed"} onClick={() => setMode("installed")}>
            {pl.content.installed}
          </ModeBtn>
          <ModeBtn active={mode === "modrinth"} onClick={() => setMode("modrinth")}>
            Modrinth
          </ModeBtn>
        </div>

        <div className="flex rounded-xl bg-raised2 p-0.5 ring-1 ring-line">
          <button
            type="button"
            title={pl.content.densityNormal}
            onClick={() => setDensityAndSave("normal")}
            className={clsx(
              "grid h-8 w-8 place-items-center rounded-lg transition",
              density === "normal" ? "bg-accent/20 text-ink" : "text-mute hover:text-ink",
            )}
          >
            <List size={15} />
          </button>
          <button
            type="button"
            title={pl.content.densityCompact}
            onClick={() => setDensityAndSave("compact")}
            className={clsx(
              "grid h-8 w-8 place-items-center rounded-lg transition",
              density === "compact" ? "bg-accent/20 text-ink" : "text-mute hover:text-ink",
            )}
          >
            <LayoutGrid size={15} />
          </button>
        </div>

        <button
          type="button"
          className="grid h-9 w-9 place-items-center rounded-xl bg-white/5 text-mute hover:bg-white/10 hover:text-ink"
          onClick={() => {
            void api.openInstanceSubdir(inst.id, kind).catch((e) => showError(String(e)));
          }}
          title={pl.content.openFolder}
        >
          <FolderPlus size={16} />
        </button>
      </div>

      <p className="mt-2 text-xs text-mute">
        {mode === "installed"
          ? pl.content.loadedCount.replace("{n}", String(list.length))
          : pl.content.modrinthHint}
      </p>

      {mode === "modrinth" ? (
        <ModrinthPanel
          inst={inst}
          projectType={projectType}
          query={q}
          onInstalled={async (warnings) => {
            await reload();
            setMode("installed");
            if (warnings.length) showError(warnings.join("\n"));
            else showOk(pl.content.installedOk);
          }}
        />
      ) : (
        <InstalledList
          inst={inst}
          kind={kind}
          projectType={projectType}
          density={density}
          loading={loading}
          list={list}
          onChange={setFiles}
          onReload={reload}
          onOpenFolder={() => {
            void api.openInstanceSubdir(inst.id, kind).catch((e) => showError(String(e)));
          }}
        />
      )}
    </div>
  );
}

function ModeBtn({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={clsx(
        "rounded-lg px-3 py-1.5 text-xs font-semibold transition",
        active ? "bg-accent/25 text-ink" : "text-mute hover:text-ink",
      )}
    >
      {children}
    </button>
  );
}

function InstalledList({
  inst,
  kind,
  projectType,
  density,
  loading,
  list,
  onChange,
  onReload,
  onOpenFolder,
}: {
  inst: Instance;
  kind: ContentKind;
  projectType: string;
  density: ContentDensity;
  loading: boolean;
  list: ContentFile[];
  onChange: (files: ContentFile[]) => void;
  onReload: () => Promise<void>;
  onOpenFolder: () => void;
}) {
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);
  const compact = density === "compact";
  const [updates, setUpdates] = useState<ContentUpdate[]>([]);
  const [checkingUpdates, setCheckingUpdates] = useState(false);
  const [updating, setUpdating] = useState<string | null>(null);
  const [titles, setTitles] = useState<Record<string, { title: string; iconUrl?: string | null }>>(
    {},
  );

  const updateByFile = useMemo(() => {
    const map = new Map<string, ContentUpdate>();
    for (const u of updates) {
      if (u.kind === kind) map.set(u.fileName, u);
    }
    return map;
  }, [updates, kind]);

  const refreshUpdates = useCallback(async () => {
    setCheckingUpdates(true);
    try {
      setUpdates(await api.checkContentUpdates(inst.id));
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setCheckingUpdates(false);
    }
  }, [inst.id, showError]);

  useEffect(() => {
    void refreshUpdates();
  }, [refreshUpdates, list.length]);

  useEffect(() => {
    let cancelled = false;
    async function enrich() {
      const next: Record<string, { title: string; iconUrl?: string | null }> = {};
      await Promise.all(
        list.map(async (f) => {
          const upd = updateByFile.get(f.name);
          if (upd?.projectTitle) {
            next[f.name] = {
              title: upd.projectTitle,
              iconUrl: modrinthIconUrl(f.projectId),
            };
            return;
          }
          if (f.slug) {
            const p = await fetchModrinthProject(f.slug);
            if (p) next[f.name] = p;
          }
        }),
      );
      if (!cancelled) setTitles((prev) => ({ ...prev, ...next }));
    }
    void enrich();
    return () => {
      cancelled = true;
    };
  }, [list, updateByFile]);

  async function updateOne(file: ContentFile, upd: ContentUpdate) {
    if (!upd.slug) return;
    setUpdating(file.name);
    try {
      await api.installModrinthContent(inst.id, upd.slug, projectType);
      await onReload();
      await refreshUpdates();
      showOk(pl.content.updatedOk);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setUpdating(null);
    }
  }

  async function updateAll() {
    const pending = list
      .map((f) => ({ f, u: updateByFile.get(f.name) }))
      .filter((x): x is { f: ContentFile; u: ContentUpdate } => Boolean(x.u?.slug));
    for (const { f, u } of pending) {
      await updateOne(f, u);
    }
  }

  const pendingCount = list.filter((f) => updateByFile.has(f.name)).length;

  if (loading) {
    return (
      <div className="mt-6 grid flex-1 place-items-center text-sm text-mute">
        <Loader2 className="animate-spin" size={22} />
      </div>
    );
  }

  if (list.length === 0) {
    return (
      <button
        type="button"
        className="mt-4 flex flex-1 flex-col items-center justify-center rounded-2xl border border-dashed border-line px-4 py-10 text-sm text-mute hover:border-accent/40 hover:text-ink"
        onClick={onOpenFolder}
      >
        <p>{pl.content.dropHint}</p>
        <p className="mt-2 text-xs">{pl.content.orModrinth}</p>
      </button>
    );
  }

  return (
    <div className="mt-3 flex min-h-0 flex-1 flex-col">
      <div className="mb-2 flex flex-wrap items-center gap-2">
        <button
          type="button"
          onClick={() => void refreshUpdates()}
          disabled={checkingUpdates}
          className="inline-flex items-center gap-1.5 rounded-lg border border-line px-2.5 py-1.5 text-[11px] font-semibold text-mute hover:border-accent/40 hover:text-ink disabled:opacity-50"
        >
          <RefreshCw size={13} className={checkingUpdates ? "animate-spin" : ""} />
          {checkingUpdates ? pl.content.checkingUpdates : "Odśwież"}
        </button>
        {pendingCount > 0 && (
          <button
            type="button"
            onClick={() => void updateAll()}
            className="inline-flex items-center gap-1.5 rounded-lg bg-good/20 px-2.5 py-1.5 text-[11px] font-bold text-good ring-1 ring-good/40 hover:bg-good/30"
          >
            <ArrowDownCircle size={13} />
            {pl.content.updateAll} ({pendingCount})
          </button>
        )}
      </div>

      {!compact && (
        <div className="mb-1 grid grid-cols-[minmax(0,1fr)_120px_140px] gap-2 px-3 text-[10px] font-semibold uppercase tracking-wider text-mute">
          <span>Projekt</span>
          <span>{pl.content.version}</span>
          <span className="text-right">Akcje</span>
        </div>
      )}

      <div className={clsx("min-h-0 flex-1 overflow-auto", compact ? "space-y-1" : "space-y-1.5")}>
        {list.map((f) => {
          const upd = updateByFile.get(f.name);
          const meta = titles[f.name];
          const title = meta?.title ?? upd?.projectTitle ?? f.displayName;
          const icon =
            meta?.iconUrl ?? modrinthIconUrl(f.projectId) ?? (f.slug ? modrinthIconUrl(null) : null);
          const version = upd?.currentVersion ?? f.name.replace(/\.jar$/i, "").replace(/\.zip$/i, "");

          return (
            <article
              key={f.name}
              className={clsx(
                "grid items-center gap-3 rounded-xl border border-line bg-raised2/40 transition hover:border-accent/25",
                compact
                  ? "grid-cols-[auto_minmax(0,1fr)_auto_auto] px-2 py-1.5"
                  : "grid-cols-[auto_minmax(0,1fr)_120px_140px] px-3 py-2.5",
                !f.enabled && "opacity-70",
              )}
            >
              <ContentIcon iconUrl={icon} title={title} compact={compact} />

              <div className="min-w-0">
                <div
                  className={clsx(
                    "font-semibold leading-tight",
                    compact ? "text-xs" : "text-sm",
                    f.enabled ? "text-ink" : "text-mute",
                  )}
                >
                  {title}
                </div>
                <div className={clsx("truncate text-mute", compact ? "text-[10px]" : "text-[11px]")}>
                  {f.name} · {formatBytes(f.size)}
                  {f.slug ? ` · ${f.slug}` : ""}
                </div>
                {upd?.latestVersion && (
                  <p className="mt-0.5 text-[10px] font-medium text-good">
                    {pl.content.updateAvailable}: {upd.latestVersion}
                  </p>
                )}
              </div>

              {!compact && (
                <div className="text-[11px] text-mute">
                  <div className="truncate font-medium text-ink/90">{version}</div>
                </div>
              )}

              <div className="flex items-center justify-end gap-1.5">
                {upd?.slug && (
                  <button
                    type="button"
                    disabled={updating === f.name}
                    title={pl.content.update}
                    onClick={() => void updateOne(f, upd)}
                    className="grid h-8 w-8 place-items-center rounded-lg bg-good/15 text-good ring-1 ring-good/30 hover:bg-good/25 disabled:opacity-50"
                  >
                    {updating === f.name ? (
                      <Loader2 size={14} className="animate-spin" />
                    ) : (
                      <ArrowDownCircle size={14} />
                    )}
                  </button>
                )}
                <EnableToggle
                  enabled={f.enabled}
                  compact={compact}
                  onToggle={async () => {
                    try {
                      onChange(await api.toggleInstanceContent(inst.id, kind, f.name));
                    } catch (e) {
                      showError(e instanceof Error ? e.message : String(e));
                    }
                  }}
                />
                <button
                  type="button"
                  className="grid h-8 w-8 shrink-0 place-items-center rounded-lg text-mute transition hover:bg-danger/15 hover:text-danger"
                  title={pl.content.remove}
                  onClick={async () => {
                    try {
                      onChange(await api.deleteInstanceContent(inst.id, kind, f.name));
                    } catch (e) {
                      showError(e instanceof Error ? e.message : String(e));
                    }
                  }}
                >
                  <Trash2 size={compact ? 14 : 16} />
                </button>
              </div>
            </article>
          );
        })}
      </div>
    </div>
  );
}

function ContentIcon({
  iconUrl,
  title,
  compact,
}: {
  iconUrl: string | null;
  title: string;
  compact: boolean;
}) {
  const size = compact ? "h-8 w-8" : "h-11 w-11";
  const [failed, setFailed] = useState(false);

  if (iconUrl && !failed) {
    return (
      <img
        src={iconUrl}
        alt=""
        className={clsx(size, "shrink-0 rounded-lg bg-black/30 object-cover ring-1 ring-white/10")}
        onError={() => setFailed(true)}
      />
    );
  }

  return (
    <div
      className={clsx(
        size,
        "grid shrink-0 place-items-center rounded-lg bg-white/5 text-xs font-bold text-mute ring-1 ring-white/10",
      )}
    >
      {title.slice(0, 1).toUpperCase()}
    </div>
  );
}

function EnableToggle({
  enabled,
  compact,
  onToggle,
}: {
  enabled: boolean;
  compact: boolean;
  onToggle: () => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={enabled}
      onClick={() => void onToggle()}
      className={clsx(
        "relative shrink-0 rounded-full border-2 transition",
        compact ? "h-6 w-11" : "h-7 w-[3.25rem]",
        enabled
          ? "border-good bg-good/30"
          : "border-line bg-white/10 hover:border-accent/40",
      )}
      title={enabled ? pl.content.enabled : pl.content.disabled}
    >
      <span
        className={clsx(
          "absolute top-1/2 block -translate-y-1/2 rounded-full bg-white shadow transition-all",
          compact ? "h-4 w-4" : "h-5 w-5",
          enabled ? (compact ? "left-[1.35rem]" : "left-[1.55rem]") : "left-0.5",
        )}
      />
    </button>
  );
}

function ModrinthPanel({
  inst,
  projectType,
  query,
  onInstalled,
}: {
  inst: Instance;
  projectType: string;
  query: string;
  onInstalled: (warnings: string[]) => void;
}) {
  const [hits, setHits] = useState<ModrinthPackHit[]>([]);
  const [loading, setLoading] = useState(false);
  const [installing, setInstalling] = useState<string | null>(null);
  const showError = useApp((s) => s.showError);

  useEffect(() => {
    let cancelled = false;
    const timer = setTimeout(() => {
      setLoading(true);
      api
        .searchModrinthContent({
          query,
          projectType,
          gameVersion: inst.gameVersion,
          loader: modrinthLoader(inst.loader),
          limit: 20,
          sort: query.trim() ? "relevance" : "downloads",
        })
        .then((res) => {
          if (!cancelled) setHits(res.hits);
        })
        .catch((e) => {
          if (!cancelled) {
            setHits([]);
            showError(e instanceof Error ? e.message : String(e));
          }
        })
        .finally(() => {
          if (!cancelled) setLoading(false);
        });
    }, 320);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [query, projectType, inst.gameVersion, inst.loader, showError]);

  async function install(hit: ModrinthPackHit) {
    setInstalling(hit.slug);
    try {
      const res = await api.installModrinthContent(inst.id, hit.slug, projectType);
      onInstalled(res.warnings);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setInstalling(null);
    }
  }

  return (
    <div className="mt-3 min-h-0 flex-1 overflow-auto">
      {loading ? (
        <div className="grid py-12 place-items-center text-mute">
          <Loader2 className="animate-spin" size={22} />
        </div>
      ) : hits.length === 0 ? (
        <p className="py-12 text-center text-sm text-mute">{pl.content.modrinthEmpty}</p>
      ) : (
        <div className="grid gap-2">
          {hits.map((hit) => (
            <article
              key={hit.slug}
              className="flex items-start gap-3 rounded-xl border border-line bg-raised2/50 p-3 transition hover:border-accent/30"
            >
              {hit.iconUrl ? (
                <img
                  src={hit.iconUrl}
                  alt=""
                  className="h-11 w-11 shrink-0 rounded-lg bg-black/30 object-cover"
                />
              ) : (
                <div className="grid h-11 w-11 shrink-0 place-items-center rounded-lg bg-white/5 text-xs text-mute">
                  ?
                </div>
              )}
              <div className="min-w-0 flex-1">
                <h3 className="truncate text-sm font-semibold">{hit.title}</h3>
                <p className="mt-0.5 line-clamp-2 text-[11px] text-mute">{hit.description}</p>
                <p className="mt-1 text-[10px] text-mute">
                  {formatCount(hit.downloads)} pobrań
                  {hit.author ? ` · ${hit.author}` : ""}
                </p>
              </div>
              <button
                type="button"
                disabled={installing === hit.slug}
                onClick={() => void install(hit)}
                className="inline-flex shrink-0 items-center gap-1.5 rounded-lg bg-accent px-3 py-2 text-xs font-bold text-white hover:brightness-110 disabled:opacity-50"
              >
                {installing === hit.slug ? (
                  <Loader2 size={14} className="animate-spin" />
                ) : (
                  <Download size={14} />
                )}
                {pl.content.install}
              </button>
            </article>
          ))}
        </div>
      )}
    </div>
  );
}
