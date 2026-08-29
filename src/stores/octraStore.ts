import { create } from "zustand";
import { pl } from "../locales/pl";

export type OctraView =
  | "home"
  | "locker"
  | "notify"
  | "relay"
  | "versions"
  | "gallery"
  | "store"
  | "host"
  | "settings";

export type ContentTab = "loader" | "mods" | "shaders" | "worlds" | "resources" | "advanced";

export const VIEW_LABELS: Record<OctraView, string> = {
  home: pl.nav.home,
  locker: pl.nav.locker,
  notify: pl.nav.notify,
  relay: pl.nav.relay,
  versions: pl.nav.versions,
  gallery: pl.nav.gallery,
  store: pl.nav.store,
  host: pl.nav.host,
  settings: pl.nav.settings,
};

export const CONTENT_TAB_LABELS: Record<ContentTab, string> = {
  loader: "Loader",
  mods: "Mody",
  shaders: "Shadery",
  worlds: "Światy",
  resources: "Zasoby",
  advanced: "Zaawansowane",
};

export type Breadcrumb = { label: string; key: string };

export type Friend = {
  id: string;
  name: string;
  status: "online" | "idle" | "ingame" | "offline";
  detail: string;
  best?: boolean;
};

export type ChatMsg = { id: string; from: "me" | "them"; text: string; at: number };

export type Overlay =
  | null
  | { kind: "content"; tab: ContentTab }
  | { kind: "crash"; instanceId: string; code: string };

export function buildBreadcrumbs(opts: {
  view: OctraView;
  profileName?: string | null;
  overlay: Overlay;
}): Breadcrumb[] {
  const crumbs: Breadcrumb[] = [{ label: pl.breadcrumb.start, key: "start" }];
  const onHome = opts.view === "home" && !opts.overlay;

  if (opts.profileName && (opts.view !== "home" || opts.overlay)) {
    crumbs.push({ label: opts.profileName, key: "profile" });
  }

  if (opts.view !== "home") {
    crumbs.push({ label: VIEW_LABELS[opts.view], key: `view-${opts.view}` });
  }

  if (opts.overlay?.kind === "content") {
    crumbs.push({
      label: CONTENT_TAB_LABELS[opts.overlay.tab],
      key: `tab-${opts.overlay.tab}`,
    });
  } else if (opts.overlay?.kind === "crash") {
    crumbs.push({ label: "Awaria", key: "crash" });
  }

  if (onHome && opts.profileName) {
    crumbs.push({ label: opts.profileName, key: "profile-home" });
  }

  return crumbs;
}

type OctraState = {
  view: OctraView;
  selectedId: string | null;
  overlay: Overlay;
  friendsTab: "friends" | "requests";
  friends: Friend[];
  requests: { id: string; name: string; when: string; dir: "in" | "out" }[];
  chatWith: string | null;
  chats: Record<string, ChatMsg[]>;
  friendsOpen: boolean;
  instanceCreatorOpen: boolean;
  octraMenuOpen: boolean;
  setFriendsOpen: (v: boolean) => void;
  openInstanceCreator: () => void;
  closeInstanceCreator: () => void;
  toggleOctraMenu: () => void;
  setOctraMenuOpen: (v: boolean) => void;
  setView: (v: OctraView) => void;
  selectInstance: (id: string | null) => void;
  openContent: (tab: ContentTab) => void;
  openCrash: (instanceId: string, code: string) => void;
  closeOverlay: () => void;
  setFriendsTab: (t: "friends" | "requests") => void;
  addFriend: (name: string) => void;
  removeFriend: (id: string) => void;
  acceptRequest: (id: string) => void;
  declineRequest: (id: string) => void;
  openChat: (id: string | null) => void;
  sendChat: (text: string) => void;
  appendRelayMessage: (
    peerId: string,
    text: string,
    from: "me" | "them",
    at?: number,
  ) => void;
  syncLanFriends: (peers: Friend[]) => void;
};

const LS = "octra-experimental-social";

function loadSocial(): Pick<OctraState, "friends" | "requests" | "chats"> {
  try {
    const raw = localStorage.getItem(LS);
    if (raw) return JSON.parse(raw);
  } catch {
    /* ignore */
  }
  return { friends: [], requests: [], chats: {} };
}

function persist(s: Pick<OctraState, "friends" | "requests" | "chats">) {
  localStorage.setItem(LS, JSON.stringify(s));
}

const DISABLED_VIEWS = new Set<OctraView>(["notify", "relay"]);

export const useOctra = create<OctraState>((set, get) => ({
  view: "home",
  selectedId: null,
  overlay: null,
  friendsTab: "friends",
  ...loadSocial(),
  chatWith: null,
  friendsOpen: false,
  instanceCreatorOpen: false,
  octraMenuOpen: false,
  setFriendsOpen: (friendsOpen) => set({ friendsOpen }),
  openInstanceCreator: () => set({ view: "versions", overlay: null, instanceCreatorOpen: true }),
  closeInstanceCreator: () => set({ instanceCreatorOpen: false }),
  toggleOctraMenu: () => set((s) => ({ octraMenuOpen: !s.octraMenuOpen })),
  setOctraMenuOpen: (octraMenuOpen) => set({ octraMenuOpen }),
  setView: (view) => {
    if (DISABLED_VIEWS.has(view)) return;
    set({
      view,
      overlay: null,
      octraMenuOpen: false,
      chatWith: view === "relay" ? get().chatWith : get().chatWith,
      instanceCreatorOpen: view === "versions" ? get().instanceCreatorOpen : false,
    });
  },
  selectInstance: (selectedId) => set({ selectedId }),
  openContent: (tab) => set({ overlay: { kind: "content", tab } }),
  openCrash: (instanceId, code) => set({ overlay: { kind: "crash", instanceId, code } }),
  closeOverlay: () => set({ overlay: null }),
  setFriendsTab: (friendsTab) => set({ friendsTab }),
  addFriend: (name) => {
    const n = name.trim();
    if (!n) return;
    set((s) => {
      const next = {
        friends: [
          ...s.friends,
          {
            id: crypto.randomUUID(),
            name: n,
            status: "offline" as const,
            detail: "Dodany lokalnie (dev)",
          },
        ],
        requests: s.requests,
        chats: s.chats,
      };
      persist(next);
      return next;
    });
  },
  removeFriend: (id) =>
    set((s) => {
      const next = {
        friends: s.friends.filter((f) => f.id !== id),
        requests: s.requests,
        chats: s.chats,
      };
      persist(next);
      return { ...next, chatWith: s.chatWith === id ? null : s.chatWith };
    }),
  acceptRequest: (id) =>
    set((s) => {
      const req = s.requests.find((r) => r.id === id);
      if (!req) return s;
      const next = {
        friends: [
          ...s.friends,
          { id: req.id, name: req.name, status: "offline" as const, detail: "Offline" },
        ],
        requests: s.requests.filter((r) => r.id !== id),
        chats: s.chats,
      };
      persist(next);
      return next;
    }),
  declineRequest: (id) =>
    set((s) => {
      const next = {
        friends: s.friends,
        requests: s.requests.filter((r) => r.id !== id),
        chats: s.chats,
      };
      persist(next);
      return next;
    }),
  openChat: (chatWith) => set({ chatWith }),
  sendChat: (text) => {
    const t = text.trim();
    const id = get().chatWith;
    if (!t || !id) return;
    set((s) => {
      const list = s.chats[id] ?? [];
      const chats = {
        ...s.chats,
        [id]: [...list, { id: crypto.randomUUID(), from: "me" as const, text: t, at: Date.now() }],
      };
      persist({ friends: s.friends, requests: s.requests, chats });
      return { chats };
    });
  },
  appendRelayMessage: (peerId, text, from, at) => {
    const t = text.trim();
    if (!t) return;
    set((s) => {
      const list = s.chats[peerId] ?? [];
      const chats = {
        ...s.chats,
        [peerId]: [
          ...list,
          {
            id: crypto.randomUUID(),
            from,
            text: t,
            at: at ?? Date.now(),
          },
        ],
      };
      persist({ friends: s.friends, requests: s.requests, chats });
      return { chats };
    });
  },
  syncLanFriends: (peers) =>
    set((s) => {
      const lan = peers.map((p) => ({
        id: p.id,
        name: p.name,
        status: "online" as const,
        detail: "W sieci LAN",
      }));
      const manual = s.friends.filter((f) => !f.detail.includes("LAN"));
      const next = { friends: [...lan, ...manual], requests: s.requests, chats: s.chats };
      persist(next);
      return next;
    }),
}));
