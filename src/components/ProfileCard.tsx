import { clsx } from "clsx";
import type { ReactNode } from "react";
import { Play, Settings2, SquarePen } from "lucide-react";
import {
  formatDate,
  formatPlayTime,
  instanceAccent,
  LOADER_LABEL,
  loaderChipClass,
} from "../lib/format";
import { useProfileWallpaper } from "../lib/useProfileWallpaper";
import type { Instance } from "../types";
import { ProfileIcon } from "./ProfileIcon";

type Props = {
  inst: Instance;
  active?: boolean;
  onClick?: () => void;
  onPlay?: () => void;
  onEdit?: () => void;
  onSettings?: () => void;
  variant?: "grid" | "compact" | "library";
  showPlaytime?: boolean;
  className?: string;
};

export function ProfileCard({
  inst,
  active,
  onClick,
  onPlay,
  onEdit,
  onSettings,
  variant = "grid",
  showPlaytime = false,
  className,
}: Props) {
  const wallpaper = useProfileWallpaper(inst);
  const [c1, c2] = instanceAccent(inst.id);
  const bg1 = inst.ledColor?.trim() || c1;
  const bg2 = inst.ledColor2?.trim() || c2;
  const compact = variant === "compact";
  const library = variant === "library";
  const iconSize = library ? 52 : compact ? 32 : 40;

  const body = (
    <>
      {wallpaper ? (
        <img
          src={wallpaper}
          alt=""
          className="absolute inset-0 h-full w-full object-cover transition duration-300 group-hover:scale-[1.03]"
          loading="lazy"
        />
      ) : (
        <div
          className="absolute inset-0 transition duration-300 group-hover:scale-[1.03]"
          style={{
            background: `linear-gradient(145deg, ${bg1} 0%, ${bg2}88 55%, #0c0c0f 100%)`,
          }}
        />
      )}
      <div className="absolute inset-0 bg-gradient-to-t from-black/85 via-black/35 to-black/15" />
      <div className="absolute inset-0 opacity-[0.14] [background-image:radial-gradient(circle_at_15%_20%,white_0,transparent_42%)]" />

      <div
        className={clsx(
          "relative flex h-full flex-col justify-between text-left",
          library ? "p-5" : compact ? "p-2.5" : "p-3.5",
        )}
      >
        <div className="flex items-start justify-between gap-2">
          <div
            className={clsx(
              "grid shrink-0 place-items-center overflow-hidden rounded-xl border border-white/20 bg-black/35 shadow-lg",
              library ? "h-16 w-16 p-1.5" : compact ? "h-9 w-9 p-0.5" : "h-11 w-11 p-1",
            )}
          >
            <ProfileIcon inst={inst} size={iconSize} />
          </div>
          {active ? (
            <span
              className={clsx(
                "rounded-full bg-accent/90 font-bold uppercase tracking-wide text-white",
                library ? "px-3 py-1 text-[10px]" : "px-2 py-0.5 text-[9px]",
              )}
            >
              Aktywny
            </span>
          ) : null}
        </div>

        <div className="min-w-0">
          <h3
            className={clsx(
              "truncate font-bold text-white",
              library ? "text-xl" : compact ? "text-[13px]" : "text-sm",
            )}
          >
            {inst.name}
          </h3>
          <div className="mt-1.5 flex flex-wrap items-center gap-1.5">
            <span
              className={clsx(
                "rounded-full font-semibold",
                library ? "px-2.5 py-1 text-[10px]" : "px-2 py-0.5 text-[9px]",
                loaderChipClass(inst.loader),
              )}
            >
              {LOADER_LABEL[inst.loader] ?? inst.loader} {inst.gameVersion}
            </span>
          </div>
          <p
            className={clsx(
              "mt-1.5 truncate text-white/55",
              library ? "text-xs" : "text-[10px]",
            )}
          >
            {formatDate(inst.lastPlayed)}
            {showPlaytime && (inst.playTimeSecs ?? 0) > 0
              ? ` · ${formatPlayTime(inst.playTimeSecs || 0)}`
              : ""}
          </p>
        </div>
      </div>

      {(onPlay || onEdit || onSettings) && (
        <div
          className={clsx(
            "absolute right-0 top-0 flex overflow-hidden rounded-bl-xl border-b border-l border-white/10 opacity-0 transition group-hover:opacity-100",
            library ? "rounded-bl-2xl" : "",
          )}
          onClick={(e) => e.stopPropagation()}
        >
          {onEdit ? (
            <IconBtn title="Zarządzaj" onClick={onEdit} library={library}>
              <SquarePen size={library ? 16 : 14} />
            </IconBtn>
          ) : null}
          {onSettings ? (
            <IconBtn title="Wygląd" onClick={onSettings} library={library}>
              <Settings2 size={library ? 16 : 14} />
            </IconBtn>
          ) : null}
          {onPlay ? (
            <button
              type="button"
              title="Graj"
              onClick={onPlay}
              className={clsx(
                "grid place-items-center bg-launch text-white shadow-lg hover:brightness-110",
                library ? "h-11 w-11" : "h-8 w-8",
              )}
            >
              <Play size={library ? 16 : 14} className="ml-0.5" fill="currentColor" />
            </button>
          ) : null}
        </div>
      )}
    </>
  );

  const shellClass = clsx(
    "group relative w-full overflow-hidden rounded-2xl text-left ring-1 transition",
    library ? "min-h-[220px]" : compact ? "min-h-[118px]" : "min-h-[148px]",
    active ? "ring-accent/60 ring-2" : "ring-line hover:ring-accent/35",
    onClick && "cursor-pointer",
    className,
  );

  if (onClick) {
    return (
      <button type="button" onClick={onClick} className={shellClass}>
        {body}
      </button>
    );
  }

  return <article className={shellClass}>{body}</article>;
}

function IconBtn({
  title,
  onClick,
  library,
  children,
}: {
  title: string;
  onClick: () => void;
  library?: boolean;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      title={title}
      onClick={onClick}
      className={clsx(
        "grid place-items-center border-r border-white/10 bg-black/55 text-white/90 hover:bg-black/75",
        library ? "h-11 w-11" : "h-8 w-8",
      )}
    >
      {children}
    </button>
  );
}
