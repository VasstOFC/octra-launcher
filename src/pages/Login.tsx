import { Copy, ExternalLink } from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";
import { api } from "../lib/api";
import { promptDialog } from "../lib/dialog";
import { pl } from "../locales/pl";
import { useApp } from "../stores/appStore";
import { Mark, WindowButtons } from "../components/WindowButtons";

const LOGIN_BG = "/login-bg.jpg";

export function LoginPage() {
  const login = useApp((s) => s.login);
  const appInfo = useApp((s) => s.appInfo);
  const setLogin = useApp((s) => s.setLogin);
  const setLoginOpen = useApp((s) => s.setLoginOpen);
  const showError = useApp((s) => s.showError);
  const refreshAccounts = useApp((s) => s.refreshAccounts);
  const [busy, setBusy] = useState(false);

  async function ms() {
    setBusy(true);
    setLoginOpen(true);
    try {
      const code = await api.startLogin();
      setLogin(code);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
      setLoginOpen(false);
    } finally {
      setBusy(false);
    }
  }

  async function offline() {
    const name = await promptDialog(pl.login.offlinePrompt, {
      title: pl.login.offlineTitle,
      confirmLabel: pl.login.offlineConfirm,
      defaultValue: "Player",
    });
    if (!name) return;
    try {
      await api.addOfflineAccount(name);
      await refreshAccounts();
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <div className="relative flex h-full flex-col overflow-hidden bg-bg">
      <img
        src={LOGIN_BG}
        alt=""
        className="pointer-events-none absolute inset-0 h-full w-full scale-105 object-cover"
        draggable={false}
      />
      <div className="pointer-events-none absolute inset-0 bg-gradient-to-b from-black/70 via-black/55 to-black/80" />
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_center,transparent_0%,rgb(0_0_0_/_0.45)_100%)]" />

      <header
        data-tauri-drag-region
        className="drag-region relative z-10 flex h-10 shrink-0 items-center justify-between px-3 text-[11px] text-white/70"
      >
        <span>
          {appInfo?.version
            ? `${pl.login.version} v${appInfo.version}`
            : pl.login.version}
        </span>
        <WindowButtons />
      </header>

      <div className="relative z-10 flex min-h-0 flex-1 items-center justify-center p-6">
        <div className="w-full max-w-[420px] rounded-2xl border border-white/12 bg-raised/88 p-8 shadow-[0_24px_80px_rgb(0_0_0_/_0.45)] backdrop-blur-xl">
          <div className="flex flex-col items-center text-center">
            <Mark size={44} />
            <h1 className="mt-4 text-2xl font-bold tracking-tight text-ink">
              {pl.login.title}
            </h1>
            <p className="mt-2 text-sm leading-relaxed text-mute">{pl.login.subtitle}</p>
          </div>

          <button
            type="button"
            className="mt-8 flex w-full items-center justify-center gap-2 rounded-xl bg-white py-3.5 text-sm font-semibold text-black transition hover:bg-white/92 disabled:opacity-60"
            onClick={() => void ms()}
            disabled={busy}
          >
            {pl.login.ms}
          </button>

          {login && (
            <div className="mt-4 rounded-xl border border-line bg-raised2/90 p-4">
              <p className="text-xs text-mute">{pl.login.msCode}</p>
              <div className="mt-2 flex items-center justify-between gap-2">
                <span className="font-mono text-lg tracking-[0.22em] text-accent">
                  {login.userCode}
                </span>
                <button
                  type="button"
                  onClick={() => navigator.clipboard.writeText(login.userCode)}
                  className="grid h-8 w-8 shrink-0 place-items-center rounded-lg text-mute hover:bg-white/6 hover:text-ink"
                  aria-label="Kopiuj kod"
                >
                  <Copy size={16} />
                </button>
              </div>
              <button
                type="button"
                className="mt-3 flex w-full items-center justify-center gap-2 rounded-xl bg-accent py-2.5 text-sm font-semibold text-white"
                onClick={() =>
                  openUrl(login.verificationUriComplete || login.verificationUri).catch(
                    () => undefined,
                  )
                }
              >
                <ExternalLink size={14} />
                {pl.login.msOpen}
              </button>
            </div>
          )}

          <button
            type="button"
            className="mt-3 w-full rounded-xl border border-white/14 py-3.5 text-sm font-semibold text-ink transition hover:bg-white/6"
            onClick={() => void offline()}
          >
            {pl.login.offline}
          </button>
        </div>
      </div>
    </div>
  );
}
