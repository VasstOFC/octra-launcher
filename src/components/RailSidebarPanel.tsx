import { clsx } from "clsx";
import { Clock3, Layers, Sparkles } from "lucide-react";
import { formatPlayTime } from "../lib/format";
import { pl } from "../locales/pl";
import { useApp } from "../stores/appStore";
import { useOctra } from "../stores/octraStore";
import { ProfileIcon } from "./ProfileIcon";

type Props = {
  expanded: boolean;
};

export function RailSidebarPanel({ expanded }: Props) {
  const instances = useApp((s) => s.instances);
  const selectedId = useOctra((s) => s.selectedId);
  const select = useOctra((s) => s.selectInstance);
  const setView = useOctra((s) => s.setView);

  const sorted = instances
    .slice()
    .sort((a, b) => (b.lastPlayed ?? "").localeCompare(a.lastPlayed ?? ""));
  const totalPlay = instances.reduce((n, i) => n + (i.playTimeSecs || 0), 0);
  const preview = sorted.slice(0, 5);

  if (!expanded) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-2 py-2">
        <div className="grid h-9 w-9 place-items-center rounded-xl border border-line bg-raised2/60 text-accent">
          <Layers size={16} />
        </div>
        <span className="text-[10px] font-bold text-mute">{instances.length}</span>
      </div>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-2xl border border-line bg-raised2/35 p-2.5">
      <div className="shrink-0 rounded-xl border border-accent/15 bg-gradient-to-br from-accent/10 via-transparent to-transparent p-3">
        <div className="flex items-center gap-2 text-accent">
          <Sparkles size={14} />
          <span className="text-[10px] font-bold uppercase tracking-wider">
            {pl.rail.library}
          </span>
        </div>
        <p className="mt-2 text-lg font-extrabold text-ink">{instances.length}</p>
        <p className="text-[11px] text-mute">
          {instances.length === 1 ? "profil w bibliotece" : "profilów w bibliotece"}
        </p>
        <div className="mt-2 flex items-center gap-1.5 text-[10px] text-mute">
          <Clock3 size={12} />
          <span>{formatPlayTime(totalPlay)} łącznie</span>
        </div>
      </div>

      {preview.length > 0 ? (
        <div className="mt-2 min-h-0 flex-1 overflow-y-auto news-scroll-y">
          <p className="mb-1.5 px-1 text-[9px] font-bold uppercase tracking-wider text-mute">
            {pl.rail.quickSwitch}
          </p>
          <ul className="space-y-1">
            {preview.map((inst) => {
              const active = selectedId === inst.id;
              return (
                <li key={inst.id}>
                  <button
                    type="button"
                    onClick={() => {
                      select(inst.id);
                      setView("home");
                    }}
                    className={clsx(
                      "flex w-full items-center gap-2 rounded-xl px-2 py-1.5 text-left transition",
                      active
                        ? "bg-accent/15 ring-1 ring-accent/30"
                        : "hover:bg-white/5",
                    )}
                  >
                    <div className="grid h-7 w-7 shrink-0 place-items-center overflow-hidden rounded-lg border border-white/10 bg-black/25 p-0.5">
                      <ProfileIcon inst={inst} size={22} />
                    </div>
                    <span className="min-w-0 flex-1 truncate text-[11px] font-medium text-ink">
                      {inst.name}
                    </span>
                  </button>
                </li>
              );
            })}
          </ul>
        </div>
      ) : (
        <button
          type="button"
          onClick={() => setView("versions")}
          className="mt-3 rounded-xl border border-dashed border-line px-3 py-4 text-center text-[11px] text-mute hover:border-accent/35 hover:text-ink"
        >
          {pl.rail.emptyLibrary}
        </button>
      )}
    </div>
  );
}
