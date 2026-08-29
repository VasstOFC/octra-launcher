import { Check, MessageCircle, Search, UserPlus, X } from "lucide-react";
import { useMemo, useState } from "react";
import { clsx } from "clsx";
import { promptDialog } from "../lib/dialog";
import { useOctra } from "../stores/octraStore";

export function FriendsPanel({ embedded }: { embedded?: boolean }) {
  const tab = useOctra((s) => s.friendsTab);
  const setTab = useOctra((s) => s.setFriendsTab);
  const friends = useOctra((s) => s.friends);
  const requests = useOctra((s) => s.requests);
  const addFriend = useOctra((s) => s.addFriend);
  const openChat = useOctra((s) => s.openChat);
  const setView = useOctra((s) => s.setView);
  const [q, setQ] = useState("");
  const online = useMemo(
    () => friends.filter((f) => f.status !== "offline"),
    [friends],
  );
  const offline = useMemo(
    () => friends.filter((f) => f.status === "offline"),
    [friends],
  );
  const filtered = (list: typeof friends) =>
    list.filter((f) => f.name.toLowerCase().includes(q.toLowerCase()));

  return (
    <aside
      className={clsx(
        "flex min-h-0 flex-1 flex-col",
        !embedded && "w-[280px] shrink-0 border-l border-line bg-raised",
      )}
    >
      <div className="flex items-center gap-4 px-4 pt-4 text-sm font-semibold">
        <button
          className={clsx(tab === "friends" ? "text-ink" : "text-mute")}
          onClick={() => setTab("friends")}
        >
          Znajomi
        </button>
        <span className="rounded-full bg-accent/15 px-2 py-0.5 text-[9px] font-bold uppercase tracking-wide text-accent">
          Chmura — wkrótce
        </span>
        <button
          className={clsx("relative", tab === "requests" ? "text-ink" : "text-mute")}
          onClick={() => setTab("requests")}
        >
          Zaproszenia
          {requests.length > 0 && (
            <span className="absolute -right-2 -top-1 h-2 w-2 rounded-full bg-warn" />
          )}
        </button>
      </div>
      <div className="mt-3 flex items-center gap-2 px-3">
        <div className="flex min-w-0 flex-1 items-center gap-2 rounded-xl bg-bg px-2.5 py-1.5 ring-1 ring-line">
          <Search size={14} className="text-mute" />
          <input
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder="Szukaj gracza…"
            className="min-w-0 flex-1 bg-transparent text-xs outline-none placeholder:text-mute"
          />
        </div>
        <button
          className="grid h-8 w-8 place-items-center rounded-xl bg-white/5 text-mute hover:text-ink"
          title="Dodaj znajomego"
          onClick={async () => {
            const name = await promptDialog("Nick znajomego (lista lokalna, tryb dev):", {
              title: "Dodaj znajomego",
              confirmLabel: "Dodaj",
            });
            if (name) addFriend(name);
          }}
        >
          <UserPlus size={15} />
        </button>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto px-2 py-3">
        {tab === "requests" ? (
          <RequestsList />
        ) : friends.length === 0 ? (
          <p className="px-3 pt-8 text-center text-xs leading-relaxed text-mute">
            Lista znajomych jest lokalna w tej wersji dev. Dodaj nick przyciskiem + — bez
            serwera społeczności.
          </p>
        ) : (
          <>
            {filtered(online).length > 0 && (
              <Section title={`${filtered(online).length} Online`}>
                {filtered(online).map((f) => (
                  <FriendRow
                    key={f.id}
                    id={f.id}
                    name={f.name}
                    detail={f.detail}
                    live={f.status === "ingame"}
                    onChat={() => {
                      openChat(f.id);
                      setView("relay");
                    }}
                  />
                ))}
              </Section>
            )}
            {filtered(offline).length > 0 && (
              <Section title={`${filtered(offline).length} Offline`}>
                {filtered(offline).map((f) => (
                  <FriendRow
                    key={f.id}
                    id={f.id}
                    name={f.name}
                    detail={f.detail}
                    dim
                    onChat={() => {
                      openChat(f.id);
                      setView("relay");
                    }}
                  />
                ))}
              </Section>
            )}
          </>
        )}
      </div>
    </aside>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="mb-3">
      <p className="px-2 pb-1 text-[10px] font-semibold uppercase tracking-wider text-mute">
        {title}
      </p>
      {children}
    </div>
  );
}

function FriendRow({
  id,
  name,
  detail,
  dim,
  live,
  onChat,
}: {
  id: string;
  name: string;
  detail: string;
  dim?: boolean;
  live?: boolean;
  onChat: () => void;
}) {
  const remove = useOctra((s) => s.removeFriend);
  const [menu, setMenu] = useState(false);
  return (
    <div
      className={clsx(
        "group relative flex items-center gap-2 rounded-xl px-2 py-1.5 hover:bg-white/5",
        dim && "opacity-55",
      )}
      onContextMenu={(e) => {
        e.preventDefault();
        setMenu((v) => !v);
      }}
    >
      <div className="relative">
        <div className="grid h-8 w-8 place-items-center rounded-lg bg-white/8 text-[11px] font-bold">
          {name.slice(0, 2).toUpperCase()}
        </div>
        <span
          className={clsx(
            "absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full ring-2 ring-raised",
            dim ? "bg-mute" : live ? "bg-good" : "bg-warn",
          )}
        />
      </div>
      <div className="min-w-0 flex-1">
        <div className="truncate text-xs font-semibold">{name}</div>
        <div className={clsx("truncate text-[10px]", live ? "text-good" : "text-mute")}>
          {detail}
        </div>
      </div>
      <button
        className="opacity-0 group-hover:opacity-100"
        onClick={onChat}
        aria-label="Wiadomość"
      >
        <MessageCircle size={14} className="text-mute" />
      </button>
      {menu && (
        <div className="absolute right-2 top-10 z-20 w-40 rounded-xl border border-line bg-raised2 p-1 text-xs shadow-xl">
          <button className="block w-full rounded-lg px-2 py-1.5 text-left hover:bg-white/8" onClick={onChat}>
            Send Message
          </button>
          <button
            className="block w-full rounded-lg px-2 py-1.5 text-left hover:bg-white/8"
            onClick={() => {
              void navigator.clipboard.writeText(name);
              setMenu(false);
            }}
          >
            Copy IGN
          </button>
          <button
            className="block w-full rounded-lg px-2 py-1.5 text-left text-danger hover:bg-white/8"
            onClick={() => {
              remove(id);
              setMenu(false);
            }}
          >
            Unfriend
          </button>
        </div>
      )}
    </div>
  );
}

function RequestsList() {
  const requests = useOctra((s) => s.requests);
  const accept = useOctra((s) => s.acceptRequest);
  const decline = useOctra((s) => s.declineRequest);
  if (requests.length === 0) {
    return <p className="px-3 pt-8 text-center text-xs text-mute">Brak zaproszeń.</p>;
  }
  const incoming = requests.filter((r) => r.dir === "in");
  const outgoing = requests.filter((r) => r.dir === "out");
  return (
    <>
      {incoming.length > 0 && (
        <Section title="Received">
          {incoming.map((r) => (
            <div key={r.id} className="flex items-center gap-2 rounded-xl px-2 py-1.5">
              <div className="grid h-8 w-8 place-items-center rounded-lg bg-white/8 text-[11px] font-bold">
                {r.name.slice(0, 2).toUpperCase()}
              </div>
              <div className="min-w-0 flex-1">
                <div className="text-xs font-semibold">{r.name}</div>
                <div className="text-[10px] text-mute">{r.when}</div>
              </div>
              <button onClick={() => accept(r.id)} className="text-good">
                <Check size={14} />
              </button>
              <button onClick={() => decline(r.id)} className="text-danger">
                <X size={14} />
              </button>
            </div>
          ))}
        </Section>
      )}
      {outgoing.length > 0 && (
        <Section title="Sent">
          {outgoing.map((r) => (
            <div key={r.id} className="flex items-center gap-2 rounded-xl px-2 py-1.5">
              <div className="grid h-8 w-8 place-items-center rounded-lg bg-white/8 text-[11px] font-bold">
                {r.name.slice(0, 2).toUpperCase()}
              </div>
              <div className="min-w-0 flex-1 text-xs font-semibold">{r.name}</div>
              <button onClick={() => decline(r.id)} className="text-danger">
                <X size={14} />
              </button>
            </div>
          ))}
        </Section>
      )}
    </>
  );
}
