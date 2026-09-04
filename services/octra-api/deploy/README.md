# Octra API — wdrożenie na VPS (IP, bez domeny)

Backend: FastAPI + SQLite + bcrypt + JWT. Obsługuje rejestrację kont Octra, logowanie i upload skinów (Bearer JWT lub legacy `X-Octra-Key`).

## Wymagania

- Ubuntu/Debian VPS z dostępem root (SSH)
- Python 3.11+
- Caddy (reverse proxy na :80)

## 1. Użytkownik i katalogi

```bash
sudo useradd --system --home /var/lib/octra-skins --shell /usr/sbin/nologin octra || true
sudo mkdir -p /opt/octra-api /var/lib/octra-skins /etc/octra
sudo chown -R octra:octra /var/lib/octra-skins
```

## 2. Kod API

Skopiuj folder `services/octra-api/` na VPS (np. `scp -r services/octra-api user@92.5.186.6:/tmp/`):

```bash
sudo rsync -a /tmp/octra-api/ /opt/octra-api/
sudo chown -R root:root /opt/octra-api
```

### MUST redeploy for chat image attachments (v1.9)

Chat screenshot sharing uploads PNG/JPEG/WebP to the API and stores them under
`$DATA_DIR/chat-media/`. Without this deploy, share still fails or only sends text.

**From your Windows/dev machine** (repo root):

```bash
scp services/octra-api/octra_api/main.py `
  services/octra-api/octra_api/db.py `
  user@92.5.186.6:/tmp/octra-api-update/
```

**On the VPS:**

```bash
sudo cp /tmp/octra-api-update/main.py /tmp/octra-api-update/db.py /opt/octra-api/octra_api/
sudo mkdir -p /var/lib/octra-skins/chat-media
sudo chown -R octra:octra /var/lib/octra-skins/chat-media
sudo systemctl restart octra-api
curl -s http://127.0.0.1:8787/health
```

Optional env in `/etc/octra/octra.env`: `CHAT_ATTACHMENT_MAX=8388608` (8 MiB default).

### MUST redeploy after passport schema change

Launcher registration no longer sends `username` — only `password`, `minecraft_nick`,
`profile_uuid`, and optional `account_type`. If the VPS still runs the old
`RegisterBody` (required `username`), clients get:

`Field required ... loc: ["body","username"]`

**From your Windows/dev machine** (repo root `G:\Nervia App` or equivalent):

```bash
# Prefer rsync (WSL/Git Bash). Adjust user@host if needed.
rsync -avz \
  services/octra-api/octra_api/main.py \
  services/octra-api/octra_api/db.py \
  services/octra-api/octra_api/auth_util.py \
  services/octra-api/requirements.txt \
  user@92.5.186.6:/tmp/octra-api-update/

# Or scp (PowerShell / OpenSSH):
scp services/octra-api/octra_api/main.py `
  services/octra-api/octra_api/db.py `
  services/octra-api/octra_api/auth_util.py `
  services/octra-api/requirements.txt `
  user@92.5.186.6:/tmp/octra-api-update/
```

**On the VPS:**

```bash
sudo mkdir -p /opt/octra-api/octra_api
sudo cp /tmp/octra-api-update/main.py /tmp/octra-api-update/db.py /tmp/octra-api-update/auth_util.py \
  /opt/octra-api/octra_api/
# only if requirements.txt changed:
# sudo /opt/octra-api/venv/bin/pip install -r /tmp/octra-api-update/requirements.txt
sudo systemctl restart octra-api
sudo systemctl status octra-api --no-pager
curl -s http://127.0.0.1:8787/health
# Then verify register accepts passport body (no username):
curl -s -X POST http://127.0.0.1:8787/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"password":"haslo1234","minecraft_nick":"RedeployCheck","profile_uuid":"00000000-0000-0000-0000-000000000099","account_type":"offline"}'
# Community list (Bearer from login/register). Without the new files this path is 404:
# curl -s http://127.0.0.1:8787/api/v1/community -H "Authorization: Bearer <token>"
```

A successful response is JSON with `token` / `username` / `minecraft_nick`. If you still see
`Field required ... username`, the old files were not copied into `/opt/octra-api/octra_api/`.

## 3. Venv i zależności

```bash
cd /opt/octra-api
sudo python3 -m venv venv
sudo ./venv/bin/pip install -r requirements.txt
```

## 4. Konfiguracja `/etc/octra/octra.env`

```bash
sudo tee /etc/octra/octra.env <<'EOF'
DATA_DIR=/var/lib/octra-skins
DATABASE_PATH=/var/lib/octra-skins/octra.db
JWT_SECRET=WYMIEN_NA_DLUGI_LOSOWY_CIEN
API_KEY=WYMIEN_NA_STARY_KLUCZ_OCTRA
PUBLIC_BASE_URL=http://92.5.186.6
EOF
sudo chmod 600 /etc/octra/octra.env
sudo chown root:octra /etc/octra/octra.env
```

Wygeneruj sekrety:

```bash
openssl rand -hex 32   # JWT_SECRET
openssl rand -hex 16   # API_KEY (opcjonalnie, dla starych launcherów)
```

`API_KEY` musi zgadzać się z `SKINS_API_KEY` w `packages/app-lib/src/nervia.rs` (fallback bez konta Octra).

## 5. systemd

```bash
sudo cp /opt/octra-api/deploy/octra-api.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now octra-api
sudo systemctl status octra-api
```

## 6. Caddy (IP :80)

Jeśli masz już Caddy z poprzednim `server.py`, zamień upstream na nowy serwis:

```
:80 {
	reverse_proxy 127.0.0.1:8787
}
```

```bash
sudo systemctl reload caddy
```

## 7. Test

```bash
curl http://127.0.0.1/health
# ok

# Rejestracja (paszport): nick + prawdziwy UUID z launchera + hasło.
# Login Octra = nick Minecraft (nick-as-login). account_type: premium|offline.
curl -X POST http://127.0.0.1/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"password":"haslo1234","minecraft_nick":"TestNick","profile_uuid":"00000000-0000-0000-0000-000000000001","account_type":"offline"}'

# Logowanie (nick lub legacy username + hasło)
curl -X POST http://127.0.0.1/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"TestNick","password":"haslo1234"}'
```

## Migracja ze starego `server.py`

- Pliki skinów w `by-uuid/` i `by-name/` pozostają w `DATA_DIR` — nie trzeba ich przenosić.
- Stary proces na :8787 zatrzymaj przed startem `octra-api`.
- SQLite (`octra.db`) tworzy się automatycznie przy pierwszym uruchomieniu.
- Istniejąca baza: przy starcie API doda kolumnę `account_type` (domyślnie `offline`) jeśli jej brakuje.
- Po wdrożeniu tej wersji skopiuj zaktualizowane `main.py` / `db.py` / `auth_util.py` i zrestartuj: `sudo systemctl restart octra-api`.
- **Passport:** bez redeployu VPS rejestracja z launchera pada na `username` Field required — zobacz sekcję „MUST redeploy” wyżej.

## Endpointy

| Metoda | Ścieżka | Opis |
|--------|---------|------|
| GET | `/health` | healthcheck |
| POST | `/api/v1/auth/register` | rejestracja: `password`, `minecraft_nick`, `profile_uuid`, opcjonalnie `account_type` (legacy opcjonalne `username` ignorowane przy nicku) |
| POST | `/api/v1/auth/login` | logowanie: `username` (nick) + `password` |
| GET | `/api/v1/auth/me` | sesja (Bearer) |
| GET | `/api/v1/community` | lista wszystkich kont Octra oprócz Ciebie (Bearer) — bez zaproszeń; zawiera `presence` (`launcher` / `ingame` / `offline`), `instance_name`, `join_address` (gdy start przez Octra/QuickPlay), `last_seen` |
| POST | `/api/v1/presence` | heartbeat: `{ "status": "launcher"|"ingame"|"offline", "instance_name": "...", "join_address": "host:port" }` (Bearer). Po ~60 s bez pulsu status spada do offline |
| GET | `/api/v1/chat/channels` | lista kanałów (DM + grupy) użytkownika (Bearer) |
| POST | `/api/v1/chat/channels/dm` | `{ "user_id": N }` — otwórz/utwórz DM |
| POST | `/api/v1/chat/channels/group` | `{ "name": "...", "member_ids": [..] }` — nowa grupa (min. 3 uczestników łącznie z twórcą) |
| GET/POST | `/api/v1/chat/channels/{id}/delete-vote` | głosowanie o usunięcie grupy (większość 2/3 „tak”) |
| GET | `/api/v1/chat/channels/{id}/messages?after_id=` | wiadomości kanału |
| POST | `/api/v1/chat/channels/{id}/messages` | `{ "text": "..." }` — wyślij na kanał |
| GET/POST | `/api/v1/chat` | legacy — Everyone usunięty (GET pusta lista, POST 410) |
| GET/PUT/POST | `/skins/{uuid}` | skin po UUID |
| GET | `/skins/MinecraftSkins/{nick}.png` | skin po nicku (legacy / SkinsRestorer / authlib textures) |
| GET | `/` lub `/index.json` | **authlib-injector** meta (`skinDomains`, `feature.non_email_login`) |
| GET | `/sessionserver/session/minecraft/profile/{uuid}` | profil Yggdrasil + textures (registry → Mojang fallthrough) |
| GET | `/sessionserver/session/minecraft/hasJoined` | `?username=` — join check (stub + registry) |
| POST | `/sessionserver/session/minecraft/join` | stub 204 (offline) |
| POST | `/api/profiles/minecraft` | lista `{id,name}` po nickach |
| POST | `/authserver/authenticate` (także refresh/validate) | stub sesji offline |
| GET | `/textures/{sha256}` | opcjonalnie PNG po hashu |

Launcher rejestruje konto Octra z domyślnego konta Minecraft (nick + UUID), bez osobnego username i bez tworzenia konta offline. Po zalogowaniu wysyła skiny z `Authorization: Bearer <jwt>`. Panel znajomych w launcherze pokazuje **wszystkie** konta z `GET /api/v1/community` (prywatny launcher — bez dodawania znajomych). Presence może zawierać `join_address`, gdy gracz wystartował multiplayer przez Octra (QuickPlay / lista serwerów) — wtedy inni widzą przycisk Dołącz. Czat: DM + grupy (min. 3 osoby; usuwanie grupy przez głosowanie 2/3). Wklejony link `.mrpack` → Instaluj w UI.

### MUST redeploy — shared remote Yggdrasil (authlib-injector)

Od tej wersji launcher wskazuje authlib-injector na **ten sam host** co registry (`http://92.5.186.6`), bez lokalnego hubu i bez CustomSkinLoader. Bez nowych tras Yggdrasil na VPS znajomi **nie zobaczą** skinów offline.

**Weryfikacja po `systemctl restart octra-api`:**

```bash
# Meta root (authlib-injector)
curl -s http://127.0.0.1:8787/ | head -c 400; echo
# Oczekuj: "serverName":"Octra", "skinDomains" zawiera 92.5.186.6

# Profil Vasstek (podstaw swój offline UUID z launchera / by-uuid)
# Przykład: najpierw znajdź plik:
ls /var/lib/octra-skins/by-name/vasstek.json
# potem:
NICK=Vasstek
UUID=$(python3 -c "import json; print(json.load(open('/var/lib/octra-skins/by-name/vasstek.json'))['uuid'])")
curl -s "http://127.0.0.1:8787/sessionserver/session/minecraft/profile/${UUID}"
# Oczekuj: properties[].name == textures, value (base64) z URL .../skins/MinecraftSkins/Vasstek.png

curl -s -o /dev/null -w "%{http_code}\n" "http://127.0.0.1:8787/skins/MinecraftSkins/${NICK}.png"
# Oczekuj: 200

# Z zewnątrz (ten sam URL co launcher):
curl -s http://92.5.186.6/ | head -c 200; echo
curl -s "http://92.5.186.6/sessionserver/session/minecraft/profile/${UUID}" | head -c 300; echo
```

Opcjonalnie ustaw `PUBLIC_BASE_URL=http://92.5.186.6` w `/etc/octra/octra.env` (domyślna wartość w kodzie jest taka sama).

**Uwaga HTTP:** authlib-injector akceptuje cleartext HTTP z ostrzeżeniem w logu JVM (`You are using HTTP protocol, which is INSECURE`). Działa na IP bez HTTPS; po cutoverze na domenę+TLS wystarczy zmienić `SKINS_URL` / `PUBLIC_BASE_URL`.

---

## HTTPS + domena (Cloudflare + Caddy) — gdy kupisz domenę

Produktowy default launchera pozostaje `http://92.5.186.6` (`SKINS_URL` w `packages/app-lib/src/nervia.rs`) dopóki nie ustawisz prawdziwej domeny. **Nie wpisuj placeholderów** typu `skins.example.com` do kodu.

### A. DNS (Cloudflare)

1. Kup domenę i dodaj ją do Cloudflare.
2. Utwórz rekord **A** (lub CNAME) wskazujący na VPS `92.5.186.6`, np. `skins.twojadomena.pl` → `92.5.186.6`.
3. Włącz **Proxy** (pomarańczowa chmura) — Cloudflare terminuje TLS po stronie klienta.
4. SSL/TLS mode w Cloudflare: **Full** (jeśli Caddy ma własny cert na origin) albo **Full (strict)** z prawdziwym certyfikatem origin. Unikaj trybu *Flexible* (HTTP do origin + HTTPS na zewnątrz).

### B. Caddy na VPS (origin)

Zastąp (lub rozszerz) blok `:80` o hostname domeny. Przykład z Let’s Encrypt bezpośrednio na Caddy (gdy Cloudflare jest w trybie DNS-only albo origin cert jest zbędny):

```
skins.twojadomena.pl {
	reverse_proxy 127.0.0.1:8787
}
```

Jeśli Cloudflare proxy zostaje włączone i wolisz cert origin od Cloudflare:

1. Cloudflare → SSL/TLS → Origin Server → utwórz Origin Certificate.
2. Zapisz cert + klucz na VPS (np. `/etc/caddy/origin.pem`, `/etc/caddy/origin.key`).
3. Caddyfile:

```
skins.twojadomena.pl {
	tls /etc/caddy/origin.pem /etc/caddy/origin.key
	reverse_proxy 127.0.0.1:8787
}
```

Opcjonalnie zostaw `:80` → ten sam upstream, żeby stare klienty na IP nadal działały w okresie przejściowym.

```bash
sudo systemctl reload caddy
curl -I https://skins.twojadomena.pl/health
```

### C. Launcher — `SKINS_URL` i rebuild

1. W `packages/app-lib/src/nervia.rs` ustaw:

```rust
pub const SKINS_URL: &str = "https://skins.twojadomena.pl";
```

   (użyj **swojej** domeny; wartość musi być bez trailing `/`).

2. Jeśli pack `.mrpack` jest hostowany na tym samym hostcie, zaktualizuj też `FEATURED_PACK_URL` w tym samym pliku.

3. Allowlist HTTP (Tauri): w `apps/app/capabilities/plugins.json` **dodaj** obok istniejącego IP:

```json
{ "url": "https://skins.twojadomena.pl/*" }
```

   Zostaw `http://92.5.186.6/*` na czas migracji albo usuń po pełnym cutoverze.

4. CSP: w `apps/app/tauri.conf.json` dopisz domenę do `connect-src` i `img-src` (obok `http://92.5.186.6`).

5. Zbuduj i opublikuj nowy launcher. `octra_skins::ygg_root()` / authlib-injector bierze URL z `nervia::skins_url()`, więc agent automatycznie dostanie HTTPS.

### D. Opcjonalny override bez rebuildu (dev / test)

Na maszynie z launcherem:

```bash
# Windows PowerShell
$env:OCTRA_SKINS_URL = "https://skins.twojadomena.pl"

# Linux / macOS
export OCTRA_SKINS_URL=https://skins.twojadomena.pl
```

Produkcyjne buildy i tak powinny mieć poprawny `SKINS_URL` w `nervia.rs` — env jest tylko wygodą lokalną.

### E. Checklist cutoveru

- [ ] DNS A/CNAME → VPS, Cloudflare proxy + SSL mode Full/Full strict
- [ ] Caddy serwuje hostname z TLS (origin cert lub Let’s Encrypt)
- [ ] `curl https://<domena>/health` zwraca OK
- [ ] `SKINS_URL` w `nervia.rs` → `https://<domena>`
- [ ] `FEATURED_PACK_URL` zaktualizowany jeśli pack jest na tym hostcie
- [ ] `plugins.json` — wpis `https://<domena>/*`
- [ ] `tauri.conf.json` — CSP `connect-src` / `img-src`
- [ ] Rebuild + dystrybucja launchera
- [ ] Test: upload skina + drugi klient Octra widzi skin przez authlib (bez CSL) po HTTPS
- [ ] `curl https://<domena>/` zwraca meta Yggdrasil; profil UUID zawiera textures
