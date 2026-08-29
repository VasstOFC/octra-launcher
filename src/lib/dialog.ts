import { create } from "zustand";

export type DialogKind = "alert" | "confirm" | "prompt";

export type DialogSpec = {
  id: number;
  kind: DialogKind;
  title: string;
  message: string;
  confirmLabel: string;
  cancelLabel: string;
  danger: boolean;
  defaultValue: string;
  resolve: (value: string | boolean | null) => void;
};

type DialogState = {
  current: DialogSpec | null;
  queue: DialogSpec[];
  enqueue: (spec: Omit<DialogSpec, "id">) => void;
  settle: (value: string | boolean | null) => void;
};

let nextId = 1;

export const useDialog = create<DialogState>((set, get) => ({
  current: null,
  queue: [],
  enqueue: (spec) => {
    const item: DialogSpec = { ...spec, id: nextId++ };
    set((s) => {
      if (!s.current) return { current: item };
      return { queue: [...s.queue, item] };
    });
  },
  settle: (value) => {
    const { current, queue } = get();
    current?.resolve(value);
    const [next, ...rest] = queue;
    set({ current: next ?? null, queue: rest });
  },
}));

export function alertDialog(
  message: string,
  title = "Octra",
): Promise<void> {
  return new Promise((resolve) => {
    useDialog.getState().enqueue({
      kind: "alert",
      title,
      message,
      confirmLabel: "OK",
      cancelLabel: "Anuluj",
      danger: false,
      defaultValue: "",
      resolve: () => resolve(),
    });
  });
}

export function confirmDialog(
  message: string,
  opts?: {
    title?: string;
    confirmLabel?: string;
    cancelLabel?: string;
    danger?: boolean;
  },
): Promise<boolean> {
  return new Promise((resolve) => {
    useDialog.getState().enqueue({
      kind: "confirm",
      title: opts?.title ?? "Potwierdź",
      message,
      confirmLabel: opts?.confirmLabel ?? "Potwierdź",
      cancelLabel: opts?.cancelLabel ?? "Anuluj",
      danger: opts?.danger ?? false,
      defaultValue: "",
      resolve: (v) => resolve(v === true),
    });
  });
}

export function promptDialog(
  message: string,
  opts?: { title?: string; defaultValue?: string; confirmLabel?: string },
): Promise<string | null> {
  return new Promise((resolve) => {
    useDialog.getState().enqueue({
      kind: "prompt",
      title: opts?.title ?? "Octra",
      message,
      confirmLabel: opts?.confirmLabel ?? "OK",
      cancelLabel: "Anuluj",
      danger: false,
      defaultValue: opts?.defaultValue ?? "",
      resolve: (v) => resolve(typeof v === "string" ? v : null),
    });
  });
}
