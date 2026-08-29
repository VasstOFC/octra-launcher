import { useEffect, useRef } from "react";
import { Trash2 } from "lucide-react";
import { pl } from "../locales/pl";
import type { Instance } from "../types";

export function ProfileContextMenu({
  x,
  y,
  inst,
  onClose,
  onDelete,
}: {
  x: number;
  y: number;
  inst: Instance;
  onClose: () => void;
  onDelete: () => void;
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

  return (
    <div
      ref={ref}
      className="fixed z-50 min-w-[180px] overflow-hidden rounded-xl border border-line bg-raised py-1 shadow-xl"
      style={{ left: x, top: y }}
      role="menu"
    >
      <p className="truncate px-3 py-1.5 text-[10px] font-semibold uppercase tracking-wider text-mute">
        {inst.name}
      </p>
      <button
        type="button"
        role="menuitem"
        className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm text-danger hover:bg-danger/10"
        onClick={onDelete}
      >
        <Trash2 size={15} />
        {pl.versions.deleteProfile}
      </button>
    </div>
  );
}
