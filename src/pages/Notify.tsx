import { Bell, CheckCheck, Trash2 } from "lucide-react";
import { clsx } from "clsx";
import { useNotifications, type NotificationKind } from "../stores/notifications";
import { usePackUpdate } from "../stores/packUpdateStore";

const KIND_LABEL: Record<NotificationKind, string> = {
  crash: "Awaria",
  update: "Aktualizacja",
  "pack-update": "Paczka",
  friend: "Znajomy",
  info: "Info",
};

const KIND_COLOR: Record<NotificationKind, string> = {
  crash: "text-danger",
  update: "text-warn",
  "pack-update": "text-warn",
  friend: "text-good",
  info: "text-accent",
};

export function NotifyPage() {
  const items = useNotifications((s) => s.items);
  const unread = useNotifications((s) => s.unread);
  const markRead = useNotifications((s) => s.markRead);
  const markAllRead = useNotifications((s) => s.markAllRead);
  const clear = useNotifications((s) => s.clear);
  const openPackUpdate = usePackUpdate((s) => s.openFor);

  function onClick(n: (typeof items)[number]) {
    markRead(n.id);
    if (n.kind === "pack-update" && n.instanceId) {
      openPackUpdate(n.instanceId);
    }
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <header className="flex items-center justify-between border-b border-line px-6 py-4">
        <div>
          <h1 className="text-xl font-extrabold">Powiadomienia</h1>
          <p className="mt-0.5 text-xs text-mute">
            Awarie, aktualizacje modów, paczek i znajomi w sieci LAN.
          </p>
        </div>
        <div className="flex gap-2">
          {unread > 0 && (
            <button
              onClick={markAllRead}
              className="inline-flex items-center gap-1 rounded-full border border-line px-3 py-1.5 text-xs text-mute hover:text-ink"
            >
              <CheckCheck size={14} />
              Oznacz wszystkie
            </button>
          )}
          {items.length > 0 && (
            <button
              onClick={clear}
              className="inline-flex items-center gap-1 rounded-full border border-line px-3 py-1.5 text-xs text-mute hover:text-danger"
            >
              <Trash2 size={14} />
              Wyczyść
            </button>
          )}
        </div>
      </header>

      <div className="min-h-0 flex-1 overflow-auto p-4">
        {items.length === 0 ? (
          <div className="mt-24 text-center">
            <Bell size={32} className="mx-auto text-mute/40" />
            <p className="mt-3 text-sm text-mute">Brak powiadomień.</p>
          </div>
        ) : (
          <ul className="mx-auto max-w-2xl space-y-2">
            {items.map((n) => (
              <li key={n.id}>
                <button
                  onClick={() => onClick(n)}
                  className={clsx(
                    "w-full rounded-2xl border px-4 py-3 text-left transition",
                    n.read
                      ? "border-line/60 bg-raised/40 opacity-70"
                      : "border-accent/30 bg-raised2",
                  )}
                >
                  <div className="flex items-start justify-between gap-2">
                    <span className={clsx("text-[10px] font-bold uppercase", KIND_COLOR[n.kind])}>
                      {KIND_LABEL[n.kind]}
                    </span>
                    <time className="text-[10px] text-mute">
                      {new Date(n.at).toLocaleString("pl-PL")}
                    </time>
                  </div>
                  <p className="mt-1 text-sm font-semibold">{n.title}</p>
                  <p className="mt-0.5 text-xs text-mute">{n.body}</p>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}

