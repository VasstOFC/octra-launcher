import { useRef, useState } from "react";
import { clsx } from "clsx";
import { MoveHorizontal } from "lucide-react";

export function SkinPreview({
  src,
  alt,
  className,
  large,
}: {
  src: string | null;
  alt: string;
  className?: string;
  large?: boolean;
}) {
  const [angle, setAngle] = useState(0);
  const drag = useRef<{ x: number; active: boolean }>({ x: 0, active: false });

  function onPointerDown(e: React.PointerEvent) {
    drag.current = { x: e.clientX, active: true };
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: React.PointerEvent) {
    if (!drag.current.active) return;
    const dx = e.clientX - drag.current.x;
    drag.current.x = e.clientX;
    setAngle((a) => a + dx * 0.6);
  }

  function onPointerUp() {
    drag.current.active = false;
  }

  return (
    <div className={clsx("flex flex-col items-center", className)}>
      <div
        className={clsx(
          "relative grid place-items-center select-none",
          large ? "h-[min(52vh,420px)] w-full" : "h-36 w-full",
        )}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerLeave={onPointerUp}
        style={{ touchAction: "none" }}
      >
        {src ? (
          <img
            src={src}
            alt={alt}
            draggable={false}
            className="max-h-full object-contain [image-rendering:pixelated]"
            style={{
              transform: `perspective(600px) rotateY(${angle}deg)`,
              transformStyle: "preserve-3d",
            }}
          />
        ) : (
          <div className="h-40 w-20 animate-pulse rounded-xl bg-white/8" />
        )}
      </div>
      {large && (
        <p className="mt-2 flex items-center gap-1 text-[10px] text-mute">
          <MoveHorizontal size={12} />
          Przeciągnij, żeby obrócić
        </p>
      )}
    </div>
  );
}
