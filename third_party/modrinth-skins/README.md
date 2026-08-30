# Modrinth Skins — kod referencyjny

Ten katalog zawiera **kod źródłowy Modrinth App** (system skinów) sklonowany wyłącznie na potrzeby rozwoju Octra Launcher.

## Licencja

Kod w `code/` pochodzi z repozytorium [modrinth/code](https://github.com/modrinth/code) i jest objęty licencją **GNU General Public License v3.0 (GPLv3)**.

Fragmenty logiki przeniesione do Octry (`src-tauri/src/minecraft_skins/`, `src/lib/skins.ts`) są adaptacją tego systemu i muszą pozostać zgodne z GPLv3, jeśli dystrybuujemy pochodne dzieła.

## Struktura (sparse checkout)

| Ścieżka w `code/` | Opis |
|---|---|
| `packages/app-lib/src/api/minecraft_skins/` | Główne API Rust (listy, equip, normalizacja PNG) |
| `packages/app-lib/src/state/minecraft_skins/` | Persystencja SQLite + Mojang HTTP |
| `apps/app-frontend/src/helpers/skins.ts` | Typy TS + invoke |
| `apps/app-frontend/src/helpers/rendering/batch-skin-renderer.ts` | Miniatury 3D (Three.js) |
| `packages/ui/src/composables/skin-rendering/` | Podgląd 3D na żywo (TresJS) |
| `packages/assets/models/` | Modele GLTF classic/slim |

## Mapowanie na Octrę

| Modrinth | Octra |
|---|---|
| `get_available_skins` | `minecraft_skins::get_available_skins` |
| `get_available_capes` | `minecraft_skins::get_available_capes` |
| `equip_skin` | `commands::equip_skin` |
| `save_custom_skin` | `commands::save_custom_skin` |
| `normalize_skin_texture` | `minecraft_skins/png_util.rs` |
| SQLite `custom_minecraft_skins` | `skin_library.rs` (JSON + pliki PNG) |
| `DEFAULT_SKINS` (embedded base64) | `mojang_skins.json` + `catalog_bundled_textures.json` |

## Do przeniesienia później

- [ ] Pełny katalog `default_skins.rs` (~64 skiny z base64)
- [ ] `batch-skin-renderer.ts` + IndexedDB (`skin-preview-storage.ts`)
- [ ] Silnik 3D TresJS (`SkinPreviewRenderer`) — obecnie Octra używa `skinview3d`
- [ ] Debounce 10s przed sync Mojang (`PENDING_SKIN_CHANGE`)
- [ ] Drag-and-drop reorder (`set_custom_skin_order`)

## Aktualizacja kopii

```powershell
cd third_party\modrinth-skins\code
git pull origin main
```
