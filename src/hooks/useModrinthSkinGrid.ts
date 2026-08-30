import { useEffect, useState } from "react";

export const SKIN_CARD_ASPECT = 31 / 40;
export const SKIN_GRID_GAP = 12;

export function columnCountForViewport(width: number): number {
  if (width >= 2050) return 6;
  if (width >= 1750) return 5;
  if (width >= 1300) return 4;
  return 3;
}

export function useModrinthSkinGrid(listWidth: number) {
  const [viewportWidth, setViewportWidth] = useState(
    typeof window !== "undefined" ? window.innerWidth : 1400,
  );

  useEffect(() => {
    const onResize = () => setViewportWidth(window.innerWidth);
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);

  const columns = columnCountForViewport(viewportWidth);
  const gap = SKIN_GRID_GAP;
  const cardWidth =
    listWidth > 0 ? Math.max(0, (listWidth - (columns - 1) * gap) / columns) : 0;
  const cardHeight = cardWidth > 0 ? cardWidth / SKIN_CARD_ASPECT : 0;

  return { columns, gap, cardWidth, cardHeight };
}
