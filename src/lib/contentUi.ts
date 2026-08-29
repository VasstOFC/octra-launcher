import type { ContentTab } from "../stores/octraStore";
import type { Loader } from "../types";

export type ContentDensity = "normal" | "compact";

const DENSITY_KEY = "octra-content-density";

export function loadContentDensity(): ContentDensity {
  try {
    const v = localStorage.getItem(DENSITY_KEY);
    if (v === "compact" || v === "normal") return v;
  } catch {
    /* ignore */
  }
  return "normal";
}

export function saveContentDensity(density: ContentDensity) {
  try {
    localStorage.setItem(DENSITY_KEY, density);
  } catch {
    /* ignore */
  }
}

export function modrinthProjectType(tab: ContentTab): string | null {
  if (tab === "mods") return "mod";
  if (tab === "shaders") return "shader";
  if (tab === "resources") return "resourcepack";
  return null;
}

export function modrinthLoader(loader: Loader): string | undefined {
  if (loader === "vanilla") return undefined;
  return loader;
}

export function contentTabLabel(tab: ContentTab): string {
  switch (tab) {
    case "loader":
      return "Loader";
    case "mods":
      return "Mody";
    case "shaders":
      return "Shadery";
    case "worlds":
      return "Światy";
    case "resources":
      return "Zasoby";
    case "appearance":
      return "Wygląd";
    case "advanced":
      return "Zaawansowane";
    default:
      return tab;
  }
}
