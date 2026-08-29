import { useEffect, useState } from "react";
import { clsx } from "clsx";
import { api } from "../lib/api";

function textureKeyFromUrl(url: string): string | null {
  const parts = url.split("/");
  return parts[parts.length - 1] || null;
}

/** Płaska miniatura peleryny (atlas 64×32, widoczny fragment 10×16). */
export function CapeThumbnail({
  textureUrl,
  alt,
  selected,
  className,
}: {
  textureUrl: string;
  alt: string;
  selected?: boolean;
  className?: string;
}) {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setSrc(null);
    const key = textureKeyFromUrl(textureUrl);
    if (!key) return;
    api
      .getMojangTexturePreview(key)
      .then((b64) => {
        if (!cancelled) {
          setSrc(b64.startsWith("data:") ? b64 : `data:image/png;base64,${b64}`);
        }
      })
      .catch(() => {
        if (!cancelled) setSrc(null);
      });
    return () => {
      cancelled = true;
    };
  }, [textureUrl]);

  return (
    <div
      className={clsx(
        "cape-thumb relative h-full w-full overflow-hidden rounded-lg bg-raised2",
        selected && "ring-2 ring-good/60",
        className,
      )}
      title={alt}
    >
      {src ? (
        <img src={src} alt={alt} className="cape-thumb-img" draggable={false} />
      ) : (
        <div className="grid h-full place-items-center text-[9px] text-mute">…</div>
      )}
      {selected && (
        <div className="pointer-events-none absolute inset-0 shadow-[inset_0_0_4px_4px_rgba(0,0,0,0.35)]" />
      )}
    </div>
  );
}
