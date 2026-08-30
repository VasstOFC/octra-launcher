import { useEffect, useState } from "react";

import { api } from "./api";

import { assetUrl, bustAssetUrl } from "./assetUrl";

import { useProfileVisualEpoch } from "./profileVisual";

import type { Instance } from "../types";



export function useProfileWallpaper(inst: Instance) {

  const [wallpaper, setWallpaper] = useState<string | null>(null);

  const epoch = useProfileVisualEpoch(inst.id);



  useEffect(() => {

    let cancelled = false;

    void api.readInstanceWallpaper(inst.id).then((path) => {

      if (!cancelled) setWallpaper(bustAssetUrl(assetUrl(path), epoch));

    });

    return () => {

      cancelled = true;

    };

  }, [inst.id, inst.wallpaperPath, epoch]);



  return wallpaper;

}
