import {
  Crop,
  House,
  Layers,
  Package,
  Server,
  Settings,
  User,
  X,
} from "lucide-react";
import { clsx } from "clsx";
import { pl } from "../locales/pl";
import { useApp, useActiveAccount } from "../stores/appStore";
import { useOctra, type OctraView, VIEW_LABELS } from "../stores/octraStore";

const NAV_ITEMS: { id: OctraView; icon: typeof House }[] = [
  { id: "home", icon: House },
  { id: "locker", icon: User },
  { id: "versions", icon: Layers },
  { id: "gallery", icon: Crop },
  { id: "host", icon: Server },
  { id: "store", icon: Package },
  { id: "settings", icon: Settings },
];

export function OctraMenu() {
  const open = useOctra((s) => s.octraMenuOpen);
  const setOpen = useOctra((s) => s.setOctraMenuOpen);
  const setView = useOctra((s) => s.setView);
  const openCreator = useOctra((s) => s.openInstanceCreator);
  const view = useOctra((s) => s.view);
  const acc = useActiveAccount();
  const appInfo = useApp((s) => s.appInfo);
  const instances = useApp((s) => s.instances);

  if (!open) return null;

  function go(target: OctraView) {
    setView(target);
    setOpen(false);
  }

  return (
    <div
      className="fixed inset-0 z-[60] grid place-items-center bg-black/55 p-4 backdrop-blur-[2px]"
      onMouseDown={() => setOpen(false)}
    >
      <div
        className="w-full max-w-lg overflow-hidden rounded-2xl border border-line bg-raised shadow-2xl"
        role="dialog"
        aria-modal="true"
        aria-label={pl.menu.title}
        onMouseDown={(e) => e.stopPropagation()}
      >
        <header className="flex items-center justify-between border-b border-line px-5 py-4">
          <div>
            <h2 className="text-base font-bold">{pl.menu.title}</h2>
            <p className="mt-0.5 text-[11px] text-mute">
              {pl.menu.subtitle} · v{appInfo?.version ?? "0.1.0"}
            </p>
          </div>
          <button
            type="button"
            className="grid h-8 w-8 place-items-center rounded-lg text-mute hover:bg-white/6 hover:text-ink"
            onClick={() => setOpen(false)}
            aria-label={pl.menu.close}
          >
            <X size={16} />
          </button>
        </header>

        <div className="max-h-[min(70vh,520px)] overflow-auto p-4">
          {acc && (
            <section className="mb-4 rounded-xl border border-line bg-raised2/50 px-3 py-2.5">
              <p className="text-[10px] font-semibold uppercase tracking-wider text-mute">
                {pl.menu.account}
              </p>
              <p className="mt-1 text-sm font-semibold">{acc.name}</p>
              <p className="text-[11px] text-mute">
                {acc.kind === "offline" ? pl.accounts.nonPremium : pl.accounts.premium}
              </p>
            </section>
          )}

          <section>
            <p className="mb-2 text-[10px] font-semibold uppercase tracking-wider text-mute">
              {pl.menu.navigation}
            </p>
            <div className="grid grid-cols-2 gap-1.5 sm:grid-cols-3">
              {NAV_ITEMS.map((item) => {
                const Icon = item.icon;
                const active = view === item.id;
                return (
                  <button
                    key={item.id}
                    type="button"
                    onClick={() => go(item.id)}
                    className={clsx(
                      "flex items-center gap-2 rounded-xl border px-3 py-2.5 text-left text-sm font-semibold transition",
                      active
                        ? "border-accent/50 bg-accent/15 text-ink"
                        : "border-line bg-raised2/40 text-mute hover:border-accent/30 hover:text-ink",
                    )}
                  >
                    <Icon size={16} strokeWidth={1.7} />
                    {VIEW_LABELS[item.id]}
                  </button>
                );
              })}
            </div>
          </section>

          <section className="mt-4">
            <p className="mb-2 text-[10px] font-semibold uppercase tracking-wider text-mute">
              {pl.menu.quickActions}
            </p>
            <div className="flex flex-wrap gap-2">
              <button
                type="button"
                className="rounded-full border border-line px-3 py-1.5 text-xs font-semibold text-mute hover:border-accent/40 hover:text-ink"
                onClick={() => {
                  openCreator();
                  setOpen(false);
                }}
              >
                {pl.versions.newProfile}
              </button>
              <button
                type="button"
                className="rounded-full border border-line px-3 py-1.5 text-xs font-semibold text-mute hover:border-accent/40 hover:text-ink"
                onClick={() => go("locker")}
              >
                {pl.nav.locker}
              </button>
              <button
                type="button"
                className="rounded-full border border-line px-3 py-1.5 text-xs font-semibold text-mute hover:border-accent/40 hover:text-ink"
                onClick={() => go("settings")}
              >
                {pl.nav.settings}
              </button>
            </div>
          </section>

          <section className="mt-4 rounded-xl border border-dashed border-line bg-raised2/30 px-3 py-2.5">
            <p className="text-[10px] font-semibold uppercase tracking-wider text-mute">
              {pl.menu.comingSoonSection}
            </p>
            <p className="mt-1 text-[11px] text-mute">
              {pl.nav.notify}, {pl.nav.relay}, {pl.menu.friends} — {pl.nav.comingSoon}
            </p>
          </section>

          <p className="mt-4 text-center text-[10px] text-mute">
            {instances.length} {pl.menu.profilesLabel} · {pl.menu.hint}
          </p>
        </div>
      </div>
    </div>
  );
}
