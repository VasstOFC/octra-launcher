import type { Instance } from "../types";

export const GALLERY_PREFIX = "gallery:";

export function galleryIconId(symbol?: string | null): string | null {
  const sym = symbol?.trim();
  if (!sym?.startsWith(GALLERY_PREFIX)) return null;
  const id = sym.slice(GALLERY_PREFIX.length).trim();
  return id || null;
}

export function galleryIconIdFromInstance(inst: Instance): string | null {
  return galleryIconId(inst.iconSymbol);
}

export function galleryIconSymbol(id: string): string {
  return `${GALLERY_PREFIX}${id}`;
}

export function galleryIconUrl(id: string): string {
  const safe = id.trim() || DEFAULT_GALLERY_ICON_ID;
  return `/mc-textures/${safe}.png`;
}

export const DEFAULT_GALLERY_ICON_ID = "grass";
