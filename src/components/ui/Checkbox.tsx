import { clsx } from "clsx";
import { Check } from "lucide-react";

type CheckboxProps = {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  className?: string;
  id?: string;
};

export function Checkbox({
  checked,
  onChange,
  disabled,
  className,
  id,
}: CheckboxProps) {
  return (
    <button
      id={id}
      type="button"
      role="checkbox"
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={clsx(
        "octra-checkbox flex size-[18px] shrink-0 items-center justify-center rounded-[5px] transition",
        checked
          ? "bg-accent text-bg-on-accent shadow-[inset_0_1px_0_rgb(255_255_255/0.12)] ring-1 ring-accent/60"
          : "bg-raised2 ring-1 ring-line hover:ring-white/14",
        disabled && "cursor-not-allowed opacity-50",
        className,
      )}
    >
      {checked ? <Check className="size-3" strokeWidth={3} /> : null}
    </button>
  );
}

type ToggleFieldProps = {
  label: string;
  hint?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
};

export function ToggleField({
  label,
  hint,
  checked,
  onChange,
  disabled,
}: ToggleFieldProps) {
  return (
    <label
      className={clsx(
        "flex cursor-pointer items-start gap-3",
        disabled && "cursor-not-allowed opacity-50",
      )}
    >
      <Checkbox
        checked={checked}
        onChange={onChange}
        disabled={disabled}
        className="mt-0.5"
      />
      <span className="min-w-0 flex-1">
        <span className="block text-[13px] text-ink">{label}</span>
        {hint ? <span className="mt-0.5 block text-xs text-mute">{hint}</span> : null}
      </span>
    </label>
  );
}
