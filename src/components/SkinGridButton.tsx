import { Check } from "lucide-react";
import { clsx } from "clsx";
import { SkinThumbnail } from "./SkinThumbnail";

export function SkinGridButton({
  selected,
  active,
  previewing,
  disabled,
  onClick,
  skinPngDataUrl,
  pngBase64,
  textureKey,
  variant = "classic",
  alt,
  className,
}: {
  selected?: boolean;
  active?: boolean;
  previewing?: boolean;
  disabled?: boolean;
  onClick: () => void;
  skinPngDataUrl?: string | null;
  pngBase64?: string | null;
  textureKey?: string | null;
  variant?: "slim" | "classic";
  alt?: string;
  className?: string;
}) {
  const b64 =
    pngBase64 ??
    (skinPngDataUrl?.startsWith("data:image/png;base64,")
      ? skinPngDataUrl.replace(/^data:image\/png;base64,/, "")
      : skinPngDataUrl?.startsWith("data:")
        ? skinPngDataUrl
        : undefined);

  return (
    <div
      className={clsx(
        "skin-grid-button group relative flex h-full w-full items-end justify-center overflow-hidden border border-solid transition-[border-color,box-shadow,filter] duration-200",
        selected
          ? "border-white/40 brightness-110"
          : "border-line/80 hover:border-line hover:brightness-105",
        disabled && "pointer-events-none opacity-65",
        previewing && !selected && "border-accent/50 ring-1 ring-accent/30",
        className,
      )}
    >
      <button
        type="button"
        disabled={disabled}
        aria-pressed={selected}
        aria-label={alt ? `Wybierz ${alt}` : "Wybierz skin"}
        onClick={onClick}
        className="absolute inset-0 z-10 cursor-pointer border-none bg-transparent p-0 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      />

      {active && !selected && (
        <span className="pointer-events-none absolute right-3 top-3 z-20 size-3 rounded-full border-2 border-raised bg-good" />
      )}

      {selected && (
        <span className="pointer-events-none absolute right-3 top-3 z-20 flex size-6 items-center justify-center rounded-full bg-white">
          <Check className="size-4 text-black" strokeWidth={3} />
        </span>
      )}

      <div className="pointer-events-none relative z-0 mb-px grid h-[95%] w-full place-items-stretch">
        {b64 || textureKey ? (
          <SkinThumbnail
            textureKey={textureKey ?? undefined}
            pngBase64={b64}
            variant={variant}
            alt={alt ?? ""}
            className="h-full w-full object-contain drop-shadow-[0_4px_8px_rgba(0,0,0,0.4)] [image-rendering:pixelated]"
          />
        ) : (
          <div className="grid h-full place-items-center text-[10px] text-mute">{alt}</div>
        )}
      </div>

      <div
        className="pointer-events-none absolute inset-0 z-[5] bg-gradient-to-b from-transparent to-[rgba(37,39,45,0.2)]"
        aria-hidden
      />
    </div>
  );
}

export function SkinAddCard({
  onFile,
  disabled,
  className,
}: {
  onFile: (file: File) => void;
  disabled?: boolean;
  className?: string;
}) {
  return (
    <label
      className={clsx(
        "skin-add-card flex h-full w-full cursor-pointer flex-col items-center justify-center border border-dashed text-center transition-colors duration-200",
        disabled ? "pointer-events-none opacity-60" : "hover:border-accent/40 hover:bg-raised2/80",
        className,
      )}
      onDragOver={(e) => {
        e.preventDefault();
        e.dataTransfer.dropEffect = "copy";
      }}
      onDrop={(e) => {
        e.preventDefault();
        const f = e.dataTransfer.files[0];
        if (f?.type === "image/png") onFile(f);
      }}
    >
      <span className="text-2xl font-light text-mute">+</span>
      <span className="mt-1 text-sm font-semibold text-ink">Dodaj skin</span>
      <span className="mt-0.5 text-xs text-mute">Przeciągnij i upuść</span>
      <input
        type="file"
        accept="image/png"
        className="hidden"
        disabled={disabled}
        onChange={(e) => {
          const f = e.target.files?.[0];
          if (f) onFile(f);
        }}
      />
    </label>
  );
}
