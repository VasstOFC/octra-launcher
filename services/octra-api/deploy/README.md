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

curl -X POST http://127.0.0.1/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"test","password":"haslo1234","minecraft_nick":"TestNick"}'
```

## Migracja ze starego `server.py`

- Pliki skinów w `by-uuid/` i `by-name/` pozostają w `DATA_DIR` — nie trzeba ich przenosić.
- Stary proces na :8787 zatrzymaj przed startem `octra-api`.
- SQLite (`octra.db`) tworzy się automatycznie przy pierwszym uruchomieniu.

## Endpointy

| Metoda | Ścieżka | Opis |
|--------|---------|------|
| GET | `/health` | healthcheck |
| POST | `/api/v1/auth/register` | rejestracja (otwarta) |
| POST | `/api/v1/auth/login` | logowanie |
| GET | `/api/v1/auth/me` | sesja (Bearer) |
| GET/PUT/POST | `/skins/{uuid}` | skin po UUID |
| GET | `/skins/MinecraftSkins/{nick}.png` | skin po nicku (CSL) |

Launcher po zalogowaniu na konto Octra wysyła skiny z `Authorization: Bearer <jwt>`.
