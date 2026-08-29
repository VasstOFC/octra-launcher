import { useOctra } from "../stores/octraStore";
import { PackGallery } from "../components/PackGallery";
import { SectionHeader } from "../components/ui/SectionHeader";

export function StorePage() {
  const select = useOctra((s) => s.selectInstance);
  const setView = useOctra((s) => s.setView);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="border-b border-line px-5 py-4">
        <SectionHeader title="Galeria paczek" />
        <p className="mt-1 text-[13px] text-mute">
          Przeglądaj i instaluj paczki modów z Modrinth.
        </p>
      </div>
      <PackGallery
        onInstalled={(id) => {
          select(id);
          setView("versions");
        }}
      />
    </div>
  );
}
