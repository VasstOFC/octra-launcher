import { useEffect, useState } from "react";
import { clsx } from "clsx";
import { X } from "lucide-react";
import type { LocalSoftware } from "../types";
import { pl } from "../locales/pl";

export type CreateServerDraft = {
  name: string;
  gameVersion: string;
  software: LocalSoftware;
  motd: string;
  port: number;
  maxPlayers: number;
  difficulty: string;
  viewDistance: number;
  memoryMb: number;
  onlineMode: boolean;
};

type Props = {
  open: boolean;
  defaultVersion: string;
  onClose: () => void;
  onSubmit: (draft: CreateServerDraft) => void;
  busy?: boolean;
};

const DIFFICULTIES = ["peaceful", "easy", "normal", "hard"] as const;

export function LocalServerCreateDialog({
  open,
  defaultVersion,
  onClose,
  onSubmit,
  busy,
}: Props) {
  const [name, setName] = useState("Mój serwer");
  const [gameVersion, setGameVersion] = useState(defaultVersion);
  const [software, setSoftware] = useState<LocalSoftware>("paper");
  const [motd, setMotd] = useState("Serwer Octra");
  const [port, setPort] = useState(25565);
  const [maxPlayers, setMaxPlayers] = useState(10);
  const [difficulty, setDifficulty] = useState("easy");
  const [viewDistance, setViewDistance] = useState(10);
  const [memoryMb, setMemoryMb] = useState(2048);
  const [onlineMode, setOnlineMode] = useState(false);

  useEffect(() => {
    if (!open) return;
    setName("Mój serwer");
    setGameVersion(defaultVersion);
    setSoftware("paper");
    setMotd("Serwer Octra");
    setPort(25565);
    setMaxPlayers(10);
    setDifficulty("easy");
    setViewDistance(10);
    setMemoryMb(2048);
    setOnlineMode(false);
  }, [open, defaultVersion]);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-black/60 p-4">
      <div
        className="max-h-[90vh] w-full max-w-lg overflow-auto rounded-2xl border border-line bg-raised shadow-2xl"
        role="dialog"
        aria-modal="true"
      >
        <header className="flex items-center justify-between border-b border-line px-5 py-4">
          <h2 className="text-base font-bold">{pl.host.createTitle}</h2>
          <button type="button" onClick={onClose} className="text-mute hover:text-ink">
            <X size={18} />
          </button>
        </header>

        <div className="space-y-3 px-5 py-4">
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder={pl.host.serverName}
            className="w-full rounded-xl bg-raised2 px-3 py-2.5 text-sm ring-1 ring-line"
          />
          <input
            value={gameVersion}
            onChange={(e) => setGameVersion(e.target.value)}
            placeholder="1.21.1"
            className="w-full rounded-xl bg-raised2 px-3 py-2.5 text-sm ring-1 ring-line"
          />
          <div className="flex flex-wrap gap-2">
            {(["paper", "vanilla", "fabric"] as const).map((s) => (
              <button
                key={s}
                type="button"
                onClick={() => setSoftware(s)}
                className={clsx(
                  "rounded-full px-3 py-1 text-xs font-semibold",
                  software === s
                    ? "bg-accent/25 ring-1 ring-accent/50"
                    : "bg-raised2 text-mute",
                )}
              >
                {s}
              </button>
            ))}
          </div>

          <input
            value={motd}
            onChange={(e) => setMotd(e.target.value)}
            placeholder={pl.host.motd}
            className="w-full rounded-xl bg-raised2 px-3 py-2.5 text-sm ring-1 ring-line"
          />

          <div className="grid grid-cols-2 gap-2">
            <input
              type="number"
              value={port}
              onChange={(e) => setPort(Number(e.target.value))}
              className="rounded-xl bg-raised2 px-3 py-2 text-sm ring-1 ring-line"
              placeholder={pl.host.port}
            />
            <input
              type="number"
              value={maxPlayers}
              onChange={(e) => setMaxPlayers(Number(e.target.value))}
              className="rounded-xl bg-raised2 px-3 py-2 text-sm ring-1 ring-line"
              placeholder={pl.host.maxPlayers}
            />
          </div>

          <select
            value={difficulty}
            onChange={(e) => setDifficulty(e.target.value)}
            className="w-full rounded-xl bg-raised2 px-3 py-2 text-sm ring-1 ring-line"
          >
            {DIFFICULTIES.map((d) => (
              <option key={d} value={d}>
                {pl.host.difficulties[d]}
              </option>
            ))}
          </select>

          <label className="block text-xs text-mute">
            {pl.host.viewDistance}: {viewDistance}
            <input
              type="range"
              min={2}
              max={32}
              value={viewDistance}
              onChange={(e) => setViewDistance(Number(e.target.value))}
              className="mt-1 w-full"
            />
          </label>

          <label className="block text-xs text-mute">
            RAM: {memoryMb} MB
            <input
              type="range"
              min={512}
              max={16384}
              step={256}
              value={memoryMb}
              onChange={(e) => setMemoryMb(Number(e.target.value))}
              className="mt-1 w-full"
            />
          </label>

          <label className="flex items-center gap-2 text-xs">
            <input
              type="checkbox"
              checked={onlineMode}
              onChange={(e) => setOnlineMode(e.target.checked)}
            />
            {pl.host.onlineMode}
          </label>
        </div>

        <footer className="flex justify-end gap-2 border-t border-line px-5 py-4">
          <button
            type="button"
            onClick={onClose}
            className="rounded-xl px-4 py-2 text-sm text-mute hover:text-ink"
          >
            {pl.creator.cancel}
          </button>
          <button
            type="button"
            disabled={busy || !name.trim()}
            onClick={() =>
              onSubmit({
                name: name.trim(),
                gameVersion: gameVersion.trim() || defaultVersion,
                software,
                motd,
                port,
                maxPlayers,
                difficulty,
                viewDistance,
                memoryMb,
                onlineMode,
              })
            }
            className="rounded-xl bg-accent px-4 py-2 text-sm font-semibold text-bg-on-accent disabled:opacity-50"
          >
            {pl.host.create}
          </button>
        </footer>
      </div>
    </div>
  );
}
