import { convertFileSrc } from "@tauri-apps/api/core";

/** Konwertuje ścieżkę pliku lokalnego na URL ładowalny w WebView. */
export function assetUrl(path: string | null | undefined): string | null {
  if (!path) return null;
  return convertFileSrc(path);
}

/** Wymusza ponowne wczytanie obrazu po zmianie pliku pod tą samą ścieżką. */
export function bustAssetUrl(url: string | null | undefined, epoch: number): string | null {
  if (!url) return null;
  if (epoch <= 0) return url;
  const sep = url.includes("?") ? "&" : "?";
  return `${url}${sep}v=${epoch}`;
}
