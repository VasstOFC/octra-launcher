import { create } from "zustand";

type PackUpdateState = {
  open: boolean;
  instanceId: string | null;
  openFor: (instanceId: string) => void;
  close: () => void;
};

export const usePackUpdate = create<PackUpdateState>((set) => ({
  open: false,
  instanceId: null,
  openFor: (instanceId) => set({ open: true, instanceId }),
  close: () => set({ open: false, instanceId: null }),
}));
