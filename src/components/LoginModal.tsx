import { Copy, ExternalLink, X } from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { pl } from "../locales/pl";
import { useApp } from "../stores/appStore";

export function LoginModal() {
  const login = useApp((s) => s.login);
  const loginOpen = useApp((s) => s.loginOpen);
  const setLogin = useApp((s) => s.setLogin);
  const setLoginOpen = useApp((s) => s.setLoginOpen);
  const showError = useApp((s) => s.showError);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!loginOpen || login) return;
    let cancelled = false;
    setBusy(true);
    void api
      .startLogin()
      .then((code) => {
        if (!cancelled) setLogin(code);
      })
      .catch((e: unknown) => {
        if (!cancelled) {
          showError(e instanceof Error ? e.message : String(e));
          setLoginOpen(false);
        }
      })
      .finally(() => {
        if (!cancelled) setBusy(false);
      });
    return () => {
      cancelled = true;
    };
  }, [loginOpen, login, setLogin, setLoginOpen, showError]);

  if (!loginOpen) return null;

  async function close() {
    setLoginOpen(false);
    setLogin(null);
    try {
      await api.cancelLogin();
    } catch {
      /* ignore */
    }
  }

  return (
    <div className="absolute inset-0 z-50 grid place-items-center bg-black/70 p-6">
      <div className="relative w-full max-w-md rounded-2xl border border-line bg-raised p-6 shadow-2xl">
        <button
          type="button"
          onClick={() => void close()}
          className="absolute right-3 top-3 grid h-8 w-8 place-items-center rounded-lg text-mute hover:bg-raised2 hover:text-ink"
          aria-label={pl.login.reauthCancel}
        >
          <X size={16} />
        </button>
        <h2 className="pr-8 text-lg font-bold text-ink">{pl.login.reauthTitle}</h2>
        <p className="mt-1 text-sm text-mute">{pl.login.reauthSubtitle}</p>

        {busy && !login ? (
          <p className="mt-6 text-sm text-mute">{pl.login.reauthLoading}</p>
        ) : null}

        {login ? (
          <div className="mt-5 rounded-xl border border-line bg-raised2 p-4">
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
        ) : null}
      </div>
    </div>
  );
}
