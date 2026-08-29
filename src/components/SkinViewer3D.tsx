import { useEffect, useRef, useState } from "react";
import { clsx } from "clsx";
import { MoveHorizontal } from "lucide-react";
import { IdleAnimation, SkinViewer } from "skinview3d";
import { api } from "../lib/api";
import { normalizeTextureUrl } from "../lib/skinRender";

const skinDataCache = new Map<string, string>();

function toDataUrl(b64: string): string {
  return b64.startsWith("data:") ? b64 : `data:image/png;base64,${b64}`;
}

function textureKeyFromUrl(url: string): string | null {
  if (!url.includes("textures.minecraft.net")) return null;
  const parts = url.split("/");
  return parts[parts.length - 1] || null;
}

async function remoteUrlToDataUrl(url: string): Promise<string> {
  const normalized = normalizeTextureUrl(url);
  const cached = skinDataCache.get(`url:${normalized}`);
  if (cached) return cached;

  const key = textureKeyFromUrl(normalized);
  if (key) {
    const b64 = await api.getMojangTexturePreview(key);
    const dataUrl = toDataUrl(b64);
    skinDataCache.set(`url:${normalized}`, dataUrl);
    skinDataCache.set(key, dataUrl);
    return dataUrl;
  }

  const b64 = await api.fetchImageBase64(normalized);
  const dataUrl = toDataUrl(b64);
  skinDataCache.set(`url:${normalized}`, dataUrl);
  return dataUrl;
}

async function resolveSkinSource(
  skinPngDataUrl: string | null,
  skinUrl: string | null,
  skinTextureKey: string | null,
): Promise<string | null> {
  if (skinPngDataUrl) {
    return skinPngDataUrl.startsWith("data:")
      ? skinPngDataUrl
      : toDataUrl(skinPngDataUrl);
  }
  if (skinTextureKey) {
    const cached = skinDataCache.get(skinTextureKey);
    if (cached) return cached;
    const b64 = await api.getMojangTexturePreview(skinTextureKey);
    const dataUrl = toDataUrl(b64);
    skinDataCache.set(skinTextureKey, dataUrl);
    return dataUrl;
  }
  if (skinUrl) {
    if (skinUrl.startsWith("data:") || skinUrl.startsWith("blob:")) {
      return skinUrl;
    }
    return remoteUrlToDataUrl(skinUrl);
  }
  return null;
}

async function resolveCapeSource(capeUrl: string | null): Promise<string | null> {
  if (!capeUrl) return null;
  if (capeUrl.startsWith("data:") || capeUrl.startsWith("blob:")) return capeUrl;
  const normalized = normalizeTextureUrl(capeUrl);
  const cacheKey = `cape:${normalized}`;
  const cached = skinDataCache.get(cacheKey);
  if (cached) return cached;
  const key = textureKeyFromUrl(normalized);
  if (key) {
    try {
      const b64 = await api.getMojangTexturePreview(key);
      const dataUrl = toDataUrl(b64);
      skinDataCache.set(cacheKey, dataUrl);
      return dataUrl;
    } catch {
      /* fallback fetch */
    }
  }
  const dataUrl = await remoteUrlToDataUrl(normalized);
  skinDataCache.set(cacheKey, dataUrl);
  return dataUrl;
}

export function SkinViewer3D({
  skinPngDataUrl,
  skinUrl,
  skinTextureKey,
  capeUrl,
  model = "classic",
  className,
  large,
}: {
  skinPngDataUrl?: string | null;
  skinUrl?: string | null;
  skinTextureKey?: string | null;
  capeUrl?: string | null;
  model?: "slim" | "classic";
  className?: string;
  large?: boolean;
}) {
  const hostRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const viewerRef = useRef<SkinViewer | null>(null);
  const [viewerReady, setViewerReady] = useState(false);
  const [skinLoaded, setSkinLoaded] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    const host = hostRef.current;
    const canvas = canvasRef.current;
    if (!host || !canvas) return;

    let viewer: SkinViewer | null = null;
    let disposed = false;

    const ensureViewer = () => {
      if (disposed || viewer) return;
      const w = host.clientWidth;
      const h = host.clientHeight;
      if (w < 32 || h < 32) return;

      viewer = new SkinViewer({
        canvas,
        width: w,
        height: h,
        background: 0x18181c,
        preserveDrawingBuffer: true,
      });
      viewer.fov = 42;
      viewer.zoom = 0.92;
      viewer.controls.enableRotate = true;
      viewer.controls.enableZoom = false;
      viewer.controls.enablePan = false;
      viewer.animation = new IdleAnimation();
      viewer.autoRotate = false;
      viewerRef.current = viewer;
      setViewerReady(true);
    };

    const onResize = () => {
      ensureViewer();
      const v = viewerRef.current;
      if (!v) return;
      const w = host.clientWidth;
      const h = host.clientHeight;
      if (w > 0 && h > 0) v.setSize(w, h);
    };

    const ro = new ResizeObserver(onResize);
    ro.observe(host);
    onResize();

    return () => {
      disposed = true;
      ro.disconnect();
      viewer?.dispose();
      viewerRef.current = null;
      setViewerReady(false);
      setSkinLoaded(false);
    };
  }, []);

  useEffect(() => {
    if (!viewerReady) return;
    const viewer = viewerRef.current;
    if (!viewer) return;

    let cancelled = false;
    setSkinLoaded(false);
    setLoadError(null);

    (async () => {
      try {
        const src = await resolveSkinSource(
          skinPngDataUrl ?? null,
          skinUrl ?? null,
          skinTextureKey ?? null,
        );
        if (cancelled) return;
        if (src) {
          const skinModel = model === "slim" ? "slim" : "default";
          await viewer.loadSkin(src, { model: skinModel });
          if (cancelled) return;
          viewer.adjustCameraDistance();
          viewer.resetCameraPose();
          viewer.render();
          setSkinLoaded(true);
        } else {
          viewer.resetSkin();
        }
      } catch (e) {
        if (!cancelled) {
          viewer.resetSkin();
          setSkinLoaded(false);
          setLoadError(e instanceof Error ? e.message : String(e));
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [viewerReady, skinPngDataUrl, skinUrl, skinTextureKey, model]);

  useEffect(() => {
    if (!viewerReady) return;
    const viewer = viewerRef.current;
    if (!viewer) return;

    let cancelled = false;

    (async () => {
      try {
        const src = await resolveCapeSource(capeUrl ?? null);
        if (cancelled) return;
        if (src) {
          await viewer.loadCape(src);
        } else {
          viewer.loadCape(null);
        }
        if (!cancelled) viewer.render();
      } catch {
        if (!cancelled) viewer.loadCape(null);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [viewerReady, capeUrl]);

  const hasSkinRequest = Boolean(skinPngDataUrl || skinTextureKey || skinUrl);

  return (
    <div className={clsx("flex min-h-0 flex-col items-center", className)}>
      <div
        ref={hostRef}
        className={clsx(
          "relative w-full min-h-[11rem] overflow-hidden rounded-xl border border-line/60 bg-[#141418]",
          large ? "h-[min(52vh,420px)] flex-1" : "h-44",
        )}
      >
        <canvas ref={canvasRef} className="block h-full w-full" />
        {hasSkinRequest && !skinLoaded && !loadError && (
          <div className="pointer-events-none absolute inset-0 grid place-items-center text-xs text-mute">
            Ładowanie skina…
          </div>
        )}
        {!hasSkinRequest && (
          <div className="pointer-events-none absolute inset-0 grid place-items-center text-xs text-mute">
            Wybierz skin
          </div>
        )}
        {loadError ? (
          <div className="pointer-events-none absolute inset-x-3 bottom-3 rounded-lg bg-danger/15 px-2 py-1.5 text-center text-[10px] text-danger">
            {loadError}
          </div>
        ) : null}
      </div>
      {large && (
        <p className="mt-2 flex items-center gap-1 text-[10px] text-mute">
          <MoveHorizontal size={12} />
          Przeciągnij, żeby obrócić
        </p>
      )}
    </div>
  );
}
