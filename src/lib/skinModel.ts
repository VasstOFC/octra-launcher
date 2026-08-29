/** Model ramion w API backendu (Rust). */
export type ApiSkinModel = "classic" | "slim";

/** Model w UI (Steve = szerokie ramiona). */
export type UiSkinModel = "wide" | "slim";

export function toApiSkinModel(model: UiSkinModel | ApiSkinModel | string): ApiSkinModel {
  return model === "slim" ? "slim" : "classic";
}

export function toUiSkinModel(model: string | null | undefined): UiSkinModel {
  return model === "slim" ? "slim" : "wide";
}
