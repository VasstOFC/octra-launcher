# Octra Launcher

**Octra App** (nowy launcher) jest na gałęzi [`app`](https://github.com/VasstOFC/octra-launcher/tree/app). `main` to jeszcze stary Octra Launcher.

Po zmianie nazwy repo na `octra-app` linki `octra-launcher/releases` nadal działają (redirect GitHuba). Wydania: [Releases](https://github.com/VasstOFC/octra-launcher/releases/latest).

---

Launcher Minecraft na Windows — interfejs według mockupów (Figtree, fiolet `#a051a2`, teal Launch).

## Szybki start (dev)

```powershell
git clone https://github.com/VasstOFC/octra-launcher.git
cd octra-launcher
npm install
npm run dev:app
```

Albo skrót **Octra Dev** na pulpicie (`npm run shortcut`).

## Instalacja

1. Pobierz **`Octra-setup.exe`** z [Releases](https://github.com/VasstOFC/octra-launcher/releases/latest).
2. Uruchom instalator — program trafi do `%LOCALAPPDATA%\Octra Launcher\`.
3. Dane gry (instancje, konta, skiny): `%APPDATA%\.octralauncher\`.

Build release włącza **auto-updater** (kanał Stable). W trybie dev (`npm run dev:app`) aktualizacje sprawdzasz ręcznie w **Ustawienia → Aktualizacje**.

Jeśli aktualizacja z bardzo starej wersji (np. v1.0.0) nie instaluje się automatycznie, pobierz instalator ręcznie **raz** — kolejne aktualizacje powinny już działać normalnie.

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

1. Podnieś wersję w `package.json`, `src-tauri/Cargo.toml`, `installer/Cargo.toml` i `src-tauri/tauri.conf.json`.
2. Ustaw sekret `TAURI_SIGNING_PRIVATE_KEY` w GitHub Actions.
3. Utwórz tag i wypchnij:

```powershell
git tag v1.1.1
git push origin v1.1.1
```

Workflow **Release** zbuduje podpisany `Octra-setup.exe`, `latest.json` i opublikuje na GitHub Releases.

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

## Funkcje (v1.2.0)

- **Galeria paczek Modrinth** — większe karty, szczegóły paczki (opis, galeria, wersje, changelog), polecana paczka
- **Aktualizacje modpacków** — sprawdzanie nowej wersji paczki z changelogiem i resync jednym kliknięciem
- **Lista serwerów** — CRUD na `servers.json`, ping (online/offline, ms, gracze), sync z `servers.dat`
- **Smart Start** — kontekstowe podpowiedzi: aktualizacja paczki, ostatni profil, featured, serwer online
- Logowanie Microsoft / offline, ponowne logowanie po wygaśnięciu tokenu
- Start: LAUNCH / STOP, profile z tapetą/ikoną, news
- Wersje, mody, shadery, światy, zasoby; import z CurseForge / Prism / MultiMC
- Galeria zrzutów F2, szafka skinów, skórki offline (CustomSkinLoader)
- Lokalny serwer (Host): Paper / Vanilla / Fabric, ustawienia `server.properties`
- Auto-updater z GitHub Releases
- Ustawienia: motyw, RAM, Java, aktualizacje, folder danych

## Funkcje (v1.1.x)

- Wersja 1.1.x — profil wyglądu, auto-updater z fallbackiem, installer zamyka proces Octra

**Wkrótce:** znajomi, czat LAN (Relay), centrum powiadomień.

## Licencja

Zobacz [LICENSE](LICENSE).
