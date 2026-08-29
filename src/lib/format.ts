import { PROFILE_PALETTES } from "./profilePalettes";

export const LOADER_LABEL: Record<string, string> = {
  vanilla: "Vanilla",
  fabric: "Fabric",
  quilt: "Quilt",
  forge: "Forge",
  neoforge: "NeoForge",
  iris: "Iris",
  optifine: "OptiFine",
  canvas: "Canvas",
};

export const LOADER_CHIP: Record<string, string> = {
  vanilla: "bg-vanilla/25 text-vanilla ring-1 ring-vanilla/35",
  fabric: "bg-fabric/25 text-fabric ring-1 ring-fabric/35",
  quilt: "bg-quilt/25 text-quilt ring-1 ring-quilt/35",
  forge: "bg-forge/25 text-forge ring-1 ring-forge/35",
  neoforge: "bg-neoforge/25 text-neoforge ring-1 ring-neoforge/35",
};

export function loaderChipClass(loader: string): string {
  return LOADER_CHIP[loader] ?? "bg-white/10 text-mute ring-1 ring-white/15";
}

export function instanceAccent(id: string): [string, string] {
  const palettes: [string, string][] = PROFILE_PALETTES.map((p) => [p.c1, p.c2]);
  let h = 0;
  for (let i = 0; i < id.length; i++) h = (h * 31 + id.charCodeAt(i)) >>> 0;
  return palettes[h % palettes.length]!;
}

export function formatDate(iso?: string | null): string {
  if (!iso) return "Nigdy";
  try {
    return new Intl.DateTimeFormat("pl-PL", {
      dateStyle: "medium",
      timeStyle: "short",
    }).format(new Date(iso));
  } catch {
    return iso;
  }
}

export function formatCount(n: number): string {
  if (n >= 1_000_000) {
    const v = n / 1_000_000;
    const s = v >= 10 ? v.toFixed(0) : v.toFixed(1).replace(".", ",");
    return `${s} mln`;
  }
  if (n >= 1_000) {
    const v = n / 1_000;
    const s = v >= 10 ? v.toFixed(0) : v.toFixed(1).replace(".", ",");
    return `${s} tys.`;
  }
  return new Intl.NumberFormat("pl-PL").format(n);
}

export function formatRam(mb: number): string {
  if (mb >= 1024) {
    const gb = mb / 1024;
    return `${gb % 1 === 0 ? gb.toFixed(0) : gb.toFixed(1)} GB`;
  }
  return `${mb} MB`;
}

export function instanceCountLabel(n: number): string {
  const abs = Math.abs(n) % 100;
  const last = abs % 10;
  let word = "profili";
  if (n === 1) word = "profil";
  else if (last >= 2 && last <= 4 && (abs < 12 || abs > 14)) word = "profile";
  return `${n} ${word}`;
}

export function formatPlayTime(secs: number): string {
  if (!secs || secs < 60) return "poniżej minuty";
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h <= 0) return `${m} min`;
  if (m <= 0) return `${h} godz.`;
  return `${h} godz. ${m} min`;
}

export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${Math.round(n / 1024)} KB`;
  const mb = n / (1024 * 1024);
  return `${mb >= 10 ? mb.toFixed(0) : mb.toFixed(1).replace(".", ",")} MB`;
}
