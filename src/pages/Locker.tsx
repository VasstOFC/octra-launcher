import { ChevronDown, Trash2 } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { clsx } from "clsx";
import { confirmDialog } from "../lib/dialog";
import { api } from "../lib/api";
import { normalizeTextureUrl } from "../lib/skinRender";
import {
  buildSavedSkinsList,
  dedupeSkinsByName,
  draftFromSkin,
  groupSkinsBySection,
  pngDataUrlFromSkin,
  skinIdentity,
  skinToUiModel,
  uiModelToSkinVariant,
  type Cape,
  type Skin,
} from "../lib/skins";
import { pl } from "../locales/pl";
import { useApp, useActiveAccount } from "../stores/appStore";
import type {
  AccountSkin,
  CatalogSkin,
  McPlayerProfile,
  SkinLibraryEntry,
} from "../types";
import { LockerLoadingScreen } from "../components/LockerLoadingScreen";
import { SkinEditModal } from "../components/SkinEditModal";
import { SkinAddCard, SkinGridButton } from "../components/SkinGridButton";
import { LockerSkinGrid } from "../components/LockerSkinGrid";
import { SkinPreviewPanel } from "../components/SkinPreviewPanel";
import { SkinUploadDialog } from "../components/SkinUploadDialog";
import { clearAccountAvatarCache } from "../components/AccountAvatar";
import {
  toApiSkinModel,
  toUiSkinModel,
  type ApiSkinModel,
  type UiSkinModel,
} from "../lib/skinModel";

function savedSkinId(skin: Skin): string {
  return skin.libraryId ?? skin.textureKey;
}

function skinTextureProps(skin: Skin) {
  const variant = skinToUiModel(skin.variant);
  const skinPngDataUrl = pngDataUrlFromSkin(skin);
  return {
    variant,
    skinPngDataUrl,
    textureKey: skinPngDataUrl ? null : skin.textureKey,
    alt: skin.name ?? skin.textureKey,
  };
}

function catalogSkinFromSkin(skin: Skin): CatalogSkin {
  return {
    id: skin.textureKey,
    name: skin.name ?? skin.textureKey,
    textureKey: skin.textureKey,
    variant: skin.variant === "SLIM" ? "slim" : "classic",
  };
}

type DraftSkin = {
  textureKey?: string;
  variant: string;
  name: string;
  pngDataUrl?: string;
  libraryId?: string;
};

function pngDataUrlFromAccountSkin(skin: AccountSkin | null): string | null {
  if (!skin?.pngBase64) return null;
  return skin.pngBase64.startsWith("data:")
    ? skin.pngBase64
    : `data:image/png;base64,${skin.pngBase64}`;
}

function pngDataUrlFromBase64(b64?: string | null): string | null {
  if (!b64) return null;
  return b64.startsWith("data:") ? b64 : `data:image/png;base64,${b64}`;
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
    a.pngDataUrl === b.pngDataUrl &&
    a.libraryId === b.libraryId
  );
}

function textureKeyFromUrl(url?: string | null): string | null {
  if (!url) return null;
  const parts = url.split("/");
  return parts[parts.length - 1] || null;
}

function committedCapeId(profile: McPlayerProfile | null, capes: Cape[] = []): string | null {
  const fromProfile =
    profile?.capes.find((c) => c.state.toUpperCase() === "ACTIVE")?.id ?? null;
  if (fromProfile) return fromProfile;
  return capes.find((c) => c.isEquipped)?.id ?? null;
}

export function LockerPage() {
  const acc = useActiveAccount();
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);
  const bump = useApp((s) => s.bumpSkin);
  const skinEpoch = useApp((s) => s.skinEpoch);

  const [allSkins, setAllSkins] = useState<Skin[]>([]);
  const [msSkin, setMsSkin] = useState<AccountSkin | null>(null);
  const [mcProfile, setMcProfile] = useState<McPlayerProfile | null>(null);
  const [loading, setLoading] = useState(false);
  const [initialLoad, setInitialLoad] = useState(true);
  const [editOpen, setEditOpen] = useState(false);
  const [model, setModel] = useState<UiSkinModel>("wide");
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});
  const [profileError, setProfileError] = useState<string | null>(null);
  const [profileLoading, setProfileLoading] = useState(false);
  const [availableCapes, setAvailableCapes] = useState<Cape[]>([]);
  const [capesLoading, setCapesLoading] = useState(false);
  const [capesError, setCapesError] = useState<string | null>(null);

  const [draftSkin, setDraftSkin] = useState<DraftSkin | null>(null);
  const [draftCapeId, setDraftCapeId] = useState<string | null>(null);
  const [pendingPng, setPendingPng] = useState<Uint8Array | null>(null);
  const [offlinePreviewUrl, setOfflinePreviewUrl] = useState<string | null>(null);
  const [previewingOfflineId, setPreviewingOfflineId] = useState<string | null>(null);
  const [previewingPremiumId, setPreviewingPremiumId] = useState<string | null>(null);

  const [uploadPending, setUploadPending] = useState<{
    buf: Uint8Array;
    previewUrl: string;
    forPremium?: boolean;
  } | null>(null);
  const [uploadModel, setUploadModel] = useState<ApiSkinModel>("classic");
  const [uploadName, setUploadName] = useState("");

  const isOffline = acc?.kind === "offline";

  const savedSkins = useMemo(() => buildSavedSkinsList(allSkins), [allSkins]);

  const displayCatalog = useMemo(
    () =>
      groupSkinsBySection(dedupeSkinsByName(allSkins.filter((s) => s.source === "default"))).map(
        (group) => ({
          id: group.title.toLowerCase().replace(/\s+/g, "-"),
          title: group.title,
          skins: group.skins.map(catalogSkinFromSkin),
        }),
      ),
    [allSkins],
  );

  const dirty = useMemo(() => {
    if (isOffline) return false;
    const baseSkin = committedSkin(msSkin);
    const baseCape = committedCapeId(mcProfile, availableCapes);
    const skinChanged = !draftSkinsEqual(draftSkin, baseSkin);
    const capeChanged = (draftCapeId ?? null) !== (baseCape ?? null);
    return skinChanged || capeChanged;
  }, [draftSkin, draftCapeId, isOffline, mcProfile, msSkin, availableCapes]);

  const offlinePreviewDirty = useMemo(() => {
    if (!isOffline || !previewingOfflineId) return false;
    const active = savedSkins.find((s) => s.isEquipped);
    return (active ? savedSkinId(active) : null) !== previewingOfflineId;
  }, [isOffline, previewingOfflineId, savedSkins]);

  const reloadCapes = useCallback(async () => {
    if (!acc || isOffline) {
      setAvailableCapes([]);
      return [];
    }
    setCapesLoading(true);
    try {
      const capes = await api.getAvailableCapes(acc.uuid);
      setAvailableCapes(capes);
      setCapesError(null);
      return capes;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setCapesError(msg);
      return [];
    } finally {
      setCapesLoading(false);
    }
  }, [acc, isOffline]);

  const reloadSkins = useCallback(async () => {
    if (!acc) return [];
    try {
      const skins = await api.getAvailableSkins(acc.uuid);
      setAllSkins(skins);
      return skins;
    } catch {
      setAllSkins([]);
      return [];
    }
  }, [acc]);

  const syncDraftFromCommitted = useCallback(() => {
    const equipped = allSkins.find((s) => s.isEquipped);
    if (!isOffline && equipped) {
      setDraftSkin(draftFromSkin(equipped));
    } else {
      setDraftSkin(committedSkin(msSkin));
    }
    setDraftCapeId(committedCapeId(mcProfile, availableCapes));
    setPendingPng(null);
    const active = savedSkins.find((s) => s.isEquipped);
    if (active?.libraryId) setPreviewingPremiumId(active.libraryId);
  }, [allSkins, availableCapes, isOffline, mcProfile, msSkin, savedSkins]);

  const reload = useCallback(
    async (refresh = false) => {
      if (!acc) return;
      setLoading(true);
      try {
        const skins = await reloadSkins();
        if (isOffline) {
          const os = await api.getOfflineSkin(acc.uuid);
          setMcProfile(null);
          setModel(toUiSkinModel(os.model));
          if (os.hasCustom && os.pngBase64) {
            setOfflinePreviewUrl(pngDataUrlFromBase64(os.pngBase64));
          } else {
            setOfflinePreviewUrl(null);
          }
          const active = skins.find((s) => s.isEquipped);
          if (active?.libraryId) {
            setPreviewingOfflineId(active.libraryId);
            setModel(active.variant === "SLIM" ? "slim" : "wide");
            if (active.texture.startsWith("data:")) {
              setOfflinePreviewUrl(active.texture);
            }
          }
        } else {
          const skin = await api.getAccountSkin(acc.uuid, refresh);
          let profile: McPlayerProfile | null = null;
          try {
            profile = await api.getMinecraftProfile(acc.uuid, refresh);
            setProfileError(null);
          } catch (e) {
            const msg = e instanceof Error ? e.message : String(e);
            setProfileError(msg);
            if (refresh) showError(msg);
          }
          setMsSkin(skin);
          setMcProfile(profile);
          setModel(toUiSkinModel(skin.model));
          const equipped = skins.find((s) => s.isEquipped);
          if (equipped) {
            setDraftSkin(draftFromSkin(equipped));
            if (equipped.libraryId) setPreviewingPremiumId(equipped.libraryId);
          } else {
            setDraftSkin(committedSkin(skin));
            setDraftCapeId(committedCapeId(profile, []));
          }
        }
      } catch (e) {
        showError(e instanceof Error ? e.message : String(e));
      } finally {
        setLoading(false);
        setInitialLoad(false);
      }
    },
    [acc, isOffline, reloadSkins, showError],
  );

  useEffect(() => {
    setInitialLoad(true);
  }, [acc?.uuid, isOffline]);

  useEffect(() => {
    void reload(false);
    void reloadCapes();
  }, [acc?.uuid, isOffline, skinEpoch, reload, reloadCapes]);

  useEffect(() => {
    if (!editOpen || !acc || isOffline || mcProfile) return;
    setProfileLoading(true);
    void api
      .getMinecraftProfile(acc.uuid, false)
      .then((profile) => {
        setMcProfile(profile);
        setProfileError(null);
      })
      .catch((e) => {
        const msg = e instanceof Error ? e.message : String(e);
        setProfileError(msg);
      })
      .finally(() => setProfileLoading(false));
  }, [editOpen, acc?.uuid, isOffline, mcProfile]);

  const previewingSavedSkin = useMemo(() => {
    const id = isOffline ? previewingOfflineId : previewingPremiumId;
    if (!id) return undefined;
    return savedSkins.find((s) => savedSkinId(s) === id);
  }, [isOffline, previewingOfflineId, previewingPremiumId, savedSkins]);

  const activeSavedSkin = useMemo(
    () => savedSkins.find((s) => s.isEquipped),
    [savedSkins],
  );

  const viewerSkinPng = useMemo(() => {
    if (!isOffline && draftSkin?.pngDataUrl) return draftSkin.pngDataUrl;
    if (!isOffline && !dirty) {
      const fromAccount = pngDataUrlFromAccountSkin(msSkin);
      if (fromAccount) return fromAccount;
      const libPng =
        (activeSavedSkin && pngDataUrlFromSkin(activeSavedSkin)) ||
        (previewingSavedSkin && pngDataUrlFromSkin(previewingSavedSkin)) ||
        null;
      if (libPng) return libPng;
    }
    if (isOffline && previewingSavedSkin) {
      const png = pngDataUrlFromSkin(previewingSavedSkin);
      if (png) return png;
    }
    if (isOffline && offlinePreviewUrl) return offlinePreviewUrl;
    return null;
  }, [
    activeSavedSkin,
    dirty,
    draftSkin?.pngDataUrl,
    isOffline,
    msSkin,
    offlinePreviewUrl,
    previewingSavedSkin,
  ]);

  const draftSkinTextureKey = useMemo(() => {
    if (viewerSkinPng) return null;
    if (isOffline) return null;
    const skin = !dirty ? (committedSkin(msSkin) ?? draftSkin) : (draftSkin ?? committedSkin(msSkin));
    if (skin?.textureKey) return skin.textureKey;
    if (!dirty) return textureKeyFromUrl(msSkin?.textureUrl) ?? null;
    return null;
  }, [dirty, draftSkin, isOffline, msSkin, viewerSkinPng]);

  const draftSkinUrl = useMemo(() => {
    if (viewerSkinPng || draftSkinTextureKey) return null;
    if (isOffline && acc) {
      return `https://mc-heads.net/skin/${encodeURIComponent(acc.name)}`;
    }
    if (!isOffline && msSkin?.textureUrl) {
      return normalizeTextureUrl(msSkin.textureUrl);
    }
    return null;
  }, [acc, draftSkinTextureKey, isOffline, msSkin?.textureUrl, viewerSkinPng]);

  const viewerModel = useMemo(() => {
    if (isOffline) {
      const variant = previewingSavedSkin?.variant ?? activeSavedSkin?.variant;
      if (variant) return skinToUiModel(variant);
      return model === "slim" ? "slim" : "classic";
    }
    if (!dirty && msSkin) {
      return msSkin.model === "slim" ? "slim" : "classic";
    }
    const variant = (draftSkin ?? committedSkin(msSkin))?.variant;
    return variant === "slim" ? "slim" : "classic";
  }, [activeSavedSkin?.variant, dirty, draftSkin, isOffline, model, msSkin, previewingSavedSkin?.variant]);

  const draftCapeUrl = useMemo(() => {
    if (!draftCapeId) return null;
    const fromProfile = mcProfile?.capes.find((c) => c.id === draftCapeId);
    if (fromProfile) return normalizeTextureUrl(fromProfile.url);
    const fromList = availableCapes.find((c) => c.id === draftCapeId);
    if (fromList) return normalizeTextureUrl(fromList.texture);
    return null;
  }, [availableCapes, draftCapeId, mcProfile]);

  async function onPremiumUpload(file: File) {
    if (!acc || isOffline) return;
    const buf = new Uint8Array(await file.arrayBuffer());
    const url = URL.createObjectURL(new Blob([buf], { type: "image/png" }));
    setPendingPng(buf);
    setPreviewingPremiumId(null);
    setDraftSkin({
      name: file.name.replace(/\.png$/i, "") || "Własny",
      variant: model === "slim" ? "slim" : "classic",
      pngDataUrl: url,
    });
  }

  function queueUpload(buf: Uint8Array, forPremium: boolean) {
    const previewUrl = URL.createObjectURL(new Blob([buf], { type: "image/png" }));
    setUploadModel(forPremium ? toApiSkinModel(model) : toApiSkinModel(model));
    setUploadName("");
    setUploadPending({ buf, previewUrl, forPremium });
  }

  function cancelUpload() {
    if (uploadPending?.previewUrl) URL.revokeObjectURL(uploadPending.previewUrl);
    setUploadPending(null);
    setUploadName("");
  }

  async function confirmUpload() {
    if (!acc || !uploadPending) return;
    setLoading(true);
    try {
      const variant = uiModelToSkinVariant(uploadModel);
      const skin: Skin = {
        textureKey: `local-pending-${Date.now()}`,
        name: uploadName.trim() || "Własny skin",
        variant,
        texture: uploadPending.previewUrl,
        source: "custom",
        isEquipped: false,
      };
      await api.saveCustomSkin(acc.uuid, skin, uploadModel, {
        png: [...uploadPending.buf],
        capeId: uploadPending.forPremium ? draftCapeId : null,
        replaceTexture: true,
      });
      await reloadSkins();
      if (uploadPending.forPremium) {
        showOk("Zapisano skin w bibliotece.");
      } else {
        await reload(false);
        showOk("Dodano skin do biblioteki.");
      }
      cancelUpload();
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  async function onUpload(file: File) {
    if (!acc) return;
    const buf = new Uint8Array(await file.arrayBuffer());
    queueUpload(buf, false);
  }

  function previewSavedSkin(skin: Skin, fromSave = false) {
    const id = savedSkinId(skin);
    if (isOffline) {
      setPreviewingOfflineId(id);
      setModel(skinToUiModel(skin.variant) === "slim" ? "slim" : "wide");
      const png = pngDataUrlFromSkin(skin);
      if (png) setOfflinePreviewUrl(png);
      return;
    }
    setPreviewingPremiumId(id);
    setDraftSkin(draftFromSkin(skin));
    setPendingPng(null);
    if (skin.capeId) setDraftCapeId(skin.capeId);
    if (fromSave && skin.texture.startsWith("data:image/png;base64,")) {
      const b64 = skin.texture.replace(/^data:image\/png;base64,/, "");
      try {
        setPendingPng(Uint8Array.from(atob(b64), (c) => c.charCodeAt(0)));
      } catch {
        /* preview only */
      }
    }
  }

  function isSavedSkinSelected(skin: Skin): boolean {
    const id = savedSkinId(skin);
    if (isOffline) {
      if (offlinePreviewDirty) return previewingOfflineId === id;
      return skin.isEquipped;
    }
    if (dirty) {
      const draftId = draftSkin?.libraryId ?? draftSkin?.textureKey;
      if (draftId) return draftId === id;
      return draftSkin?.textureKey === skin.textureKey;
    }
    return skin.isEquipped;
  }

  function isSavedSkinPreviewing(skin: Skin): boolean {
    const id = savedSkinId(skin);
    if (isOffline) return offlinePreviewDirty && previewingOfflineId === id;
    return dirty && previewingPremiumId === id;
  }

  function canDeleteSavedSkin(skin: Skin): boolean {
    return skin.source === "custom" && Boolean(skin.libraryId);
  }

  function previewPremiumLibraryEntry(entry: SkinLibraryEntry, fromSave = false) {
    const skin = savedSkins.find((s) => savedSkinId(s) === entry.id);
    if (skin) previewSavedSkin(skin, fromSave);
  }

  async function equipOfflinePreview() {
    if (!acc || !previewingOfflineId) return;
    const skin = savedSkins.find((s) => savedSkinId(s) === previewingOfflineId);
    if (!skin) return;
    setLoading(true);
    try {
      await api.equipSkin(acc.uuid, { ...skin, capeId: undefined });
      bump();
      clearAccountAvatarCache();
      await reload(false);
      showOk("Założono skin.");
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  async function deleteSavedSkin(skin: Skin) {
    if (!acc || !canDeleteSavedSkin(skin)) return;
    const id = savedSkinId(skin);
    if (
      !(await confirmDialog(`Usunąć „${skin.name ?? "skin"}" z biblioteki?`, {
        title: "Usuń skin",
        confirmLabel: "Usuń",
        danger: true,
      }))
    )
      return;
    setLoading(true);
    try {
      await api.removeCustomSkin(acc.uuid, skin);
      if (isOffline) {
        if (previewingOfflineId === id) setPreviewingOfflineId(null);
        await reload(false);
      } else {
        if (previewingPremiumId === id) syncDraftFromCommitted();
        await reloadSkins();
      }
      showOk("Usunięto skin z biblioteki.");
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
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
    setPreviewingPremiumId(null);
    setDraftSkin({
      textureKey: item.textureKey,
      variant: item.variant,
      name: item.name,
    });
    setPendingPng(null);
  }

  async function saveDraftToLibrary() {
    if (!acc || isOffline || !draftSkin) return;
    setLoading(true);
    try {
      const skin: Skin = {
        textureKey: draftSkin.textureKey ?? `local-${Date.now()}`,
        name: draftSkin.name || "Profil",
        variant: uiModelToSkinVariant(draftSkin.variant),
        capeId: draftCapeId ?? undefined,
        texture: draftSkin.pngDataUrl ?? `https://textures.minecraft.net/texture/${draftSkin.textureKey}`,
        source: "custom",
        isEquipped: false,
        libraryId: draftSkin.libraryId,
      };
      const entry = await api.saveCustomSkin(acc.uuid, skin, draftSkin.variant, {
        capeId: draftCapeId,
        png: pendingPng ? [...pendingPng] : null,
        replaceTexture: true,
      });
      await reloadSkins();
      previewPremiumLibraryEntry(entry, true);
      showOk("Zapisano profil w bibliotece.");
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  async function commitChanges() {
    if (!acc || isOffline || !draftSkin) return;
    const baseSkin = committedSkin(msSkin);
    const baseCape = committedCapeId(mcProfile, availableCapes);
    const skinChanged = !draftSkinsEqual(draftSkin, baseSkin) || Boolean(pendingPng);
    const capeChanged = (draftCapeId ?? null) !== (baseCape ?? null);
    if (!skinChanged && !capeChanged) return;

    setLoading(true);
    try {
      if (!skinChanged && capeChanged) {
        const profile = await api.setMinecraftCape(acc.uuid, draftCapeId);
        if (profile) setMcProfile(profile);
        await reloadCapes();
        bump();
        clearAccountAvatarCache();
        setEditOpen(false);
        showOk("Zastosowano pelerynę na koncie Mojang.");
        return;
      }

      const skin: Skin = {
        textureKey: draftSkin.textureKey ?? `local-${Date.now()}`,
        name: draftSkin.name,
        variant: uiModelToSkinVariant(draftSkin.variant),
        capeId: draftCapeId ?? undefined,
        texture:
          draftSkin.pngDataUrl ??
          (draftSkin.textureKey
            ? `https://textures.minecraft.net/texture/${draftSkin.textureKey}`
            : ""),
        source: draftSkin.libraryId ? "custom" : draftSkin.textureKey ? "default" : "custom_external",
        isEquipped: true,
        libraryId: draftSkin.libraryId,
      };
      const profile = await api.equipSkin(
        acc.uuid,
        skin,
        pendingPng ? [...pendingPng] : null,
      );
      if (profile) setMcProfile(profile);
      setPendingPng(null);
      bump();
      clearAccountAvatarCache();
      await reload(false);
      await reloadCapes();
      setEditOpen(false);
      showOk(
        capeChanged
          ? "Zastosowano skin i pelerynę na koncie Mojang."
          : "Zastosowano skin na koncie Mojang.",
      );
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

  if (initialLoad) {
    return (
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <div className="min-h-0 flex-1 overflow-y-auto p-4">
          <LockerLoadingScreen accountName={acc.name} />
        </div>
      </div>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <div className="min-h-0 flex-1 overflow-y-auto p-4">
        <div className="locker-layout mx-auto max-w-[1400px]">
          <div className="sticky top-4 self-start">
            <h1 className="m-0 text-2xl font-bold">Wybór skina</h1>
            <p className="mt-1 text-sm text-mute">
              {isOffline ? pl.locker.offlineHint : pl.locker.premiumHint}
            </p>
            <div className="mt-4">
              <SkinPreviewPanel
                nametag={acc.name}
                skinPngDataUrl={viewerSkinPng}
                skinUrl={draftSkinUrl}
                skinTextureKey={draftSkinTextureKey}
                capeUrl={isOffline ? null : draftCapeUrl}
                model={viewerModel}
                previewing={!isOffline ? dirty : offlinePreviewDirty}
                dirty={!isOffline ? dirty : offlinePreviewDirty}
                loading={loading}
                onApply={() => {
                  if (isOffline) void equipOfflinePreview();
                  else void commitChanges();
                }}
                onReset={() => {
                  if (isOffline) {
                    const active = savedSkins.find((s) => s.isEquipped);
                    if (active) previewSavedSkin(active);
                    else void reload(false);
                  } else {
                    cancelDraft();
                  }
                }}
                onSaveToLibrary={!isOffline && dirty ? () => void saveDraftToLibrary() : undefined}
                onEdit={() => setEditOpen(true)}
                editDisabled={loading}
                showCapes={!isOffline}
                availableCapes={availableCapes}
                draftCapeId={draftCapeId}
                onDraftCapeChange={setDraftCapeId}
                capesLoading={capesLoading}
                capesError={capesError}
              />
            </div>
          </div>

          <div className="min-w-0 pt-2">
            <section className="mb-8">
              <h2 className="mb-3 flex items-center gap-2 text-base font-semibold text-ink">
                {pl.locker.savedSkins}
              </h2>
              <LockerSkinGrid>
                {(cardHeight) => (
                  <>
                    <div style={{ height: cardHeight }}>
                      <SkinAddCard
                        disabled={loading}
                        onFile={(f) => {
                          void (async () => {
                            const buf = new Uint8Array(await f.arrayBuffer());
                            if (isOffline) queueUpload(buf, false);
                            else queueUpload(buf, true);
                          })();
                        }}
                      />
                    </div>
                    {savedSkins.map((skin) => {
                      const props = skinTextureProps(skin);
                      return (
                        <div
                          key={skinIdentity(skin)}
                          className="group relative"
                          style={{ height: cardHeight }}
                        >
                          <SkinGridButton
                            selected={isSavedSkinSelected(skin)}
                            active={skin.isEquipped}
                            previewing={isSavedSkinPreviewing(skin)}
                            disabled={loading}
                            alt={props.alt}
                            variant={props.variant}
                            skinPngDataUrl={props.skinPngDataUrl}
                            textureKey={props.textureKey}
                            onClick={() => previewSavedSkin(skin)}
                          />
                          {canDeleteSavedSkin(skin) && (
                            <button
                              type="button"
                              disabled={loading}
                              title="Usuń z biblioteki"
                              onClick={() => void deleteSavedSkin(skin)}
                              className="absolute right-2 top-2 z-30 grid h-7 w-7 place-items-center rounded-lg bg-black/70 text-mute opacity-0 transition hover:bg-bad/80 hover:text-white group-hover:opacity-100 disabled:opacity-40"
                            >
                              <Trash2 size={13} />
                            </button>
                          )}
                        </div>
                      );
                    })}
                  </>
                )}
              </LockerSkinGrid>
            </section>

            {displayCatalog.map((group) => (
              <section key={group.id} className="mb-8">
                <button
                  type="button"
                  className="mb-3 flex w-full items-center gap-2 text-left text-base font-semibold text-ink"
                  onClick={() => toggleGroup(group.id)}
                >
                  <ChevronDown
                    size={18}
                    className={clsx("text-mute transition", collapsed[group.id] && "-rotate-90")}
                  />
                  {group.title}
                </button>
                {!collapsed[group.id] && (
                  <LockerSkinGrid>
                    {(cardHeight) =>
                      group.skins.map((item) => {
                        const selected =
                          !isOffline &&
                          !draftSkin?.pngDataUrl &&
                          draftSkin?.textureKey === item.textureKey &&
                          draftSkin.variant === item.variant;
                        return (
                          <div
                            key={`${item.textureKey}-${item.variant}`}
                            style={{ height: cardHeight }}
                          >
                            <SkinGridButton
                              selected={selected}
                              active={selected}
                              disabled={loading}
                              alt={item.name}
                              variant={item.variant === "slim" ? "slim" : "classic"}
                              textureKey={item.textureKey}
                              onClick={() => onCatalogPick(item)}
                            />
                          </div>
                        );
                      })
                    }
                  </LockerSkinGrid>
                )}
              </section>
            ))}
          </div>
        </div>
      </div>

      <SkinUploadDialog
        open={uploadPending !== null}
        previewUrl={uploadPending?.previewUrl ?? null}
        model={uploadModel}
        name={uploadName}
        onNameChange={setUploadName}
        onModelChange={setUploadModel}
        onConfirm={() => void confirmUpload()}
        onCancel={cancelUpload}
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
        profileError={profileError ?? capesError}
        profileLoading={profileLoading || capesLoading}
        availableCapes={availableCapes}
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
          await reloadCapes();
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
