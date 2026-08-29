import { useEffect, useState, type ReactNode } from "react";
import { clsx } from "clsx";
import { api } from "../lib/api";
import { pl } from "../locales/pl";
import { useApp } from "../stores/appStore";
import type { LocalServerInfo } from "../types";

const DIFFICULTIES = ["peaceful", "easy", "normal", "hard"] as const;

type Props = {
  server: LocalServerInfo;
  onSaved: () => void;
  disabled?: boolean;
};

export function LocalServerSettings({ server, onSaved, disabled }: Props) {
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);
  const [saving, setSaving] = useState(false);
  const [motd, setMotd] = useState(server.motd);
  const [port, setPort] = useState(server.port);
  const [maxPlayers, setMaxPlayers] = useState(server.maxPlayers);
  const [difficulty, setDifficulty] = useState(server.difficulty);
  const [viewDistance, setViewDistance] = useState(server.viewDistance);
  const [memoryMb, setMemoryMb] = useState(server.memoryMb);
  const [onlineMode, setOnlineMode] = useState(server.onlineMode);

  useEffect(() => {
    setMotd(server.motd);
    setPort(server.port);
    setMaxPlayers(server.maxPlayers);
    setDifficulty(server.difficulty);
    setViewDistance(server.viewDistance);
    setMemoryMb(server.memoryMb);
    setOnlineMode(server.onlineMode);
  }, [server.id, server.motd, server.port, server.maxPlayers, server.difficulty, server.viewDistance, server.memoryMb, server.onlineMode]);

  async function save() {
    setSaving(true);
    try {
      await api.updateLocalServer(server.id, {
        motd: motd.trim() || undefined,
        port,
        maxPlayers,
        difficulty,
        viewDistance,
        memoryMb,
        onlineMode,
      });
      showOk(pl.host.settingsSaved);
      onSaved();
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  }

  const running = server.status === "running";

  return (
    <div className="flex h-full flex-col overflow-auto p-4">
      <h2 className="text-sm font-bold text-ink">{pl.host.settingsTitle}</h2>
      <p className="mt-1 text-[11px] text-mute">{pl.host.settingsHint}</p>

      <div className="mt-4 space-y-3">
        <Field label={pl.host.motd}>
          <input
            value={motd}
            onChange={(e) => setMotd(e.target.value)}
            disabled={disabled || saving}
            className="w-full rounded-lg bg-raised px-2.5 py-2 text-xs ring-1 ring-line"
            placeholder="Minecraft Server"
          />
        </Field>

        <div className="grid grid-cols-2 gap-2">
          <Field label={pl.host.port}>
            <input
              type="number"
              min={1}
              max={65535}
              value={port}
              onChange={(e) => setPort(Number(e.target.value))}
              disabled={disabled || saving || running}
              className="w-full rounded-lg bg-raised px-2.5 py-2 text-xs ring-1 ring-line disabled:opacity-50"
            />
          </Field>
          <Field label={pl.host.maxPlayers}>
            <input
              type="number"
              min={1}
              max={200}
              value={maxPlayers}
              onChange={(e) => setMaxPlayers(Number(e.target.value))}
              disabled={disabled || saving}
              className="w-full rounded-lg bg-raised px-2.5 py-2 text-xs ring-1 ring-line"
            />
          </Field>
        </div>

        <Field label={pl.host.difficulty}>
          <select
            value={difficulty}
            onChange={(e) => setDifficulty(e.target.value)}
            disabled={disabled || saving}
            className="w-full rounded-lg bg-raised px-2.5 py-2 text-xs ring-1 ring-line"
          >
            {DIFFICULTIES.map((d) => (
              <option key={d} value={d}>
                {pl.host.difficulties[d]}
              </option>
            ))}
          </select>
        </Field>

        <Field label={`${pl.host.viewDistance}: ${viewDistance}`}>
          <input
            type="range"
            min={2}
            max={32}
            value={viewDistance}
            onChange={(e) => setViewDistance(Number(e.target.value))}
            disabled={disabled || saving}
            className="w-full"
          />
        </Field>

        <Field label={`RAM: ${memoryMb} MB`}>
          <input
            type="range"
            min={512}
            max={16384}
            step={256}
            value={memoryMb}
            onChange={(e) => setMemoryMb(Number(e.target.value))}
            disabled={disabled || saving || running}
            className="w-full disabled:opacity-50"
          />
        </Field>

        <label className="flex items-center gap-2 text-xs text-ink">
          <input
            type="checkbox"
            checked={onlineMode}
            onChange={(e) => setOnlineMode(e.target.checked)}
            disabled={disabled || saving}
          />
          {pl.host.onlineMode}
        </label>
      </div>

      {running ? (
        <p className="mt-3 text-[10px] text-warn">{pl.host.restartHint}</p>
      ) : null}

      <button
        type="button"
        disabled={disabled || saving}
        onClick={() => void save()}
        className={clsx(
          "mt-4 w-full rounded-xl bg-accent px-3 py-2 text-xs font-semibold text-bg-on-accent",
          (disabled || saving) && "opacity-50",
        )}
      >
        {saving ? pl.host.saving : pl.host.saveSettings}
      </button>

      <button
        type="button"
        onClick={() => void api.openLocalServerProperties(server.id).catch((e) => showError(String(e)))}
        className="mt-2 w-full rounded-xl border border-line px-3 py-2 text-xs text-mute hover:text-ink"
      >
        {pl.host.openProperties}
      </button>
    </div>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="block">
      <span className="text-[10px] font-semibold uppercase tracking-wider text-mute">{label}</span>
      <div className="mt-1">{children}</div>
    </label>
  );
}
