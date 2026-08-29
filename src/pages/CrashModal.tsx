import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { useApp } from "../stores/appStore";
import { useOctra } from "../stores/octraStore";

export function CrashModal() {
  const overlay = useOctra((s) => s.overlay);
  const close = useOctra((s) => s.closeOverlay);
  const play = useApp((s) => s.playInstance);
  const [cause, setCause] = useState<string>("Nieznana przyczyna.");
  const [log, setLog] = useState("");

  useEffect(() => {
    if (!overlay || overlay.kind !== "crash") return;
    const id = overlay.instanceId;
    Promise.all([api.listCrashReports(id), api.readInstanceLog(id)])
      .then(([reports, txt]) => {
        const last = reports[0];
        const jar = txt.match(/([A-Za-z0-9._-]+\.jar)/)?.[1];
        setCause(
          jar
            ? `Podejrzany plik: ${jar}`
            : last
              ? `Ostatni raport: ${last.name}`
              : "Brak pliku crash-report.",
        );
        setLog(txt.slice(-4000));
      })
      .catch(() => undefined);
  }, [overlay]);

  if (!overlay || overlay.kind !== "crash") return null;

  return (
    <div className="absolute inset-0 z-40 grid place-items-center bg-black/70 p-6">
      <div className="w-full max-w-lg rounded-3xl border border-danger/40 bg-raised p-6 text-center">
        <div className="mx-auto mb-3 h-14 w-14 rounded-xl bg-danger/20 text-2xl leading-[56px] text-danger">
          ✕
        </div>
        <h2 className="text-lg font-bold">Game Crash Detected</h2>
        <p className="mt-1 text-xs text-mute">Exit: {overlay.code}</p>
        <p className="mt-3 text-sm text-ink">{cause}</p>
        <pre className="mt-3 max-h-32 overflow-auto rounded-xl bg-bg p-2 text-left text-[10px] text-mute">
          {log || "Brak logu."}
        </pre>
        <div className="mt-4 flex flex-col gap-2">
          <button
            className="rounded-full bg-white py-2 text-sm font-semibold text-black"
            onClick={() => {
              close();
              void play(overlay.instanceId);
            }}
          >
            Relaunch Game
          </button>
          <div className="flex gap-2">
            <button
              className="flex-1 rounded-full bg-white/10 py-2 text-xs"
              onClick={() => navigator.clipboard.writeText(log)}
            >
              Copy Crash Log
            </button>
            <button
              className="flex-1 rounded-full bg-white/10 py-2 text-xs"
              onClick={() => api.openInstanceSubdir(overlay.instanceId, "crash-reports")}
            >
              Open Crash Log
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
