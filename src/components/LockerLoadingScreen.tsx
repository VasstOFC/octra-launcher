import { Loader2 } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useModrinthSkinGrid } from "../hooks/useModrinthSkinGrid";
import { pl } from "../locales/pl";

function BlockySilhouette() {
  return (
    <div className="locker-loading-character" aria-hidden>
      <div className="locker-loading-head" />
      <div className="locker-loading-body" />
      <div className="locker-loading-arm locker-loading-arm--left" />
      <div className="locker-loading-arm locker-loading-arm--right" />
      <div className="locker-loading-legs">
        <div />
        <div />
      </div>
    </div>
  );
}

function SkeletonGrid() {
  const ref = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(0);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const ro = new ResizeObserver(() => setWidth(el.clientWidth));
    ro.observe(el);
    setWidth(el.clientWidth);
    return () => ro.disconnect();
  }, []);

  const { columns, gap, cardHeight } = useModrinthSkinGrid(width);
  const count = Math.max(columns * 2, 6);

  return (
    <div
      ref={ref}
      className="locker-loading-grid"
      style={{
        gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
        gap: `${gap}px`,
      }}
    >
      {Array.from({ length: count }, (_, i) => (
        <div
          key={i}
          className="locker-loading-card"
          style={{
            height: cardHeight,
            animationDelay: `${(i % columns) * 70 + Math.floor(i / columns) * 120}ms`,
          }}
        />
      ))}
    </div>
  );
}

export function LockerLoadingScreen({ accountName }: { accountName?: string }) {
  return (
    <div className="locker-loading-screen" role="status" aria-live="polite">
      <div className="locker-layout mx-auto max-w-[1400px]">
        <div className="locker-loading-preview">
          <div className="locker-loading-title-shimmer h-8 w-48 rounded-lg" />
          <div className="locker-loading-title-shimmer mt-2 h-4 w-64 max-w-full rounded-md" />

          <div className="relative mt-6 flex flex-col items-center">
            {accountName ? (
              <div className="locker-loading-nametag mb-4">{accountName}</div>
            ) : null}

            <div className="relative flex h-[min(62vh,440px)] w-full max-w-[min(100%,380px)] items-center justify-center">
              <div className="skin-preview-spotlight pointer-events-none absolute left-1/2 z-0 -translate-x-1/2" />
              <BlockySilhouette />
              <div className="absolute inset-x-0 bottom-8 z-10 flex flex-col items-center gap-2">
                <Loader2 className="size-7 animate-spin text-accent" />
                <p className="text-sm font-medium text-ink">{pl.locker.loadingTitle}</p>
                <p className="max-w-[220px] text-center text-xs text-mute">
                  {pl.locker.loadingHint}
                </p>
              </div>
            </div>
          </div>
        </div>

        <div className="min-w-0 pt-2">
          <div className="locker-loading-title-shimmer mb-3 h-5 w-36 rounded-md" />
          <SkeletonGrid />
          <div className="mt-8">
            <div className="locker-loading-title-shimmer mb-3 h-5 w-28 rounded-md" />
            <SkeletonGrid />
          </div>
        </div>
      </div>
    </div>
  );
}
