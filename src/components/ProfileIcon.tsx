import { useEffect, useState } from "react";

import { clsx } from "clsx";

import { api } from "../lib/api";

import { assetUrl, bustAssetUrl } from "../lib/assetUrl";

import {

  DEFAULT_GALLERY_ICON_ID,

  galleryIconIdFromInstance,

  galleryIconUrl,

} from "../lib/profileIconResolve";

import { useProfileVisualEpoch } from "../lib/profileVisual";

import type { Instance } from "../types";



export function ProfileIcon({

  inst,

  size = 56,

  className,

}: {

  inst: Instance;

  size?: number;

  className?: string;

}) {

  const galleryId = galleryIconIdFromInstance(inst);

  const epoch = useProfileVisualEpoch(inst.id);

  const [fileUrl, setFileUrl] = useState<string | null>(null);

  const [fileFailed, setFileFailed] = useState(false);



  useEffect(() => {

    let cancelled = false;

    setFileFailed(false);

    if (!inst.iconPath?.trim()) {

      setFileUrl(null);

      return;

    }

    void api.readInstanceIcon(inst.id).then((path) => {

      if (!cancelled) setFileUrl(bustAssetUrl(assetUrl(path), epoch));

    });

    return () => {

      cancelled = true;

    };

  }, [inst.id, inst.iconPath, inst.iconSymbol, epoch]);



  const presetId = galleryId ?? (!inst.iconPath?.trim() || fileFailed ? DEFAULT_GALLERY_ICON_ID : null);



  if (!fileFailed && fileUrl && inst.iconPath?.trim() && !galleryId) {

    return (

      <img

        src={fileUrl}

        alt=""

        draggable={false}

        width={size}

        height={size}

        onError={() => setFileFailed(true)}

        className={clsx("rounded-lg object-cover [image-rendering:pixelated]", className)}

      />

    );

  }



  return (

    <img

      src={galleryIconUrl(presetId ?? DEFAULT_GALLERY_ICON_ID)}

      alt=""

      draggable={false}

      width={size}

      height={size}

      onError={(e) => {

        (e.target as HTMLImageElement).src = galleryIconUrl(DEFAULT_GALLERY_ICON_ID);

      }}

      className={clsx("rounded-lg [image-rendering:pixelated]", className)}

    />

  );

}
