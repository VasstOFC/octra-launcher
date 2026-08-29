import { Download, Loader2, Sparkles } from "lucide-react";
import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { pl } from "../locales/pl";
import { useApp } from "../stores/appStore";
import { useOctra } from "../stores/octraStore";
import type { FeaturedPackInfo } from "../types";
import { PackGallery } from "../components/PackGallery";
import { SectionHeader } from "../components/ui/SectionHeader";

export function StorePage() {
  const select = useOctra((s) => s.selectInstance);
  const setView = useOctra((s) => s.setView);
  const instances = useApp((s) => s.instances);
  const refresh = useApp((s) => s.refreshInstances);
  const progress = useApp((s) => s.progress);
  const showError = useApp((s) => s.showError);
  const showOk = useApp((s) => s.showOk);
  const [featured, setFeatured] = useState<FeaturedPackInfo | null>(null);
  const [installingFeatured, setInstallingFeatured] = useState(false);

  useEffect(() => {
    void api.getFeaturedPack().then(setFeatured).catch(() => setFeatured(null));
  }, []);

  const featuredInstalled =
    featured?.enabled &&
    featured.slug &&
    instances.some((i) => i.linkedPack === featured.slug);

  async function installFeatured() {
    setInstallingFeatured(true);
    try {
      const inst = await api.installFeaturedPack();
      await refresh();
      showOk(`Zainstalowano „${featured?.title ?? "polecana paczka"}".`);
      select(inst.id);
      setView("versions");
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e));
    } finally {
      setInstallingFeatured(false);
    }
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="border-b border-line px-5 py-4">
        <SectionHeader title="Galeria paczek" />
        <p className="mt-1 text-[13px] text-mute">
          Przeglądaj i instaluj paczki modów z Modrinth.
        </p>
      </div>

      {featured?.enabled && !featuredInstalled ? (
        <div className="border-b border-line bg-gradient-to-r from-accent/15 via-raised2/80 to-raised px-5 py-5">
          <div className="flex flex-wrap items-center justify-between gap-4">
            <div className="flex items-start gap-3">
              <div className="grid h-12 w-12 place-items-center rounded-xl bg-accent/20 text-accent">
                <Sparkles size={22} />
              </div>
              <div>
                <p className="text-[10px] font-bold uppercase tracking-wider text-accent">
                  {pl.store.featuredTitle}
                </p>
                <h2 className="text-lg font-bold text-ink">{featured.title}</h2>
                {featured.blurb ? (
                  <p className="mt-1 max-w-xl text-sm text-mute">{featured.blurb}</p>
                ) : null}
              </div>
            </div>
            <button
              type="button"
              disabled={Boolean(progress) || installingFeatured}
              onClick={() => void installFeatured()}
              className="inline-flex items-center gap-2 rounded-full bg-accent px-4 py-2 text-sm font-semibold text-bg-on-accent disabled:opacity-50"
            >
              {installingFeatured ? (
                <Loader2 size={14} className="animate-spin" />
              ) : (
                <Download size={14} />
              )}
              {pl.store.featuredInstall}
            </button>
          </div>
        </div>
      ) : null}

      <PackGallery
        onInstalled={(id) => {
          select(id);
          setView("versions");
        }}
      />
    </div>
  );
}
