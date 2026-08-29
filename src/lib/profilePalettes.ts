export type ProfilePalette = {
  id: string;
  name: string;
  c1: string;
  c2: string;
  glyph: string;
};

export const PROFILE_PALETTES: ProfilePalette[] = [
  { id: "forest", name: "Leśny", c1: "#14532d", c2: "#4ade80", glyph: "#dcfce7" },
  { id: "ocean", name: "Ocean", c1: "#164e63", c2: "#22d3ee", glyph: "#cffafe" },
  { id: "sky", name: "Niebo", c1: "#1e3a8a", c2: "#60a5fa", glyph: "#dbeafe" },
  { id: "violet", name: "Fiolet", c1: "#4c1d95", c2: "#c4a7ff", glyph: "#f3e8ff" },
  { id: "magenta", name: "Magenta", c1: "#6b21a8", c2: "#e879f9", glyph: "#fae8ff" },
  { id: "rose", name: "Róż", c1: "#831843", c2: "#fb7185", glyph: "#ffe4e6" },
  { id: "ember", name: "Żar", c1: "#7c2d12", c2: "#fb923c", glyph: "#ffedd5" },
  { id: "gold", name: "Złoto", c1: "#713f12", c2: "#fbbf24", glyph: "#fef3c7" },
  { id: "teal", name: "Morski", c1: "#134e4a", c2: "#2dd4bf", glyph: "#ccfbf1" },
  { id: "indigo", name: "Indygo", c1: "#1e1b4b", c2: "#818cf8", glyph: "#e0e7ff" },
  { id: "lime", name: "Limonka", c1: "#3f6212", c2: "#a3e635", glyph: "#ecfccb" },
  { id: "mint", name: "Mięta", c1: "#0f766e", c2: "#5eead4", glyph: "#ccfbf1" },
];

export function paletteById(id: string): ProfilePalette {
  return PROFILE_PALETTES.find((p) => p.id === id) ?? PROFILE_PALETTES[3]!;
}
