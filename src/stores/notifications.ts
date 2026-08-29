import { create } from "zustand";

export type NotificationKind = "crash" | "update" | "pack-update" | "friend" | "info";

export type Notification = {
  id: string;
  kind: NotificationKind;
  title: string;
  body: string;
  at: number;
  read: boolean;
  instanceId?: string;
};

type NotificationState = {
  items: Notification[];
  unread: number;
  push: (n: Omit<Notification, "id" | "at" | "read"> & { at?: number }) => void;
  markRead: (id: string) => void;
  markAllRead: () => void;
  clear: () => void;
};

const LS = "octra-notifications";

function load(): Notification[] {
  try {
    const raw = localStorage.getItem(LS);
    if (raw) return JSON.parse(raw) as Notification[];
  } catch {
    /* ignore */
  }
  return [];
}

function persist(items: Notification[]) {
  localStorage.setItem(LS, JSON.stringify(items.slice(0, 100)));
}

export const useNotifications = create<NotificationState>((set, _get) => ({
  items: load(),
  unread: load().filter((n) => !n.read).length,
  push: (n) =>
    set((s) => {
      const item: Notification = {
        id: crypto.randomUUID(),
        at: n.at ?? Date.now(),
        read: false,
        kind: n.kind,
        title: n.title,
        body: n.body,
        instanceId: n.instanceId,
      };
      const items = [item, ...s.items].slice(0, 100);
      persist(items);
      return { items, unread: items.filter((i) => !i.read).length };
    }),
  markRead: (id) =>
    set((s) => {
      const items = s.items.map((i) => (i.id === id ? { ...i, read: true } : i));
      persist(items);
      return { items, unread: items.filter((i) => !i.read).length };
    }),
  markAllRead: () =>
    set((s) => {
      const items = s.items.map((i) => ({ ...i, read: true }));
      persist(items);
      return { items, unread: 0 };
    }),
  clear: () => {
    persist([]);
    set({ items: [], unread: 0 });
  },
}));

export function notifyCrash(instanceId: string, code: string, instanceName?: string) {
  useNotifications.getState().push({
    kind: "crash",
    title: "Awaria gry",
    body: `${instanceName ?? "Profil"} zakończył się z kodem ${code}.`,
    instanceId,
  });
}

export function notifyContentUpdates(instanceName: string, count: number) {
  if (count <= 0) return;
  useNotifications.getState().push({
    kind: "update",
    title: "Aktualizacje modów",
    body: `${instanceName}: dostępne ${count} aktualizacji treści.`,
  });
}

export function notifyPackUpdate(instanceId: string, instanceName: string) {
  useNotifications.getState().push({
    kind: "pack-update",
    title: "Aktualizacja paczki",
    body: `„${instanceName}”: dostępna nowa wersja paczki.`,
    instanceId,
  });
}

export function notifyFriendOnline(name: string) {
  useNotifications.getState().push({
    kind: "friend",
    title: "Znajomy online",
    body: `${name} jest dostępny w sieci LAN.`,
  });
}
