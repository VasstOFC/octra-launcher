import { useEffect, useState } from "react";

import { Link2Off, Loader2 } from "lucide-react";

import { api } from "../lib/api";

import { assetUrl } from "../lib/assetUrl";

import { confirmDialog } from "../lib/dialog";

import {

  paletteById,

  paletteIdFromInstance,

  PROFILE_PALETTES,

} from "../lib/profilePalettes";

import {

  galleryIconIdFromInstance,

  galleryIconSymbol,

} from "../lib/profileIconResolve";

import { useApplyProfileVisualUpdate } from "../lib/profileVisual";

import { useApp } from "../stores/appStore";

import { pl } from "../locales/pl";

import {

  ProfileAppearanceEditor,

  type ProfileIconDraft,

} from "./ProfileAppearanceEditor";



export function ProfileAppearancePane({ id }: { id: string }) {

  const instances = useApp((s) => s.instances);

  const applyVisual = useApplyProfileVisualUpdate();

  const showError = useApp((s) => s.showError);

  const showOk = useApp((s) => s.showOk);

  const inst = instances.find((i) => i.id === id);



  const [paletteId, setPaletteId] = useState(PROFILE_PALETTES[3]!.id);

  const [icon, setIcon] = useState<ProfileIconDraft>({ kind: "default" });

  const [wallpaperPreviewUrl, setWallpaperPreviewUrl] = useState<string | null>(null);

  const [wallpaperBusy, setWallpaperBusy] = useState(false);

  const [saving, setSaving] = useState(false);

  const [unlinking, setUnlinking] = useState(false);



  useEffect(() => {

    if (!inst) return;

    setPaletteId(paletteIdFromInstance(inst));

    const gid = galleryIconIdFromInstance(inst);

    if (inst.iconPath?.trim()) {

      void api.readInstanceIcon(inst.id).then((path) => {

        const url = assetUrl(path);

        if (url) setIcon({ kind: "file", previewUrl: url, bytes: new Uint8Array() });

        else if (gid) setIcon({ kind: "preset", id: gid });

        else setIcon({ kind: "default" });

      });

    } else if (gid) {

      setIcon({ kind: "preset", id: gid });

    } else {

      setIcon({ kind: "default" });

    }

    void api.readInstanceWallpaper(inst.id).then((path) => {

      setWallpaperPreviewUrl(assetUrl(path));

    });

  }, [inst?.id, inst?.iconPath, inst?.iconSymbol, inst?.wallpaperPath, inst?.ledColor, inst?.ledColor2]);



  if (!inst) return null;



  const locked = Boolean(inst.packLocked);



  async function savePalette(nextPaletteId: string) {

    const palette = paletteById(nextPaletteId);

    setSaving(true);

    try {

      const updated = await api.updateInstance({

        ...inst!,

        ledColor: palette.c1,

        ledColor2: palette.c2,

      });

      applyVisual(updated);

    } catch (e) {

      showError(e instanceof Error ? e.message : String(e));

    } finally {

      setSaving(false);

    }

  }



  async function onPaletteIdChange(nextPaletteId: string) {

    setPaletteId(nextPaletteId);

    await savePalette(nextPaletteId);

  }



  async function onIconChange(next: ProfileIconDraft) {

    setIcon(next);

    if (locked) return;

    setSaving(true);

    try {

      let updated;

      if (next.kind === "default") {

        updated = await api.updateInstance({

          ...inst!,

          iconPath: "",

          iconSymbol: "",

          iconColor: "",

        });

      } else if (next.kind === "preset") {

        updated = await api.updateInstance({

          ...inst!,

          iconPath: "",

          iconSymbol: galleryIconSymbol(next.id),

          iconColor: "",

        });

      } else if (next.bytes.length > 0) {

        updated = await api.setInstanceIconBytes(inst!.id, [...next.bytes]);

      } else {

        return;

      }

      applyVisual(updated);

      showOk("Ikona profilu została zaktualizowana.");

    } catch (e) {

      showError(e instanceof Error ? e.message : String(e));

    } finally {

      setSaving(false);

    }

  }



  async function unlinkPack() {

    const ok = await confirmDialog(pl.versions.unlinkPackConfirm, {

      title: pl.versions.unlinkPack,

      confirmLabel: pl.versions.unlinkPack,

    });

    if (!ok) return;

    setUnlinking(true);

    try {

      const updated = await api.unlinkInstancePack(inst!.id);

      applyVisual(updated);

      showOk(pl.versions.unlinkPackDone);

    } catch (e) {

      showError(e instanceof Error ? e.message : String(e));

    } finally {

      setUnlinking(false);

    }

  }



  async function onPickWallpaper() {

    setWallpaperBusy(true);

    try {

      const updated = await api.pickProfileWallpaper(inst!.id);

      if (!updated) return;

      applyVisual(updated);

      const path = await api.readInstanceWallpaper(updated.id);

      setWallpaperPreviewUrl(assetUrl(path));

      showOk("Tapeta profilu została zaktualizowana.");

    } catch (e) {

      showError(e instanceof Error ? e.message : String(e));

    } finally {

      setWallpaperBusy(false);

    }

  }



  async function onClearWallpaper() {

    setWallpaperBusy(true);

    try {

      const updated = await api.clearProfileWallpaper(inst!.id);

      applyVisual(updated);

      setWallpaperPreviewUrl(null);

      showOk("Tapeta została usunięta.");

    } catch (e) {

      showError(e instanceof Error ? e.message : String(e));

    } finally {

      setWallpaperBusy(false);

    }

  }



  return (

    <div className="h-full overflow-auto p-6">

      <h2 className="text-lg font-semibold">Wygląd profilu</h2>

      <p className="mt-1 text-sm text-mute">

        Kolor tła, ikona i tapeta widoczne na ekranie startowym.

      </p>

      {locked ? (

        <div className="mt-3 flex flex-wrap items-center justify-between gap-3 rounded-xl border border-warn/30 bg-warn/10 px-3 py-2.5">

          <div className="min-w-0 text-xs text-warn">

            <p>{pl.versions.packLockedHint}</p>

            {inst.linkedPack ? (

              <p className="mt-1 text-[11px] text-mute">

                {pl.versions.linkedPack}: <span className="text-ink">{inst.linkedPack}</span>

              </p>

            ) : null}

          </div>

          <button

            type="button"

            disabled={unlinking}

            onClick={() => void unlinkPack()}

            className="inline-flex shrink-0 items-center gap-1.5 rounded-lg bg-warn/20 px-3 py-1.5 text-xs font-semibold text-warn ring-1 ring-warn/35 hover:bg-warn/30 disabled:opacity-50"

          >

            {unlinking ? <Loader2 size={14} className="animate-spin" /> : <Link2Off size={14} />}

            {pl.versions.unlinkPack}

          </button>

        </div>

      ) : null}

      <div className="mt-5 max-w-xl">

        <ProfileAppearanceEditor

          name={inst.name}

          loader={inst.loader}

          gameVersion={inst.gameVersion}

          paletteId={paletteId}

          onPaletteIdChange={(next) => void onPaletteIdChange(next)}

          icon={icon}

          onIconChange={(next) => void onIconChange(next)}

          wallpaperPreviewUrl={wallpaperPreviewUrl}

          onPickWallpaper={() => void onPickWallpaper()}

          onClearWallpaper={() => void onClearWallpaper()}

          wallpaperBusy={wallpaperBusy}

          disabled={saving}

          iconDisabled={locked}

        />

      </div>

    </div>

  );

}
