import { useEffect } from "react";
import { Save, X } from "lucide-react";
import type { ApiSkinModel } from "../lib/skinModel";
import { SkinViewer3D } from "./SkinViewer3D";

export function SkinUploadDialog({
  open,
  previewUrl,
  model,
  name,
  onNameChange,
  onModelChange,
  onConfirm,
  onCancel,
  busy,
  confirmLabel = "Zapisz do biblioteki",
  showName = true,
}: {
  open: boolean;
  previewUrl: string | null;
  model: ApiSkinModel;
  name?: string;
  onNameChange?: (name: string) => void;
  onModelChange: (m: ApiSkinModel) => void;
  onConfirm: () => void;
  onCancel: () => void;
  busy?: boolean;
  confirmLabel?: string;
  showName?: boolean;
}) {
  useEffect(() => {
    if (!open) return;
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") onCancel();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onCancel]);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-[80] flex items-center justify-center bg-black/75 p-4 backdrop-blur-sm">
      <div className="w-full max-w-md rounded-2xl border border-line bg-raised shadow-2xl">
        <div className="flex items-center justify-between border-b border-line px-5 py-4">
          <h2 className="text-lg font-bold">Dodaj skin</h2>
          <button
            type="button"
            onClick={onCancel}
            className="grid h-9 w-9 place-items-center rounded-lg text-mute hover:bg-white/6 hover:text-ink"
          >
            <X size={18} />
          </button>
        </div>

        <div className="space-y-5 p-5">
          {previewUrl && (
            <div className="mx-auto h-48 w-full max-w-[10rem] overflow-hidden rounded-xl border border-line bg-[#141418]">
              <SkinViewer3D
                compact
                skinPngDataUrl={previewUrl}
                model={model === "slim" ? "slim" : "classic"}
                className="h-full"
              />
            </div>
          )}

          {showName && onNameChange && (
            <div>
              <label className="text-xs font-semibold uppercase tracking-wider text-mute">
                Nazwa (opcjonalnie)
              </label>
              <input
                type="text"
                value={name ?? ""}
                onChange={(e) => onNameChange(e.target.value)}
                placeholder="Mój skin"
                className="mt-2 w-full rounded-xl border border-line bg-raised2 px-3 py-2 text-sm outline-none focus:border-accent/50"
              />
            </div>
          )}

          <div>
            <p className="text-xs font-semibold uppercase tracking-wider text-mute">
              Grubość ramion
            </p>
            <div className="mt-2 flex gap-4 text-sm">
              <label className="flex cursor-pointer items-center gap-2">
                <input
                  type="radio"
                  name="upload-arm"
                  checked={model === "classic"}
                  onChange={() => onModelChange("classic")}
                />
                Steve (szerokie)
              </label>
              <label className="flex cursor-pointer items-center gap-2">
                <input
                  type="radio"
                  name="upload-arm"
                  checked={model === "slim"}
                  onChange={() => onModelChange("slim")}
                />
                Alex (smukłe)
              </label>
            </div>
          </div>
        </div>

        <div className="flex justify-end gap-2 border-t border-line px-5 py-4">
          <button
            type="button"
            onClick={onCancel}
            className="rounded-xl px-4 py-2 text-sm text-mute hover:bg-white/5"
          >
            Anuluj
          </button>
          <button
            type="button"
            disabled={busy}
            onClick={onConfirm}
            className="inline-flex items-center gap-2 rounded-xl bg-good px-4 py-2 text-sm font-semibold text-black disabled:opacity-50"
          >
            <Save size={15} />
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
