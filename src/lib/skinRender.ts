function dataUrl(pngBase64: string): string {
  return pngBase64.startsWith("data:")
    ? pngBase64
    : `data:image/png;base64,${pngBase64}`;
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error("skin"));
    img.src = src;
  });
}

function drawPart(
  ctx: CanvasRenderingContext2D,
  img: HTMLImageElement,
  sx: number,
  sy: number,
  sw: number,
  sh: number,
  dx: number,
  dy: number,
  dw: number,
  dh: number,
  unit: number,
) {
  ctx.drawImage(img, sx * unit, sy * unit, sw * unit, sh * unit, dx, dy, dw, dh);
}

/** Podgląd postaci z przodu (jak w Modrinth). */
export async function bodyFromSkinPngBase64(
  pngBase64: string,
  model: "slim" | "classic" = "classic",
): Promise<string> {
  const img = await loadImage(dataUrl(pngBase64));
  const unit = img.width / 64;
  const scale = 10;
  const w = 16 * scale;
  const h = 32 * scale;
  const canvas = document.createElement("canvas");
  canvas.width = w;
  canvas.height = h;
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("canvas");
  ctx.imageSmoothingEnabled = false;

  const armW = model === "slim" ? 3 : 4;
  const armSx = model === "slim" ? 47 : 44;

  drawPart(ctx, img, 20, 20, 8, 12, 4 * scale, 8 * scale, 8 * scale, 12 * scale, unit);
  drawPart(ctx, img, 36, 52, armW, 12, 0, 8 * scale, armW * scale, 12 * scale, unit);
  drawPart(ctx, img, armSx, 20, armW, 12, 12 * scale, 8 * scale, armW * scale, 12 * scale, unit);
  drawPart(ctx, img, 4, 20, 4, 12, 4 * scale, 20 * scale, 4 * scale, 12 * scale, unit);
  drawPart(ctx, img, 20, 52, 4, 12, 8 * scale, 20 * scale, 4 * scale, 12 * scale, unit);
  drawPart(ctx, img, 8, 8, 8, 8, 4 * scale, 0, 8 * scale, 8 * scale, unit);
  drawPart(ctx, img, 40, 8, 8, 8, 4 * scale, 0, 8 * scale, 8 * scale, unit);

  return canvas.toDataURL("image/png");
}

/** Miniatura bust (głowa + tors) do siatki skinów. */
export async function bustFromSkinPngBase64(
  pngBase64: string,
  model: "slim" | "classic" = "classic",
): Promise<string> {
  const img = await loadImage(dataUrl(pngBase64));
  const unit = img.width / 64;
  const scale = 8;
  const w = 14 * scale;
  const h = 18 * scale;
  const canvas = document.createElement("canvas");
  canvas.width = w;
  canvas.height = h;
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("canvas");
  ctx.imageSmoothingEnabled = false;

  const armW = model === "slim" ? 3 : 4;
  const armSx = model === "slim" ? 47 : 44;

  drawPart(ctx, img, 20, 20, 8, 8, 3 * scale, 8 * scale, 8 * scale, 8 * scale, unit);
  drawPart(ctx, img, 36, 52, armW, 8, 0, 8 * scale, armW * scale, 8 * scale, unit);
  drawPart(ctx, img, armSx, 20, armW, 8, 11 * scale, 8 * scale, armW * scale, 8 * scale, unit);
  drawPart(ctx, img, 8, 8, 8, 8, 3 * scale, 0, 8 * scale, 8 * scale, unit);
  drawPart(ctx, img, 40, 8, 8, 8, 3 * scale, 0, 8 * scale, 8 * scale, unit);

  return canvas.toDataURL("image/png");
}

export function mcHeadsBodyUrl(uuidOrName: string, size = 128): string {
  const id = uuidOrName.replace(/-/g, "");
  return `https://mc-heads.net/body/${id}/${size}`;
}

export function mojangTextureUrl(textureKey: string): string {
  return `https://textures.minecraft.net/texture/${textureKey}`;
}

export function normalizeTextureUrl(url: string): string {
  return url.replace(/^http:\/\//i, "https://");
}
