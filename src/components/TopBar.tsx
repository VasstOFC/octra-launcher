import { ChevronRight } from "lucide-react";
import { useMemo } from "react";
import { useApp } from "../stores/appStore";
import { buildBreadcrumbs, useOctra } from "../stores/octraStore";

export function TopBar() {
  const view = useOctra((s) => s.view);
  const overlay = useOctra((s) => s.overlay);
  const selectedId = useOctra((s) => s.selectedId);
  const instances = useApp((s) => s.instances);

  const profileName = useMemo(() => {
    const inst =
      instances.find((i) => i.id === selectedId) ??
      instances
        .slice()
        .sort((a, b) => (b.lastPlayed ?? "").localeCompare(a.lastPlayed ?? ""))[0];
    return inst?.name ?? null;
  }, [instances, selectedId]);

  const crumbs = buildBreadcrumbs({ view, profileName, overlay });

  return (
    <nav
      className="flex h-9 shrink-0 items-center gap-1 border-b border-line bg-raised/60 px-4 text-[12px]"
      aria-label="Nawigacja"
    >
      {crumbs.map((c, i) => (
        <span key={c.key} className="inline-flex items-center gap-1">
          {i > 0 && <ChevronRight size={12} className="text-mute/60" />}
          <span className={i === crumbs.length - 1 ? "font-semibold text-ink" : "text-mute"}>
            {c.label}
          </span>
        </span>
      ))}
    </nav>
  );
}
