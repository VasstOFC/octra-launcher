import { useEffect, useRef, useState, type ReactNode } from "react";
import { useModrinthSkinGrid } from "../hooks/useModrinthSkinGrid";

export function LockerSkinGrid({
  children,
  className,
}: {
  children: (cardHeight: number) => ReactNode;
  className?: string;
}) {
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

  return (
    <div
      ref={ref}
      className={className}
      style={{
        display: "grid",
        gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
        gap: `${gap}px`,
      }}
    >
      {children(cardHeight)}
    </div>
  );
}
