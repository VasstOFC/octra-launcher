import { Download, X, ZoomIn } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { api } from "../lib/api";
import { assetUrl } from "../lib/assetUrl";
import { formatBytes } from "../lib/format";
import { pl } from "../locales/pl";
import { ScreenshotContextMenu } from "../components/ScreenshotContextMenu";
import { SectionHeader } from "../components/ui/SectionHeader";
import { useApp } from "../stores/appStore";
import type { GlobalScreenshotEntry } from "../types";

export function GalleryPage() {
  const instances = useApp((s) => s.instances);
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);
  const [items, setItems] = useState<GlobalScreenshotEntry[]>([]);
  const [src, setSrc] = useState<Record<string, string>>({});
  const [profileFilter, setProfileFilter] = useState<string>("all");
  const [q, setQ] = useState("");
  const [lightbox, setLightbox] = useState<GlobalScreenshotEntry | null>(null);
  const [ctxMenu, setCtxMenu] = useState<{
    x: number;
    y: number;
    item: GlobalScreenshotEntry;
  } | null>(null);

  useEffect(() => {
    api
      .listAllScreenshots()
      .then(async (list) => {
        setItems(list);
        const slice = list.slice(0, 32);
        const entries = await Promise.all(
          slice.map(async (it) => {
            const key = `${it.instanceId}/${it.name}`;
            try {
              const path = await api.readScreenshot(it.instanceId, it.name, false);
              return [key, assetUrl(path)] as const;
            } catch {
              return null;
            }
          }),
        );
        const next: Record<string, string> = {};
        for (const e of entries) {
          if (e && e[1]) next[e[0]] = e[1];
        }
        setSrc(next);
      })
      .catch((e) => showError(String(e)));
  }, [showError]);

  const filtered = useMemo(() => {
    return items
      .filter((i) => profileFilter === "all" || i.instanceId === profileFilter)
      .filter((i) => i.name.toLowerCase().includes(q.toLowerCase()))
      .sort((a, b) => (b.modified ?? "").localeCompare(a.modified ?? ""));
  }, [items, profileFilter, q]);

  async function ensureFullSrc(item: GlobalScreenshotEntry) {
    const key = `${item.instanceId}/${item.name}`;
    if (src[key]) return src[key]!;
    const path = await api.readScreenshot(item.instanceId, item.name, true);
    const url = assetUrl(path);
    if (!url) throw new Error("Nie udało się wczytać zrzutu.");
    setSrc((s) => ({ ...s, [key]: url }));
    return url;
  }

  async function openLightbox(item: GlobalScreenshotEntry) {
    try {
      await ensureFullSrc(item);
      setLightbox(item);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    }
  }

  async function downloadShot(item: GlobalScreenshotEntry) {
    try {
      const saved = await api.saveScreenshotAs(item.instanceId, item.name);
      if (saved) showOk(pl.gallery.downloaded);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    }
  }

  function openFolder(item: GlobalScreenshotEntry) {
    void api.openInstanceSubdir(item.instanceId, "screenshots").catch((e) =>
      showError(e instanceof Error ? e.message : String(e)),
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-auto p-5">
      <SectionHeader
        title={pl.nav.gallery}
        action={
          <select
            value={profileFilter}
            onChange={(e) => setProfileFilter(e.target.value)}
            className="rounded-lg bg-raised px-3 py-1.5 text-xs ring-1 ring-line"
          >
            <option value="all">{pl.gallery.allProfiles}</option>
            {instances.map((i) => (
              <option key={i.id} value={i.id}>
                {i.name}
              </option>
            ))}
          </select>
        }
      />
      <input
        value={q}
        onChange={(e) => setQ(e.target.value)}
        placeholder={pl.gallery.search}
        className="mt-3 w-full max-w-md rounded-lg bg-raised px-3 py-2 text-sm ring-1 ring-line"
      />
      {filtered.length === 0 ? (
        <div className="mt-24 text-center text-sm text-mute">
          <p>{pl.gallery.empty}</p>
        </div>
      ) : (
        <div className="mt-4 grid grid-cols-2 gap-3 md:grid-cols-3 xl:grid-cols-4">
          {filtered.map((i) => {
            const key = `${i.instanceId}/${i.name}`;
            return (
              <figure
                key={key}
                className="group relative cursor-pointer overflow-hidden rounded-lg ring-1 ring-line hover:ring-accent/40"
                onClick={() => void openLightbox(i)}
                onContextMenu={(e) => {
                  e.preventDefault();
                  setCtxMenu({ x: e.clientX, y: e.clientY, item: i });
                }}
              >
                {src[key] ? (
                  <img src={src[key]} alt="" className="aspect-video w-full object-cover" loading="lazy" />
                ) : (
                  <div className="aspect-video bg-raised2" />
                )}
                <div className="absolute inset-0 flex items-center justify-center gap-0 bg-black/0 opacity-0 transition group-hover:bg-black/25 group-hover:opacity-100">
                  <button
                    type="button"
                    title={pl.gallery.preview}
                    onClick={(e) => {
                      e.stopPropagation();
                      void openLightbox(i);
                    }}
                    className="grid h-10 w-10 place-items-center border-r border-white/15 bg-black/45 text-white hover:bg-black/65"
                  >
                    <ZoomIn size={18} />
                  </button>
                  <button
                    type="button"
                    title={pl.gallery.download}
                    onClick={(e) => {
                      e.stopPropagation();
                      void downloadShot(i);
                    }}
                    className="grid h-10 w-10 place-items-center bg-black/45 text-white hover:bg-black/65"
                  >
                    <Download size={18} />
                  </button>
                </div>
                <figcaption className="px-2 py-1.5 text-[10px] text-mute">
                  <span className="font-semibold text-ink">{i.instanceName}</span>
                  <br />
                  {i.name} · {formatBytes(i.size)}
                </figcaption>
              </figure>
            );
          })}
        </div>
      )}

      {lightbox && (
        <div
          className="fixed inset-0 z-50 grid place-items-center bg-black/85 p-8"
          onClick={() => setLightbox(null)}
        >
          <button
            className="absolute right-6 top-20 grid h-10 w-10 place-items-center rounded-full bg-white/10"
            onClick={() => setLightbox(null)}
          >
            <X size={18} />
          </button>
          <img
            src={src[`${lightbox.instanceId}/${lightbox.name}`]}
            alt=""
            className="max-h-[80vh] max-w-[90vw] rounded-lg object-contain"
            onClick={(e) => e.stopPropagation()}
          />
          <div
            className="absolute bottom-8 flex flex-wrap items-center justify-center gap-0 overflow-hidden rounded-xl border border-line bg-raised shadow-xl"
            onClick={(e) => e.stopPropagation()}
          >
            <button
              type="button"
              onClick={() => void downloadShot(lightbox)}
              className="inline-flex min-w-[7rem] items-center justify-center gap-2 border-r border-line px-4 py-2.5 text-sm font-semibold text-ink hover:bg-white/6"
            >
              <Download size={16} />
              {pl.gallery.download}
            </button>
            <button
              type="button"
              onClick={() => openFolder(lightbox)}
              className="inline-flex min-w-[7rem] items-center justify-center gap-2 px-4 py-2.5 text-sm font-semibold text-ink hover:bg-white/6"
            >
              {pl.gallery.openFolder}
            </button>
          </div>
          <p className="absolute bottom-24 text-sm text-mute">
            {lightbox.instanceName} · {lightbox.name}
          </p>
        </div>
      )}

      {ctxMenu && (
        <ScreenshotContextMenu
          x={ctxMenu.x}
          y={ctxMenu.y}
          item={ctxMenu.item}
          onClose={() => setCtxMenu(null)}
          onPreview={() => {
            const item = ctxMenu.item;
            setCtxMenu(null);
            void openLightbox(item);
          }}
          onDownload={() => {
            const item = ctxMenu.item;
            setCtxMenu(null);
            void downloadShot(item);
          }}
          onOpenFolder={() => {
            const item = ctxMenu.item;
            setCtxMenu(null);
            openFolder(item);
          }}
        />
      )}
    </div>
  );
}
