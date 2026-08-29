import type { Instance } from "../types";
import { instanceAccent, LOADER_LABEL } from "../lib/format";
import { Layers } from "lucide-react";

/** Domyślna grafika profilu gdy brak tapety. */
export function ProfileBackdrop({
  inst,
  className = "",
  size = "hero",
}: {
  inst: Instance;
  className?: string;
  size?: "hero" | "card";
}) {
  const [defaultC1, defaultC2] = instanceAccent(inst.id);
  const c1 = inst.ledColor?.trim() || defaultC1;
  const c2 = inst.ledColor2?.trim() || defaultC2;
  const glyph =
    inst.iconSymbol?.trim() ||
    inst.name.slice(0, 2).toUpperCase() ||
    "?";
  const loader = LOADER_LABEL[inst.loader] ?? inst.loader;

  return (
    <div
      className={`absolute inset-0 ${className}`}
      style={{
        background: `linear-gradient(135deg, ${c1} 0%, ${c2}55 45%, #0f0f12 100%)`,
      }}
    >
      <div className="absolute inset-0 opacity-[0.12] [background-image:radial-gradient(circle_at_20%_30%,white_0,transparent_45%),radial-gradient(circle_at_80%_70%,white_0,transparent_40%)]" />
      <div
        className={`absolute flex flex-col items-center justify-center text-white/90 ${
          size === "hero" ? "right-[18%] top-1/2 -translate-y-1/2" : "inset-0"
        }`}
      >
        <div
          className={`grid place-items-center rounded-2xl border border-white/15 bg-black/25 font-bold ${
            size === "hero" ? "h-20 w-20 text-2xl" : "h-10 w-10 text-sm"
          }`}
          style={{ color: inst.iconColor || "#fff" }}
        >
          {glyph.length <= 2 ? glyph : <Layers size={size === "hero" ? 28 : 16} />}
        </div>
        {size === "hero" && (
          <span className="mt-2 text-[10px] font-semibold uppercase tracking-widest text-white/50">
            {loader}
          </span>
        )}
      </div>
    </div>
  );
}
