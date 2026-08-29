import { ImageIcon, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { api } from "../lib/api";
import { assetUrl } from "../lib/assetUrl";
import { formatPlayTime, LOADER_LABEL, loaderChipClass } from "../lib/format";
import { useMojangNews } from "../lib/mojangNews";
import { pl } from "../locales/pl";
import { Button } from "../components/ui/Button";
import { ProfileBackdrop } from "../components/ProfileBackdrop";
import { useApp, useActiveAccount } from "../stores/appStore";
import { useOctra } from "../stores/octraStore";
import type { Instance } from "../types";

export function HomePage() {
  const instances = useApp((s) => s.instances);
  const progress = useApp((s) => s.progress);
  const launching = useApp((s) => s.launchingId);
  const isPlaying = useApp((s) => s.isPlaying);
  const playInstance = useApp((s) => s.playInstance);
  const stopInstance = useApp((s) => s.stopInstance);
  const refreshInstances = useApp((s) => s.refreshInstances);
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
  const recent = sorted.filter((i) => i.id !== selected?.id).slice(0, 2);
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
      {/* Nagłówek — wyśrodkowany względem szerokości treści */}
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
          <StatChip
            label={pl.home.totalPlaytime}
            value={formatPlayTime(playSecs)}
          />
        </div>
      </header>

      {!online && (
        <div className="mt-3 rounded-lg bg-warn/15 px-3 py-2 text-sm text-warn">
          {pl.home.offline}
        </div>
      )}

      {/* Główny panel — profil + GRAJ w jednej ramce */}
      <div className="mt-4 overflow-hidden rounded-2xl border border-line bg-raised shadow-[inset_0_1px_0_rgb(255_255_255/0.04)]">
        <div className="grid min-h-[220px] grid-cols-1 lg:grid-cols-5">
          <div className="relative lg:col-span-3">
            {selected ? (
              <ProfileHeroCard
                inst={selected}
                onRefresh={() => void refreshInstances()}
              />
            ) : (
              <button
                onClick={openCreator}
                className="flex h-full min-h-[220px] w-full items-center justify-center border-b border-line bg-raised2/40 text-sm text-mute lg:border-b-0 lg:border-r"
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

      {/* Dolna połowa — symetryczna siatka 50/50 */}
      <div className="mt-4 grid min-h-0 flex-1 grid-cols-1 gap-4 lg:grid-cols-2">
        <section className="flex flex-col rounded-2xl border border-line bg-raised/80 p-4">
          <h2 className="text-[11px] font-semibold uppercase tracking-wider text-mute">
            {pl.home.recentProfiles}
          </h2>
          <div className="mt-3 grid flex-1 grid-cols-1 gap-3 sm:grid-cols-2">
            {recent.length === 0 ? (
              <button
                onClick={openCreator}
                className="col-span-full flex min-h-[100px] items-center justify-center rounded-xl border border-dashed border-line bg-raised2/40 text-xs text-mute hover:border-accent/30 hover:text-ink"
              >
                {pl.home.createProfile}
              </button>
            ) : (
              recent.map((i) => (
                <RecentProfileCard
                  key={i.id}
                  inst={i}
                  active={selected?.id === i.id}
                  onClick={() => select(i.id)}
                />
              ))
            )}
            {recent.length === 1 && (
              <button
                onClick={openCreator}
                className="flex min-h-[100px] items-center justify-center rounded-xl border border-dashed border-line bg-raised2/30 text-xs text-mute hover:border-accent/30"
              >
                + {pl.home.createProfile}
              </button>
            )}
          </div>
        </section>

        <section className="flex flex-col rounded-2xl border border-line bg-raised/80 p-4">
          <h2 className="text-[11px] font-semibold uppercase tracking-wider text-mute">
            {pl.home.newsTitle}
          </h2>
          <div className="mt-3 min-h-[100px] flex-1">
            {newsLoading && !news.length ? (
              <p className="text-[13px] text-mute">{pl.home.newsLoading}</p>
            ) : newsOffline && !news.length ? (
              <div className="flex h-full min-h-[100px] flex-col items-center justify-center rounded-xl border border-line bg-raised2/40 px-4 text-center">
                <p className="text-[13px] text-warn">{pl.home.newsOffline}</p>
                <p className="mt-1 text-[11px] text-mute">
                  Aktualności pojawią się po przywróceniu połączenia.
                </p>
              </div>
            ) : (
              <ul className="grid gap-2 sm:grid-cols-1">
                {news.slice(0, 4).map((item) => (
                  <li key={item.id}>
                    <a
                      href={item.link}
                      target="_blank"
                      rel="noreferrer"
                      className="block rounded-xl border border-line bg-raised2/50 px-3 py-2.5 transition hover:border-accent/35 hover:bg-raised2"
                    >
                      <div className="flex items-start justify-between gap-2">
                        <h3 className="line-clamp-2 text-[13px] font-semibold leading-snug text-ink">
                          {item.title}
                        </h3>
                        {item.published && (
                          <span className="shrink-0 text-[10px] text-mute">
                            {new Date(item.published).toLocaleDateString("pl-PL")}
                          </span>
                        )}
                      </div>
                      {item.summary && (
                        <p className="mt-1 line-clamp-2 text-[12px] leading-relaxed text-mute">
                          {item.summary}
                        </p>
                      )}
                    </a>
                  </li>
                ))}
              </ul>
            )}
            {newsOffline && news.length > 0 && (
              <p className="mt-2 text-[10px] text-mute">{pl.home.newsOffline}</p>
            )}
          </div>
        </section>
      </div>
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

function ProfileHeroCard({
  inst,
  onRefresh,
}: {
  inst: Instance;
  onRefresh: () => void;
}) {
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);
  const [wallpaper, setWallpaper] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void api.readInstanceWallpaper(inst.id).then((path) => {
      if (!cancelled) setWallpaper(assetUrl(path));
    });
    return () => {
      cancelled = true;
    };
  }, [inst.id, inst.wallpaperPath]);

  async function pickWallpaper() {
    try {
      const next = await api.pickProfileWallpaper(inst.id);
      if (!next) return;
      onRefresh();
      const path = await api.readInstanceWallpaper(inst.id);
      setWallpaper(assetUrl(path));
      showOk("Tapeta profilu została zaktualizowana.");
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    }
  }

  const hasWallpaper = Boolean(wallpaper);

  return (
    <article className="relative h-full min-h-[220px]">
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
      <div className="absolute inset-0 bg-gradient-to-r from-black/75 via-black/55 to-black/40" />
      <div className="relative z-10 flex h-full min-h-[220px] flex-col justify-end p-5 lg:border-r lg:border-line/50">
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0">
            <p className="text-[10px] font-semibold uppercase tracking-widest text-white/50">
              Aktywny profil
            </p>
            <h2 className="mt-1 truncate text-2xl font-extrabold text-white">{inst.name}</h2>
            <span
              className={`mt-2 inline-block rounded-full px-2.5 py-0.5 text-[10px] font-semibold ${loaderChipClass(inst.loader)}`}
            >
              {LOADER_LABEL[inst.loader] ?? inst.loader} {inst.gameVersion}
            </span>
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
        <p className="mt-3 text-sm text-white/80">
          {pl.home.totalPlaytime}:{" "}
          <span className="font-semibold">{formatPlayTime(inst.playTimeSecs || 0)}</span>
        </p>
        {inst.lastPlayed && (
          <p className="mt-1 text-[11px] text-white/55">
            {pl.home.lastPlayed}: {new Date(inst.lastPlayed).toLocaleString("pl-PL")}
          </p>
        )}
        <button
          type="button"
          onClick={() => void pickWallpaper()}
          className="mt-2 w-fit text-[11px] font-medium text-accent hover:underline"
        >
          {hasWallpaper ? pl.home.changeWallpaper : pl.home.setWallpaper}
        </button>
      </div>
    </article>
  );
}

function RecentProfileCard({
  inst,
  active,
  onClick,
}: {
  inst: Instance;
  active: boolean;
  onClick: () => void;
}) {
  const [wallpaper, setWallpaper] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void api.readInstanceWallpaper(inst.id).then((path) => {
      if (!cancelled) setWallpaper(assetUrl(path));
    });
    return () => {
      cancelled = true;
    };
  }, [inst.id, inst.wallpaperPath]);

  return (
    <button
      onClick={onClick}
      className={`relative min-h-[100px] overflow-hidden rounded-xl text-left ring-1 transition ${
        active ? "ring-accent/50" : "ring-line hover:ring-accent/25"
      }`}
    >
      {wallpaper ? (
        <img
          src={wallpaper}
          alt=""
          className="absolute inset-0 h-full w-full object-cover"
          loading="lazy"
        />
      ) : (
        <ProfileBackdrop inst={inst} size="card" />
      )}
      <div className="absolute inset-0 bg-black/50" />
      <div className="relative z-10 flex h-full min-h-[100px] flex-col justify-end p-3">
        <div className="truncate text-sm font-semibold text-white">{inst.name}</div>
        <div className="mt-0.5 text-[10px] text-white/70">
          {LOADER_LABEL[inst.loader]} {inst.gameVersion}
        </div>
      </div>
    </button>
  );
}
