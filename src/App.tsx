import { memo, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { DialogHost } from "./components/DialogHost";
import { FriendsDrawer } from "./components/FriendsDrawer";
import { Rail } from "./components/Rail";
import { Titlebar } from "./components/Titlebar";
import { CrashModal } from "./pages/CrashModal";
import { ContentOverlay } from "./pages/ContentOverlay";
import { GalleryPage } from "./pages/Gallery";
import { HomePage } from "./pages/Home";
import { LockerPage } from "./pages/Locker";
import { LoginPage } from "./pages/Login";
import { NotifyPage } from "./pages/Notify";
import { StorePage } from "./pages/Store";
import { RelayPage } from "./pages/Relay";
import { OctraMenu } from "./components/OctraMenu";
import { installKeyboardGuard } from "./lib/keyboardGuard";
import { HostPage } from "./pages/Host";
import { SettingsPage } from "./pages/Settings";
import { VersionsPage } from "./pages/Versions";
import { useApp } from "./stores/appStore";
import { useOctra } from "./stores/octraStore";
import {
  notifyCrash,
  notifyContentUpdates,
  notifyFriendOnline,
} from "./stores/notifications";
import { api } from "./lib/api";
import type { Account, InstallProgress } from "./types";
import { checkForUpdates } from "./lib/updater";

function sessionFromGameEvent(
  payload: unknown,
): { instanceId: string; accountUuid: string } | undefined {
  if (!payload || typeof payload !== "object") return undefined;
  const p = payload as Record<string, unknown>;
  const nested =
    p.payload && typeof p.payload === "object"
      ? (p.payload as Record<string, unknown>)
      : undefined;
  const instanceId =
    p.instanceId ?? p.instance_id ?? nested?.instanceId ?? nested?.instance_id;
  const accountUuid =
    p.accountUuid ?? p.account_uuid ?? nested?.accountUuid ?? nested?.account_uuid;
  if (typeof instanceId !== "string" || !instanceId) return undefined;
  if (typeof accountUuid !== "string" || !accountUuid) return undefined;
  return { instanceId, accountUuid };
}

function MainView({ view }: { view: string }) {
  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      {view === "home" && <HomePage />}
      {view === "versions" && <VersionsPage />}
      {view === "locker" && <LockerPage />}
      {view === "notify" && <NotifyPage />}
      {view === "relay" && <RelayPage />}
      {view === "gallery" && <GalleryPage />}
      {view === "store" && <StorePage />}
      {view === "host" && <HostPage />}
      {view === "settings" && <SettingsPage />}
    </div>
  );
}

const MemoMainView = memo(MainView);

export default function App() {
  const loadAll = useApp((s) => s.loadAll);
  const ready = useApp((s) => s.ready);
  const accounts = useApp((s) => s.accounts);
  const activeAccountId = useApp((s) => s.accounts.active ?? "");
  const view = useOctra((s) => s.view);
  const toggleOctraMenu = useOctra((s) => s.toggleOctraMenu);
  const setOctraMenuOpen = useOctra((s) => s.setOctraMenuOpen);
  const octraMenuOpen = useOctra((s) => s.octraMenuOpen);

  useEffect(() => {
    return installKeyboardGuard(toggleOctraMenu);
  }, [toggleOctraMenu]);

  useEffect(() => {
    if (!octraMenuOpen) return;
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") {
        e.preventDefault();
        setOctraMenuOpen(false);
      }
    }
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [octraMenuOpen, setOctraMenuOpen]);

  useEffect(() => {
    if (view === "notify" || view === "relay") {
      useOctra.getState().setView("home");
    }
  }, [view]);

  useEffect(() => {
    void loadAll();
    const unsubs: Array<() => void> = [];
    const {
      setProgress,
      refreshInstances,
      refreshAccounts,
      setLogin,
      setLoginOpen,
      markPlaying,
      markStopped,
      showError,
      showOk,
    } = useApp.getState();
    const { openCrash, syncLanFriends } = useOctra.getState();

    void listen<InstallProgress>("install-progress", (e) => setProgress(e.payload)).then(
      (u) => unsubs.push(u),
    );
    void listen("install-finished", () => {
      setProgress(null);
      void refreshInstances();
    }).then((u) => unsubs.push(u));
    void listen<Account>("auth-success", () => {
      setLogin(null);
      setLoginOpen(false);
      void refreshAccounts();
      showOk("Zalogowano.");
    }).then((u) => unsubs.push(u));
    void listen<string>("auth-error", (e) => {
      setLogin(null);
      setLoginOpen(false);
      showError(typeof e.payload === "string" ? e.payload : String(e.payload));
    }).then((u) => unsubs.push(u));
    void listen<unknown>("game-started", (e) => {
      const session = sessionFromGameEvent(e.payload);
      if (session) markPlaying(session.instanceId, session.accountUuid);
    }).then((u) => unsubs.push(u));
    void listen<unknown>("game-exited", (e) => {
      const session = sessionFromGameEvent(e.payload);
      if (session) {
        markStopped(session.instanceId, session.accountUuid);
        const code =
          typeof e.payload === "object" && e.payload && "code" in e.payload
            ? String((e.payload as { code?: unknown }).code ?? "exit")
            : "exit";
        if (code && code !== "0" && code !== "exit") {
          openCrash(session.instanceId, code);
          const inst = useApp
            .getState()
            .instances.find((i) => i.id === session.instanceId);
          notifyCrash(session.instanceId, code, inst?.name);
        }
      }
    }).then((u) => unsubs.push(u));
    void listen<{ id: string; name: string }>("relay-peer-online", (e) => {
      notifyFriendOnline(e.payload.name);
      syncLanFriends([
        {
          id: e.payload.id,
          name: e.payload.name,
          status: "online",
          detail: "W sieci LAN",
        },
      ]);
    }).then((u) => unsubs.push(u));

    const contentCheck = setInterval(() => {
      for (const inst of useApp.getState().instances) {
        api
          .checkContentUpdates(inst.id)
          .then((updates) => notifyContentUpdates(inst.name, updates.length))
          .catch(() => {});
      }
    }, 30 * 60 * 1000);

    return () => {
      unsubs.forEach((u) => u());
      clearInterval(contentCheck);
    };
  }, [loadAll]);

  useEffect(() => {
    if (!ready) return;
    (window as Window & { __octraBootFinish?: () => void }).__octraBootFinish?.();
  }, [ready]);

  useEffect(() => {
    if (!ready) return;
    const { settings, appInfo } = useApp.getState();
    if (settings?.autoCheckUpdates === false) return;
    void checkForUpdates(appInfo)
      .then((status) => {
        if (status.state === "available") {
          useApp
            .getState()
            .showOk(
              `Dostępna aktualizacja ${status.version}. Otwórz Ustawienia, aby zainstalować.`,
            );
        }
      })
      .catch(() => {});
  }, [ready]);

  if (!ready) return null;
  if (accounts.accounts.length === 0) {
    return (
      <>
        <LoginPage />
        <DialogHost />
      </>
    );
  }

  return (
    <div className="relative flex h-full flex-col bg-bg">
      <Titlebar />
      <div className="relative flex min-h-0 flex-1">
        <Rail />
        <main className="relative flex min-w-0 flex-1 flex-col">
          <MemoMainView key={activeAccountId} view={view} />
          <ContentOverlay />
          <CrashModal />
        </main>
      </div>
      <FriendsDrawer />
      <OctraMenu />
      <DialogHost />
    </div>
  );
}
