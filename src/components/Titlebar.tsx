import { Mark, WindowButtons } from "./WindowButtons";
import { useApp } from "../stores/appStore";

export function Titlebar({ subtitle }: { subtitle?: string }) {
  const appInfo = useApp((s) => s.appInfo);
  const online = typeof navigator !== "undefined" ? navigator.onLine : true;
  return (
    <header
      data-tauri-drag-region
      className="drag-region flex h-10 shrink-0 items-center gap-3 border-b border-line bg-bg px-3"
    >
      <div className="flex items-center gap-2 text-[12px] text-mute">
        <Mark size={18} />
        <span className="font-semibold tracking-wide text-ink">Octra</span>
        <span className="text-mute/60">·</span>
        <span>v{appInfo?.version ?? "0.1.0"}</span>
        <span
          className={`h-1.5 w-1.5 rounded-full ${online ? "bg-good" : "bg-danger"}`}
          title={online ? "Online" : "Offline"}
        />
        {subtitle && (
          <>
            <span className="text-mute/60">·</span>
            <span>{subtitle}</span>
          </>
        )}
      </div>
      <div className="ml-auto">
        <WindowButtons />
      </div>
    </header>
  );
}
