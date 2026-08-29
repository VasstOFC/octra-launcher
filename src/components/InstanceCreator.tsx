import { ChevronLeft, ChevronRight, Loader2, Search, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { clsx } from "clsx";
import { api } from "../lib/api";
import { confirmDialog } from "../lib/dialog";
import { LOADER_LABEL } from "../lib/format";
import {
  filterVersions,
  isLegacyMinecraftVersion,
  versionBucket,
  versionTypeLabel,
  type VersionBucket,
} from "../lib/minecraftVersions";
import { paletteById, PROFILE_PALETTES } from "../lib/profilePalettes";
import { pl } from "../locales/pl";
import { Button } from "./ui/Button";
import type { Loader, ManifestVersion } from "../types";

const LOADERS: Loader[] = ["vanilla", "fabric", "quilt", "forge", "neoforge"];

const STEPS = ["basics", "version", "look"] as const;
type Step = (typeof STEPS)[number];

type Props = {
  open: boolean;
  onClose: () => void;
  onCreated: (id: string) => void;
};

export function InstanceCreator({ open, onClose, onCreated }: Props) {
  const [step, setStep] = useState<Step>("basics");
  const [name, setName] = useState("");
  const [loader, setLoader] = useState<Loader>("vanilla");
  const [gameVersion, setGameVersion] = useState<ManifestVersion | null>(null);
  const [loaderVersion, setLoaderVersion] = useState("");
  const [paletteId, setPaletteId] = useState(PROFILE_PALETTES[3]!.id);
  const [versions, setVersions] = useState<ManifestVersion[]>([]);
  const [loaderVersions, setLoaderVersions] = useState<string[]>([]);
  const [loaderRecommended, setLoaderRecommended] = useState<string | null>(null);
  const [versionQuery, setVersionQuery] = useState("");
  const [versionFilter, setVersionFilter] = useState<VersionBucket | "all">("all");
  const [legacyExpanded, setLegacyExpanded] = useState(false);
  const [loadingVersions, setLoadingVersions] = useState(false);
  const [loadingLoader, setLoadingLoader] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setStep("basics");
    setName("");
    setLoader("vanilla");
    setGameVersion(null);
    setLoaderVersion("");
    setPaletteId(PROFILE_PALETTES[3]!.id);
    setVersionQuery("");
    setVersionFilter("all");
    setLegacyExpanded(false);
    setError(null);
    setLoadingVersions(true);
    api
      .fetchMinecraftVersions(true)
      .then((list) => {
        setVersions(list);
        const latest = list.find((v) => v.versionType === "release") ?? list[0] ?? null;
        setGameVersion(latest);
        if (latest) setName(`Vanilla ${latest.id}`);
      })
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoadingVersions(false));
  }, [open]);

  useEffect(() => {
    if (!open || !gameVersion || loader === "vanilla") {
      setLoaderVersions([]);
      setLoaderRecommended(null);
      setLoaderVersion("");
      return;
    }
    let cancelled = false;
    setLoadingLoader(true);
    api
      .fetchLoaderVersions(loader, gameVersion.id)
      .then((lv) => {
        if (cancelled) return;
        setLoaderVersions(lv.versions);
        setLoaderRecommended(lv.recommended ?? lv.versions[0] ?? null);
        setLoaderVersion(lv.recommended ?? lv.versions[0] ?? "");
      })
      .catch(() => {
        if (!cancelled) {
          setLoaderVersions([]);
          setLoaderRecommended(null);
          setLoaderVersion("");
        }
      })
      .finally(() => {
        if (!cancelled) setLoadingLoader(false);
      });
    return () => {
      cancelled = true;
    };
  }, [open, gameVersion?.id, loader]);

  const filteredVersions = useMemo(
    () => filterVersions(versions, versionQuery, versionFilter),
    [versions, versionQuery, versionFilter],
  );

  const grouped = useMemo(() => {
    const current: ManifestVersion[] = [];
    const snapshot: ManifestVersion[] = [];
    const legacy: ManifestVersion[] = [];
    for (const v of filteredVersions) {
      const bucket = versionBucket(v);
      if (bucket === "legacy") legacy.push(v);
      else if (bucket === "snapshot") snapshot.push(v);
      else current.push(v);
    }
    return { current, snapshot, legacy };
  }, [filteredVersions]);

  const palette = paletteById(paletteId);
  const stepIndex = STEPS.indexOf(step);

  if (!open) return null;

  async function pickVersion(v: ManifestVersion) {
    if (isLegacyMinecraftVersion(v)) {
      const ok = await confirmDialog(pl.creator.legacyConfirm.replace("{version}", v.id), {
        title: pl.creator.legacyTitle,
        confirmLabel: pl.creator.legacyConfirmBtn,
        cancelLabel: pl.creator.cancel,
      });
      if (!ok) return;
    }
    setGameVersion(v);
    setName((prev) => {
      const trimmed = prev.trim();
      if (!trimmed || /^Vanilla\s/.test(trimmed)) {
        return `${LOADER_LABEL[loader] ?? loader} ${v.id}`;
      }
      return prev;
    });
  }

  async function submit() {
    if (!gameVersion) {
      setError(pl.creator.pickVersion);
      return;
    }
    const trimmed = name.trim();
    if (!trimmed) {
      setError(pl.creator.nameRequired);
      return;
    }
    if (loader !== "vanilla" && !loaderVersion) {
      setError(pl.creator.loaderVersionRequired);
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const created = await api.createInstance({
        name: trimmed,
        gameVersion: gameVersion.id,
        loader,
        loaderVersion: loader === "vanilla" ? undefined : loaderVersion,
      });
      const glyph = trimmed.slice(0, 2).toUpperCase();
      const styled = await api.updateInstance({
        ...created,
        iconColor: palette.glyph,
        iconSymbol: glyph,
        ledColor: palette.c1,
        ledColor2: palette.c2,
      });
      onCreated(styled.id);
      onClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSubmitting(false);
    }
  }

  function next() {
    if (step === "basics") {
      if (!name.trim()) {
        setError(pl.creator.nameRequired);
        return;
      }
      setError(null);
      setStep("version");
      return;
    }
    if (step === "version") {
      if (!gameVersion) {
        setError(pl.creator.pickVersion);
        return;
      }
      if (loader !== "vanilla" && !loaderVersion) {
        setError(pl.creator.loaderVersionRequired);
        return;
      }
      setError(null);
      setStep("look");
      return;
    }
    void submit();
  }

  function back() {
    setError(null);
    if (step === "version") setStep("basics");
    else if (step === "look") setStep("version");
  }

  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-black/60 p-4">
      <div
        className="flex max-h-[min(720px,92vh)] w-full max-w-2xl flex-col overflow-hidden rounded-2xl border border-line bg-raised shadow-2xl"
        role="dialog"
        aria-modal="true"
        aria-labelledby="instance-creator-title"
      >
        <header className="flex items-center justify-between border-b border-line px-5 py-4">
          <div>
            <h2 id="instance-creator-title" className="text-base font-bold">
              {pl.creator.title}
            </h2>
            <p className="mt-0.5 text-[11px] text-mute">
              {pl.creator.step} {stepIndex + 1}/{STEPS.length} — {pl.creator.steps[step]}
            </p>
          </div>
          <button
            type="button"
            className="grid h-8 w-8 place-items-center rounded-lg text-mute hover:bg-white/6 hover:text-ink"
            onClick={onClose}
            aria-label={pl.creator.close}
          >
            <X size={16} />
          </button>
        </header>

        <div className="flex gap-1 border-b border-line px-5 py-2">
          {STEPS.map((s, i) => (
            <div
              key={s}
              className={clsx(
                "h-1 flex-1 rounded-full transition",
                i <= stepIndex ? "bg-accent" : "bg-white/10",
              )}
            />
          ))}
        </div>

        <div className="min-h-0 flex-1 overflow-auto px-5 py-4">
          {step === "basics" && (
            <div className="space-y-4">
              <label className="block">
                <span className="text-xs font-semibold text-mute">{pl.creator.nameLabel}</span>
                <input
                  className="mt-1.5 w-full rounded-xl border border-line bg-raised2 px-3 py-2.5 text-sm outline-none focus:border-accent/50"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder={pl.creator.namePlaceholder}
                  autoFocus
                />
              </label>
              <div>
                <span className="text-xs font-semibold text-mute">{pl.creator.loaderLabel}</span>
                <div className="mt-2 flex flex-wrap gap-2">
                  {LOADERS.map((l) => (
                    <button
                      key={l}
                      type="button"
                      onClick={() => setLoader(l)}
                      className={clsx(
                        "rounded-full px-3 py-1.5 text-xs font-semibold transition",
                        loader === l
                          ? "bg-accent/25 text-ink ring-1 ring-accent/50"
                          : "bg-raised2 text-mute hover:text-ink",
                      )}
                    >
                      {LOADER_LABEL[l] ?? l}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          )}

          {step === "version" && (
            <div className="flex min-h-[360px] flex-col gap-3">
              <div className="relative">
                <Search size={14} className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-mute" />
                <input
                  className="w-full rounded-xl border border-line bg-raised2 py-2 pl-9 pr-3 text-sm outline-none focus:border-accent/50"
                  placeholder={pl.creator.searchVersion}
                  value={versionQuery}
                  onChange={(e) => setVersionQuery(e.target.value)}
                />
              </div>

              <div className="flex flex-wrap gap-1.5">
                {(
                  [
                    ["all", pl.creator.filterAll],
                    ["current", pl.creator.filterCurrent],
                    ["snapshot", pl.creator.filterSnapshots],
                    ["legacy", pl.creator.filterLegacy],
                  ] as const
                ).map(([id, label]) => (
                  <button
                    key={id}
                    type="button"
                    onClick={() => setVersionFilter(id)}
                    className={clsx(
                      "rounded-full px-2.5 py-1 text-[10px] font-semibold",
                      versionFilter === id
                        ? "bg-accent/25 text-ink ring-1 ring-accent/40"
                        : "bg-raised2 text-mute hover:text-ink",
                    )}
                  >
                    {label}
                  </button>
                ))}
              </div>

              {loadingVersions ? (
                <div className="grid flex-1 place-items-center text-sm text-mute">
                  <Loader2 className="animate-spin" size={20} />
                </div>
              ) : (
                <div className="min-h-0 flex-1 space-y-3 overflow-auto pr-1">
                  <VersionSection
                    title={pl.creator.sectionCurrent}
                    versions={grouped.current}
                    selectedId={gameVersion?.id}
                    onSelect={(v) => void pickVersion(v)}
                  />
                  <VersionSection
                    title={pl.creator.sectionSnapshots}
                    versions={grouped.snapshot}
                    selectedId={gameVersion?.id}
                    onSelect={(v) => void pickVersion(v)}
                  />
                  <LegacyVersionSection
                    title={pl.creator.sectionLegacy}
                    hint={pl.creator.legacyHint}
                    versions={grouped.legacy}
                    selectedId={gameVersion?.id}
                    expanded={legacyExpanded || versionFilter === "legacy"}
                    onToggle={() => setLegacyExpanded((v) => !v)}
                    onSelect={(v) => void pickVersion(v)}
                  />
                  {filteredVersions.length === 0 && (
                    <p className="py-8 text-center text-sm text-mute">{pl.creator.noVersions}</p>
                  )}
                </div>
              )}

              {loader !== "vanilla" && gameVersion && (
                <div className="rounded-xl border border-line bg-raised2/60 p-3">
                  <label className="block text-xs font-semibold text-mute">
                    {pl.creator.loaderVersionLabel} ({LOADER_LABEL[loader]})
                  </label>
                  {loadingLoader ? (
                    <p className="mt-2 text-xs text-mute">{pl.creator.loadingLoader}</p>
                  ) : loaderVersions.length === 0 ? (
                    <p className="mt-2 text-xs text-warn">{pl.creator.noLoaderVersions}</p>
                  ) : (
                    <select
                      className="mt-2 w-full rounded-lg border border-line bg-raised px-3 py-2 text-sm outline-none focus:border-accent/50"
                      value={loaderVersion}
                      onChange={(e) => setLoaderVersion(e.target.value)}
                    >
                      {loaderVersions.map((v) => (
                        <option key={v} value={v}>
                          {v}
                          {v === loaderRecommended ? ` (${pl.creator.recommended})` : ""}
                        </option>
                      ))}
                    </select>
                  )}
                </div>
              )}
            </div>
          )}

          {step === "look" && (
            <div className="space-y-4">
              <p className="text-sm text-mute">{pl.creator.colorHint}</p>
              <div className="grid grid-cols-3 gap-2 sm:grid-cols-4">
                {PROFILE_PALETTES.map((p) => (
                  <button
                    key={p.id}
                    type="button"
                    onClick={() => setPaletteId(p.id)}
                    className={clsx(
                      "overflow-hidden rounded-xl border p-2 text-left transition",
                      paletteId === p.id
                        ? "border-accent ring-1 ring-accent/40"
                        : "border-line hover:border-accent/30",
                    )}
                  >
                    <div
                      className="h-12 rounded-lg"
                      style={{
                        background: `linear-gradient(135deg, ${p.c1} 0%, ${p.c2} 100%)`,
                      }}
                    />
                    <span className="mt-1.5 block text-[10px] font-semibold">{p.name}</span>
                  </button>
                ))}
              </div>

              <div className="overflow-hidden rounded-2xl border border-line">
                <div
                  className="relative h-36"
                  style={{
                    background: `linear-gradient(135deg, ${palette.c1} 0%, ${palette.c2}55 45%, #0f0f12 100%)`,
                  }}
                >
                  <div className="absolute right-[20%] top-1/2 flex -translate-y-1/2 flex-col items-center">
                    <div
                      className="grid h-14 w-14 place-items-center rounded-xl border border-white/15 bg-black/25 text-lg font-bold"
                      style={{ color: palette.glyph }}
                    >
                      {(name.trim() || "MC").slice(0, 2).toUpperCase()}
                    </div>
                    <span className="mt-2 text-[10px] font-semibold uppercase tracking-widest text-white/50">
                      {LOADER_LABEL[loader]}
                    </span>
                  </div>
                </div>
                <div className="border-t border-line bg-raised2/50 px-4 py-3 text-sm">
                  <p className="font-semibold">{name.trim() || "—"}</p>
                  <p className="mt-0.5 text-xs text-mute">
                    {LOADER_LABEL[loader]} · {gameVersion?.id ?? "—"}
                  </p>
                </div>
              </div>
            </div>
          )}

          {error && (
            <p className="mt-3 rounded-lg bg-danger/15 px-3 py-2 text-xs text-danger">{error}</p>
          )}
        </div>

        <footer className="flex items-center justify-between border-t border-line px-5 py-4">
          <Button variant="ghost" onClick={step === "basics" ? onClose : back} disabled={submitting}>
            {step === "basics" ? (
              pl.creator.cancel
            ) : (
              <>
                <ChevronLeft size={14} className="mr-1" />
                {pl.creator.back}
              </>
            )}
          </Button>
          <Button variant="primary" onClick={() => void next()} disabled={submitting || loadingVersions}>
            {submitting ? (
              <>
                <Loader2 size={14} className="mr-1 animate-spin" />
                {pl.creator.creating}
              </>
            ) : step === "look" ? (
              pl.creator.create
            ) : (
              <>
                {pl.creator.next}
                <ChevronRight size={14} className="ml-1" />
              </>
            )}
          </Button>
        </footer>
      </div>
    </div>
  );
}

function VersionSection({
  title,
  versions,
  selectedId,
  onSelect,
}: {
  title: string;
  versions: ManifestVersion[];
  selectedId?: string;
  onSelect: (v: ManifestVersion) => void;
}) {
  if (versions.length === 0) return null;
  return (
    <section>
      <h3 className="mb-1.5 text-[10px] font-bold uppercase tracking-wider text-mute">{title}</h3>
      <div className="max-h-44 overflow-auto rounded-xl border border-line">
        {versions.map((v) => (
          <VersionRow key={v.id} v={v} active={selectedId === v.id} onSelect={() => onSelect(v)} />
        ))}
      </div>
    </section>
  );
}

function LegacyVersionSection({
  title,
  hint,
  versions,
  selectedId,
  expanded,
  onToggle,
  onSelect,
}: {
  title: string;
  hint: string;
  versions: ManifestVersion[];
  selectedId?: string;
  expanded: boolean;
  onToggle: () => void;
  onSelect: (v: ManifestVersion) => void;
}) {
  if (versions.length === 0) return null;
  return (
    <section className="rounded-xl border border-warn/25 bg-warn/5">
      <button
        type="button"
        className="flex w-full items-center justify-between px-3 py-2 text-left"
        onClick={onToggle}
      >
        <div>
          <h3 className="text-[10px] font-bold uppercase tracking-wider text-warn">{title}</h3>
          <p className="mt-0.5 text-[10px] text-mute">{hint}</p>
        </div>
        <ChevronRight
          size={14}
          className={clsx("text-mute transition", expanded && "rotate-90")}
        />
      </button>
      {expanded && (
        <div className="max-h-52 overflow-auto border-t border-warn/20">
          {versions.map((v) => (
            <VersionRow
              key={v.id}
              v={v}
              active={selectedId === v.id}
              legacy
              onSelect={() => onSelect(v)}
            />
          ))}
        </div>
      )}
    </section>
  );
}

function VersionRow({
  v,
  active,
  legacy,
  onSelect,
}: {
  v: ManifestVersion;
  active: boolean;
  legacy?: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onSelect}
      className={clsx(
        "flex w-full items-center justify-between gap-2 border-b border-line/60 px-3 py-2 text-left text-sm last:border-b-0",
        active ? "bg-accent/15 text-ink" : "hover:bg-white/4",
        legacy && !active && "text-mute",
      )}
    >
      <span className="font-medium">{v.id}</span>
      <span className="shrink-0 text-[10px] text-mute">{versionTypeLabel(v.versionType)}</span>
    </button>
  );
}
