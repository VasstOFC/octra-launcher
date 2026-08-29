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

export async function checkForUpdates(
  appInfo: AppInfo | null,
): Promise<UpdateStatus> {
  const currentVersion = appInfo?.version ?? "0.0.0";

  const [tauriUpdate, githubResult] = await Promise.all([
    checkTauriUpdate(),
    checkGithubUpdate().catch((e: unknown) => ({
      kind: "error" as const,
      message: e instanceof Error ? e.message : String(e),
    })),
  ]);

  if (githubResult.kind === "error") {
    if (tauriUpdate) {
      return tauriAvailable(tauriUpdate);
    }
    return { state: "error", message: githubResult.message };
  }

  const githubStatus = mapGithubRelease(githubResult, currentVersion);

  if (tauriUpdate) {
    const remote = parseVersion(tauriUpdate.version);
    const local = parseVersion(currentVersion);
    if (!remote || !local || remote > local) {
      return tauriAvailable(tauriUpdate);
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

function tauriAvailable(update: Update): UpdateStatus {
  return {
    state: "available",
    version: update.version,
    notes: update.body ?? "",
    htmlUrl: `${RELEASES_URL}/latest`,
    installerUrl: null,
    mode: "tauri",
    tauriUpdate: update,
  };
}

function mapGithubRelease(
  release: GithubReleaseCheck,
  currentVersion: string,
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
        installerUrl: release.installerUrl,
        mode: release.hasLatestJson ? "tauri" : "manual",
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

    await status.tauriUpdate.downloadAndInstall();
    await relaunch();
    return { state: "installing" };
  }

  const url = status.installerUrl ?? status.htmlUrl;
  await openUrl(url);
  return status;
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
