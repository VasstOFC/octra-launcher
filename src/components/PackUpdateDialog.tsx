import { Download, Loader2, X } from "lucide-react";
import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { pl } from "../locales/pl";
import { useApp } from "../stores/appStore";
import { usePackUpdate } from "../stores/packUpdateStore";
import type { PackUpdateInfo } from "../types";

function stripMarkdown(md: string): string {
  return md
    .replace(/```[\s\S]*?```/g, "")
    .replace(/`([^`]+)`/g, "$1")
    .replace(/!\[[^\]]*\]\([^)]+\)/g, "")
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .replace(/^#{1,6}\s+/gm, "")
    .replace(/[*_~]/g, "")
    .trim();
}

export function PackUpdateDialog() {
  const open = usePackUpdate((s) => s.open);
  const instanceId = usePackUpdate((s) => s.instanceId);
  const close = usePackUpdate((s) => s.close);
  const instances = useApp((s) => s.instances);
  const refresh = useApp((s) => s.refreshInstances);
  const progress = useApp((s) => s.progress);
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);
  const [info, setInfo] = useState<PackUpdateInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [updating, setUpdating] = useState(false);

  const inst = instances.find((i) => i.id === instanceId) ?? null;

  useEffect(() => {
    if (!open || !instanceId) {
      setInfo(null);
      return;
    }
    let cancelled = false;
    setLoading(true);
    void api
      .checkPackUpdate(instanceId)
      .then((res) => {
        if (!cancelled) setInfo(res);
      })
      .catch((e: unknown) => {
        if (!cancelled) {
          showError(e instanceof Error ? e.message : String(e));
          close();
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [open, instanceId, close, showError]);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") close();
    }
    if (open) document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [open, close]);

  if (!open || !inst) return null;

  const busy = Boolean(progress) || updating;

  async function updatePack() {
    if (!instanceId) return;
    setUpdating(true);
    try {
      await api.resyncInstancePack(instanceId);
      await refresh();
      showOk(pl.packUpdate.updated.replace("{name}", inst!.name));
      close();
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setUpdating(false);
    }
  }

  return (
    <div className="absolute inset-0 z-50 grid place-items-center bg-black/70 p-6">
      <div className="relative w-full max-w-lg rounded-2xl border border-line bg-raised p-6 shadow-2xl">
        <button
          type="button"
          onClick={close}
          className="absolute right-3 top-3 grid h-8 w-8 place-items-center rounded-lg text-mute hover:bg-raised2 hover:text-ink"
          aria-label={pl.common.close}
        >
          <X size={16} />
        </button>
        <h2 className="pr-8 text-lg font-bold text-ink">{pl.packUpdate.title}</h2>
        <p className="mt-1 text-sm text-mute">{inst.name}</p>

        {loading ? (
          <div className="mt-6 flex items-center gap-2 text-sm text-mute">
            <Loader2 size={16} className="animate-spin" />
            {pl.packUpdate.checking}
          </div>
        ) : info && !info.hasUpdate ? (
          <p className="mt-6 text-sm text-mute">{pl.packUpdate.upToDate}</p>
        ) : info ? (
          <div className="mt-4 space-y-3">
            <div className="grid grid-cols-2 gap-3 text-sm">
              <div className="rounded-xl border border-line bg-raised2/50 p-3">
                <p className="text-[10px] font-bold uppercase text-mute">
                  {pl.packUpdate.current}
                </p>
                <p className="mt-1 font-semibold">{info.currentVersion ?? "—"}</p>
              </div>
              <div className="rounded-xl border border-accent/30 bg-accent/10 p-3">
                <p className="text-[10px] font-bold uppercase text-mute">
                  {pl.packUpdate.latest}
                </p>
                <p className="mt-1 font-semibold text-ink">{info.latestVersion}</p>
              </div>
            </div>
            {info.changelog ? (
              <div className="max-h-40 overflow-auto rounded-xl border border-line bg-raised2/30 p-3 text-sm text-mute">
                <p className="mb-1 text-[10px] font-bold uppercase">{pl.store.changelog}</p>
                <p className="whitespace-pre-wrap">{stripMarkdown(info.changelog)}</p>
              </div>
            ) : null}
          </div>
        ) : null}

        <div className="mt-6 flex justify-end gap-2">
          <button
            type="button"
            onClick={close}
            className="rounded-full border border-line px-4 py-2 text-sm text-mute hover:text-ink"
          >
            {info?.hasUpdate ? pl.packUpdate.later : pl.common.close}
          </button>
          {info?.hasUpdate ? (
            <button
              type="button"
              disabled={busy}
              onClick={() => void updatePack()}
              className="inline-flex items-center gap-2 rounded-full bg-accent px-4 py-2 text-sm font-semibold text-bg-on-accent disabled:opacity-50"
            >
              {updating ? (
                <Loader2 size={14} className="animate-spin" />
              ) : (
                <Download size={14} />
              )}
              {pl.packUpdate.update}
            </button>
          ) : null}
        </div>
      </div>
    </div>
  );
}
