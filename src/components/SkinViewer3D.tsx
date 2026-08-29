import { useEffect, useRef, useState } from "react";
import { clsx } from "clsx";
import { MoveHorizontal } from "lucide-react";
import { SkinViewer } from "skinview3d";
import { api } from "../lib/api";
import { normalizeTextureUrl } from "../lib/skinRender";

const skinDataCache = new Map<string, string>();

function toDataUrl(b64: string): string {
  return b64.startsWith("data:") ? b64 : `data:image/png;base64,${b64}`;
}

async function resolveSkinSource(
  skinPngDataUrl: string | null,
  skinUrl: string | null,
  skinTextureKey: string | null,
): Promise<string | null> {
  if (skinPngDataUrl) return skinPngDataUrl;
  if (skinTextureKey) {
    const cached = skinDataCache.get(skinTextureKey);
    if (cached) return cached;
    const b64 = await api.getMojangTexturePreview(skinTextureKey);
    const dataUrl = toDataUrl(b64);
    skinDataCache.set(skinTextureKey, dataUrl);
    return dataUrl;
  }
  if (skinUrl) return skinUrl;
  return null;
}

async function resolveCapeSource(capeUrl: string | null): Promise<string | null> {
  if (!capeUrl) return null;
  const key = capeUrl.split("/").pop() ?? capeUrl;
  const cached = skinDataCache.get(`cape:${key}`);
  if (cached) return cached;
  if (capeUrl.startsWith("data:")) return capeUrl;
  if (capeUrl.includes("textures.minecraft.net")) {
    try {
      const b64 = await api.getMojangTexturePreview(key);
      const dataUrl = toDataUrl(b64);
      skinDataCache.set(`cape:${key}`, dataUrl);
      return dataUrl;
    } catch {
      return normalizeTextureUrl(capeUrl);
    }
  }
  return capeUrl;
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

  useEffect(() => {
    const canvas = canvasRef.current;
    const host = hostRef.current;
    if (!canvas || !host) return;

    const viewer = new SkinViewer({
      canvas,
      width: host.clientWidth || 280,
      height: host.clientHeight || 360,
      background: 0x18181c,
    });
    viewer.fov = 42;
    viewer.zoom = 0.9;
    viewer.controls.enableRotate = true;
    viewer.controls.enableZoom = false;
    viewer.controls.enablePan = false;
    viewerRef.current = viewer;
    setViewerReady(true);

    const ro = new ResizeObserver(() => {
      const w = host.clientWidth;
      const h = host.clientHeight;
      if (w > 0 && h > 0) viewer.setSize(w, h);
    });
    ro.observe(host);

    return () => {
      ro.disconnect();
      viewer.dispose();
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

    (async () => {
      try {
        const src = await resolveSkinSource(
          skinPngDataUrl ?? null,
          skinUrl ?? null,
          skinTextureKey ?? null,
        );
        if (cancelled) return;
        const skinModel = model === "slim" ? "slim" : "default";
        if (src) {
          await viewer.loadSkin(src, { model: skinModel });
          if (!cancelled) setSkinLoaded(true);
        } else {
          viewer.resetSkin();
        }
      } catch {
        if (!cancelled) {
          viewer.resetSkin();
          setSkinLoaded(false);
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
    <div className={clsx("flex flex-col items-center", className)}>
      <div
        ref={hostRef}
        className={clsx(
          "relative w-full overflow-hidden rounded-xl bg-raised2/80",
          large ? "h-[min(52vh,420px)]" : "h-44",
        )}
      >
        <canvas ref={canvasRef} className="h-full w-full" />
        {hasSkinRequest && !skinLoaded && (
          <div className="pointer-events-none absolute inset-0 grid place-items-center text-xs text-mute">
            Ładowanie skina…
          </div>
        )}
        {!hasSkinRequest && (
          <div className="pointer-events-none absolute inset-0 grid place-items-center text-xs text-mute">
            Wybierz skin
          </div>
        )}
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
