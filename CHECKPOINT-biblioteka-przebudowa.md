# CHECKPOINT — przebudowa modułu biblioteki

Data: 2026-09-12
Branch: `experiment/play-dock-lavender`
HEAD: `63e5a61307686038fcf9539cc71715a196175a41`

Ten plik opisuje działający stan projektu, do którego wracamy po eksperymentach z wyglądem.

## Stan zweryfikowany

- `vue-tsc --noEmit` (app-frontend) — OK, zero błędów
- `eslint` na wszystkich plikach biblioteki — OK, zero błędów i ostrzeżeń

## Pliki zmienione w ramach przebudowy (MOJE — nie ruszać przy revercie eksperymentów)

Modyfikacje:

- `apps/app-frontend/src/components/ui/library/use-library.ts` (~170 linii mniej; typy przeniesione, stan w composables, komparatory na cache timestampów)
- `apps/app-frontend/src/components/ui/library/library-toolbar/index.vue` (search podpięty pod `searchInput`/`setSearchInput`)
- `apps/app-frontend/src/components/ui/library/instance-group/index.vue` (dodany blok `<style>` z `content-visibility: auto`)
- `packages/ui/src/components/base/Avatar.vue` (fix migotania ikon: `detecting` → `opacity: 0` + `transition: opacity 0.2s ease-in-out`)

Nowe pliki:

- `apps/app-frontend/src/components/ui/library/library-types.ts`
- `apps/app-frontend/src/components/ui/library/composables/use-library-search.ts`
- `apps/app-frontend/src/components/ui/library/composables/use-library-timestamps.ts`
- `apps/app-frontend/src/components/ui/library/composables/use-library-server-types.ts`
- `apps/app-frontend/src/components/ui/library/composables/use-library-selection.ts`

## Pliki zmienione PRZED przebudową (nie moje — zostawić w spokoju)

- `AGENTS.md`
- `apps/installer/gen/schemas/capabilities.json`
- `services/octra-client-skins/bin/main/pl/octra/clientskins/SkinFetcher.class`

## Jak wrócić do tego stanu

Eksperymenty z wyglądem rób na osobnym branchu:

```powershell
git checkout -b eksperyment/wyglad-biblioteki
# ... eksperymenty ...
```

Powrót do checkpointu (porzucenie eksperymentów):

```powershell
git checkout experiment/play-dock-lavender
git branch -D eksperyment/wyglad-biblioteki
```

Kopia zapasowa samych zmian z przebudowy (patch):

```powershell
git diff -- apps/app-frontend/src/components/ui/library packages/ui/src/components/base/Avatar.vue > biblioteka-przebudowa.patch
git diff --stat
```

Przywrócenie z patcha:

```powershell
git apply biblioteka-przebudowa.patch
```

## Zakres przebudowy (co działa)

1. Podział monolitu `use-library.ts` (1458 linii) na 4 composables + `library-types.ts`; publiczne API (`provideLibrary`/`useLibrary`) bez zmian.
2. Debounce wyszukiwarki 200 ms (czyszczenie natychmiastowe).
3. Memoizacja dat (`toEpochMs` raz na instancję, sortowanie na liczbach — koniec alokacji `dayjs` w komparatorach).
4. Cache 60 s + single-flight dla detekcji typów serwerowych (koniec burzy requestów).
5. `content-visibility: auto` na grupach instancji.
6. Fix migotania ikon w `Avatar.vue`.

## Celowo odłożone (nie ruszać bez decyzji)

- Migracja `pages/Index.vue` na TanStack Query (`instanceListQueryOptions`)
- Unifikacja DnD (usunięcie `vuedraggable`)
- Virtual scroll / Grid-List toggle / bulk actions
