import type { ButtonHTMLAttributes, ReactNode } from "react";
import { clsx } from "clsx";

type Variant = "primary" | "secondary" | "ghost" | "launch" | "danger";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: Variant;
  children: ReactNode;
};

const VARIANT: Record<Variant, string> = {
  primary: "bg-accent/20 text-ink ring-1 ring-accent/40 hover:bg-accent/30",
  secondary: "bg-raised text-ink ring-1 ring-line hover:bg-raised2",
  ghost: "text-mute hover:bg-white/6 hover:text-ink",
  launch: "bg-launch text-white hover:brightness-110",
  danger: "bg-danger text-white hover:brightness-110",
};

export function Button({
  variant = "secondary",
  className,
  children,
  ...props
}: ButtonProps) {
  return (
    <button
      type="button"
      className={clsx(
        "inline-flex items-center justify-center rounded-lg px-3 py-2 text-sm font-semibold transition disabled:opacity-50",
        VARIANT[variant],
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
