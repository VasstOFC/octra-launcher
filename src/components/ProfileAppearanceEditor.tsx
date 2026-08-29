import { useEffect } from "react";
import { clsx } from "clsx";
import { ImageIcon, Trash2, Upload } from "lucide-react";
import { PROFILE_PALETTES, paletteById } from "../lib/profilePalettes";
import { galleryIconUrl } from "../lib/profileIconResolve";
import { LOADER_LABEL } from "../lib/format";
import { ProfileDefaultIcon } from "./ProfileDefaultIcon";
import { ProfileIconGallery } from "./ProfileIconGallery";
import type { Loader } from "../types";

export type ProfileIconDraft =
  | { kind: "default" }
  | { kind: "preset"; id: string }
  | { kind: "file"; previewUrl: string; bytes: Uint8Array };

type Props = {
  name: string;
  loader: Loader;
  gameVersion?: string;
  paletteId: string;
  onPaletteIdChange: (id: string) => void;
  icon: ProfileIconDraft;
  onIconChange: (icon: ProfileIconDraft) => void;
  wallpaperPreviewUrl?: string | null;
  onPickWallpaper?: () => void;
  onPickWallpaperFile?: (file: File) => void;
  onClearWallpaper?: () => void;
  wallpaperBusy?: boolean;
  disabled?: boolean;
  iconDisabled?: boolean;
};

export function ProfileAppearanceEditor({
  name,
  loader,
  gameVersion,
  paletteId,
  onPaletteIdChange,
  icon,
  onIconChange,
  wallpaperPreviewUrl,
  onPickWallpaper,
  onPickWallpaperFile,
  onClearWallpaper,
  wallpaperBusy,
  disabled,
  iconDisabled,
}: Props) {
  const palette = paletteById(paletteId);
  const iconsLocked = disabled || iconDisabled;

  function onIconFile(file: File | undefined) {
    if (!file || iconsLocked) return;
    const previewUrl = URL.createObjectURL(file);
    void file.arrayBuffer().then((buf) => {
      onIconChange({ kind: "file", previewUrl, bytes: new Uint8Array(buf) });
    });
  }

  useEffect(() => {
    return () => {
      if (icon.kind === "file") URL.revokeObjectURL(icon.previewUrl);
    };
  }, [icon]);

  return (
    <div className="space-y-5">
      <div>
        <p className="text-sm text-mute">Kolor tła na ekranie startowym i w liście profili.</p>
        <div className="mt-3 grid grid-cols-3 gap-2 sm:grid-cols-4">
          {PROFILE_PALETTES.map((p) => (
            <button
              key={p.id}
              type="button"
              disabled={disabled}
              onClick={() => onPaletteIdChange(p.id)}
              className={clsx(
                "overflow-hidden rounded-xl border p-2 text-left transition",
                paletteId === p.id
                  ? "border-accent ring-1 ring-accent/40"
                  : "border-line hover:border-accent/30",
                disabled && "opacity-50",
              )}
            >
              <div
                className="h-12 rounded-lg"
                style={{
                  background: `linear-gradient(135deg, ${p.c1} 0%, ${p.c2} 100%)`,
                }}
              />
              <span className="mt-1.5 block text-[10px] font-semibold">{p.name}</span>
            </button>
          ))}
        </div>
      </div>

      <div>
        <p className="text-sm font-semibold">Ikona profilu</p>
        <p className="mt-1 text-xs text-mute">
          Oficjalne tekstury z Minecraft. Możesz też wgrać własny PNG.
        </p>
        <div className="mt-3 flex flex-wrap gap-2">
          <button
            type="button"
            disabled={iconsLocked}
            onClick={() => onIconChange({ kind: "default" })}
            className={clsx(
              "rounded-lg px-3 py-1.5 text-xs font-semibold",
              icon.kind === "default"
                ? "bg-accent/25 text-ink ring-1 ring-accent/50"
                : "bg-raised2 text-mute hover:text-ink",
              iconsLocked && "opacity-50",
            )}
          >
            Domyślna (trawa)
          </button>
          <label
            className={clsx(
              "inline-flex cursor-pointer items-center gap-1.5 rounded-lg bg-raised2 px-3 py-1.5 text-xs font-semibold text-mute hover:text-ink",
              iconsLocked && "pointer-events-none opacity-50",
            )}
          >
            <Upload size={14} />
            Wgraj ikonę
            <input
              type="file"
              accept="image/png,image/jpeg,image/webp,image/gif"
              className="hidden"
              disabled={iconsLocked}
              onChange={(e) => {
                onIconFile(e.target.files?.[0]);
                e.target.value = "";
              }}
            />
          </label>
          {icon.kind === "file" || icon.kind === "preset" ? (
            <button
              type="button"
              disabled={iconsLocked}
              onClick={() => onIconChange({ kind: "default" })}
              className="inline-flex items-center gap-1 rounded-lg px-2 py-1.5 text-xs text-mute hover:text-danger"
            >
              <Trash2 size={13} />
              Reset ikony
            </button>
          ) : null}
        </div>

        <div className="mt-4">
          <p className="text-xs font-semibold text-mute">Galeria ikon</p>
          <div className="mt-2">
            <ProfileIconGallery
              disabled={iconsLocked}
              selectedId={
                icon.kind === "preset"
                  ? icon.id
                  : icon.kind === "default"
                    ? "grass"
                    : null
              }
              onSelect={(item) => onIconChange({ kind: "preset", id: item.id })}
            />
          </div>
        </div>
      </div>

      {(onPickWallpaper || onPickWallpaperFile) ? (
        <div>
          <p className="text-sm font-semibold">Tapeta profilu</p>
          <p className="mt-1 text-xs text-mute">Opcjonalny obraz na karcie profilu (ekran Start).</p>
          <div className="mt-3 flex flex-wrap gap-2">
            {onPickWallpaper ? (
              <button
                type="button"
                disabled={disabled || wallpaperBusy}
                onClick={onPickWallpaper}
                className="inline-flex items-center gap-1.5 rounded-lg bg-raised2 px-3 py-1.5 text-xs font-semibold text-mute hover:text-ink disabled:opacity-50"
              >
                <ImageIcon size={14} />
                {wallpaperPreviewUrl ? "Zmień tapetę" : "Wybierz tapetę"}
              </button>
            ) : (
              <label
                className={clsx(
                  "inline-flex cursor-pointer items-center gap-1.5 rounded-lg bg-raised2 px-3 py-1.5 text-xs font-semibold text-mute hover:text-ink",
                  disabled && "pointer-events-none opacity-50",
                )}
              >
                <ImageIcon size={14} />
                {wallpaperPreviewUrl ? "Zmień tapetę" : "Wybierz tapetę"}
                <input
                  type="file"
                  accept="image/png,image/jpeg,image/webp"
                  className="hidden"
                  disabled={disabled}
                  onChange={(e) => {
                    const file = e.target.files?.[0];
                    if (file) onPickWallpaperFile?.(file);
                    e.target.value = "";
                  }}
                />
              </label>
            )}
            {wallpaperPreviewUrl && onClearWallpaper ? (
              <button
                type="button"
                disabled={disabled || wallpaperBusy}
                onClick={onClearWallpaper}
                className="inline-flex items-center gap-1 rounded-lg px-2 py-1.5 text-xs text-mute hover:text-danger"
              >
                <Trash2 size={13} />
                Usuń tapetę
              </button>
            ) : null}
          </div>
        </div>
      ) : null}

      <div className="overflow-hidden rounded-2xl border border-line">
        <div
          className="relative h-36"
          style={{
            background: wallpaperPreviewUrl
              ? `url(${wallpaperPreviewUrl}) center/cover no-repeat`
              : `linear-gradient(135deg, ${palette.c1} 0%, ${palette.c2}55 45%, #0f0f12 100%)`,
          }}
        >
          {!wallpaperPreviewUrl ? (
            <div className="absolute inset-0 opacity-[0.12] [background-image:radial-gradient(circle_at_20%_30%,white_0,transparent_45%)]" />
          ) : (
            <div className="absolute inset-0 bg-black/35" />
          )}
          <div className="absolute right-[20%] top-1/2 flex -translate-y-1/2 flex-col items-center">
            <div className="grid h-14 w-14 place-items-center overflow-hidden rounded-xl border border-white/15 bg-black/25">
              {icon.kind === "file" ? (
                <img
                  src={icon.previewUrl}
                  alt=""
                  className="h-full w-full object-cover [image-rendering:pixelated]"
                />
              ) : icon.kind === "preset" ? (
                <img
                  src={galleryIconUrl(icon.id)}
                  alt=""
                  className="h-full w-full object-cover [image-rendering:pixelated]"
                  onError={(e) => {
                    (e.target as HTMLImageElement).src = galleryIconUrl("grass");
                  }}
                />
              ) : (
                <ProfileDefaultIcon size={44} />
              )}
            </div>
            <span className="mt-2 text-[10px] font-semibold uppercase tracking-widest text-white/50">
              {LOADER_LABEL[loader]}
            </span>
          </div>
        </div>
        <div className="border-t border-line bg-raised2/50 px-4 py-3 text-sm">
          <p className="font-semibold">{name.trim() || "—"}</p>
          <p className="mt-0.5 text-xs text-mute">
            {LOADER_LABEL[loader]} · {gameVersion ?? "—"}
          </p>
        </div>
      </div>
    </div>
  );
}
