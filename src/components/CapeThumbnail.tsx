import { useEffect, useState } from "react";
import { clsx } from "clsx";
import { api } from "../lib/api";
import { capeFrontFromPngBase64 } from "../lib/skinRender";

const cache = new Map<string, string>();

function textureKeyFromUrl(url: string): string | null {
  const parts = url.split("/");
  return parts[parts.length - 1] || null;
}

/** Płaska miniatura peleryny (tył 10×16), jak w Modrinth. */
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
  const [src, setSrc] = useState<string | null>(() => cache.get(textureUrl) ?? null);

  useEffect(() => {
    let cancelled = false;
    const cached = cache.get(textureUrl);
    if (cached) {
      setSrc(cached);
      return;
    }
    setSrc(null);

    const key = textureKeyFromUrl(textureUrl);
    const load = key
      ? api.getMojangTexturePreview(key).then((b64) => capeFrontFromPngBase64(b64))
      : api.fetchImageBase64(textureUrl).then((b64) => capeFrontFromPngBase64(b64));

    void load
      .then((url) => {
        if (!cancelled) {
          cache.set(textureUrl, url);
          setSrc(url);
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
        "relative h-full w-full overflow-hidden rounded-lg bg-raised2",
        selected && "ring-2 ring-good/60",
        className,
      )}
      title={alt}
    >
      {src ? (
        <img
          src={src}
          alt={alt}
          className="h-full w-full object-contain [image-rendering:pixelated]"
          draggable={false}
        />
      ) : (
        <div className="grid h-full place-items-center text-[9px] text-mute">…</div>
      )}
      {selected && (
        <div className="pointer-events-none absolute inset-0 shadow-[inset_0_0_4px_4px_rgba(0,0,0,0.35)]" />
      )}
    </div>
  );
}
