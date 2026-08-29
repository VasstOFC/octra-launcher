import { Paperclip, Send, Smile, Wifi, WifiOff } from "lucide-react";
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "../lib/api";
import { pl } from "../locales/pl";
import { useApp, useActiveAccount } from "../stores/appStore";
import { useOctra } from "../stores/octraStore";
import type { RelayPeerInfo } from "../types";

export function RelayPage() {
  const acc = useActiveAccount();
  const chatWith = useOctra((s) => s.chatWith);
  const openChat = useOctra((s) => s.openChat);
  const chats = useOctra((s) => s.chats);
  const appendRelayMessage = useOctra((s) => s.appendRelayMessage);
  const [peers, setPeers] = useState<RelayPeerInfo[]>([]);
  const [online, setOnline] = useState(false);
  const [q, setQ] = useState("");
  const [draft, setDraft] = useState("");
  const showError = useApp((s) => s.showError);

  const peer = peers.find((f) => f.id === chatWith);
  const msgs = chatWith ? chats[chatWith] ?? [] : [];

  useEffect(() => {
    if (!acc) return;
    api.relayStart(acc.name).then(() => setOnline(true)).catch((e) => showError(String(e)));
    return () => {
      api.relayStop().catch(() => {});
    };
  }, [acc?.name, showError]);

  useEffect(() => {
    const poll = setInterval(() => {
      api.relayListPeers().then(setPeers).catch(() => {});
    }, 10000);
    const unsubs: Array<() => void> = [];
    void listen<RelayPeerInfo>("relay-peer-online", () => {
      api.relayListPeers().then(setPeers).catch(() => {});
    }).then((u) => unsubs.push(u));
    void listen<{ peerId: string; peerName: string; text: string; at: number }>(
      "relay-message",
      (e) => {
        appendRelayMessage(e.payload.peerId, e.payload.text, "them", e.payload.at);
      },
    ).then((u) => unsubs.push(u));
    return () => {
      clearInterval(poll);
      unsubs.forEach((u) => u());
    };
  }, [appendRelayMessage]);

  async function send(text: string) {
    if (!chatWith || !text.trim()) return;
    appendRelayMessage(chatWith, text, "me");
    try {
      await api.relaySend(chatWith, text);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <div className="flex min-h-0 flex-1">
      <aside className="flex w-64 shrink-0 flex-col border-r border-line bg-raised">
        <div className="p-3">
          <div className="flex items-center gap-2">
            <h2 className="text-sm font-bold">{pl.nav.relay}</h2>
            {online ? (
              <Wifi size={14} className="text-good" aria-label="LAN aktywny" />
            ) : (
              <WifiOff size={14} className="text-mute" />
            )}
          </div>
          <p className="mt-1 text-[10px] text-mute">Czat P2P w sieci lokalnej między klientami Octra.</p>
          <input
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder="Szukaj w LAN…"
            className="mt-2 w-full rounded-lg bg-bg px-2 py-1.5 text-xs ring-1 ring-line"
          />
        </div>
        <div className="min-h-0 flex-1 overflow-auto">
          {peers
            .filter((f) => f.name.toLowerCase().includes(q.toLowerCase()))
            .map((f) => (
              <button
                key={f.id}
                onClick={() => openChat(f.id)}
                className={`flex w-full items-center gap-2 px-3 py-2 text-left ${
                  chatWith === f.id ? "bg-accent/20" : "hover:bg-white/5"
                }`}
              >
                <div className="grid h-8 w-8 place-items-center rounded-full bg-white/10 text-[10px] font-bold">
                  {f.name.slice(0, 2).toUpperCase()}
                </div>
                <div className="min-w-0">
                  <div className="truncate text-xs font-semibold">{f.name}</div>
                  <div className="truncate text-[10px] text-good">W sieci LAN</div>
                </div>
              </button>
            ))}
          {peers.length === 0 && (
            <p className="px-3 py-6 text-xs text-mute">
              Brak innych klientów Octra w LAN. Upewnij się, że firewall zezwala na UDP 47894.
            </p>
          )}
        </div>
      </aside>
      <section className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-12 items-center border-b border-line px-4 text-sm font-semibold">
          {peer ? peer.name : "Wybierz rozmowę"}
          {peer && <span className="ml-2 text-xs font-normal text-good">LAN</span>}
        </header>
        <div className="min-h-0 flex-1 space-y-2 overflow-auto p-4">
          {msgs.map((m) => (
            <div
              key={m.id}
              className={`max-w-[70%] rounded-2xl px-3 py-2 text-sm ${
                m.from === "me" ? "ml-auto bg-accent text-bg-on-accent" : "bg-white/8"
              }`}
            >
              {m.text}
            </div>
          ))}
        </div>
        <form
          className="flex items-center gap-2 border-t border-line p-3"
          onSubmit={(e) => {
            e.preventDefault();
            void send(draft);
            setDraft("");
          }}
        >
          <Paperclip size={16} className="text-mute" />
          <Smile size={16} className="text-mute" />
          <input
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            placeholder={peer ? `Wiadomość do ${peer.name}…` : "Wybierz kontakt z LAN"}
            className="flex-1 rounded-xl bg-raised px-3 py-2 text-sm outline-none ring-1 ring-line"
            disabled={!peer}
          />
          <button
            className="grid h-9 w-9 place-items-center rounded-full bg-accent text-bg-on-accent"
            type="submit"
          >
            <Send size={14} />
          </button>
        </form>
      </section>
    </div>
  );
}
