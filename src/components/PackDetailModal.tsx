import { Download, Loader2, X } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { api } from "../lib/api";
import { formatCount } from "../lib/format";
import { pl } from "../locales/pl";
import { useApp } from "../stores/appStore";
import type { ModrinthPackHit, ModrinthPackVersionHit, ModrinthProjectDetail } from "../types";

type Props = {
  pack: ModrinthPackHit | null;
  onClose: () => void;
  onInstalled?: (instanceId: string) => void;
};

function stripMarkdown(md: string): string {
  return md
    .replace(/```[\s\S]*?```/g, "")
    .replace(/`([^`]+)`/g, "$1")
    .replace(/!\[[^\]]*\]\([^)]+\)/g, "")
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .replace(/^#{1,6}\s+/gm, "")
    .replace(/[*_~]/g, "")
    .trim();
}

export function PackDetailModal({ pack, onClose, onInstalled }: Props) {
  const [detail, setDetail] = useState<ModrinthProjectDetail | null>(null);
  const [versions, setVersions] = useState<ModrinthPackVersionHit[]>([]);
  const [selectedVersionId, setSelectedVersionId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [installing, setInstalling] = useState(false);
  const progress = useApp((s) => s.progress);
  const refresh = useApp((s) => s.refreshInstances);
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);

  const load = useCallback(async () => {
    if (!pack) return;
    setLoading(true);
    try {
      const [proj, vers] = await Promise.all([
        api.getModrinthProject(pack.slug),
        api.getModrinthPackVersions(pack.slug),
      ]);
      setDetail(proj);
      setVersions(vers);
      setSelectedVersionId(vers[0]?.id ?? null);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
      onClose();
    } finally {
      setLoading(false);
    }
  }, [pack, onClose, showError]);

  useEffect(() => {
    if (!pack) {
      setDetail(null);
      setVersions([]);
      setSelectedVersionId(null);
      return;
    }
    void load();
  }, [pack, load]);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [onClose]);

  if (!pack) return null;

  const selectedVersion = versions.find((v) => v.id === selectedVersionId) ?? versions[0];
  const busy = Boolean(progress) || installing;

  async function install() {
    if (!pack || !selectedVersion) return;
    setInstalling(true);
    try {
      const query =
        selectedVersion.id === versions[0]?.id
          ? pack.slug
          : `${pack.slug}/${selectedVersion.id}`;
      const inst = await api.importModrinthPack(query, pack.iconUrl);
      await refresh();
      showOk(`Zainstalowano „${pack.title}".`);
      onInstalled?.(inst.id);
      onClose();
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setInstalling(false);
    }
  }

  return (
    <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/70 p-4">
      <div className="flex max-h-[90vh] w-full max-w-3xl flex-col overflow-hidden rounded-2xl border border-line bg-raised shadow-2xl">
        <header className="flex items-start gap-4 border-b border-line p-5">
          <div className="h-16 w-16 shrink-0 overflow-hidden rounded-xl bg-raised2">
            {pack.iconUrl ? (
              <img src={pack.iconUrl} alt="" className="h-full w-full object-cover" />
            ) : (
              <div className="grid h-full place-items-center text-2xl font-bold text-mute">
                {pack.title.slice(0, 1)}
              </div>
            )}
          </div>
          <div className="min-w-0 flex-1">
            <h2 className="text-lg font-bold text-ink">{detail?.title ?? pack.title}</h2>
            <p className="mt-1 line-clamp-2 text-sm text-mute">
              {detail?.description ?? pack.description}
            </p>
            <div className="mt-2 flex flex-wrap gap-2 text-[11px] text-mute">
              <span>{formatCount(detail?.downloads ?? pack.downloads)} pobrań</span>
              <span>·</span>
              <span>{formatCount(detail?.follows ?? pack.follows)} obserwujących</span>
            </div>
            <div className="mt-2 flex flex-wrap gap-1">
              {(detail?.loaders ?? pack.loaders).slice(0, 4).map((l) => (
                <span
                  key={l}
                  className="rounded-full bg-white/8 px-2 py-0.5 text-[10px] capitalize"
                >
                  {l}
                </span>
              ))}
            </div>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="grid h-8 w-8 shrink-0 place-items-center rounded-lg text-mute hover:bg-raised2 hover:text-ink"
            aria-label={pl.common.close}
          >
            <X size={16} />
          </button>
        </header>

        <div className="min-h-0 flex-1 overflow-auto p-5">
          {loading ? (
            <div className="flex items-center justify-center gap-2 py-16 text-sm text-mute">
              <Loader2 size={18} className="animate-spin" />
              {pl.store.loadingDetail}
            </div>
          ) : (
            <>
              {detail?.gallery && detail.gallery.length > 0 ? (
                <div className="mb-4 flex gap-2 overflow-x-auto pb-1">
                  {detail.gallery.map((url) => (
                    <img
                      key={url}
                      src={url}
                      alt=""
                      className="h-28 w-auto shrink-0 rounded-xl object-cover ring-1 ring-line"
                    />
                  ))}
                </div>
              ) : null}

              {detail?.body ? (
                <div className="mb-4 rounded-xl border border-line bg-raised2/50 p-4">
                  <p className="whitespace-pre-wrap text-sm text-ink/90">
                    {stripMarkdown(detail.body).slice(0, 2000)}
                    {detail.body.length > 2000 ? "…" : ""}
                  </p>
                </div>
              ) : null}

              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <h3 className="text-xs font-bold uppercase tracking-wider text-mute">
                    {pl.store.versions}
                  </h3>
                  <ul className="mt-2 max-h-48 space-y-1 overflow-auto rounded-xl border border-line bg-raised2/30 p-2">
                    {versions.map((v) => (
                      <li key={v.id}>
                        <button
                          type="button"
                          onClick={() => setSelectedVersionId(v.id)}
                          className={`w-full rounded-lg px-3 py-2 text-left text-sm transition ${
                            selectedVersion?.id === v.id
                              ? "bg-accent/15 text-ink ring-1 ring-accent/30"
                              : "text-mute hover:bg-white/5 hover:text-ink"
                          }`}
                        >
                          <span className="font-semibold">{v.versionNumber}</span>
                          {v.name ? (
                            <span className="ml-2 text-xs text-mute">{v.name}</span>
                          ) : null}
                        </button>
                      </li>
                    ))}
                    {versions.length === 0 && (
                      <li className="px-3 py-4 text-center text-xs text-mute">
                        {pl.store.noVersions}
                      </li>
                    )}
                  </ul>
                </div>
                <div>
                  <h3 className="text-xs font-bold uppercase tracking-wider text-mute">
                    {pl.store.changelog}
                  </h3>
                  <div className="mt-2 max-h-48 overflow-auto rounded-xl border border-line bg-raised2/30 p-3 text-sm text-mute">
                    {selectedVersion?.changelog ? (
                      <p className="whitespace-pre-wrap">
                        {stripMarkdown(selectedVersion.changelog)}
                      </p>
                    ) : (
                      <p className="text-xs italic">{pl.store.noChangelog}</p>
                    )}
                  </div>
                </div>
              </div>
            </>
          )}
        </div>

        <footer className="flex justify-end gap-2 border-t border-line p-4">
          <button
            type="button"
            onClick={onClose}
            className="rounded-full border border-line px-4 py-2 text-sm text-mute hover:text-ink"
          >
            {pl.common.cancel}
          </button>
          <button
            type="button"
            disabled={busy || !selectedVersion || loading}
            onClick={() => void install()}
            className="inline-flex items-center gap-2 rounded-full bg-accent px-4 py-2 text-sm font-semibold text-bg-on-accent disabled:opacity-50"
          >
            {installing ? (
              <Loader2 size={14} className="animate-spin" />
            ) : (
              <Download size={14} />
            )}
            {pl.store.install}
          </button>
        </footer>
      </div>
    </div>
  );
}
