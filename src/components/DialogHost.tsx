import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { AnimatePresence, motion } from "framer-motion";
import { useDialog } from "../lib/dialog";

export function DialogHost() {
  const current = useDialog((s) => s.current);
  const settle = useDialog((s) => s.settle);
  const [value, setValue] = useState("");
  const inputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    setValue(current?.defaultValue ?? "");
    if (current?.kind === "prompt") {
      const t = window.setTimeout(() => inputRef.current?.select(), 30);
      return () => window.clearTimeout(t);
    }
  }, [current?.id, current?.defaultValue, current?.kind]);

  useEffect(() => {
    if (!current) return;
    const spec = current;
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") {
        e.preventDefault();
        settle(spec.kind === "confirm" ? false : spec.kind === "prompt" ? null : true);
      }
      if (e.key === "Enter" && spec.kind !== "prompt") {
        e.preventDefault();
        settle(true);
      }
    }
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [current, settle]);

  const confirm = () => {
    if (!current) return;
    if (current.kind === "prompt") settle(value);
    else settle(true);
  };
  const cancel = () => {
    if (!current) return;
    settle(current.kind === "prompt" ? null : current.kind === "confirm" ? false : true);
  };

  return createPortal(
    <AnimatePresence>
      {current && (
        <motion.div
          key={current.id}
          className="fixed inset-0 z-[400] grid place-items-center bg-black/70 p-6"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onMouseDown={(e) => {
            if (e.target === e.currentTarget) cancel();
          }}
        >
          <motion.div
            className="w-full max-w-md rounded-3xl border border-line bg-raised p-6 shadow-2xl"
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 6 }}
          >
            <h2 className="text-lg font-semibold">{current.title}</h2>
            <p className="mt-3 select-text whitespace-pre-wrap text-sm leading-relaxed text-mute">
              {current.message}
            </p>
            {current.kind === "prompt" && (
              <input
                ref={inputRef}
                className="mt-4 w-full rounded-xl border border-line bg-bg px-3 py-2 text-sm outline-none focus:border-accent"
                value={value}
                onChange={(e) => setValue(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    confirm();
                  }
                }}
                autoFocus
              />
            )}
            <div className="mt-5 flex justify-end gap-2">
              {current.kind !== "alert" && (
                <button
                  className="rounded-xl px-4 py-2 text-sm font-medium text-mute hover:text-ink"
                  onClick={cancel}
                >
                  {current.cancelLabel}
                </button>
              )}
              <button
                className={`rounded-xl px-4 py-2 text-sm font-semibold ${
                  current.danger ? "bg-danger text-white" : "bg-accent text-white"
                }`}
                onClick={confirm}
              >
                {current.confirmLabel}
              </button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>,
    document.body,
  );
}
