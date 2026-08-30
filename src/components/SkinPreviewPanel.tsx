import { Check, Eye, Pencil, RotateCcw, UnfoldHorizontal } from "lucide-react";
import { SkinViewer3D } from "./SkinViewer3D";
import { CapePicker } from "./CapePicker";
import type { Cape } from "../lib/skins";

export function SkinPreviewPanel({
  nametag,
  skinPngDataUrl,
  skinUrl,
  skinTextureKey,
  capeUrl,
  model,
  previewing,
  dirty,
  loading,
  onApply,
  onReset,
  onEdit,
  onSaveToLibrary,
  editDisabled,
  showCapes,
  availableCapes,
  draftCapeId,
  onDraftCapeChange,
  capesLoading,
  capesError,
}: {
  nametag: string;
  skinPngDataUrl?: string | null;
  skinUrl?: string | null;
  skinTextureKey?: string | null;
  capeUrl?: string | null;
  model: "slim" | "classic";
  previewing?: boolean;
  dirty?: boolean;
  loading?: boolean;
  onApply?: () => void;
  onReset?: () => void;
  onEdit?: () => void;
  onSaveToLibrary?: () => void;
  editDisabled?: boolean;
  showCapes?: boolean;
  availableCapes?: Cape[];
  draftCapeId?: string | null;
  onDraftCapeChange?: (capeId: string | null) => void;
  capesLoading?: boolean;
  capesError?: string | null;
}) {
  return (
    <div className="flex min-h-0 flex-col">
      <div className="relative mx-auto w-full max-w-[min(100%,380px)] flex-1">
        <div className="relative h-[min(62vh,440px)] w-full pt-8">
          {previewing && (
            <div className="absolute left-1/2 top-0 z-20 flex -translate-x-1/2 items-center gap-1.5 rounded-full border border-accent/40 bg-accent/15 px-3 py-1 text-sm font-semibold text-accent">
              <Eye className="size-4 shrink-0" />
              Podglądasz
            </div>
          )}

          {nametag ? (
            <div
              className="skin-preview-nametag pointer-events-none absolute left-1/2 z-20 -translate-x-1/2"
              style={{ top: previewing ? "2.25rem" : "0.5rem" }}
            >
              {nametag}
            </div>
          ) : null}

          <div className="absolute inset-x-0 bottom-0 top-12">
            <SkinViewer3D
              large
              modrinth
              skinPngDataUrl={skinPngDataUrl}
              skinUrl={skinUrl}
              skinTextureKey={skinTextureKey}
              capeUrl={capeUrl}
              model={model}
              className="h-full"
            />
          </div>

          <div className="pointer-events-none absolute inset-x-0 bottom-[30%] z-10 flex justify-center">
            <span className="flex items-center gap-1.5 text-sm font-medium text-mute/80">
              <UnfoldHorizontal className="size-4 shrink-0" />
              Przeciągnij, żeby obrócić
            </span>
          </div>
        </div>

        <div className="relative z-10 mt-2 flex flex-col items-center gap-4">
          {showCapes && availableCapes && onDraftCapeChange ? (
            <CapePicker
              capes={availableCapes}
              draftCapeId={draftCapeId ?? null}
              onDraftCapeChange={onDraftCapeChange}
              loading={capesLoading}
              error={capesError}
              disabled={loading}
            />
          ) : null}

          {dirty ? (
            <div className="flex w-full flex-wrap items-center justify-center gap-2">
              <button
                type="button"
                disabled={loading}
                onClick={onReset}
                className="inline-flex items-center gap-2 rounded-xl border border-line bg-raised2 px-4 py-2.5 text-sm font-semibold text-ink transition hover:bg-white/6 disabled:opacity-50"
              >
                <RotateCcw className="size-4" />
                Cofnij
              </button>
              {onSaveToLibrary && (
                <button
                  type="button"
                  disabled={loading}
                  onClick={onSaveToLibrary}
                  className="inline-flex items-center gap-2 rounded-xl border border-accent/35 bg-accent/10 px-4 py-2.5 text-sm font-semibold text-ink transition hover:bg-accent/20 disabled:opacity-50"
                >
                  Zapisz
                </button>
              )}
              <button
                type="button"
                disabled={loading}
                onClick={onApply}
                className="inline-flex items-center gap-2 rounded-xl bg-good px-5 py-2.5 text-sm font-semibold text-black transition hover:brightness-110 disabled:opacity-50"
              >
                <Check className="size-4" />
                Zastosuj
              </button>
            </div>
          ) : (
            <button
              type="button"
              disabled={editDisabled || loading}
              onClick={onEdit}
              className="inline-flex items-center gap-2 rounded-xl border border-line bg-raised2 px-5 py-2.5 text-sm font-semibold text-ink transition hover:bg-white/6 disabled:opacity-50"
            >
              <Pencil className="size-4" />
              Edytuj skin
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
