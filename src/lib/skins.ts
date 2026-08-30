/**
 * Minecraft skins API — adaptacja Modrinth App (GPLv3).
 * Źródło: apps/app-frontend/src/helpers/skins.ts
 */
import { invoke } from "@tauri-apps/api/core";
import type { McPlayerProfile } from "../types";

export interface Cape {
  id: string;
  name: string;
  texture: string;
  isEquipped: boolean;
}

export type SkinModel = "CLASSIC" | "SLIM" | "UNKNOWN";
export type SkinSource = "default" | "custom_external" | "custom";

export interface Skin {
  textureKey: string;
  name?: string;
  section?: string;
  variant: SkinModel;
  capeId?: string;
  texture: string;
  source: SkinSource;
  isEquipped: boolean;
  libraryId?: string;
}

export const DEFAULT_MODEL_SORTING = ["Steve", "Alex"] as const;

export const DEFAULT_MODELS: Record<string, SkinModel> = {
  Steve: "CLASSIC",
  Alex: "SLIM",
  Zuri: "CLASSIC",
  Sunny: "CLASSIC",
  Noor: "SLIM",
  Makena: "SLIM",
  Kai: "CLASSIC",
  Efe: "SLIM",
  Ari: "CLASSIC",
};

function unwrap<T>(p: Promise<T>): Promise<T> {
  return p.catch((e: unknown) => {
    throw new Error(typeof e === "string" ? e : String(e));
  });
}

export function filterSavedSkins(list: Skin[]): Skin[] {
  const custom = list.filter((s) => s.source === "custom");
  void fixUnknownSkins(custom);
  return custom;
}

export function skinIdentity(skin: Skin): string {
  return `${skin.textureKey}:${skin.variant}:${skin.capeId ?? ""}`;
}

/** Zapisane skiny + aktywny skin na górze (nawet gdy nie ma go w bibliotece). */
export function buildSavedSkinsList(allSkins: Skin[]): Skin[] {
  const saved = [...filterSavedSkins(allSkins)];
  const equipped = allSkins.find((s) => s.isEquipped);
  if (!equipped) return saved;

  const eqKey = skinIdentity(equipped);
  const existingIdx = saved.findIndex((s) => skinIdentity(s) === eqKey);
  if (existingIdx >= 0) {
    const [item] = saved.splice(existingIdx, 1);
    return [{ ...item, isEquipped: true }, ...saved];
  }
  return [{ ...equipped, isEquipped: true }, ...saved];
}

export function filterDefaultSkins(list: Skin[]): Skin[] {
  return list
    .filter(
      (s) =>
        s.source === "default" &&
        (!s.name || !(s.name in DEFAULT_MODELS) || s.variant === DEFAULT_MODELS[s.name]),
    )
    .sort((a, b) => {
      const aIndex = a.name ? DEFAULT_MODEL_SORTING.indexOf(a.name as (typeof DEFAULT_MODEL_SORTING)[number]) : -1;
      const bIndex = b.name ? DEFAULT_MODEL_SORTING.indexOf(b.name as (typeof DEFAULT_MODEL_SORTING)[number]) : -1;
      return (aIndex === -1 ? Infinity : aIndex) - (bIndex === -1 ? Infinity : bIndex);
    });
}

export function groupSkinsBySection(list: Skin[]): { title: string; skins: Skin[] }[] {
  const map = new Map<string, Skin[]>();
  for (const skin of filterDefaultSkins(list)) {
    const section = skin.section ?? "Domyślne skiny";
    const arr = map.get(section) ?? [];
    arr.push(skin);
    map.set(section, arr);
  }
  const order = ["Domyślne skiny", "Default skins"];
  return [...map.entries()]
    .sort(([a], [b]) => {
      const ai = order.indexOf(a);
      const bi = order.indexOf(b);
      return (ai === -1 ? 999 : ai) - (bi === -1 ? 999 : bi) || a.localeCompare(b, "pl");
    })
    .map(([title, skins]) => ({ title, skins }));
}

export function dedupeSkinsByName(skins: Skin[]): Skin[] {
  const byName = new Map<string, Skin>();
  for (const skin of skins) {
    const name = skin.name ?? skin.textureKey;
    const prefer = name === "Steve" ? "CLASSIC" : "SLIM";
    const existing = byName.get(name);
    if (!existing) {
      byName.set(name, skin);
      continue;
    }
    if (skin.variant === prefer && existing.variant !== prefer) {
      byName.set(name, skin);
    }
  }
  return [...byName.values()];
}

export async function determineModelType(texture: string): Promise<"SLIM" | "CLASSIC"> {
  return new Promise((resolve, reject) => {
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    if (!context) return reject(new Error("Brak kontekstu canvas."));
    const image = new Image();
    image.crossOrigin = "anonymous";
    image.src = texture;
    image.onload = () => {
      canvas.width = image.width;
      canvas.height = image.height;
      context.drawImage(image, 0, 0);
      const data = context.getImageData(54, 20, 2, 12).data;
      for (let i = 3; i < data.length; i += 4) {
        if (data[i] !== 0) {
          canvas.remove();
          resolve("CLASSIC");
          return;
        }
      }
      canvas.remove();
      resolve("SLIM");
    };
    image.onerror = () => {
      canvas.remove();
      reject(new Error("Nie udało się wczytać tekstury."));
    };
  });
}

export async function fixUnknownSkins(list: Skin[]): Promise<void> {
  for (const skin of list.filter((s) => s.variant === "UNKNOWN")) {
    try {
      skin.variant = await determineModelType(skin.texture);
    } catch {
      skin.variant = "CLASSIC";
    }
  }
}

export function skinToUiModel(variant: SkinModel): "slim" | "classic" {
  return variant === "SLIM" ? "slim" : "classic";
}

export function uiModelToSkinVariant(model: string): SkinModel {
  return model === "slim" || model === "SLIM" ? "SLIM" : "CLASSIC";
}

export function draftFromSkin(skin: Skin): {
  textureKey: string;
  variant: "slim" | "classic";
  name: string;
  pngDataUrl?: string;
  libraryId?: string;
} {
  return {
    textureKey: skin.textureKey,
    variant: skinToUiModel(skin.variant),
    name: skin.name ?? "Skin",
    pngDataUrl: skin.texture.startsWith("data:") ? skin.texture : undefined,
    libraryId: skin.libraryId,
  };
}

export function pngDataUrlFromSkin(skin: Skin): string | null {
  if (skin.texture.startsWith("data:")) return skin.texture;
  return null;
}

export async function getAvailableCapes(uuid: string): Promise<Cape[]> {
  return unwrap(invoke<Cape[]>("get_available_capes", { uuid }));
}

export async function getAvailableSkins(uuid: string): Promise<Skin[]> {
  return unwrap(invoke<Skin[]>("get_available_skins", { uuid }));
}

export async function equipSkin(
  uuid: string,
  skin: Skin,
  png?: number[] | null,
): Promise<McPlayerProfile | null> {
  return unwrap(
    invoke<McPlayerProfile | null>("equip_skin", {
      req: { uuid, skin, png: png ?? null },
    }),
  );
}

export async function removeCustomSkin(uuid: string, skin: Skin): Promise<void> {
  return unwrap(invoke("remove_custom_skin", { uuid, skin }));
}

export async function saveCustomSkin(
  uuid: string,
  skin: Skin,
  variant: SkinModel,
  options?: {
    capeId?: string | null;
    png?: number[] | null;
    replaceTexture?: boolean;
  },
): Promise<import("../types").SkinLibraryEntry> {
  return unwrap(
    invoke("save_custom_skin", {
      req: {
        uuid,
        skin,
        variant: variant === "SLIM" ? "slim" : "classic",
        capeId: options?.capeId ?? null,
        png: options?.png ?? null,
        replaceTexture: options?.replaceTexture ?? true,
      },
    }),
  );
}

export async function normalizeSkinTexture(texture: string | Uint8Array): Promise<Uint8Array> {
  const payload =
    typeof texture === "string" ? texture : [...texture];
  const data = await unwrap(invoke<number[]>("normalize_skin_texture", { texture: payload }));
  return Uint8Array.from(data);
}

export async function getNormalizedSkinTextureUrl(skin: Skin): Promise<string> {
  const data = await normalizeSkinTexture(skin.texture);
  const b64 = btoa(String.fromCharCode(...data));
  return `data:image/png;base64,${b64}`;
}

export async function flushPendingSkinChange(): Promise<void> {
  return unwrap(invoke("flush_pending_skin_change"));
}
