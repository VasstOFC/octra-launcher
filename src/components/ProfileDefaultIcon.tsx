import { clsx } from "clsx";
import { DEFAULT_GALLERY_ICON_ID, galleryIconUrl } from "../lib/profileIconResolve";

export function ProfileDefaultIcon({
  className,
  size = 56,
}: {
  className?: string;
  size?: number;
}) {
  return (
    <img
      src={galleryIconUrl(DEFAULT_GALLERY_ICON_ID)}
      alt=""
      draggable={false}
      width={size}
      height={size}
      className={clsx("rounded-lg [image-rendering:pixelated]", className)}
    />
  );
}
