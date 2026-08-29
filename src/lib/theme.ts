import type { Settings } from "../types";

export type AccentPreset = "violet" | "cyber" | "ember" | "mono";

export const ACCENT_PRESETS: { id: AccentPreset; label: string }[] = [
  { id: "violet", label: "Fiolet" },
  { id: "cyber", label: "Cyber" },
  { id: "ember", label: "Ember" },
  { id: "mono", label: "Mono" },
];

const VALID: AccentPreset[] = ["violet", "cyber", "ember", "mono"];

export function normalizePreset(value?: string | null): AccentPreset {
  if (value && (VALID as string[]).includes(value)) return value as AccentPreset;
  return "violet";
}

export function applyPreset(preset: AccentPreset) {
  document.documentElement.setAttribute("data-preset", preset);
}

export function applySettingsTheme(settings: Settings | null | undefined) {
  applyPreset(normalizePreset(settings?.accentPreset));
}
