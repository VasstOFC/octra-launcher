import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api } from "./api";
import { confirmDialog } from "./dialog";
import type { AppInfo, GithubReleaseCheck } from "../types";

export type UpdateStatus =
  | { state: "idle" }
  | { state: "checking" }
  | { state: "current"; version: string }
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

export async function checkForUpdates(
  appInfo: AppInfo | null,
): Promise<UpdateStatus> {
  if (appInfo?.updatesEnabled) {
    try {
      const update = await check();
      if (update) {
        return {
          state: "available",
          version: update.version,
          notes: update.body ?? "",
          htmlUrl: "https://github.com/VasstOFC/octra-launcher/releases/latest",
          installerUrl: null,
          mode: "tauri",
          tauriUpdate: update,
        };
      }
      return {
        state: "current",
        version: appInfo.version,
      };
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      if (!message.toLowerCase().includes("unsupported")) {
        return { state: "error", message };
      }
    }
  }

  const release = await api.checkGithubRelease();
  return mapGithubRelease(release, appInfo?.version ?? "0.0.0");
}

function mapGithubRelease(
  release: GithubReleaseCheck,
  currentVersion: string,
): UpdateStatus {
  switch (release.kind) {
    case "notFound":
      return { state: "error", message: "Brak publikacji na GitHubie." };
    case "current":
      return { state: "current", version: release.version };
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

export async function installUpdate(status: Extract<
  UpdateStatus,
  { state: "available" }
>): Promise<UpdateStatus> {
  if (status.mode === "tauri" && status.tauriUpdate) {
    const ok = await confirmDialog(
      `Pobrać i zainstalować Octra ${status.version}? Launcher uruchomi się ponownie.`,
      { title: "Aktualizacja", confirmLabel: "Zainstaluj" },
    );
    if (!ok) return status;

    await status.tauriUpdate.downloadAndInstall((event) => {
      if (event.event === "Started") {
        // progress UI polls separately if needed
      }
    });
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
