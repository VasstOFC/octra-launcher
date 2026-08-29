import { useApp } from "../stores/appStore";
import type { Instance } from "../types";

export function useProfileVisualEpoch(instanceId: string): number {
  return useApp((s) => s.profileVisualEpoch[instanceId] ?? 0);
}

export function useApplyProfileVisualUpdate() {
  const patchInstance = useApp((s) => s.patchInstance);
  const bumpProfileVisual = useApp((s) => s.bumpProfileVisual);
  return (inst: Instance) => {
    patchInstance(inst);
    bumpProfileVisual(inst.id);
  };
}
