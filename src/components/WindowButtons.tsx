import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";
import { useEffect, useState } from "react";

export function WindowButtons() {
  const [max, setMax] = useState(false);
  useEffect(() => {
    const w = getCurrentWindow();
    w.isMaximized().then(setMax).catch(() => undefined);
  }, []);
  async function toggle() {
    const w = getCurrentWindow();
    await w.toggleMaximize();
    setMax(await w.isMaximized());
  }
  return (
    <div className="no-drag flex items-center">
      <button
        className="grid h-8 w-10 place-items-center text-mute hover:bg-white/8 hover:text-ink"
        onClick={() => getCurrentWindow().minimize()}
        aria-label="Minimalizuj"
      >
        <Minus size={14} />
      </button>
      <button
        className="grid h-8 w-10 place-items-center text-mute hover:bg-white/8 hover:text-ink"
        onClick={() => void toggle()}
        aria-label={max ? "Przywróć" : "Maksymalizuj"}
      >
        <Square size={12} />
      </button>
      <button
        className="grid h-8 w-10 place-items-center text-mute hover:bg-danger hover:text-white"
        onClick={() => getCurrentWindow().close()}
        aria-label="Zamknij"
      >
        <X size={14} />
      </button>
    </div>
  );
}

export function Mark({ size = 28, animated = false }: { size?: number; animated?: boolean }) {
  const accent = "var(--preset-accent, #00d4aa)";
  return (
    <svg width={size} height={size} viewBox="0 0 64 64" aria-hidden>
      <defs>
        <radialGradient id="octra-glow" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor={accent} stopOpacity="0.35" />
          <stop offset="100%" stopColor={accent} stopOpacity="0" />
        </radialGradient>
      </defs>
      {animated && <circle cx="32" cy="32" r="30" fill="url(#octra-glow)" />}
      <rect width="64" height="64" rx="14" fill="#120c14" stroke={accent} strokeWidth="2" />
      <circle cx="32" cy="32" r="14" fill="none" stroke={accent} strokeWidth="6" />
      {animated && (
        <circle
          cx="32"
          cy="10"
          r="3"
          fill={accent}
          style={{ filter: `drop-shadow(0 0 4px ${accent})` }}
        />
      )}
    </svg>
  );
}
