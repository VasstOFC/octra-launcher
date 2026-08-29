import { create } from "zustand";
import { api } from "../lib/api";
import { alertDialog } from "../lib/dialog";
import { applySettingsTheme } from "../lib/theme";
import { useOctra } from "./octraStore";
import type {
  AccountsFile,
  AppInfo,
  DeviceCode,
  Instance,
  InstallProgress,
  JavaStatus,
  Settings,
} from "../types";

export type RunningLocalServer = {
  id: string;
  name: string;
  status: string;
};

export type PlayingSession = {
  instanceId: string;
  accountUuid: string;
};

function sameSession(
  a: PlayingSession,
  instanceId: string,
  accountUuid: string,
) {
  return a.instanceId === instanceId && a.accountUuid === accountUuid;
}

interface AppStore {
  ready: boolean;
  settings: Settings | null;
  instances: Instance[];
  accounts: AccountsFile;
  java: JavaStatus | null;
  dataDir: string;
  appInfo: AppInfo | null;
  progress: InstallProgress | null;
  login: DeviceCode | null;
  loginOpen: boolean;
  offlineOpen: boolean;
  skinEditUuid: string | null;
  skinEpoch: number;
  launchingId: string | null;
  playingSessions: PlayingSession[];
  runningServers: RunningLocalServer[];
  banner: { type: "error" | "ok"; text: string; at: number } | null;
  loadAll: () => Promise<void>;
  setSettings: (settings: Settings) => void;
  refreshInstances: () => Promise<void>;
  refreshAccounts: () => Promise<void>;
  setProgress: (p: InstallProgress | null) => void;
  setLogin: (d: DeviceCode | null) => void;
  setLoginOpen: (v: boolean) => void;
  setOfflineOpen: (v: boolean) => void;
  setSkinEditUuid: (uuid: string | null) => void;
  bumpSkin: () => void;
  setLaunching: (id: string | null) => void;
  markPlaying: (instanceId: string, accountUuid: string) => void;
  markStopped: (instanceId: string, accountUuid: string) => void;
  markServerStatus: (id: string, name: string, status: string) => void;
  isPlaying: (instanceId: string, accountUuid?: string) => boolean;
  showError: (text: string) => void;
  showOk: (text: string) => void;
  clearBanner: () => void;
  playInstance: (id: string) => Promise<void>;
  stopInstance: (id: string) => Promise<void>;
  togglePlay: (id: string) => Promise<void>;
}

const defaultAccounts: AccountsFile = { accounts: [] };

export const useApp = create<AppStore>((set, get) => ({
  ready: false,
  settings: null,
  instances: [],
  accounts: defaultAccounts,
  java: null,
  dataDir: "",
  appInfo: null,
  progress: null,
  login: null,
  loginOpen: false,
  offlineOpen: false,
  skinEditUuid: null,
  skinEpoch: 0,
  launchingId: null,
  playingSessions: [],
  runningServers: [],
  banner: null,
  loadAll: async () => {
    try {
      const [settings, instances, accounts, java, dataDir, appInfo] = await Promise.all([
        api.getSettings(),
        api.listInstances(),
        api.getAccounts(),
        api.scanJava(),
        api.getDataDir(),
        api.getAppInfo().catch(() => null),
      ]);
      set({ settings, instances, accounts, java, dataDir, appInfo, ready: true });
      applySettingsTheme(settings);
      const sorted = [...instances].sort((a, b) =>
        (b.lastPlayed ?? "").localeCompare(a.lastPlayed ?? ""),
      );
      if (sorted[0] && !useOctra.getState().selectedId) {
        useOctra.getState().selectInstance(sorted[0].id);
      }
      try {
        const local = await api.listLocalServers();
        for (const s of local) {
          if (s.status !== "stopped") {
            get().markServerStatus(s.id, s.name, s.status);
          }
        }
      } catch {
        /* stary backend bez hostingu lokalnego */
      }
    } catch (e) {
      set({ ready: true });
      get().showError(e instanceof Error ? e.message : String(e));
    }
  },
  setSettings: (settings) => {
    applySettingsTheme(settings);
    set({ settings });
  },
  refreshInstances: async () => {
    set({ instances: await api.listInstances() });
  },
  refreshAccounts: async () => {
    set({ accounts: await api.getAccounts() });
  },
  setProgress: (() => {
    let timer: ReturnType<typeof setTimeout> | null = null;
    let latest: InstallProgress | null = null;
    return (progress: InstallProgress | null) => {
      if (progress === null) {
        if (timer) clearTimeout(timer);
        timer = null;
        latest = null;
        set({ progress: null });
        return;
      }
      latest = progress;
      if (timer) return;
      set({ progress: latest });
      timer = setTimeout(() => {
        timer = null;
        if (latest) set({ progress: latest });
      }, 100);
    };
  })(),
  setLogin: (login) => set({ login }),
  setLoginOpen: (loginOpen) => set({ loginOpen }),
  setOfflineOpen: (offlineOpen) => set({ offlineOpen }),
  setSkinEditUuid: (skinEditUuid) => set({ skinEditUuid }),
  bumpSkin: () => set((s) => ({ skinEpoch: s.skinEpoch + 1 })),
  setLaunching: (launchingId) => set({ launchingId }),
  markPlaying: (instanceId, accountUuid) =>
    set((s) =>
      s.playingSessions.some((x) => sameSession(x, instanceId, accountUuid))
        ? s
        : {
            playingSessions: [
              ...s.playingSessions,
              { instanceId, accountUuid },
            ],
          },
    ),
  markStopped: (instanceId, accountUuid) =>
    set((s) => ({
      playingSessions: s.playingSessions.filter(
        (x) => !sameSession(x, instanceId, accountUuid),
      ),
    })),
  markServerStatus: (id, name, status) =>
    set((s) => {
      if (status === "stopped") {
        return {
          runningServers: s.runningServers.filter((x) => x.id !== id),
        };
      }
      const rest = s.runningServers.filter((x) => x.id !== id);
      return { runningServers: [...rest, { id, name, status }] };
    }),
  isPlaying: (instanceId, accountUuid) => {
    const uuid = accountUuid ?? get().accounts.active;
    if (!uuid) return false;
    return get().playingSessions.some((x) => sameSession(x, instanceId, uuid));
  },
  showError: (text) => {
    void alertDialog(text, "Błąd");
  },
  showOk: (text) => set({ banner: { type: "ok", text, at: Date.now() } }),
  clearBanner: () => set({ banner: null }),
  playInstance: async (id) => {
    const {
      accounts,
      setLaunching,
      refreshInstances,
      showError,
      markPlaying,
      setLoginOpen,
    } = get();
    if (!accounts.active) {
      showError(
        "Dodaj konto z menu w prawym górnym rogu — Microsoft albo offline.",
      );
      return;
    }
    setLaunching(id);
    try {
      await api.launchInstance(id);
      markPlaying(id, accounts.active!);
      await refreshInstances();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      if (/tokenu|zaloguj/i.test(msg)) {
        setLoginOpen(true);
      }
      showError(msg);
    } finally {
      setLaunching(null);
    }
  },
  stopInstance: async (id) => {
    const { accounts, showError, markStopped } = get();
    const accountUuid = accounts.active;
    if (!accountUuid) {
      showError("Wybierz konto, dla którego chcesz zatrzymać grę.");
      return;
    }
    try {
      await api.stopInstance(id, accountUuid);
      markStopped(id, accountUuid);
    } catch (e) {
      markStopped(id, accountUuid);
      showError(e instanceof Error ? e.message : String(e));
    }
  },
  togglePlay: async (id) => {
    const { isPlaying, playInstance, stopInstance } = get();
    if (isPlaying(id)) {
      await stopInstance(id);
    } else {
      await playInstance(id);
    }
  },
}));

export function useActiveAccount() {
  return useApp(
    (s) => s.accounts.accounts.find((a) => a.uuid === s.accounts.active) ?? null,
  );
}

export function activeAccount() {
  const { accounts } = useApp.getState();
  return accounts.accounts.find((a) => a.uuid === accounts.active) ?? null;
}
