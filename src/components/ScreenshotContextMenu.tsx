import { useEffect, useRef } from "react";
import { clsx } from "clsx";
import { Download, FolderOpen, ZoomIn } from "lucide-react";
import { pl } from "../locales/pl";
import type { GlobalScreenshotEntry } from "../types";

type Props = {
  x: number;
  y: number;
  item: GlobalScreenshotEntry;
  onClose: () => void;
  onPreview: () => void;
  onDownload: () => void;
  onOpenFolder: () => void;
};

export function ScreenshotContextMenu({
  x,
  y,
  item,
  onClose,
  onPreview,
  onDownload,
  onOpenFolder,
}: Props) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function onDoc(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    document.addEventListener("mousedown", onDoc);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDoc);
      document.removeEventListener("keydown", onKey);
    };
  }, [onClose]);

  const actions = [
    { id: "preview", label: pl.gallery.preview, icon: ZoomIn, onClick: onPreview },
    { id: "download", label: pl.gallery.download, icon: Download, onClick: onDownload },
    {
      id: "folder",
      label: pl.gallery.openFolder,
      icon: FolderOpen,
      onClick: onOpenFolder,
    },
  ];

  return (
    <div
      ref={ref}
      className="fixed z-50 overflow-hidden rounded-xl border border-line bg-raised shadow-2xl"
      style={{ left: x, top: y }}
      role="menu"
    >
      <p className="truncate border-b border-line bg-raised2/60 px-3 py-2 text-[10px] font-semibold uppercase tracking-wider text-mute">
        {item.name}
      </p>
      <div className="flex">
        {actions.map((action, index) => {
          const Icon = action.icon;
          return (
            <button
              key={action.id}
              type="button"
              role="menuitem"
              title={action.label}
              onClick={action.onClick}
              className={clsx(
                "flex min-w-[4.5rem] flex-col items-center justify-center gap-1 px-2.5 py-2.5 text-center transition hover:bg-white/6",
                index > 0 && "border-l border-line",
              )}
            >
              <Icon size={16} strokeWidth={1.75} className="text-mute" />
              <span className="text-[9px] font-semibold leading-tight text-ink">
                {action.label}
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
