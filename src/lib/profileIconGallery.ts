export type ProfileIconCategory = {
  id: string;
  label: string;
};

export type ProfileGalleryIcon = {
  id: string;
  label: string;
  category: string;
};

export const PROFILE_ICON_CATEGORIES: ProfileIconCategory[] = [
  { id: "blocks", label: "Bloki Minecraft" },
  { id: "ores", label: "Rudy i szlachetne" },
  { id: "packs", label: "Paczki i mody" },
];

export const PROFILE_GALLERY_ICONS: ProfileGalleryIcon[] = [
  { id: "grass", label: "Trawa", category: "blocks" },
  { id: "dirt", label: "Ziemia", category: "blocks" },
  { id: "stone", label: "Kamień", category: "blocks" },
  { id: "cobblestone", label: "Bruk", category: "blocks" },
  { id: "oak_log", label: "Dębowy pień", category: "blocks" },
  { id: "sand", label: "Piasek", category: "blocks" },
  { id: "netherrack", label: "Netherrack", category: "blocks" },
  { id: "obsidian", label: "Obsydian", category: "blocks" },
  { id: "end_stone", label: "End stone", category: "blocks" },
  { id: "tnt", label: "TNT", category: "blocks" },
  { id: "chest", label: "Skrzynia", category: "blocks" },
  { id: "crafting_table", label: "Stół rzemieślniczy", category: "blocks" },
  { id: "beacon", label: "Beacon", category: "blocks" },
  { id: "diamond", label: "Diament", category: "ores" },
  { id: "emerald", label: "Szmaragd", category: "ores" },
  { id: "gold", label: "Złoto", category: "ores" },
  { id: "iron", label: "Żelazo", category: "ores" },
  { id: "redstone", label: "Redstone", category: "ores" },
  { id: "netherite", label: "Netherite", category: "ores" },
  { id: "lapis", label: "Lapis", category: "ores" },
  { id: "cobblemon", label: "Cobblemon", category: "packs" },
  { id: "pixelmon", label: "Pixelmon", category: "packs" },
  { id: "aged", label: "Aged", category: "packs" },
  { id: "create", label: "Create", category: "packs" },
  { id: "botania", label: "Botania", category: "packs" },
  { id: "twilight", label: "Twilight Forest", category: "packs" },
  { id: "atm", label: "All the Mods", category: "packs" },
  { id: "skyblock", label: "Skyblock", category: "packs" },
  { id: "vault_hunters", label: "Vault Hunters", category: "packs" },
  { id: "ftb", label: "FTB", category: "packs" },
  { id: "better_mc", label: "Better MC", category: "packs" },
  { id: "rlcraft", label: "RLCraft", category: "packs" },
];

export function galleryIconsForCategory(categoryId: string): ProfileGalleryIcon[] {
  return PROFILE_GALLERY_ICONS.filter((i) => i.category === categoryId);
}

export function galleryIconById(id: string): ProfileGalleryIcon | undefined {
  return PROFILE_GALLERY_ICONS.find((i) => i.id === id);
}
