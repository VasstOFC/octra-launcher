import { useCallback, useEffect, useRef, useState } from "react";
import { clsx } from "clsx";
import { HitAnimation, IdleAnimation, SkinViewer, WaveAnimation } from "skinview3d";
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
    if (skinPngDataUrl.startsWith("data:") || skinPngDataUrl.startsWith("blob:")) {
      return skinPngDataUrl;
    }
    return toDataUrl(skinPngDataUrl);
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
      /* fallback */
    }
  }
  const dataUrl = await remoteUrlToDataUrl(normalized);
  skinDataCache.set(cacheKey, dataUrl);
  return dataUrl;
}

const INITIAL_YAW = 0;
const MODRINTH_MODEL_Y_OFFSET = -0.3;

export function SkinViewer3D({
  skinPngDataUrl,
  skinUrl,
  skinTextureKey,
  capeUrl,
  model = "classic",
  nametag,
  className,
  large,
  compact,
  modrinth,
}: {
  skinPngDataUrl?: string | null;
  skinUrl?: string | null;
  skinTextureKey?: string | null;
  capeUrl?: string | null;
  model?: "slim" | "classic";
  nametag?: string;
  className?: string;
  large?: boolean;
  compact?: boolean;
  modrinth?: boolean;
}) {
  const hostRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const viewerRef = useRef<SkinViewer | null>(null);
  const pointerRef = useRef({ down: false, x: 0, y: 0, moved: false });
  const idleTimerRef = useRef<number | null>(null);
  const [viewerReady, setViewerReady] = useState(false);
  const [skinLoaded, setSkinLoaded] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [visible, setVisible] = useState(false);

  const playClickBounce = useCallback(() => {
    const viewer = viewerRef.current;
    if (!viewer || compact) return;
    const hit = new HitAnimation();
    hit.speed = 1.2;
    viewer.animation = hit;
    window.setTimeout(() => {
      if (viewerRef.current === viewer) viewer.animation = new IdleAnimation();
    }, 450);
  }, [compact]);

  const scheduleIdleVariation = useCallback(() => {
    if (idleTimerRef.current !== null) window.clearTimeout(idleTimerRef.current);
    if (compact || !modrinth) return;
    idleTimerRef.current = window.setTimeout(() => {
      const viewer = viewerRef.current;
      if (!viewer) return;
      const wave = new WaveAnimation();
      wave.speed = 0.85;
      viewer.animation = wave;
      window.setTimeout(() => {
        if (viewerRef.current === viewer) {
          viewer.animation = new IdleAnimation();
          scheduleIdleVariation();
        }
      }, 2200);
    }, 7000 + Math.random() * 4000);
  }, [compact, modrinth]);

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
        background: modrinth && large ? undefined : 0x18181c,
        preserveDrawingBuffer: true,
      });
      viewer.fov = compact ? 42 : modrinth ? 38 : 40;
      viewer.zoom = compact ? 0.72 : large ? (modrinth ? 0.88 : 0.78) : 0.88;
      viewer.controls.enableRotate = !compact;
      viewer.controls.enableZoom = false;
      viewer.controls.enablePan = false;
      if (compact) {
        viewer.animation = null;
        viewer.autoRotate = false;
        viewer.playerObject.rotation.y = 0;
      } else {
        viewer.animation = new IdleAnimation();
        viewer.autoRotate = false;
        viewer.playerObject.rotation.y = INITIAL_YAW;
        if (modrinth) {
          viewer.playerWrapper.position.y = MODRINTH_MODEL_Y_OFFSET;
        }
      }
      viewerRef.current = viewer;
      setViewerReady(true);
      if (modrinth && !compact) scheduleIdleVariation();
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
      if (idleTimerRef.current !== null) {
        window.clearTimeout(idleTimerRef.current);
        idleTimerRef.current = null;
      }
      viewer?.dispose();
      viewerRef.current = null;
      setViewerReady(false);
      setSkinLoaded(false);
      setVisible(false);
    };
  }, [compact, large, modrinth, scheduleIdleVariation]);

  useEffect(() => {
    if (!viewerReady) return;
    const viewer = viewerRef.current;
    if (!viewer) return;

    let cancelled = false;
    setSkinLoaded(false);
    setLoadError(null);
    setVisible(false);

    void (async () => {
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
          if (compact) viewer.zoom = 0.72;
          else if (large) viewer.zoom = modrinth ? 0.88 : 0.78;
          if (!compact) {
            viewer.playerObject.rotation.y = INITIAL_YAW;
            if (modrinth) viewer.playerWrapper.position.y = MODRINTH_MODEL_Y_OFFSET;
          }
          viewer.resetCameraPose();
          viewer.render();
          setSkinLoaded(true);
          requestAnimationFrame(() => setVisible(true));
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
  }, [viewerReady, skinPngDataUrl, skinUrl, skinTextureKey, model, compact, large, modrinth]);

  useEffect(() => {
    if (!viewerReady) return;
    const viewer = viewerRef.current;
    if (!viewer) return;

    let cancelled = false;
    void (async () => {
      try {
        const src = await resolveCapeSource(capeUrl ?? null);
        if (cancelled) return;
        if (src) await viewer.loadCape(src);
        else viewer.loadCape(null);
        if (!cancelled) viewer.render();
      } catch {
        if (!cancelled) viewer.loadCape(null);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [viewerReady, capeUrl]);

  useEffect(() => {
    const viewer = viewerRef.current;
    if (!viewer || compact || modrinth) return;
    viewer.nameTag = nametag?.trim() ? nametag.trim() : null;
    viewer.render();
  }, [nametag, viewerReady, compact, skinLoaded, modrinth]);

  const hasSkinRequest = Boolean(skinPngDataUrl || skinTextureKey || skinUrl);

  const onPointerDown = (e: React.PointerEvent) => {
    pointerRef.current = { down: true, x: e.clientX, y: e.clientY, moved: false };
  };
  const onPointerMove = (e: React.PointerEvent) => {
    if (!pointerRef.current.down) return;
    const dx = Math.abs(e.clientX - pointerRef.current.x);
    const dy = Math.abs(e.clientY - pointerRef.current.y);
    if (dx > 4 || dy > 4) pointerRef.current.moved = true;
  };
  const onPointerUp = () => {
    if (pointerRef.current.down && !pointerRef.current.moved && modrinth && large) {
      playClickBounce();
    }
    pointerRef.current.down = false;
  };

  return (
    <div
      className={clsx(
        compact ? "h-full w-full" : "flex min-h-0 flex-col items-center",
        className,
      )}
    >
      <div
        ref={hostRef}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerLeave={onPointerUp}
        className={clsx(
          "relative w-full overflow-hidden",
          compact
            ? "h-full bg-transparent"
            : modrinth && large
              ? "h-full cursor-grab bg-transparent active:cursor-grabbing"
              : "min-h-[11rem] rounded-xl border border-line/60 bg-[#141418]",
          !compact && !modrinth && (large ? "h-[min(46vh,360px)] flex-1" : "h-44"),
        )}
      >
        {modrinth && large && (
          <div className="skin-preview-spotlight pointer-events-none absolute left-1/2 z-0 -translate-x-1/2" />
        )}
        <canvas
          ref={canvasRef}
          className={clsx(
            "relative z-[1] block h-full w-full transition-opacity duration-500",
            visible || compact ? "opacity-100" : "opacity-0",
          )}
        />
        {hasSkinRequest && !skinLoaded && !loadError && !compact && (
          <div className="pointer-events-none absolute inset-0 z-[2] grid place-items-center bg-[#141418]/40 backdrop-blur-[1px]">
            <div className="flex flex-col items-center gap-2 rounded-xl border border-line/50 bg-raised/80 px-4 py-3 shadow-lg">
              <div className="size-6 animate-spin rounded-full border-2 border-accent/25 border-t-accent" />
              <span className="text-xs font-medium text-mute">Ładowanie podglądu…</span>
            </div>
          </div>
        )}
        {!hasSkinRequest && !compact && (
          <div className="pointer-events-none absolute inset-0 z-[2] grid place-items-center text-sm text-mute">
            Wybierz skin
          </div>
        )}
        {loadError ? (
          <div className="pointer-events-none absolute inset-x-3 bottom-3 z-[2] rounded-lg bg-danger/15 px-2 py-1.5 text-center text-[10px] text-danger">
            {loadError}
          </div>
        ) : null}
      </div>
    </div>
  );
}
