import { clsx } from "clsx";
import type { Cape } from "../lib/skins";
import { CapeThumbnail } from "./CapeThumbnail";

export function CapePicker({
  capes,
  draftCapeId,
  onDraftCapeChange,
  loading,
  error,
  disabled,
}: {
  capes: Cape[];
  draftCapeId: string | null;
  onDraftCapeChange: (capeId: string | null) => void;
  loading?: boolean;
  error?: string | null;
  disabled?: boolean;
}) {
  const sorted = [...capes].sort((a, b) =>
    (a.name || a.id).localeCompare(b.name || b.id, undefined, { sensitivity: "base" }),
  );

  return (
    <div className="w-full">
      <p className="text-xs font-semibold uppercase tracking-wider text-mute">Peleryna</p>
      <div className="mt-2 grid max-h-40 grid-cols-4 gap-2 overflow-y-auto sm:grid-cols-5">
        <button
          type="button"
          disabled={disabled}
          onClick={() => onDraftCapeChange(null)}
          className={clsx(
            "flex h-[5.5rem] w-full flex-col items-center justify-center rounded-xl border bg-raised2 transition",
            !draftCapeId
              ? "border-good ring-2 ring-good/50"
              : "border-line hover:border-accent/40",
            disabled && "pointer-events-none opacity-50",
          )}
        >
          <span className="text-lg text-mute">×</span>
          <span className="mt-1 text-[9px] text-mute">Brak</span>
        </button>
        {sorted.map((cape) => (
          <button
            key={cape.id}
            type="button"
            disabled={disabled}
            title={cape.name || cape.id}
            onClick={() => onDraftCapeChange(cape.id)}
            className={clsx(
              "h-[5.5rem] w-full overflow-hidden rounded-xl border p-0 transition",
              draftCapeId === cape.id
                ? "border-good ring-2 ring-good/50"
                : "border-line hover:border-accent/40",
              disabled && "pointer-events-none opacity-50",
            )}
          >
            <CapeThumbnail
              textureUrl={cape.texture}
              alt={cape.name || cape.id}
              selected={draftCapeId === cape.id}
            />
          </button>
        ))}
      </div>
      {loading && <p className="mt-2 text-[10px] text-mute">Ładowanie peleryn…</p>}
      {error && <p className="mt-2 text-[10px] text-danger">{error}</p>}
      {!loading && !error && capes.length === 0 && (
        <p className="mt-2 text-[10px] text-mute">To konto nie ma peleryn Mojang.</p>
      )}
    </div>
  );
}
