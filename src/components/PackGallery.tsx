import { Download, Loader2, Search } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { api } from "../lib/api";
import { useApp } from "../stores/appStore";
import type { ModrinthPackHit } from "../types";

type Props = {
  onInstalled?: (instanceId: string) => void;
};

export function PackGallery({ onInstalled }: Props) {
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState("downloads");
  const [hits, setHits] = useState<ModrinthPackHit[]>([]);
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const [loading, setLoading] = useState(false);
  const [installing, setInstalling] = useState<string | null>(null);
  const progress = useApp((s) => s.progress);
  const refresh = useApp((s) => s.refreshInstances);
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);

  const load = useCallback(
    async (reset = false) => {
      setLoading(true);
      try {
        const off = reset ? 0 : offset;
        const res = await api.searchModrinthPacks({
          query: query.trim(),
          offset: off,
          limit: 20,
          sort,
        });
        setHits((prev) => (reset ? res.hits : [...prev, ...res.hits]));
        setTotal(res.totalHits);
        setOffset(off + res.hits.length);
      } catch (e) {
        showError(e instanceof Error ? e.message : String(e));
      } finally {
        setLoading(false);
      }
    },
    [query, sort, offset, showError],
  );

  useEffect(() => {
    void load(true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [query, sort]);

  async function installPack(pack: ModrinthPackHit) {
    setInstalling(pack.slug);
    try {
      const inst = await api.importModrinthPack(pack.slug, pack.iconUrl);
      await refresh();
      showOk(`Zainstalowano „${pack.title}".`);
      onInstalled?.(inst.id);
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setInstalling(null);
    }
  }

  const busy = Boolean(progress) || Boolean(installing);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex flex-wrap items-center gap-3 border-b border-line px-6 py-4">
        <div className="flex min-w-[220px] flex-1 items-center gap-2 rounded-xl bg-raised px-3 py-2 ring-1 ring-line">
          <Search size={16} className="text-mute" />
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Szukaj paczek Modrinth…"
            className="min-w-0 flex-1 bg-transparent text-sm outline-none"
          />
        </div>
        <select
          value={sort}
          onChange={(e) => setSort(e.target.value)}
          className="rounded-xl bg-raised px-3 py-2 text-sm ring-1 ring-line"
        >
          <option value="downloads">Popularność</option>
          <option value="follows">Obserwujący</option>
          <option value="updated">Ostatnia aktualizacja</option>
          <option value="relevance">Trafność</option>
        </select>
        <span className="text-xs text-mute">{total} wyników</span>
      </div>

      <div className="min-h-0 flex-1 overflow-auto p-6">
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
          {hits.map((p) => (
            <article
              key={p.slug}
              className="flex gap-3 rounded-2xl border border-line bg-raised2 p-3 transition hover:border-accent/40"
            >
              <div className="h-14 w-14 shrink-0 overflow-hidden rounded-xl bg-raised">
                {p.iconUrl ? (
                  <img src={p.iconUrl} alt="" className="h-full w-full object-cover" />
                ) : (
                  <div className="grid h-full place-items-center text-lg font-bold text-mute">
                    {p.title.slice(0, 1)}
                  </div>
                )}
              </div>
              <div className="min-w-0 flex-1">
                <h3 className="truncate text-sm font-bold">{p.title}</h3>
                <p className="mt-0.5 line-clamp-2 text-[11px] text-mute">{p.description}</p>
                <div className="mt-2 flex flex-wrap gap-1">
                  {p.loaders.slice(0, 3).map((l) => (
                    <span
                      key={l}
                      className="rounded-full bg-white/8 px-2 py-0.5 text-[10px] capitalize"
                    >
                      {l}
                    </span>
                  ))}
                </div>
                <button
                  disabled={busy}
                  onClick={() => void installPack(p)}
                  className="mt-2 inline-flex items-center gap-1 rounded-full bg-accent px-3 py-1 text-xs font-semibold text-bg-on-accent disabled:opacity-50"
                >
                  {installing === p.slug ? (
                    <Loader2 size={12} className="animate-spin" />
                  ) : (
                    <Download size={12} />
                  )}
                  Instaluj
                </button>
              </div>
            </article>
          ))}
        </div>
        {hits.length < total && (
          <button
            disabled={loading}
            onClick={() => void load(false)}
            className="mx-auto mt-6 block rounded-full border border-line px-4 py-2 text-sm text-mute hover:text-ink"
          >
            {loading ? "Ładowanie…" : "Więcej paczek"}
          </button>
        )}
        {!loading && hits.length === 0 && (
          <p className="mt-16 text-center text-sm text-mute">Brak wyników — spróbuj innej frazy.</p>
        )}
      </div>
    </div>
  );
}
