import { clsx } from "clsx";
import type { CSSProperties, InputHTMLAttributes } from "react";

type RangeSliderProps = Omit<
  InputHTMLAttributes<HTMLInputElement>,
  "type" | "onChange"
> & {
  onChange: (value: number) => void;
};

export function RangeSlider({
  className,
  onChange,
  disabled,
  value,
  min = 0,
  max = 100,
  ...props
}: RangeSliderProps) {
  const minN = Number(min);
  const maxN = Number(max);
  const valueN = Number(value ?? minN);
  const pct =
    maxN > minN ? Math.min(100, Math.max(0, ((valueN - minN) / (maxN - minN)) * 100)) : 0;

  return (
    <input
      type="range"
      className={clsx("octra-range w-full", className)}
      style={{ "--range-pct": `${pct}%` } as CSSProperties}
      value={value}
      min={min}
      max={max}
      disabled={disabled}
      onChange={(e) => onChange(Number(e.target.value))}
      {...props}
    />
  );
}
