import { useEffect, useState, type ReactNode } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { FolderOpen, RefreshCw, ExternalLink, ChevronDown } from "lucide-react";
import { useApp } from "../stores/appStore";
import { api } from "../lib/api";
import { formatRam } from "../lib/format";
import { ACCENT_PRESETS, normalizePreset } from "../lib/theme";
import {
  checkForUpdates,
  formatChannel,
  installUpdate,
  updateStatusMessage,
  type UpdateStatus,
} from "../lib/updater";
import { Card } from "../components/ui/Card";
import { SectionHeader } from "../components/ui/SectionHeader";
import { Button } from "../components/ui/Button";
import type { JavaRuntime, Settings } from "../types";

function SettingRow({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <div className="flex flex-col gap-1 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <p className="text-[13px] text-ink">{label}</p>
        {hint ? <p className="text-xs text-mute">{hint}</p> : null}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

function ChannelBadge({ channel }: { channel: string }) {
  const label = formatChannel(channel);
  const tone =
    channel === "dev"
      ? "bg-amber-500/15 text-amber-200 ring-amber-400/30"
      : "bg-accent/15 text-ink ring-accent/30";
  return (
    <span className={`rounded-md px-2 py-0.5 text-[11px] font-semibold ring-1 ${tone}`}>
      {label}
    </span>
  );
}

export function SettingsPage() {
  const settings = useApp((s) => s.settings);
  const setSettings = useApp((s) => s.setSettings);
  const dataDir = useApp((s) => s.dataDir);
  const appInfo = useApp((s) => s.appInfo);
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus>({ state: "idle" });
  const [javaRuntimes, setJavaRuntimes] = useState<JavaRuntime[]>([]);
  const [javaLoading, setJavaLoading] = useState(false);
  const [advancedOpen, setAdvancedOpen] = useState(false);

  useEffect(() => {
    void api.scanJava().then((j) => setJavaRuntimes(j.runtimes)).catch(() => {});
  }, []);

  if (!settings) return null;

  async function save(patch: Partial<Settings>) {
    try {
      const next = await api.saveSettings({ ...settings!, ...patch });
      setSettings(next);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    }
  }

  async function runUpdateCheck() {
    setUpdateStatus({ state: "checking" });
    try {
      const status = await checkForUpdates(appInfo);
      setUpdateStatus(status);
      const msg = updateStatusMessage(status);
      if (status.state === "current" && msg) {
        showOk(msg);
      }
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setUpdateStatus({ state: "error", message });
      showError(message);
    }
  }

  async function runInstall() {
    if (updateStatus.state !== "available") return;
    try {
      await installUpdate(updateStatus);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    }
  }

  async function rescanJava() {
    setJavaLoading(true);
    try {
      const j = await api.scanJava();
      setJavaRuntimes(j.runtimes);
      showOk(`Znaleziono ${j.runtimes.length} instalacji Javy.`);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setJavaLoading(false);
    }
  }

  const updatesEnabled = appInfo?.updatesEnabled ?? false;

  return (
    <div className="min-h-0 flex-1 overflow-auto p-5 pb-10">
      <SectionHeader title="Ustawienia" />
      <p className="mt-2 max-w-2xl text-[13px] text-mute">
        Konfiguracja launchera, aktualizacje i środowisko gry.
      </p>

      <Card className="mt-4 space-y-3">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h2 className="text-sm font-semibold">O Octrze</h2>
            <p className="mt-1 text-xs text-mute">
              {appInfo?.displayName ?? "Octra Launcher"}
            </p>
          </div>
          {appInfo ? <ChannelBadge channel={appInfo.channel} /> : null}
        </div>
        <div className="grid gap-2 text-[13px] sm:grid-cols-2">
          <div>
            <span className="text-mute">Wersja</span>
            <p className="font-mono text-ink">v{appInfo?.version ?? "—"}</p>
          </div>
          <div>
            <span className="text-mute">Kanał</span>
            <p className="text-ink">{appInfo ? formatChannel(appInfo.channel) : "—"}</p>
          </div>
        </div>
        <div className="flex flex-wrap gap-2 pt-1">
          <Button
            variant="ghost"
            className="gap-1.5 text-xs"
            onClick={() =>
              openUrl("https://github.com/VasstOFC/octra-launcher").catch(() => {})
            }
          >
            <ExternalLink className="size-3.5" />
            GitHub
          </Button>
          <Button
            variant="ghost"
            className="gap-1.5 text-xs"
            onClick={() =>
              openUrl("https://github.com/VasstOFC/octra-launcher/releases").catch(
                () => {},
              )
            }
          >
            <ExternalLink className="size-3.5" />
            Wydania
          </Button>
        </div>
      </Card>

      <Card className="mt-4 space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div>
            <h2 className="text-sm font-semibold">Aktualizacje</h2>
            <p className="mt-1 text-xs text-mute">
              Przycisk sprawdza GitHub Releases i manifest auto-updatera (
              <code className="text-[11px]">latest.json</code>).
              {updatesEnabled
                ? " W buildzie release aktualizacje mogą instalować się automatycznie."
                : " W trybie dev sprawdzanie jest ręczne lub przy starcie (jeśli włączone)."}
            </p>
          </div>
          <Button
            variant="secondary"
            className="gap-1.5 text-xs"
            disabled={updateStatus.state === "checking"}
            onClick={() => void runUpdateCheck()}
          >
            <RefreshCw
              className={`size-3.5 ${updateStatus.state === "checking" ? "animate-spin" : ""}`}
            />
            Sprawdź aktualizacje
          </Button>
        </div>

        <label className="flex items-center gap-2 text-[13px]">
          <input
            type="checkbox"
            checked={settings.autoCheckUpdates !== false}
            onChange={(e) => void save({ autoCheckUpdates: e.target.checked })}
          />
          Sprawdzaj przy starcie launchera
        </label>

        {updateStatus.state === "current" || updateStatus.state === "noReleases" ? (
          <p className="text-xs text-mute">{updateStatusMessage(updateStatus)}</p>
        ) : null}

        {updateStatus.state === "available" ? (
          <div className="rounded-lg bg-raised2 p-3 ring-1 ring-line">
            <p className="text-sm font-semibold text-ink">
              Dostępna wersja {updateStatus.version}
            </p>
            {updateStatus.notes ? (
              <p className="mt-2 whitespace-pre-wrap text-xs text-mute">
                {updateStatus.notes.slice(0, 500)}
                {updateStatus.notes.length > 500 ? "…" : ""}
              </p>
            ) : null}
            <div className="mt-3 flex flex-wrap gap-2">
              <Button variant="primary" className="text-xs" onClick={() => void runInstall()}>
                {updateStatus.mode === "tauri" ? "Pobierz i zainstaluj" : "Pobierz instalator"}
              </Button>
              <Button
                variant="ghost"
                className="text-xs"
                onClick={() => openUrl(updateStatus.htmlUrl).catch(() => {})}
              >
                Strona wydania
              </Button>
            </div>
          </div>
        ) : null}

        {updateStatus.state === "error" ? (
          <p className="text-xs text-danger">{updateStatus.message}</p>
        ) : null}
      </Card>

      <Card className="mt-4">
        <h2 className="text-sm font-semibold">Motyw kolorystyczny</h2>
        <div className="mt-3 flex flex-wrap gap-2">
          {ACCENT_PRESETS.map((p) => (
            <button
              key={p.id}
              type="button"
              onClick={() => void save({ accentPreset: p.id })}
              className={`rounded-lg px-3 py-1.5 text-xs font-semibold ${
                normalizePreset(settings.accentPreset) === p.id
                  ? "bg-accent/20 text-ink ring-1 ring-accent/40"
                  : "bg-raised2 text-mute hover:text-ink"
              }`}
            >
              {p.label}
            </button>
          ))}
        </div>
      </Card>

      <Card className="mt-4 space-y-4">
        <h2 className="text-sm font-semibold">Gra</h2>
        <div>
          <div className="flex items-center justify-between gap-2">
            <p className="text-[13px] text-ink">Pamięć RAM (domyślnie)</p>
            <span className="font-mono text-xs text-mute">
              {formatRam(settings.memoryMaxMb)}
            </span>
          </div>
          <input
            type="range"
            min={1024}
            max={32768}
            step={256}
            className="mt-2 w-full"
            value={settings.memoryMaxMb}
            onChange={(e) => void save({ memoryMaxMb: Number(e.target.value) })}
          />
        </div>

        <div className="border-t border-line pt-4">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div>
              <p className="text-[13px] font-medium text-ink">Java</p>
              <p className="text-xs text-mute">
                {javaRuntimes.length > 0
                  ? `Wykryto: ${javaRuntimes.map((r) => `Java ${r.major}`).join(", ")}`
                  : "Brak wykrytych instalacji — skanuj lub pobierz przy starcie gry."}
              </p>
            </div>
            <Button
              variant="secondary"
              className="gap-1.5 text-xs"
              disabled={javaLoading}
              onClick={() => void rescanJava()}
            >
              <RefreshCw className={`size-3.5 ${javaLoading ? "animate-spin" : ""}`} />
              Skanuj
            </Button>
          </div>
          <div className="mt-3">
            <label className="text-xs text-mute">Tryb wyboru Javy</label>
            <select
              className="mt-1 w-full rounded-lg bg-raised2 px-3 py-2 text-[13px] text-ink ring-1 ring-line"
              value={settings.javaMode || "auto"}
              onChange={(e) => void save({ javaMode: e.target.value })}
            >
              <option value="auto">Automatyczny (zalecane)</option>
              <option value="custom">Własna ścieżka</option>
            </select>
          </div>
          {settings.javaMode === "custom" ? (
            <input
              type="text"
              className="mt-2 w-full rounded-lg bg-raised2 px-3 py-2 font-mono text-xs text-ink ring-1 ring-line"
              placeholder="C:\Program Files\Java\..."
              value={settings.javaPath ?? ""}
              onChange={(e) => void save({ javaPath: e.target.value })}
            />
          ) : null}
        </div>
      </Card>

      <Card className="mt-4 space-y-3">
        <h2 className="text-sm font-semibold">Launcher</h2>
        <SettingRow label="Ukryj po starcie gry" hint="Zminimalizuj okno po uruchomieniu Minecrafta.">
          <input
            type="checkbox"
            checked={Boolean(settings.closeOnLaunch)}
            onChange={(e) => void save({ closeOnLaunch: e.target.checked })}
          />
        </SettingRow>
        <SettingRow label="Zasobnik systemowy" hint="Zamknięcie okna chowa launcher do tray.">
          <input
            type="checkbox"
            checked={settings.hideToTray !== false}
            onChange={(e) => void save({ hideToTray: e.target.checked })}
          />
        </SettingRow>
        <SettingRow label="Discord Rich Presence">
          <input
            type="checkbox"
            checked={settings.discordRpc !== false}
            onChange={(e) => void save({ discordRpc: e.target.checked })}
          />
        </SettingRow>
        <SettingRow label="Wersje snapshot" hint="Pokaż wersje rozwojowe Minecrafta.">
          <input
            type="checkbox"
            checked={Boolean(settings.showSnapshots)}
            onChange={(e) => void save({ showSnapshots: e.target.checked })}
          />
        </SettingRow>
      </Card>

      <Card className="mt-4 space-y-3">
        <h2 className="text-sm font-semibold">Dane</h2>
        <p className="break-all font-mono text-xs text-mute">{dataDir}</p>
        <Button
          variant="secondary"
          className="gap-1.5 text-xs"
          onClick={() => api.openDataDir().catch((e) => showError(String(e)))}
        >
          <FolderOpen className="size-3.5" />
          Otwórz folder danych
        </Button>
      </Card>

      <Card className="mt-4">
        <button
          type="button"
          className="flex w-full items-center justify-between text-left"
          onClick={() => setAdvancedOpen((v) => !v)}
        >
          <h2 className="text-sm font-semibold">Zaawansowane</h2>
          <ChevronDown
            className={`size-4 text-mute transition ${advancedOpen ? "rotate-180" : ""}`}
          />
        </button>
        {advancedOpen ? (
          <div className="mt-4 space-y-4 border-t border-line pt-4">
            <div>
              <label className="text-xs text-mute">Równoległe pobierania</label>
              <input
                type="number"
                min={1}
                max={32}
                className="mt-1 w-full rounded-lg bg-raised2 px-3 py-2 text-[13px] ring-1 ring-line"
                value={settings.maxConcurrentDownloads ?? 10}
                onChange={(e) =>
                  void save({ maxConcurrentDownloads: Number(e.target.value) })
                }
              />
            </div>
            <div>
              <label className="text-xs text-mute">URL serwera skinów Lumen (opcjonalnie)</label>
              <input
                type="text"
                className="mt-1 w-full rounded-lg bg-raised2 px-3 py-2 font-mono text-xs ring-1 ring-line"
                placeholder="http://192.168.x.x:port"
                value={settings.skinsUrl ?? ""}
                onChange={(e) => void save({ skinsUrl: e.target.value })}
              />
            </div>
          </div>
        ) : null}
      </Card>
    </div>
  );
}
