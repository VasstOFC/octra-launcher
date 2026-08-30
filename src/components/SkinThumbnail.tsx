import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { bodyFromSkinPngBase64 } from "../lib/skinRender";

const cache = new Map<string, string>();

function cacheKey(textureKey: string | undefined, pngBase64: string | undefined, variant: string) {
  if (textureKey) return `tk:${textureKey}:${variant}`;
  if (pngBase64) return `png:${pngBase64.length}:${pngBase64.slice(0, 48)}:${variant}`;
  return "";
}

export function SkinThumbnail({
  textureKey,
  pngBase64,
  variant,
  alt,
  className,
}: {
  textureKey?: string;
  pngBase64?: string | null;
  variant: "slim" | "classic";
  alt: string;
  className?: string;
}) {
  const key = cacheKey(textureKey, pngBase64 ?? undefined, variant);
  const [src, setSrc] = useState<string | null>(() => (key ? (cache.get(key) ?? null) : null));

  useEffect(() => {
    if (!key) {
      setSrc(null);
      return;
    }
    let cancelled = false;
    setSrc(null);
    const cached = cache.get(key);
    if (cached) {
      setSrc(cached);
      return;
    }
    const load =
      pngBase64 != null && pngBase64 !== ""
        ? bodyFromSkinPngBase64(pngBase64, variant)
        : textureKey
          ? api
              .getMojangTexturePreview(textureKey)
              .then((b64) => bodyFromSkinPngBase64(b64, variant))
          : Promise.reject(new Error("brak skina"));

    void load
      .then((url) => {
        if (!cancelled) {
          cache.set(key, url);
          setSrc(url);
        }
      })
      .catch(() => {
        if (!cancelled) setSrc(null);
      });
    return () => {
      cancelled = true;
    };
  }, [key, textureKey, pngBase64, variant]);

  if (!src) {
    return <div className={className ?? "h-full w-full animate-pulse bg-white/8"} />;
  }

  return (
    <img
      src={src}
      alt={alt}
      className={className ?? "h-full w-full object-contain p-2 [image-rendering:pixelated]"}
      loading="lazy"
    />
  );
}
