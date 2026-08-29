import { useEffect, useRef } from "react";
import { LogOut, UserRound } from "lucide-react";
import { pl } from "../locales/pl";
import type { Account } from "../types";

export function AccountContextMenu({
  x,
  y,
  acc,
  isActive,
  onClose,
  onSwitch,
  onRemove,
}: {
  x: number;
  y: number;
  acc: Account;
  isActive: boolean;
  onClose: () => void;
  onSwitch: () => void;
  onRemove: () => void;
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

  const typeLabel =
    acc.kind === "offline" ? pl.accounts.nonPremium : pl.accounts.premium;

  return (
    <div
      ref={ref}
      className="fixed z-[60] min-w-[200px] overflow-hidden rounded-xl border border-line bg-raised py-1 shadow-xl"
      style={{ left: x, top: y }}
      role="menu"
    >
      <p className="truncate px-3 py-1.5 text-sm font-semibold">{acc.name}</p>
      <p className="truncate px-3 pb-1 text-[10px] text-mute">{typeLabel}</p>
      {!isActive && (
        <button
          type="button"
          role="menuitem"
          className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm hover:bg-white/6"
          onClick={onSwitch}
        >
          <UserRound size={15} />
          {pl.accounts.contextSwitch}
        </button>
      )}
      <button
        type="button"
        role="menuitem"
        className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm text-danger hover:bg-danger/10"
        onClick={onRemove}
      >
        <LogOut size={15} />
        {pl.accounts.contextLogout}
      </button>
    </div>
  );
}
