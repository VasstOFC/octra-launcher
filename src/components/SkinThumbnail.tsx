import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { bustFromSkinPngBase64 } from "../lib/skinRender";

const cache = new Map<string, string>();

export function SkinThumbnail({
  textureKey,
  variant,
  alt,
  className,
}: {
  textureKey: string;
  variant: "slim" | "classic";
  alt: string;
  className?: string;
}) {
  const [src, setSrc] = useState<string | null>(() => cache.get(textureKey) ?? null);

  useEffect(() => {
    let cancelled = false;
    const cached = cache.get(textureKey);
    if (cached) {
      setSrc(cached);
      return;
    }
    api
      .getMojangTexturePreview(textureKey)
      .then(async (b64) => {
        const url = await bustFromSkinPngBase64(b64, variant);
        if (!cancelled) {
          cache.set(textureKey, url);
          setSrc(url);
        }
      })
      .catch(() => {
        if (!cancelled) setSrc(null);
      });
    return () => {
      cancelled = true;
    };
  }, [textureKey, variant]);

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
