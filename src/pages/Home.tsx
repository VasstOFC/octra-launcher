import { ExternalLink, ImageIcon, Plus, X } from "lucide-react";
import { useEffect, useMemo } from "react";
import { api } from "../lib/api";
import { formatPlayTime, LOADER_LABEL, loaderChipClass } from "../lib/format";
import { useMojangNews } from "../lib/mojangNews";
import { useProfileWallpaper } from "../lib/useProfileWallpaper";
import { useApplyProfileVisualUpdate } from "../lib/profileVisual";
import { pl } from "../locales/pl";
import { Button } from "../components/ui/Button";
import { SmartSuggestionsStrip } from "../components/SmartSuggestionsStrip";
import { ProfileBackdrop } from "../components/ProfileBackdrop";
import { ProfileCard } from "../components/ProfileCard";
import { ProfileIcon } from "../components/ProfileIcon";
import { useApp, useActiveAccount } from "../stores/appStore";
import { useOctra } from "../stores/octraStore";
import type { Instance } from "../types";
import type { MojangNewsItem } from "../types";

export function HomePage() {
  const instances = useApp((s) => s.instances);
  const progress = useApp((s) => s.progress);
  const launching = useApp((s) => s.launchingId);
  const isPlaying = useApp((s) => s.isPlaying);
  const playInstance = useApp((s) => s.playInstance);
  const stopInstance = useApp((s) => s.stopInstance);
  const selectedId = useOctra((s) => s.selectedId);
  const select = useOctra((s) => s.selectInstance);
  const setView = useOctra((s) => s.setView);
  const openCreator = useOctra((s) => s.openInstanceCreator);
  const acc = useActiveAccount();
  const online = typeof navigator !== "undefined" ? navigator.onLine : true;
  const { items: news, loading: newsLoading, offline: newsOffline } = useMojangNews();

  const sorted = useMemo(
    () =>
      instances
        .slice()
        .sort((a, b) => (b.lastPlayed ?? "").localeCompare(a.lastPlayed ?? "")),
    [instances],
  );

  useEffect(() => {
    if (!sorted.length) return;
    if (!selectedId || !instances.some((i) => i.id === selectedId)) {
      select(sorted[0]!.id);
    }
  }, [sorted, selectedId, instances, select]);

  const selected =
    instances.find((i) => i.id === selectedId) ?? sorted[0] ?? null;
  const playSecs = instances.reduce((n, i) => n + (i.playTimeSecs || 0), 0);
  const last = sorted[0];

  const busy = Boolean(progress) || launching === selected?.id;
  const playing = selected && acc ? isPlaying(selected.id, acc.uuid) : false;

  async function launch() {
    if (!selected) {
      setView("versions");
      return;
    }
    if (playing) await stopInstance(selected.id);
    else await playInstance(selected.id);
  }

  return (
    <div className="home-surface flex min-h-0 flex-1 flex-col overflow-auto p-5">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-lg font-bold text-ink">{pl.nav.home}</h1>
          <p className="mt-0.5 text-[13px] text-mute">
            {pl.home.greeting},{" "}
            <span className="font-semibold text-ink">{acc?.name ?? pl.home.guest}</span>
          </p>
        </div>
        <div className="flex flex-wrap gap-2 text-[11px]">
          <StatChip label={pl.home.lastPlayed} value={last?.name ?? "—"} />
          <StatChip label={pl.home.totalPlaytime} value={formatPlayTime(playSecs)} />
        </div>
      </header>

      {!online && (
        <div className="mt-3 rounded-lg bg-warn/15 px-3 py-2 text-sm text-warn">
          {pl.home.offline}
        </div>
      )}

      <NewsStrip
        items={news}
        loading={newsLoading}
        offline={newsOffline}
      />

      <SmartSuggestionsStrip />

      <div className="mt-4 overflow-hidden rounded-2xl border border-line bg-raised shadow-[inset_0_1px_0_rgb(255_255_255/0.04)]">
        <div className="grid min-h-[240px] grid-cols-1 lg:grid-cols-5">
          <div className="relative lg:col-span-3">
            {selected ? (
              <ProfileHeroCard inst={selected} />
            ) : (
              <button
                onClick={openCreator}
                className="flex h-full min-h-[240px] w-full items-center justify-center border-b border-line bg-raised2/40 text-sm text-mute lg:border-b-0 lg:border-r"
              >
                {pl.home.noProfile}
              </button>
            )}
          </div>

          <div className="flex flex-col items-center justify-center gap-3 border-line bg-raised2/30 p-6 lg:col-span-2 lg:border-l">
            <div className="relative w-full max-w-[200px]">
              <Button
                variant={playing ? "danger" : busy ? "secondary" : "launch"}
                onClick={() => void launch()}
                disabled={!acc || busy}
                className="h-[88px] w-full flex-col gap-1 text-lg font-extrabold tracking-wide"
              >
                {busy
                  ? pl.home.downloading
                  : playing
                    ? pl.home.stop
                    : pl.home.launch}
                {selected && (
                  <span className="text-[11px] font-medium opacity-85">
                    {LOADER_LABEL[selected.loader] ?? selected.loader} ·{" "}
                    {selected.gameVersion}
                  </span>
                )}
              </Button>
              {busy && (
                <button
                  className="absolute right-2 top-1/2 grid h-7 w-7 -translate-y-1/2 place-items-center rounded-md bg-black/30 text-white"
                  title={pl.home.cancelDownload}
                  onClick={() => void api.cancelInstall()}
                >
                  <X size={14} />
                </button>
              )}
            </div>

            {!acc ? (
              <p className="text-center text-xs text-mute">{pl.home.addAccountHint}</p>
            ) : null}

            {busy && progress && (
              <div className="w-full max-w-[200px]">
                <div className="h-1.5 overflow-hidden rounded-full bg-white/10">
                  <div
                    className="h-full bg-good transition-[width] duration-150"
                    style={{
                      width: `${progress.total > 0 ? Math.min(100, (100 * progress.current) / progress.total) : 12}%`,
                    }}
                  />
                </div>
                <p className="mt-1.5 truncate text-center text-[10px] text-mute">
                  {progress.message}
                </p>
              </div>
            )}

            <button
              className="text-xs font-medium text-mute hover:text-accent"
              onClick={() => setView("versions")}
            >
              {pl.home.changeVersion} ⌄
            </button>
          </div>
        </div>
      </div>

      <section className="mt-5">
        <div className="flex items-center justify-between gap-3">
          <div>
            <h2 className="text-[11px] font-semibold uppercase tracking-wider text-mute">
              {pl.home.allProfiles}
            </h2>
            <p className="mt-0.5 text-xs text-mute">
              {sorted.length}{" "}
              {sorted.length === 1 ? "profil" : sorted.length < 5 ? "profile" : "profilów"}
            </p>
          </div>
          <button
            type="button"
            onClick={openCreator}
            className="inline-flex items-center gap-1.5 rounded-lg border border-line bg-raised2 px-3 py-1.5 text-xs font-semibold text-mute hover:border-accent/40 hover:text-ink"
          >
            <Plus size={14} />
            {pl.home.newProfile}
          </button>
        </div>

        {sorted.length === 0 ? (
          <button
            onClick={openCreator}
            className="mt-3 flex min-h-[140px] w-full items-center justify-center rounded-2xl border border-dashed border-line bg-raised2/40 text-sm text-mute hover:border-accent/30 hover:text-ink"
          >
            {pl.home.createProfile}
          </button>
        ) : (
          <div className="mt-3 grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
            {sorted.map((inst) => (
              <ProfileCard
                key={inst.id}
                inst={inst}
                active={selected?.id === inst.id}
                onClick={() => select(inst.id)}
                variant={sorted.length > 8 ? "compact" : "grid"}
                showPlaytime
              />
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

function StatChip({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-line bg-raised/80 px-3 py-1.5">
      <span className="text-mute">{label}: </span>
      <span className="font-semibold text-ink">{value}</span>
    </div>
  );
}

function NewsStrip({
  items,
  loading,
  offline,
}: {
  items: MojangNewsItem[];
  loading: boolean;
  offline: boolean;
}) {
  return (
    <section className="news-strip mt-3 overflow-hidden rounded-xl border border-accent/20 bg-gradient-to-r from-accent/10 via-raised/90 to-raised/70">
      <div className="flex items-stretch gap-0">
        <div className="flex shrink-0 items-center border-r border-accent/15 bg-accent/10 px-3 py-2">
          <span className="text-[10px] font-bold uppercase tracking-wider text-accent">
            {pl.home.newsTitle}
          </span>
        </div>
        <div className="news-scroll flex min-w-0 flex-1 items-center gap-2 overflow-x-auto px-3 py-2">
          {loading && !items.length ? (
            <span className="text-[11px] text-mute">{pl.home.newsLoading}</span>
          ) : offline && !items.length ? (
            <span className="text-[11px] text-warn">{pl.home.newsOffline}</span>
          ) : (
            items.slice(0, 6).map((item) => (
              <a
                key={item.id}
                href={item.link}
                target="_blank"
                rel="noreferrer"
                className="news-pill inline-flex max-w-[260px] shrink-0 items-center gap-1.5 rounded-full border border-line/80 bg-raised2/90 px-3 py-1.5 text-[11px] font-medium text-ink transition hover:border-accent/45 hover:bg-raised2"
                title={item.title}
              >
                <span className="truncate">{item.title}</span>
                <ExternalLink size={11} className="shrink-0 text-accent/80" />
              </a>
            ))
          )}
        </div>
      </div>
    </section>
  );
}

function ProfileHeroCard({ inst }: { inst: Instance }) {
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);
  const applyVisual = useApplyProfileVisualUpdate();
  const wallpaper = useProfileWallpaper(inst);
  const hasWallpaper = Boolean(wallpaper);

  async function pickWallpaper() {
    try {
      const updated = await api.pickProfileWallpaper(inst.id);
      if (!updated) return;
      applyVisual(updated);
      showOk("Tapeta profilu została zaktualizowana.");
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <article className="relative h-full min-h-[240px]">
      {hasWallpaper ? (
        <img
          src={wallpaper!}
          alt=""
          className="absolute inset-0 h-full w-full object-cover"
          loading="lazy"
        />
      ) : (
        <ProfileBackdrop inst={inst} size="hero" />
      )}
      <div className="absolute inset-0 bg-gradient-to-r from-black/80 via-black/50 to-black/25" />
      <div className="relative z-10 flex h-full min-h-[240px] flex-col justify-end p-5 lg:border-r lg:border-line/50">
        <div className="flex items-end justify-between gap-4">
          <div className="flex min-w-0 items-end gap-4">
            <div className="grid h-[72px] w-[72px] shrink-0 place-items-center overflow-hidden rounded-2xl border border-white/20 bg-black/35 p-1.5 shadow-xl">
              <ProfileIcon inst={inst} size={60} />
            </div>
            <div className="min-w-0 pb-0.5">
              <p className="text-[10px] font-semibold uppercase tracking-widest text-white/50">
                Aktywny profil
              </p>
              <h2 className="mt-1 truncate text-2xl font-extrabold text-white">{inst.name}</h2>
              <span
                className={`mt-2 inline-block rounded-full px-2.5 py-0.5 text-[10px] font-semibold ${loaderChipClass(inst.loader)}`}
              >
                {LOADER_LABEL[inst.loader] ?? inst.loader} {inst.gameVersion}
              </span>
              <p className="mt-2 text-sm text-white/75">
                {pl.home.totalPlaytime}:{" "}
                <span className="font-semibold text-white">
                  {formatPlayTime(inst.playTimeSecs || 0)}
                </span>
              </p>
              {inst.lastPlayed && (
                <p className="mt-0.5 text-[11px] text-white/50">
                  {pl.home.lastPlayed}: {new Date(inst.lastPlayed).toLocaleString("pl-PL")}
                </p>
              )}
            </div>
          </div>
          <button
            type="button"
            title={pl.home.setWallpaper}
            onClick={() => void pickWallpaper()}
            className="grid h-9 w-9 shrink-0 place-items-center rounded-lg border border-white/10 bg-black/35 text-white/70 hover:text-white"
          >
            <ImageIcon size={15} />
          </button>
        </div>
      </div>
    </article>
  );
}
