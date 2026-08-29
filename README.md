# Octra Launcher

Launcher Minecraft na Windows — interfejs według mockupów (Figtree, fiolet `#a051a2`, teal Launch).

**Repozytorium:** [github.com/VasstOFC/octra-launcher](https://github.com/VasstOFC/octra-launcher)

## Szybki start (dev)

```powershell
git clone https://github.com/VasstOFC/octra-launcher.git
cd octra-launcher
npm install
npm run dev:app
```

Albo skrót **Octra Dev** na pulpicie (`npm run shortcut`).

## Instalacja (testy ze znajomymi)

1. Pobierz **`Octra-setup.exe`** z [Releases](https://github.com/VasstOFC/octra-launcher/releases/latest).
2. Uruchom instalator — program trafi do `%LOCALAPPDATA%\Octra Launcher\`.
3. Dane gry (instancje, konta, skiny): `%APPDATA%\.octralauncher\`.

Build release włącza **auto-updater** (kanał Stable). W trybie dev (`npm run dev:app`) aktualizacje sprawdzasz ręcznie w **Ustawienia → Aktualizacje**.

## Skrypty

| Komenda | Co robi |
|---------|---------|
| `npm run dev:app` | Uruchom Octra w trybie deweloperskim (hot reload) |
| `npm start` | Uruchom zbudowany `octra.exe` (release); pierwszy raz sam buduje |
| `npm run build:app` | Zbuduj `octra.exe` bez instalatora |
| `npm run dist:windows` | Instalator `dist-installer/Octra-setup.exe` |
| `npm run installer:pack -- --sign --write-manifest` | Podpis + `latest.json` dla auto-updatera |
| `npm run shortcut` | Skróty na pulpicie: **Octra** + **Octra Dev** |

## Wydanie nowej wersji

1. Podnieś wersję w `package.json`, `src-tauri/Cargo.toml` i `src-tauri/tauri.conf.json`.
2. Ustaw sekrety w GitHub Actions: `TAURI_SIGNING_PRIVATE_KEY` (+ opcjonalnie hasło).
3. Utwórz tag i wypchnij:

```powershell
git tag v0.1.0
git push origin v0.1.0
```

Workflow **Release** zbuduje `Octra-setup.exe`, `latest.json` i opublikuje na GitHub Releases.

Lokalnie (bez CI):

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content .\.keys\octra.key -Raw
npm run dist:windows
node scripts/pack-installer.mjs --sign --write-manifest
```

## Dane

- Profil Octra: `%APPDATA%\.octralauncher\`
- Instalacja programu: `%LOCALAPPDATA%\Octra Launcher\`
- Nadpisanie: zmienna `OCTRA_DATA_DIR`
- Kanał dev vs stable: build debug = Dev; build release = Stable (`OCTRA_CHANNEL=stable` wymusza release)

## Wymagania

- Node.js 22+
- Rust **1.88+** (`rust-toolchain.toml` — `rustup toolchain install 1.88.0`)
- Visual Studio Build Tools (C++) + WebView2

## Funkcje (0.1.0)

- Logowanie Microsoft / offline
- Start: LAUNCH / STOP, profile, news
- Wersje, mody, shadery, światy, zasoby
- Znajomi i czat lokalnie, Relay, galeria F2, szafka skinów
- Skiny multiplayer (Lumen + CustomSkinLoader) bez SkinRestorer
- Modal po crashu
- Ustawienia: motyw, RAM, Java, aktualizacje, folder danych

## Licencja

Zobacz [LICENSE](LICENSE).
