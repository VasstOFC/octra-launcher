import { useEffect, useState } from "react";
import { clsx } from "clsx";
import { api } from "../lib/api";
import {
  defaultAvatarUrl,
  headFromSkinPngBase64,
  premiumAvatarUrl,
} from "../lib/skinAvatar";
import { useApp } from "../stores/appStore";
import type { Account } from "../types";

const cache = new Map<string, string>();

export function clearAccountAvatarCache() {
  cache.clear();
}

export function AccountAvatar({
  account,
  className,
  size = 32,
}: {
  account: Account;
  className?: string;
  size?: number;
}) {
  const skinEpoch = useApp((s) => s.skinEpoch);
  const [src, setSrc] = useState<string | null>(
    () => cache.get(cacheKey(account, skinEpoch)) ?? null,
  );

  useEffect(() => {
    let cancelled = false;
    const key = cacheKey(account, skinEpoch);
    const cached = cache.get(key);
    if (cached) {
      setSrc(cached);
      return;
    }

    async function load() {
      try {
        let url: string;
        if (account.kind === "offline") {
          const skin = await api.getOfflineSkin(account.uuid);
          if (skin.hasCustom && skin.pngBase64) {
            url = await headFromSkinPngBase64(skin.pngBase64);
          } else {
            url = defaultAvatarUrl(skin.model === "slim" ? "slim" : "classic");
          }
        } else {
          let uuid = account.uuid;
          url = premiumAvatarUrl(uuid);
          try {
            const ms = await api.getAccountSkin(account.uuid, false);
            if (ms.uuid) uuid = ms.uuid;
            if (ms.pngBase64) {
              url = await headFromSkinPngBase64(ms.pngBase64);
            } else {
              url = premiumAvatarUrl(uuid);
            }
          } catch {
            /* mc-heads fallback */
          }
        }
        if (!cancelled) {
          cache.set(key, url);
          setSrc(url);
        }
      } catch {
        if (!cancelled) {
          setSrc(defaultAvatarUrl("classic"));
        }
      }
    }

    void load();
    return () => {
      cancelled = true;
    };
  }, [account.uuid, account.kind, skinEpoch]);

  return (
    <span
      className={clsx(
        "grid shrink-0 overflow-hidden rounded-lg bg-black/30 ring-1 ring-white/10",
        className,
      )}
      style={{ width: size, height: size }}
    >
      {src ? (
        <img
          src={src}
          alt=""
          className="h-full w-full object-cover [image-rendering:pixelated]"
        />
      ) : (
        <span className="h-full w-full animate-pulse bg-white/10" />
      )}
    </span>
  );
}

function cacheKey(account: Account, skinEpoch: number) {
  return `${account.uuid}:${skinEpoch}`;
}
