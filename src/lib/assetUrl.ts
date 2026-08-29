import { convertFileSrc } from "@tauri-apps/api/core";

/** Konwertuje ścieżkę pliku lokalnego na URL ładowalny w WebView. */
export function assetUrl(path: string | null | undefined): string | null {
  if (!path) return null;
  return convertFileSrc(path);
}
