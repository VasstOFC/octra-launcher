import { useEffect, useState } from "react";
import { clsx } from "clsx";
import {
  Bell,
  Crop,
  Globe,
  House,
  Layers,
  Package,
  PanelLeftClose,
  PanelLeftOpen,
  Server,
  Settings,
  User,
  Users,
} from "lucide-react";
import { Mark } from "./WindowButtons";
import { AccountMenu } from "./AccountMenu";
import { RailSidebarPanel } from "./RailSidebarPanel";
import { useApp } from "../stores/appStore";
import { useOctra, type OctraView } from "../stores/octraStore";
import { pl } from "../locales/pl";

const RAIL_EXPANDED_KEY = "octra.railExpanded";

const DISABLED_NAV = new Set<OctraView>(["notify"]);

const ITEMS: { id: OctraView; icon: typeof House; label: string }[] = [
  { id: "home", icon: House, label: pl.nav.home },
  { id: "locker", icon: User, label: pl.nav.locker },
  { id: "notify", icon: Bell, label: pl.nav.notify },
  { id: "versions", icon: Layers, label: pl.nav.versions },
  { id: "gallery", icon: Crop, label: pl.nav.gallery },
  { id: "host", icon: Server, label: pl.nav.host },
  { id: "servers", icon: Globe, label: pl.nav.servers },
];

export function Rail() {
  const view = useOctra((s) => s.view);
  const setView = useOctra((s) => s.setView);
  const progress = useApp((s) => s.progress);
  const [expanded, setExpanded] = useState(
    () => localStorage.getItem(RAIL_EXPANDED_KEY) !== "0",
  );

  useEffect(() => {
    localStorage.setItem(RAIL_EXPANDED_KEY, expanded ? "1" : "0");
  }, [expanded]);

  const pct =
    progress && progress.total > 0
      ? Math.min(100, Math.round((100 * progress.current) / progress.total))
      : progress
        ? 12
        : null;

  return (
    <aside
      className={clsx(
        "rail-surface flex shrink-0 flex-col border-r border-line py-3 transition-[width] duration-200 ease-out",
        expanded ? "w-[15.5rem] px-2.5" : "w-[4.75rem] items-center px-1.5",
      )}
    >
      <button
        type="button"
        className={clsx(
          "relative mb-3 flex shrink-0 items-center rounded-xl transition hover:bg-white/5",
          expanded ? "h-12 w-full gap-3 px-2.5" : "h-11 w-11 justify-center",
        )}
        onClick={() => setView("home")}
        aria-label="Octra"
      >
        <Mark size={expanded ? 34 : 30} />
        {expanded ? (
          <div className="min-w-0 text-left">
            <p className="truncate text-sm font-semibold tracking-tight text-ink">Octra</p>
            <p className="text-[10px] text-mute">Launcher</p>
          </div>
        ) : null}
        {pct !== null && (
          <span
            className={clsx(
              "absolute rounded-full bg-good px-1.5 py-0.5 text-[9px] font-bold leading-none text-bg",
              expanded ? "right-2 top-2" : "-bottom-1 left-1/2 -translate-x-1/2",
            )}
          >
            {pct}%
          </span>
        )}
      </button>

      <nav
        className={clsx(
          "flex flex-col gap-0.5",
          expanded ? "w-full" : "items-center",
        )}
        aria-label="Nawigacja"
      >
        {ITEMS.slice(0, 3).map((it) => (
          <RailBtn
            key={it.id}
            {...it}
            expanded={expanded}
            active={view === it.id}
            disabled={DISABLED_NAV.has(it.id)}
            disabledHint={pl.nav.comingSoon}
            onClick={() => setView(it.id)}
          />
        ))}

        <div className={clsx("my-2 h-px bg-line", expanded ? "mx-1 w-auto" : "w-9")} />

        {ITEMS.slice(3).map((it) => (
          <RailBtn
            key={it.id}
            {...it}
            expanded={expanded}
            active={view === it.id}
            onClick={() => setView(it.id)}
          />
        ))}
      </nav>

      <div className="my-2 flex min-h-0 flex-1 flex-col">
        <RailSidebarPanel expanded={expanded} />
      </div>

      <div
        className={clsx(
          "flex shrink-0 flex-col gap-0.5",
          expanded ? "w-full" : "items-center",
        )}
      >
        <AccountMenu expanded={expanded} />

        <RailBtn
          id="store"
          icon={Package}
          label={pl.nav.store}
          expanded={expanded}
          active={view === "store"}
          onClick={() => setView("store")}
        />
        <RailBtn
          id="friends"
          icon={Users}
          label={pl.nav.friends}
          expanded={expanded}
          active={false}
          disabled
          disabledHint={pl.nav.comingSoon}
          onClick={() => undefined}
        />
        <RailBtn
          id="settings"
          icon={Settings}
          label={pl.nav.settings}
          expanded={expanded}
          active={view === "settings"}
          onClick={() => setView("settings")}
        />

        <div className={clsx("my-1 h-px bg-line", expanded ? "mx-1" : "w-9")} />

        <div className="group relative w-full">
          <button
            type="button"
            onClick={() => setExpanded((v) => !v)}
            title={expanded ? pl.nav.collapseRail : pl.nav.expandRail}
            aria-label={expanded ? pl.nav.collapseRail : pl.nav.expandRail}
            className={clsx(
              "flex items-center rounded-xl text-mute transition hover:bg-white/5 hover:text-ink",
              expanded ? "h-11 w-full gap-3 px-3" : "h-11 w-11 justify-center",
            )}
          >
            {expanded ? (
              <PanelLeftClose size={20} strokeWidth={1.75} />
            ) : (
              <PanelLeftOpen size={20} strokeWidth={1.75} />
            )}
            {expanded ? (
              <span className="truncate text-[13px] font-medium">{pl.nav.collapseRail}</span>
            ) : null}
          </button>
          {!expanded ? (
            <span className="pointer-events-none absolute left-full top-1/2 z-50 ml-2 hidden w-max -translate-y-1/2 rounded-lg border border-line bg-raised px-2.5 py-1.5 text-[11px] font-medium text-ink shadow-lg group-hover:block">
              {pl.nav.expandRail}
            </span>
          ) : null}
        </div>
      </div>
    </aside>
  );
}

function RailBtn({
  icon: Icon,
  label,
  active,
  expanded,
  onClick,
  disabled,
  disabledHint,
}: {
  id: string;
  icon: typeof House;
  label: string;
  active: boolean;
  expanded: boolean;
  onClick: () => void;
  disabled?: boolean;
  disabledHint?: string;
}) {
  return (
    <div className="group relative w-full">
      <button
        type="button"
        title={disabled ? disabledHint : label}
        aria-label={label}
        aria-disabled={disabled}
        onClick={disabled ? undefined : onClick}
        className={clsx(
          "relative flex items-center rounded-xl transition",
          expanded ? "h-11 w-full gap-3 px-3" : "h-11 w-11 justify-center",
          disabled
            ? "cursor-not-allowed opacity-35"
            : active
              ? "bg-accent/12 text-accent ring-1 ring-accent/25"
              : "text-mute hover:bg-white/5 hover:text-ink",
        )}
      >
        {active && !expanded ? (
          <span className="absolute left-0 top-1/2 h-6 w-0.5 -translate-y-1/2 rounded-full bg-accent" />
        ) : null}
        <Icon size={21} strokeWidth={1.75} className="shrink-0" />
        {expanded ? (
          <span className="truncate text-[13px] font-medium">{label}</span>
        ) : null}
      </button>
      {!expanded && disabled && disabledHint ? (
        <span className="pointer-events-none absolute left-full top-1/2 z-50 ml-2 hidden w-max max-w-[200px] -translate-y-1/2 rounded-lg border border-line bg-raised px-2.5 py-1.5 text-[11px] font-medium text-mute shadow-lg group-hover:block">
          {disabledHint}
        </span>
      ) : null}
      {!expanded && !disabled ? (
        <span className="pointer-events-none absolute left-full top-1/2 z-50 ml-2 hidden w-max -translate-y-1/2 rounded-lg border border-line bg-raised px-2.5 py-1.5 text-[11px] font-medium text-ink shadow-lg group-hover:block">
          {label}
        </span>
      ) : null}
    </div>
  );
}
