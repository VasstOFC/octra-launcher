import { clsx } from "clsx";

import {

  Bell,

  Crop,

  House,

  Layers,

  MessageCircle,

  Package,

  Server,

  Settings,

  User,

  Users,

} from "lucide-react";

import { Mark } from "./WindowButtons";

import { AccountMenu } from "./AccountMenu";

import { useApp } from "../stores/appStore";

import { useOctra, type OctraView } from "../stores/octraStore";

import { pl } from "../locales/pl";



const DISABLED_NAV = new Set<OctraView>(["notify", "relay"]);



const ITEMS: { id: OctraView; icon: typeof House; label: string }[] = [

  { id: "home", icon: House, label: pl.nav.home },

  { id: "locker", icon: User, label: pl.nav.locker },

  { id: "notify", icon: Bell, label: pl.nav.notify },

  { id: "relay", icon: MessageCircle, label: pl.nav.relay },

  { id: "versions", icon: Layers, label: pl.nav.versions },

  { id: "gallery", icon: Crop, label: pl.nav.gallery },

  { id: "host", icon: Server, label: pl.nav.host },

];



export function Rail() {

  const view = useOctra((s) => s.view);

  const setView = useOctra((s) => s.setView);

  const progress = useApp((s) => s.progress);



  const pct =

    progress && progress.total > 0

      ? Math.min(100, Math.round((100 * progress.current) / progress.total))

      : progress

        ? 12

        : null;



  return (

    <aside className="flex w-16 shrink-0 flex-col items-center gap-0.5 border-r border-line bg-raised py-3">

      <button

        className="relative mb-2 grid h-9 w-9 place-items-center"

        onClick={() => setView("home")}

        aria-label="Octra"

      >

        <Mark size={32} />

        {pct !== null && (

          <span className="absolute -bottom-1 left-1/2 -translate-x-1/2 rounded-full bg-good px-1.5 py-0.5 text-[9px] font-bold leading-none text-bg">

            {pct}%

          </span>

        )}

      </button>

      {ITEMS.slice(0, 4).map((it) => (

        <RailBtn

          key={it.id}

          {...it}

          active={view === it.id}

          disabled={DISABLED_NAV.has(it.id)}

          disabledHint={pl.nav.comingSoon}

          onClick={() => setView(it.id)}

        />

      ))}

      <div className="my-1.5 h-px w-7 bg-line" />

      {ITEMS.slice(4).map((it) => (

        <RailBtn key={it.id} {...it} active={view === it.id} onClick={() => setView(it.id)} />

      ))}

      <div className="mt-auto flex flex-col items-center gap-0.5">

        <AccountMenu />

        <RailBtn

          id="store"

          icon={Package}

          label={pl.nav.store}

          active={view === "store"}

          onClick={() => setView("store")}

        />

        <RailBtn

          id="friends"

          icon={Users}

          label={pl.nav.friends}

          active={false}

          disabled

          disabledHint={pl.nav.comingSoon}

          onClick={() => undefined}

        />

        <RailBtn

          id="settings"

          icon={Settings}

          label={pl.nav.settings}

          active={view === "settings"}

          onClick={() => setView("settings")}

        />

      </div>

    </aside>

  );

}



function RailBtn({

  icon: Icon,

  label,

  active,

  onClick,

  disabled,

  disabledHint,

}: {

  id: string;

  icon: typeof House;

  label: string;

  active: boolean;

  onClick: () => void;

  disabled?: boolean;

  disabledHint?: string;

}) {

  return (

    <div className="group relative">

      <button

        type="button"

        title={disabled ? disabledHint : label}

        aria-label={label}

        aria-disabled={disabled}

        onClick={disabled ? undefined : onClick}

        className={clsx(

          "relative grid h-10 w-10 place-items-center rounded-lg transition",

          disabled

            ? "cursor-not-allowed opacity-30"

            : active

              ? "bg-white/8 text-accent"

              : "text-mute hover:bg-white/5 hover:text-ink",

        )}

      >

        <Icon size={18} strokeWidth={1.7} />

      </button>

      {disabled && disabledHint && (

        <span className="pointer-events-none absolute left-full top-1/2 z-50 ml-2 hidden w-max max-w-[180px] -translate-y-1/2 rounded-lg border border-line bg-raised px-2.5 py-1.5 text-[10px] font-medium text-mute shadow-lg group-hover:block">

          {disabledHint}

        </span>

      )}

    </div>

  );

}


