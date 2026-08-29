import { ChevronDown, Pencil, Plus, Save, X } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { clsx } from "clsx";
import { api } from "../lib/api";
import {
  normalizeTextureUrl,
} from "../lib/skinRender";
import { pl } from "../locales/pl";
import { useApp, useActiveAccount } from "../stores/appStore";
import type { AccountSkin, CatalogGroup, CatalogSkin, McPlayerProfile } from "../types";
import { SkinEditModal } from "../components/SkinEditModal";
import { SkinThumbnail } from "../components/SkinThumbnail";
import { SkinUploadDialog } from "../components/SkinUploadDialog";
import { SkinViewer3D } from "../components/SkinViewer3D";
import { clearAccountAvatarCache } from "../components/AccountAvatar";
import {
  toApiSkinModel,
  toUiSkinModel,
  type ApiSkinModel,
  type UiSkinModel,
} from "../lib/skinModel";

type DraftSkin = {
  textureKey?: string;
  variant: string;
  name: string;
  pngDataUrl?: string;
};

function pngDataUrlFromAccountSkin(skin: AccountSkin | null): string | null {
  if (!skin?.pngBase64) return null;
  return skin.pngBase64.startsWith("data:")
    ? skin.pngBase64
    : `data:image/png;base64,${skin.pngBase64}`;
}

function committedSkin(msSkin: AccountSkin | null): DraftSkin | null {
  const key = textureKeyFromUrl(msSkin?.textureUrl);
  const pngDataUrl = pngDataUrlFromAccountSkin(msSkin);
  if (!key && !pngDataUrl) return null;
  return {
    textureKey: key ?? undefined,
    variant: msSkin?.model === "slim" ? "slim" : "classic",
    name: "Aktywny",
    pngDataUrl: pngDataUrl ?? undefined,
  };
}

function draftSkinsEqual(a: DraftSkin | null, b: DraftSkin | null): boolean {
  if (!a && !b) return true;
  if (!a || !b) return false;
  return (
    a.textureKey === b.textureKey &&
    a.variant === b.variant &&
    a.pngDataUrl === b.pngDataUrl
  );
}

function textureKeyFromUrl(url?: string | null): string | null {
  if (!url) return null;
  const parts = url.split("/");
  return parts[parts.length - 1] || null;
}

function committedCapeId(profile: McPlayerProfile | null): string | null {
  return profile?.capes.find((c) => c.state.toUpperCase() === "ACTIVE")?.id ?? null;
}

export function LockerPage() {
  const acc = useActiveAccount();
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);
  const bump = useApp((s) => s.bumpSkin);
  const skinEpoch = useApp((s) => s.skinEpoch);

  const [catalog, setCatalog] = useState<CatalogGroup[]>([]);
  const [msSkin, setMsSkin] = useState<AccountSkin | null>(null);
  const [mcProfile, setMcProfile] = useState<McPlayerProfile | null>(null);
  const [loading, setLoading] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [model, setModel] = useState<UiSkinModel>("wide");
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});
  const [profileError, setProfileError] = useState<string | null>(null);
  const [profileLoading, setProfileLoading] = useState(false);

  const [draftSkin, setDraftSkin] = useState<DraftSkin | null>(null);
  const [draftCapeId, setDraftCapeId] = useState<string | null>(null);
  const [pendingPng, setPendingPng] = useState<Uint8Array | null>(null);
  const [offlinePreviewUrl, setOfflinePreviewUrl] = useState<string | null>(null);
  const [uploadPending, setUploadPending] = useState<{
    buf: Uint8Array;
    previewUrl: string;
  } | null>(null);
  const [uploadModel, setUploadModel] = useState<ApiSkinModel>("classic");

  const isOffline = acc?.kind === "offline";

  const dirty = useMemo(() => {
    if (isOffline) return false;
    const baseSkin = committedSkin(msSkin);
    const baseCape = committedCapeId(mcProfile);
    const skinChanged = !draftSkinsEqual(draftSkin, baseSkin);
    const capeChanged = (draftCapeId ?? null) !== (baseCape ?? null);
    return skinChanged || capeChanged;
  }, [draftSkin, draftCapeId, isOffline, mcProfile, msSkin]);

  const syncDraftFromCommitted = useCallback(() => {
    setDraftSkin(committedSkin(msSkin));
    setDraftCapeId(committedCapeId(mcProfile));
    setPendingPng(null);
  }, [mcProfile, msSkin]);

  useEffect(() => {
    api.getMojangSkinCatalog().then(setCatalog).catch(() => setCatalog([]));
  }, []);

  const reload = useCallback(
    async (refresh = false) => {
      if (!acc) return;
      setLoading(true);
      try {
        if (isOffline) {
          const os = await api.getOfflineSkin(acc.uuid);
          setMcProfile(null);
          setModel(toUiSkinModel(os.model));
          if (os.hasCustom && os.pngBase64) {
            const src = os.pngBase64.startsWith("data:")
              ? os.pngBase64
              : `data:image/png;base64,${os.pngBase64}`;
            setOfflinePreviewUrl(src);
          } else {
            setOfflinePreviewUrl(null);
          }
        } else {
          const skin = await api.getAccountSkin(acc.uuid, refresh);
          let profile: McPlayerProfile | null = null;
          try {
            profile = await api.getMinecraftProfile(acc.uuid);
            setProfileError(null);
          } catch (e) {
            const msg = e instanceof Error ? e.message : String(e);
            setProfileError(msg);
            if (refresh) showError(msg);
          }
          setMsSkin(skin);
          setMcProfile(profile);
          setModel(toUiSkinModel(skin.model));
          const nextSkin = committedSkin(skin);
          const nextCape = committedCapeId(profile);
          setDraftSkin(nextSkin);
          setDraftCapeId(nextCape);
        }
      } catch (e) {
        showError(e instanceof Error ? e.message : String(e));
      } finally {
        setLoading(false);
      }
    },
    [acc, isOffline, showError],
  );

  useEffect(() => {
    void reload(false);
  }, [acc?.uuid, isOffline, skinEpoch, reload]);

  useEffect(() => {
    if (!editOpen || !acc || isOffline) return;
    setProfileLoading(true);
    void api
      .getMinecraftProfile(acc.uuid)
      .then((profile) => {
        setMcProfile(profile);
        setProfileError(null);
      })
      .catch((e) => {
        const msg = e instanceof Error ? e.message : String(e);
        setProfileError(msg);
      })
      .finally(() => setProfileLoading(false));
  }, [editOpen, acc?.uuid, isOffline]);

  const displaySkin = draftSkin ?? committedSkin(msSkin);

  const viewerSkinPng = useMemo(() => {
    if (draftSkin?.pngDataUrl) return draftSkin.pngDataUrl;
    if (!dirty) return pngDataUrlFromAccountSkin(msSkin);
    return null;
  }, [dirty, draftSkin?.pngDataUrl, msSkin]);

  const draftSkinTextureKey = useMemo(() => {
    if (draftSkin?.pngDataUrl) return null;
    const skin = draftSkin ?? committedSkin(msSkin);
    return skin?.textureKey ?? null;
  }, [draftSkin, msSkin]);

  const draftSkinUrl = useMemo(() => {
    if (isOffline) {
      if (offlinePreviewUrl) return offlinePreviewUrl;
      if (acc) return `https://mc-heads.net/skin/${encodeURIComponent(acc.name)}`;
      return null;
    }
    return null;
  }, [acc, isOffline, offlinePreviewUrl]);

  const viewerModel = useMemo(() => {
    if (isOffline) return model === "slim" ? "slim" : "classic";
    const variant = (draftSkin ?? committedSkin(msSkin))?.variant;
    return variant === "slim" ? "slim" : "classic";
  }, [draftSkin, isOffline, model, msSkin]);

  const draftCapeUrl = useMemo(() => {
    if (!draftCapeId || !mcProfile) return null;
    const cape = mcProfile.capes.find((c) => c.id === draftCapeId);
    return cape ? normalizeTextureUrl(cape.url) : null;
  }, [draftCapeId, mcProfile]);

  async function onPremiumUpload(file: File) {
    if (!acc || isOffline) return;
    const buf = new Uint8Array(await file.arrayBuffer());
    const url = URL.createObjectURL(new Blob([buf], { type: "image/png" }));
    setPendingPng(buf);
    setDraftSkin({
      name: file.name.replace(/\.png$/i, "") || "Własny",
      variant: model === "slim" ? "slim" : "classic",
      pngDataUrl: url,
    });
  }

  function queueOfflineUpload(buf: Uint8Array) {
    const previewUrl = URL.createObjectURL(new Blob([buf], { type: "image/png" }));
    setUploadModel(toApiSkinModel(model));
    setUploadPending({ buf, previewUrl });
  }

  function cancelOfflineUpload() {
    if (uploadPending?.previewUrl) URL.revokeObjectURL(uploadPending.previewUrl);
    setUploadPending(null);
  }

  async function confirmOfflineUpload() {
    if (!acc || !isOffline || !uploadPending) return;
    setLoading(true);
    try {
      await api.saveOfflineSkin(acc.uuid, [...uploadPending.buf], uploadModel);
      setModel(uploadModel === "slim" ? "slim" : "wide");
      cancelOfflineUpload();
      bump();
      clearAccountAvatarCache();
      await reload(false);
      showOk("Zapisano skin.");
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  async function onUpload(file: File) {
    if (!acc || !isOffline) return;
    const buf = new Uint8Array(await file.arrayBuffer());
    queueOfflineUpload(buf);
  }

  function onCatalogPick(item: CatalogSkin) {
    if (!acc) return;
    if (isOffline) {
      if (item.name === "Steve") {
        void (async () => {
          try {
            await api.resetOfflineSkin(acc.uuid);
            bump();
            clearAccountAvatarCache();
            await reload(false);
            showOk("Przywrócono domyślny skin Steve.");
          } catch (e) {
            showError(e instanceof Error ? e.message : String(e));
          }
        })();
      } else {
        showError("Offline: wgraj własny PNG albo wybierz Steve z domyślnych.");
      }
      return;
    }
    setDraftSkin({
      textureKey: item.textureKey,
      variant: item.variant,
      name: item.name,
    });
    setPendingPng(null);
  }

  async function commitChanges() {
    if (!acc || isOffline || !draftSkin) return;
    setLoading(true);
    try {
      if (pendingPng) {
        const profile = await api.uploadMojangSkin(
          acc.uuid,
          [...pendingPng],
          draftSkin.variant,
        );
        setMcProfile(profile);
        setPendingPng(null);
      } else if (draftSkin.textureKey) {
        const profile = await api.equipMojangSkin(
          acc.uuid,
          draftSkin.textureKey,
          draftSkin.variant,
        );
        setMcProfile(profile);
      } else {
        showError("Brak skina do zapisania.");
        return;
      }
      const updated = await api.setMinecraftCape(acc.uuid, draftCapeId);
      setMcProfile(updated);
      bump();
      clearAccountAvatarCache();
      await reload(true);
      setEditOpen(false);
      showOk("Zapisano skin i pelerynę.");
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  function cancelDraft() {
    syncDraftFromCommitted();
    setEditOpen(false);
  }

  function toggleGroup(id: string) {
    setCollapsed((c) => ({ ...c, [id]: !c[id] }));
  }

  if (!acc) {
    return (
      <div className="grid flex-1 place-items-center p-8 text-mute">
        Wybierz konto, żeby zarządzać skinem.
      </div>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <div className="flex min-h-0 flex-1 overflow-hidden">
        <aside className="flex w-[min(100%,320px)] shrink-0 flex-col border-r border-line bg-raised/40 p-5">
          <span className="mx-auto rounded-lg bg-black/40 px-3 py-1 text-xs font-semibold">
            {acc.name}
          </span>
          <SkinViewer3D
            large
            className="mt-4 flex-1"
            skinPngDataUrl={loading ? null : viewerSkinPng}
            skinUrl={isOffline && !loading ? draftSkinUrl : null}
            skinTextureKey={loading ? null : draftSkinTextureKey}
            capeUrl={isOffline ? null : draftCapeUrl}
            model={viewerModel}
          />
          <button
            type="button"
            onClick={() => setEditOpen(true)}
            className="mt-4 flex w-full items-center justify-center gap-2 rounded-xl border border-line bg-raised2 py-2.5 text-sm font-semibold hover:bg-white/6"
          >
            <Pencil size={15} />
            Edytuj skin
          </button>
        </aside>

        <div className="min-w-0 flex-1 overflow-y-auto p-6">
          <h1 className="text-2xl font-extrabold tracking-tight">Wybór skina</h1>
          <p className="mt-1 text-sm text-mute">
            {isOffline
              ? pl.accounts.nonPremium
              : `${pl.accounts.premium} — kliknij skin, potem zatwierdź zmiany`}
          </p>

          {!isOffline && (
            <section className="mt-8">
              <p className="text-sm font-semibold text-mute">Saved skins</p>
              <div className="mt-3 grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                <label className="flex aspect-[3/4] cursor-pointer flex-col items-center justify-center rounded-2xl border border-dashed border-line bg-raised2/50 text-center text-xs text-mute hover:border-accent/40">
                  <Plus size={22} className="mb-2 text-ink" />
                  Dodaj skin
                  <input
                    type="file"
                    accept="image/png"
                    className="hidden"
                    onChange={(e) => {
                      const f = e.target.files?.[0];
                      if (f) void onPremiumUpload(f);
                    }}
                  />
                </label>
                <button
                  type="button"
                  onClick={() => syncDraftFromCommitted()}
                  className={clsx(
                    "relative aspect-[3/4] overflow-hidden rounded-2xl border bg-raised2 transition hover:border-accent/40",
                    !dirty &&
                      draftSkinsEqual(displaySkin, committedSkin(msSkin))
                      ? "border-good ring-2 ring-good/40"
                      : "border-line",
                  )}
                >
                  {displaySkin?.pngDataUrl ? (
                    <img
                      src={displaySkin.pngDataUrl}
                      alt=""
                      className="h-full w-full object-contain p-2 [image-rendering:pixelated]"
                    />
                  ) : displaySkin?.textureKey ? (
                    <SkinThumbnail
                      textureKey={displaySkin.textureKey}
                      variant={displaySkin.variant === "slim" ? "slim" : "classic"}
                      alt="Aktywny"
                    />
                  ) : (
                    <div className="grid h-full place-items-center text-xs text-mute">Aktywny</div>
                  )}
                  <span className="absolute bottom-2 left-2 rounded-md bg-black/60 px-2 py-0.5 text-[10px]">
                    Aktywny
                  </span>
                </button>
              </div>
            </section>
          )}

          {isOffline && (
            <section className="mt-8">
              <p className="text-sm font-semibold text-mute">Własne skiny</p>
              <div className="mt-2 flex flex-wrap gap-4 text-sm text-mute">
                <span>Model przy uploadzie:</span>
                <label className="flex items-center gap-2">
                  <input
                    type="radio"
                    name="offline-arm"
                    checked={model === "wide"}
                    onChange={() => setModel("wide")}
                  />
                  Steve (szerokie)
                </label>
                <label className="flex items-center gap-2">
                  <input
                    type="radio"
                    name="offline-arm"
                    checked={model === "slim"}
                    onChange={() => setModel("slim")}
                  />
                  Alex (smukłe)
                </label>
              </div>
              <div className="mt-3 grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                <label className="flex aspect-[3/4] cursor-pointer flex-col items-center justify-center rounded-2xl border border-dashed border-line bg-raised2/50 text-center text-xs text-mute hover:border-accent/40">
                  <Plus size={22} className="mb-2 text-ink" />
                  Dodaj skin
                  <input
                    type="file"
                    accept="image/png"
                    className="hidden"
                    onChange={(e) => {
                      const f = e.target.files?.[0];
                      if (f) void onUpload(f);
                    }}
                  />
                </label>
              </div>
            </section>
          )}

          {catalog.map((group) => (
            <section key={group.id} className="mt-8">
              <button
                type="button"
                className="flex w-full items-center gap-2 text-left text-sm font-semibold text-mute"
                onClick={() => toggleGroup(group.id)}
              >
                <ChevronDown
                  size={16}
                  className={clsx("transition", collapsed[group.id] && "-rotate-90")}
                />
                {group.title}
              </button>
              {!collapsed[group.id] && (
                <div className="mt-3 grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                  {group.skins.map((item) => {
                    const selected =
                      !draftSkin?.pngDataUrl &&
                      draftSkin?.textureKey === item.textureKey &&
                      draftSkin.variant === item.variant;
                    return (
                      <button
                        key={`${item.textureKey}-${item.variant}`}
                        type="button"
                        disabled={loading}
                        onClick={() => onCatalogPick(item)}
                        className={clsx(
                          "group relative aspect-[3/4] overflow-hidden rounded-2xl border bg-raised2 transition hover:border-accent/40 disabled:opacity-60",
                          selected ? "border-good ring-2 ring-good/40" : "border-line",
                        )}
                      >
                        <SkinThumbnail
                          textureKey={item.textureKey}
                          variant={item.variant === "slim" ? "slim" : "classic"}
                          alt={item.name}
                        />
                        <span className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/85 to-transparent px-2 py-2 text-left text-[10px] font-medium">
                          {item.name}
                        </span>
                      </button>
                    );
                  })}
                </div>
              )}
            </section>
          ))}
        </div>
      </div>

      {!isOffline && dirty && (
        <div className="flex shrink-0 items-center justify-end gap-2 border-t border-line bg-raised/95 px-5 py-3 backdrop-blur-sm">
          <button
            type="button"
            disabled={loading}
            onClick={cancelDraft}
            className="inline-flex items-center gap-2 rounded-xl border border-line px-4 py-2 text-sm text-mute hover:bg-white/5 disabled:opacity-50"
          >
            <X size={15} />
            Anuluj
          </button>
          <button
            type="button"
            disabled={loading || !draftSkin}
            onClick={() => void commitChanges()}
            className="inline-flex items-center gap-2 rounded-xl bg-good px-4 py-2 text-sm font-semibold text-black disabled:opacity-50"
          >
            <Save size={15} />
            Zapisz skin
          </button>
        </div>
      )}

      <SkinUploadDialog
        open={uploadPending !== null}
        previewUrl={uploadPending?.previewUrl ?? null}
        model={uploadModel}
        onModelChange={setUploadModel}
        onConfirm={() => void confirmOfflineUpload()}
        onCancel={cancelOfflineUpload}
        busy={loading}
      />

      <SkinEditModal
        open={editOpen}
        account={acc}
        mcProfile={mcProfile}
        skinPngDataUrl={viewerSkinPng}
        skinUrl={draftSkinUrl}
        skinTextureKey={draftSkinTextureKey}
        capeUrl={draftCapeUrl}
        viewerModel={viewerModel}
        draftCapeId={draftCapeId}
        profileError={profileError}
        profileLoading={profileLoading}
        model={model}
        dirty={dirty}
        onModelChange={setModel}
        onClose={cancelDraft}
        onSaveOffline={async () => {
          if (!acc) return;
          try {
            await api.setOfflineSkinModel(acc.uuid, toApiSkinModel(model));
            bump();
            clearAccountAvatarCache();
            await reload(false);
            setEditOpen(false);
          } catch (e) {
            showError(e instanceof Error ? e.message : String(e));
          }
        }}
        onUpload={(f) => void onUpload(f)}
        onPremiumUpload={(f) => void onPremiumUpload(f)}
        onRefreshPremium={async () => {
          await reload(true);
          bump();
          clearAccountAvatarCache();
        }}
        onDraftCapeChange={setDraftCapeId}
        onSave={() => void commitChanges()}
        busy={loading}
      />
    </div>
  );
}
