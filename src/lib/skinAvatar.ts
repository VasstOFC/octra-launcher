/** Wycina głowę (8×8) z PNG skina Minecraft 64×64. */
export async function headFromSkinPngBase64(pngBase64: string): Promise<string> {
  const src = pngBase64.startsWith("data:")
    ? pngBase64
    : `data:image/png;base64,${pngBase64}`;
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => {
      const canvas = document.createElement("canvas");
      const size = 48;
      canvas.width = size;
      canvas.height = size;
      const ctx = canvas.getContext("2d");
      if (!ctx) {
        reject(new Error("canvas"));
        return;
      }
      const unit = img.width / 64;
      ctx.imageSmoothingEnabled = false;
      ctx.drawImage(img, 8 * unit, 8 * unit, 8 * unit, 8 * unit, 0, 0, size, size);
      ctx.drawImage(img, 40 * unit, 8 * unit, 8 * unit, 8 * unit, 0, 0, size, size);
      resolve(canvas.toDataURL("image/png"));
    };
    img.onerror = () => reject(new Error("skin"));
    img.src = src;
  });
}

const DEFAULT_STEVE = "https://mc-heads.net/avatar/Steve/48";
const DEFAULT_ALEX = "https://mc-heads.net/avatar/Alex/48";

export function premiumAvatarUrl(uuid: string): string {
  const plain = uuid.replace(/-/g, "");
  return `https://mc-heads.net/avatar/${plain}/48`;
}

export function defaultAvatarUrl(model: "slim" | "classic" | string): string {
  return model === "slim" ? DEFAULT_ALEX : DEFAULT_STEVE;
}
