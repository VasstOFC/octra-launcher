import { useEffect, useRef } from "react";
import { clsx } from "clsx";
import {
  FolderOpen,
  PackageCheck,
  Palette,
  Play,
  Settings2,
  SquarePen,
  Trash2,
} from "lucide-react";
import { pl } from "../locales/pl";
import type { Instance } from "../types";

type Action = {
  id: string;
  label: string;
  icon: typeof Play;
  onClick: () => void;
  danger?: boolean;
};

export function ProfileContextMenu({
  x,
  y,
  inst,
  onClose,
  onPlay,
  onMods,
  onAppearance,
  onAdvanced,
  onOpenFolder,
  onDelete,
  onCheckPackUpdate,
}: {
  x: number;
  y: number;
  inst: Instance;
  onClose: () => void;
  onPlay: () => void;
  onMods: () => void;
  onAppearance: () => void;
  onAdvanced: () => void;
  onOpenFolder: () => void;
  onDelete: () => void;
  onCheckPackUpdate?: () => void;
}) {
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

  const actions: Action[] = [
    { id: "play", label: pl.versions.play, icon: Play, onClick: onPlay },
    { id: "mods", label: pl.versions.mods, icon: SquarePen, onClick: onMods },
    {
      id: "appearance",
      label: pl.versions.appearance,
      icon: Palette,
      onClick: onAppearance,
    },
    {
      id: "advanced",
      label: pl.versions.advanced,
      icon: Settings2,
      onClick: onAdvanced,
    },
    ...(inst.packLocked && onCheckPackUpdate
      ? [
          {
            id: "pack-update",
            label: pl.versions.checkPackUpdate,
            icon: PackageCheck,
            onClick: onCheckPackUpdate,
          } satisfies Action,
        ]
      : []),
    {
      id: "folder",
      label: pl.versions.openFolder,
      icon: FolderOpen,
      onClick: onOpenFolder,
    },
    {
      id: "delete",
      label: pl.versions.deleteProfile,
      icon: Trash2,
      onClick: onDelete,
      danger: true,
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
        {inst.name}
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
                "flex min-w-[4.25rem] flex-col items-center justify-center gap-1 px-2 py-2.5 text-center transition",
                index > 0 && "border-l border-line",
                action.danger
                  ? "text-danger hover:bg-danger/12"
                  : "text-ink hover:bg-white/6",
              )}
            >
              <Icon size={16} strokeWidth={1.75} className={action.danger ? "" : "text-mute"} />
              <span className="text-[9px] font-semibold leading-tight">{action.label}</span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
