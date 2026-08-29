import { useState } from "react";
import { clsx } from "clsx";
import {
  galleryIconUrl,
  DEFAULT_GALLERY_ICON_ID,
} from "../lib/profileIconResolve";
import {
  PROFILE_ICON_CATEGORIES,
  galleryIconsForCategory,
  type ProfileGalleryIcon,
} from "../lib/profileIconGallery";

type Props = {
  selectedId: string | null;
  onSelect: (icon: ProfileGalleryIcon) => void;
  disabled?: boolean;
};

export function ProfileIconGallery({ selectedId, onSelect, disabled }: Props) {
  const [category, setCategory] = useState(PROFILE_ICON_CATEGORIES[0]!.id);
  const icons = galleryIconsForCategory(category);
  const activeId = selectedId ?? DEFAULT_GALLERY_ICON_ID;

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap gap-1.5">
        {PROFILE_ICON_CATEGORIES.map((cat) => (
          <button
            key={cat.id}
            type="button"
            disabled={disabled}
            onClick={() => setCategory(cat.id)}
            className={clsx(
              "rounded-full px-2.5 py-1 text-[10px] font-semibold transition",
              category === cat.id
                ? "bg-accent/25 text-ink ring-1 ring-accent/45"
                : "bg-raised2 text-mute hover:text-ink",
            )}
          >
            {cat.label}
          </button>
        ))}
      </div>
      <div className="grid max-h-44 grid-cols-6 gap-2 overflow-y-auto pr-1 sm:grid-cols-8 news-scroll-y">
        {icons.map((icon) => (
          <button
            key={icon.id}
            type="button"
            disabled={disabled}
            title={icon.label}
            onClick={() => onSelect(icon)}
            className={clsx(
              "grid aspect-square place-items-center rounded-xl border bg-raised2/80 p-1.5 transition hover:border-accent/40",
              activeId === icon.id
                ? "border-accent ring-2 ring-accent/40"
                : "border-line",
              disabled && "opacity-50",
            )}
          >
            <img
              src={galleryIconUrl(icon.id)}
              alt=""
              className="h-full w-full [image-rendering:pixelated]"
              draggable={false}
              onError={(e) => {
                (e.target as HTMLImageElement).src = galleryIconUrl("grass");
              }}
            />
          </button>
        ))}
      </div>
    </div>
  );
}
