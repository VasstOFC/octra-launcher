import { useEffect, useMemo } from "react";
import { Save, Upload, X } from "lucide-react";
import type { Account, McCape, McPlayerProfile } from "../types";
import type { Cape } from "../lib/skins";
import { CapePicker } from "./CapePicker";
import { SkinViewer3D } from "./SkinViewer3D";

function capeLabel(cape: McCape | Cape): string {
  if ("alias" in cape && cape.alias?.trim()) return cape.alias.trim();
  if ("name" in cape && cape.name?.trim()) return cape.name.trim();
  return cape.id;
}

function capeTexture(cape: McCape | Cape): string {
  return "url" in cape ? cape.url : cape.texture;
}

export function SkinEditModal({
  open,
  account,
  mcProfile,
  skinUrl,
  skinPngDataUrl,
  skinTextureKey,
  capeUrl,
  viewerModel,
  draftCapeId,
  profileError,
  profileLoading,
  model,
  dirty,
  onModelChange,
  onClose,
  onSaveOffline,
  onUpload,
  onPremiumUpload,
  onRefreshPremium,
  onDraftCapeChange,
  onSave,
  busy,
  availableCapes,
}: {
  open: boolean;
  account: Account;
  mcProfile: McPlayerProfile | null;
  skinUrl: string | null;
  skinPngDataUrl: string | null;
  skinTextureKey: string | null;
  capeUrl: string | null;
  viewerModel: "slim" | "classic";
  draftCapeId: string | null;
  profileError: string | null;
  profileLoading: boolean;
  model: "wide" | "slim";
  dirty: boolean;
  onModelChange: (m: "wide" | "slim") => void;
  onClose: () => void;
  onSaveOffline: () => void;
  onUpload: (file: File) => void;
  onPremiumUpload: (file: File) => void;
  onRefreshPremium: () => void;
  onDraftCapeChange: (capeId: string | null) => void;
  onSave: () => void;
  busy: boolean;
  availableCapes?: Cape[];
}) {
  const isOffline = account.kind === "offline";

  const capesForPicker = useMemo(() => {
    if (availableCapes && availableCapes.length > 0) return availableCapes;
    return (mcProfile?.capes ?? []).map((cape) => ({
      id: cape.id,
      name: capeLabel(cape),
      texture: capeTexture(cape),
      isEquipped: cape.state.toUpperCase() === "ACTIVE",
    }));
  }, [availableCapes, mcProfile?.capes]);

  useEffect(() => {
    if (!open) return;
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-[70] flex items-center justify-center bg-black/70 p-4 backdrop-blur-sm">
      <div className="flex max-h-[92vh] w-full max-w-4xl flex-col overflow-hidden rounded-2xl border border-line bg-raised shadow-2xl">
        <div className="flex items-center justify-between border-b border-line px-5 py-4">
          <h2 className="text-lg font-bold">Edycja skina</h2>
          <button
            type="button"
            onClick={onClose}
            className="grid h-9 w-9 place-items-center rounded-lg text-mute hover:bg-white/6 hover:text-ink"
          >
            <X size={18} />
          </button>
        </div>

        <div className="grid min-h-0 flex-1 gap-6 overflow-y-auto p-5 md:grid-cols-[1fr_1.05fr]">
          <SkinViewer3D
            large
            skinPngDataUrl={skinPngDataUrl}
            skinUrl={skinUrl}
            skinTextureKey={skinTextureKey}
            capeUrl={isOffline ? null : capeUrl}
            model={viewerModel}
          />

          <div className="space-y-5">
            <div>
              <p className="text-xs font-semibold uppercase tracking-wider text-mute">Tekstura</p>
              {isOffline ? (
                <label className="mt-2 flex cursor-pointer items-center justify-center gap-2 rounded-xl border border-dashed border-line py-3 text-sm hover:border-accent/40">
                  <Upload size={16} />
                  Zamień teksturę (PNG)
                  <input
                    type="file"
                    accept="image/png"
                    className="hidden"
                    onChange={(e) => {
                      const f = e.target.files?.[0];
                      if (f) onUpload(f);
                    }}
                  />
                </label>
              ) : (
                <div className="mt-2 space-y-2">
                  <label className="flex cursor-pointer items-center justify-center gap-2 rounded-xl border border-dashed border-line py-3 text-sm hover:border-accent/40">
                    <Upload size={16} />
                    Zamień teksturę (PNG)
                    <input
                      type="file"
                      accept="image/png"
                      className="hidden"
                      onChange={(e) => {
                        const f = e.target.files?.[0];
                        if (f) onPremiumUpload(f);
                      }}
                    />
                  </label>
                  <p className="text-sm text-mute">
                    Wybierz skin z galerii — podgląd 3D po lewej. Zatwierdź przyciskiem „Zapisz skin”.
                  </p>
                  <button
                    type="button"
                    disabled={busy}
                    onClick={onRefreshPremium}
                    className="rounded-xl border border-line px-4 py-2 text-sm hover:bg-white/5 disabled:opacity-50"
                  >
                    Odśwież z Mojang
                  </button>
                </div>
              )}
            </div>

            <div>
              <p className="text-xs font-semibold uppercase tracking-wider text-mute">
                Model ramion
              </p>
              <div className="mt-2 flex gap-4 text-sm">
                <label className="flex items-center gap-2">
                  <input
                    type="radio"
                    name="arm"
                    checked={model === "wide"}
                    disabled={!isOffline}
                    onChange={() => onModelChange("wide")}
                  />
                  Szeroki (Steve)
                </label>
                <label className="flex items-center gap-2">
                  <input
                    type="radio"
                    name="arm"
                    checked={model === "slim"}
                    disabled={!isOffline}
                    onChange={() => onModelChange("slim")}
                  />
                  Smukły (Alex)
                </label>
              </div>
            </div>

            <div>
              <p className="text-xs font-semibold uppercase tracking-wider text-mute">Peleryna</p>
              {isOffline ? (
                <p className="mt-2 text-sm text-mute">Peleryny są dostępne tylko na koncie Premium.</p>
              ) : (
                <div className="mt-2">
                  <CapePicker
                    capes={capesForPicker}
                    draftCapeId={draftCapeId}
                    onDraftCapeChange={onDraftCapeChange}
                    loading={profileLoading}
                    error={profileError}
                    disabled={busy}
                  />
                </div>
              )}
            </div>
          </div>
        </div>

        <div className="flex justify-end gap-2 border-t border-line px-5 py-4">
          <button
            type="button"
            onClick={onClose}
            className="inline-flex items-center gap-2 rounded-xl px-4 py-2 text-sm text-mute hover:bg-white/5"
          >
            <X size={15} />
            Anuluj
          </button>
          {isOffline ? (
            <button
              type="button"
              disabled={busy}
              onClick={onSaveOffline}
              className="inline-flex items-center gap-2 rounded-xl bg-good px-4 py-2 text-sm font-semibold text-black disabled:opacity-50"
            >
              <Save size={15} />
              Zapisz skin
            </button>
          ) : (
            <button
              type="button"
              disabled={busy || !dirty}
              onClick={onSave}
              className="inline-flex items-center gap-2 rounded-xl bg-good px-4 py-2 text-sm font-semibold text-black disabled:opacity-50"
            >
              <Save size={15} />
              Zapisz skin
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
