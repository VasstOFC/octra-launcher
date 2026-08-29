export type CatalogSkin = {
  id: string;
  name: string;
  model: "slim" | "classic";
  /** mc-heads identyfikator (nick lub UUID) */
  previewId: string;
  /** offline: reset do Steve/Alex */
  offlineDefault?: "steve" | "alex";
};

export type SkinGroup = {
  id: string;
  title: string;
  skins: CatalogSkin[];
};

export const SKIN_CATALOG: SkinGroup[] = [
  {
    id: "defaults",
    title: "Domyślne",
    skins: [
      {
        id: "steve",
        name: "Steve",
        model: "classic",
        previewId: "Steve",
        offlineDefault: "steve",
      },
      {
        id: "alex",
        name: "Alex",
        model: "slim",
        previewId: "Alex",
        offlineDefault: "alex",
      },
    ],
  },
  {
    id: "classic",
    title: "Klasyki",
    skins: [
      { id: "ari", name: "Ari", model: "slim", previewId: "Ari" },
      { id: "efe", name: "Efe", model: "slim", previewId: "Efe" },
      { id: "kai", name: "Kai", model: "slim", previewId: "Kai" },
      { id: "makena", name: "Makena", model: "slim", previewId: "Makena" },
      { id: "noor", name: "Noor", model: "slim", previewId: "Noor" },
      { id: "sunny", name: "Sunny", model: "slim", previewId: "Sunny" },
      { id: "zuri", name: "Zuri", model: "slim", previewId: "Zuri" },
    ],
  },
  {
    id: "villagers",
    title: "Osadnicy",
    skins: [
      { id: "villager", name: "Osadnik", model: "classic", previewId: "Villager" },
      { id: "wandering", name: "Wędrowny handlarz", model: "classic", previewId: "WanderingTrader" },
    ],
  },
];

export function catalogPreviewUrl(previewId: string): string {
  return `https://mc-heads.net/body/${encodeURIComponent(previewId)}/128`;
}
