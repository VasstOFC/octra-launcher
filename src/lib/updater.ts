import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api } from "./api";
import { confirmDialog } from "./dialog";
import type { AppInfo, GithubReleaseCheck } from "../types";

export type UpdateStatus =
  | { state: "idle" }
  | { state: "checking" }
  | { state: "current"; version: string; detail?: string }
  | { state: "noReleases"; version: string }
  | { state: "error"; message: string }
  | {
      state: "available";
      version: string;
      notes: string;
      htmlUrl: string;
      installerUrl: string | null;
      mode: "tauri" | "manual";
      tauriUpdate?: Update;
    }
  | { state: "downloading"; percent: number }
  | { state: "installing" };

const RELEASES_URL = "https://github.com/VasstOFC/octra-launcher/releases";
export const INSTALLER_URL = `${RELEASES_URL}/latest/download/Octra-setup.exe`;

/** Wersje przed rotacją klucza podpisu — wymagają jednorazowej ręcznej instalacji. */
const LEGACY_MANUAL_UPGRADE_BEFORE: [number, number, number] = [1, 1, 0];

type GithubNewer = Extract<GithubReleaseCheck, { kind: "newer" }>;

export function isUpdaterSignatureError(error: unknown): boolean {
  const msg = (error instanceof Error ? error.message : String(error)).toLowerCase();
  return (
    msg.includes("different key") ||
    msg.includes("signature") ||
    msg.includes("minisign") ||
    msg.includes("public key")
  );
}

export function needsLegacyManualUpgrade(currentVersion: string): boolean {
  const local = parseVersion(currentVersion);
  if (!local) return false;
  return versionLt(local, LEGACY_MANUAL_UPGRADE_BEFORE);
}

function versionLt(a: number[], b: [number, number, number]): boolean {
  for (let i = 0; i < 3; i++) {
    const av = a[i] ?? 0;
    const bv = b[i] ?? 0;
    if (av < bv) return true;
    if (av > bv) return false;
  }
  return false;
}

export async function checkForUpdates(
  appInfo: AppInfo | null,
): Promise<UpdateStatus> {
  const currentVersion = appInfo?.version ?? "0.0.0";
  const legacyManual = needsLegacyManualUpgrade(currentVersion);

  const [tauriUpdate, githubResult] = await Promise.all([
    legacyManual ? Promise.resolve(null) : checkTauriUpdate(),
    checkGithubUpdate().catch((e: unknown) => ({
      kind: "error" as const,
      message: e instanceof Error ? e.message : String(e),
    })),
  ]);

  if (githubResult.kind === "error") {
    if (tauriUpdate && !legacyManual) {
      return tauriAvailable(tauriUpdate);
    }
    return { state: "error", message: githubResult.message };
  }

  const githubStatus = mapGithubRelease(githubResult, currentVersion, legacyManual);

  if (tauriUpdate && !legacyManual) {
    const remote = parseVersion(tauriUpdate.version);
    const local = parseVersion(currentVersion);
    if (!remote || !local || remote > local) {
      const githubNewer = githubResult.kind === "newer" ? githubResult : undefined;
      return tauriAvailable(tauriUpdate, githubNewer);
    }
  }

  if (githubStatus.state === "available") {
    return githubStatus;
  }

  if (githubStatus.state === "current") {
    return githubStatus;
  }

  if (githubResult.kind === "notFound") {
    return {
      state: "noReleases",
      version: currentVersion,
    };
  }

  return githubStatus;
}

async function checkTauriUpdate(): Promise<Update | null> {
  try {
    return (await check()) ?? null;
  } catch {
    return null;
  }
}

async function checkGithubUpdate(): Promise<GithubReleaseCheck> {
  return api.checkGithubRelease();
}

function tauriAvailable(update: Update, github?: GithubNewer): UpdateStatus {
  return {
    state: "available",
    version: update.version,
    notes: github?.notes || update.body || "",
    htmlUrl: github?.htmlUrl ?? `${RELEASES_URL}/latest`,
    installerUrl: github?.installerUrl ?? INSTALLER_URL,
    mode: "tauri",
    tauriUpdate: update,
  };
}

function mapGithubRelease(
  release: GithubReleaseCheck,
  currentVersion: string,
  legacyManual = false,
): UpdateStatus {
  switch (release.kind) {
    case "notFound":
      return { state: "noReleases", version: currentVersion };
    case "current":
      return {
        state: "current",
        version: release.version,
        detail: "Masz najnowszą opublikowaną wersję.",
      };
    case "newer":
      return {
        state: "available",
        version: release.version,
        notes: release.notes,
        htmlUrl: release.htmlUrl,
        installerUrl: release.installerUrl ?? INSTALLER_URL,
        mode: legacyManual || !release.hasLatestJson ? "manual" : "tauri",
      };
    case "unversioned":
      return {
        state: "available",
        version: currentVersion,
        notes: release.name,
        htmlUrl: release.htmlUrl,
        installerUrl: release.installerUrl,
        mode: "manual",
      };
  }
}

function parseVersion(raw: string): number[] | null {
  const core = raw.trim().replace(/^[vV]/, "").split(/[-+]/)[0];
  const parts = core.split(".").map((p) => Number.parseInt(p, 10));
  if (parts.length < 3 || parts.some((n) => Number.isNaN(n))) return null;
  return parts;
}

export async function installUpdate(
  status: Extract<UpdateStatus, { state: "available" }>,
): Promise<UpdateStatus> {
  if (status.mode === "tauri" && status.tauriUpdate) {
    const ok = await confirmDialog(
      `Pobrać i zainstalować Octra ${status.version}? Launcher uruchomi się ponownie.`,
      { title: "Aktualizacja", confirmLabel: "Zainstaluj" },
    );
    if (!ok) return status;

    try {
      await status.tauriUpdate.downloadAndInstall();
      await relaunch();
      return { state: "installing" };
    } catch (e) {
      if (!isUpdaterSignatureError(e)) throw e;
      return manualInstallFallback(status);
    }
  }

  const url = status.installerUrl ?? status.htmlUrl;
  await openUrl(url);
  return status;
}

async function manualInstallFallback(
  status: Extract<UpdateStatus, { state: "available" }>,
): Promise<UpdateStatus> {
  const url = status.installerUrl ?? INSTALLER_URL;
  const ok = await confirmDialog(
    `Ta wersja launchera (v${status.version}) wymaga jednorazowego pobrania instalatora. ` +
      "Po ponownej instalacji kolejne aktualizacje będą działać automatycznie.\n\nOtworzyć pobieranie?",
    { title: "Aktualizacja", confirmLabel: "Pobierz instalator" },
  );
  if (ok) await openUrl(url);
  return status;
}

export async function openManualInstaller(
  status: Extract<UpdateStatus, { state: "available" }>,
): Promise<void> {
  await openUrl(status.installerUrl ?? INSTALLER_URL);
}

export function formatChannel(channel: string): string {
  switch (channel) {
    case "stable":
      return "Stabilny";
    case "beta":
      return "Beta";
    case "dev":
      return "Dev";
    default:
      return channel;
  }
}

export function updateStatusMessage(status: UpdateStatus): string | null {
  switch (status.state) {
    case "current":
      return status.detail ?? `Masz najnowszą wersję (${status.version}).`;
    case "noReleases":
      return `Sprawdzono GitHub — nie ma jeszcze opublikowanego wydania. Lokalna wersja: ${status.version}.`;
    case "available":
      return `Dostępna nowsza wersja ${status.version}.`;
    case "error":
      return status.message;
    default:
      return null;
  }
}
