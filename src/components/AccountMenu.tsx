import { useEffect, useRef, useState } from "react";

import { clsx } from "clsx";

import {
  ArrowLeft,
  Check,
  Copy,
  ExternalLink,
  Plus,
  Settings2,
  Trash2,
  UserRound,
} from "lucide-react";

import { openUrl } from "@tauri-apps/plugin-opener";

import { api } from "../lib/api";

import { confirmDialog, promptDialog } from "../lib/dialog";

import { pl } from "../locales/pl";

import { useApp, useActiveAccount } from "../stores/appStore";

import type { Account } from "../types";

import { AccountAvatar, clearAccountAvatarCache } from "./AccountAvatar";
import { AccountContextMenu } from "./AccountContextMenu";

function accountTypeLabel(acc: Account): string {
  return acc.kind === "offline" ? pl.accounts.nonPremium : pl.accounts.premium;
}

export function AccountMenu({ expanded = false }: { expanded?: boolean }) {
  const accounts = useApp((s) => s.accounts);
  const refreshAccounts = useApp((s) => s.refreshAccounts);
  const bumpSkin = useApp((s) => s.bumpSkin);
  const setLogin = useApp((s) => s.setLogin);
  const login = useApp((s) => s.login);
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);

  const [open, setOpen] = useState(false);
  const [manage, setManage] = useState(false);
  const [busy, setBusy] = useState(false);
  const [ctxMenu, setCtxMenu] = useState<{
    x: number;
    y: number;
    acc: Account;
  } | null>(null);

  const ref = useRef<HTMLDivElement>(null);
  const active = useActiveAccount();

  useEffect(() => {
    if (!open) return;

    function onDoc(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
        setManage(false);
      }
    }

    document.addEventListener("mousedown", onDoc);
    return () => document.removeEventListener("mousedown", onDoc);
  }, [open]);

  function openAccountContext(e: React.MouseEvent, acc: Account) {
    e.preventDefault();
    e.stopPropagation();
    setCtxMenu({ x: e.clientX, y: e.clientY, acc });
  }

  async function switchTo(uuid: string) {
    if (manage) return;

    if (uuid === accounts.active) {
      setOpen(false);
      return;
    }

    try {
      await api.setActiveAccount(uuid);
      await refreshAccounts();
      bumpSkin();
      clearAccountAvatarCache();
      showOk("Przełączono konto.");
      setOpen(false);
      setCtxMenu(null);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    }
  }

  async function addMicrosoft() {
    setBusy(true);
    try {
      const code = await api.startLogin();
      setLogin(code);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function addOffline() {
    const name = await promptDialog(pl.accounts.offlineNickPrompt, {
      title: pl.accounts.addNonPremium,
      confirmLabel: "Dodaj",
      defaultValue: "Player",
    });
    if (!name) return;

    try {
      await api.addOfflineAccount(name);
      await refreshAccounts();
      showOk(pl.accounts.addedNonPremium);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    }
  }

  async function removeAccount(acc: Account) {
    const ok = await confirmDialog(
      pl.accounts.removeConfirm.replace("{name}", acc.name),
      { title: pl.accounts.removeTitle, confirmLabel: pl.accounts.remove, danger: true },
    );
    if (!ok) return;

    try {
      await api.logoutAccount(acc.uuid);
      await refreshAccounts();
      bumpSkin();
      clearAccountAvatarCache();
      showOk(pl.accounts.removed.replace("{name}", acc.name));
      setCtxMenu(null);
      setOpen(false);
      setManage(false);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    }
  }

  function renderAccountRow(acc: Account, options?: { showDeleteButton?: boolean }) {
    const isActive = accounts.active === acc.uuid;

    return (
      <li
        key={acc.uuid}
        className={clsx(
          "flex items-center gap-2 rounded-lg px-2 py-2",
          manage ? "hover:bg-white/4" : "",
        )}
        onContextMenu={(e) => openAccountContext(e, acc)}
      >
        {manage ? (
          <>
            <AccountAvatar account={acc} size={36} />
            <span className="min-w-0 flex-1">
              <span className="block truncate text-sm font-semibold">{acc.name}</span>
              <span className="text-[10px] text-mute">{accountTypeLabel(acc)}</span>
            </span>
            {isActive && (
              <span className="text-[10px] font-medium text-good">{pl.accounts.active}</span>
            )}
            {options?.showDeleteButton && (
              <button
                type="button"
                title={pl.accounts.remove}
                className="grid h-8 w-8 shrink-0 place-items-center rounded-lg text-mute hover:bg-danger/15 hover:text-danger"
                onClick={() => void removeAccount(acc)}
              >
                <Trash2 size={15} />
              </button>
            )}
          </>
        ) : (
          <button
            type="button"
            onClick={() => void switchTo(acc.uuid)}
            className="flex w-full items-center gap-2 rounded-lg text-left hover:bg-white/6"
          >
            <AccountAvatar account={acc} size={36} />
            <span className="min-w-0 flex-1">
              <span className="block truncate text-sm font-semibold">{acc.name}</span>
              <span className="text-[10px] text-mute">{accountTypeLabel(acc)}</span>
            </span>
            {isActive && <Check size={14} className="shrink-0 text-good" />}
          </button>
        )}
      </li>
    );
  }

  return (
    <div ref={ref} className={clsx("relative", expanded ? "w-full" : "")}>
      <button
        type="button"
        title={active ? `${pl.accounts.title} — ${pl.accounts.contextHint}` : pl.accounts.title}
        aria-label={pl.accounts.title}
        onClick={() => setOpen((v) => !v)}
        onContextMenu={(e) => {
          if (active) openAccountContext(e, active);
        }}
        className={clsx(
          "relative flex items-center rounded-xl transition",
          expanded ? "h-11 w-full gap-3 px-3" : "h-11 w-11 justify-center",
          open ? "bg-white/8 text-accent" : "text-mute hover:bg-white/5 hover:text-ink",
        )}
      >
        {active ? (
          <AccountAvatar
            account={active}
            size={expanded ? 26 : 28}
            className="shrink-0 rounded-md ring-0"
          />
        ) : (
          <UserRound size={21} strokeWidth={1.75} className="shrink-0" />
        )}
        {expanded ? (
          <span className="min-w-0 truncate text-left text-[13px] font-medium">
            {active?.name ?? pl.accounts.title}
          </span>
        ) : null}
      </button>

      {open && (
        <div className="absolute bottom-0 left-full z-50 ml-2 w-80 rounded-xl border border-line bg-raised p-2 shadow-xl">
          {manage ? (
            <>
              <div className="flex items-center gap-2 px-1 py-1">
                <button
                  type="button"
                  onClick={() => setManage(false)}
                  className="grid h-8 w-8 place-items-center rounded-lg text-mute hover:bg-white/6 hover:text-ink"
                >
                  <ArrowLeft size={16} />
                </button>
                <p className="text-sm font-semibold">{pl.accounts.manage}</p>
              </div>
              <ul className="mt-1 max-h-56 overflow-y-auto">
                {accounts.accounts.map((acc) =>
                  renderAccountRow(acc, { showDeleteButton: true }),
                )}
              </ul>
              <p className="mt-2 px-2 text-[10px] leading-relaxed text-mute">
                {pl.accounts.manageHint}
              </p>
              <p className="px-2 text-[10px] leading-relaxed text-mute">
                {pl.accounts.contextHint}
              </p>
            </>
          ) : (
            <>
              <p className="px-2 py-1 text-[10px] font-semibold uppercase tracking-wider text-mute">
                {pl.accounts.title}
              </p>
              <ul className="max-h-48 overflow-y-auto">
                {accounts.accounts.map((acc) => renderAccountRow(acc))}
              </ul>

              <div className="mt-1 border-t border-line pt-1">
                <button
                  type="button"
                  disabled={busy}
                  onClick={() => void addMicrosoft()}
                  className="flex w-full items-center gap-2 rounded-lg px-2 py-2 text-sm hover:bg-white/6"
                >
                  <Plus size={14} />
                  {pl.accounts.addPremium}
                </button>
                <button
                  type="button"
                  onClick={() => void addOffline()}
                  className="flex w-full items-center gap-2 rounded-lg px-2 py-2 text-sm hover:bg-white/6"
                >
                  <Plus size={14} />
                  {pl.accounts.addNonPremium}
                </button>
                <button
                  type="button"
                  onClick={() => setManage(true)}
                  className="flex w-full items-center gap-2 rounded-lg px-2 py-2 text-sm text-mute hover:bg-white/6 hover:text-ink"
                >
                  <Settings2 size={14} />
                  {pl.accounts.manage}
                </button>
              </div>

              {login && (
                <div className="mt-2 rounded-lg border border-line bg-raised2 p-2">
                  <p className="text-[10px] text-mute">Kod Microsoft</p>
                  <div className="mt-1 flex items-center justify-between">
                    <span className="font-mono text-sm tracking-widest text-accent">
                      {login.userCode}
                    </span>
                    <button
                      type="button"
                      onClick={() => navigator.clipboard.writeText(login.userCode)}
                      className="text-mute hover:text-ink"
                    >
                      <Copy size={14} />
                    </button>
                  </div>
                  <button
                    type="button"
                    className="mt-2 flex w-full items-center justify-center gap-1 rounded-lg bg-accent py-1.5 text-xs font-semibold text-white"
                    onClick={() =>
                      openUrl(login.verificationUriComplete || login.verificationUri).catch(
                        () => undefined,
                      )
                    }
                  >
                    <ExternalLink size={12} />
                    Otwórz microsoft.com/link
                  </button>
                </div>
              )}

              {active && (
                <p className="mt-2 px-2 text-[10px] text-mute">
                  {pl.accounts.active}: <span className="text-ink">{active.name}</span>
                  <span className="block text-[9px] opacity-80">{pl.accounts.contextHint}</span>
                </p>
              )}
            </>
          )}
        </div>
      )}

      {ctxMenu && (
        <AccountContextMenu
          x={ctxMenu.x}
          y={ctxMenu.y}
          acc={ctxMenu.acc}
          isActive={accounts.active === ctxMenu.acc.uuid}
          onClose={() => setCtxMenu(null)}
          onSwitch={() => {
            setCtxMenu(null);
            void switchTo(ctxMenu.acc.uuid);
          }}
          onRemove={() => {
            setCtxMenu(null);
            void removeAccount(ctxMenu.acc);
          }}
        />
      )}
    </div>
  );
}
