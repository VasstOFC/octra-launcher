import { X } from "lucide-react";
import { FriendsPanel } from "./FriendsPanel";
import { useOctra } from "../stores/octraStore";

export function FriendsDrawer() {
  const open = useOctra((s) => s.friendsOpen);
  const setFriendsOpen = useOctra((s) => s.setFriendsOpen);

  if (!open) return null;

  return (
    <>
      <button
        type="button"
        aria-label="Zamknij panel znajomych"
        className="fixed inset-0 z-40 bg-black/50"
        onClick={() => setFriendsOpen(false)}
      />
      <div className="fixed bottom-0 right-0 top-10 z-50 flex w-[280px] flex-col border-l border-line bg-raised shadow-xl">
        <div className="flex items-center justify-between border-b border-line px-3 py-2">
          <span className="text-sm font-semibold">Znajomi</span>
          <button
            type="button"
            className="grid h-8 w-8 place-items-center rounded-md text-mute hover:bg-white/6 hover:text-ink"
            onClick={() => setFriendsOpen(false)}
          >
            <X size={16} />
          </button>
        </div>
        <FriendsPanel embedded />
      </div>
    </>
  );
}
