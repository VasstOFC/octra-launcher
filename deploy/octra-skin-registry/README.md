# Serwer skinów Octra na Ubuntu (Oracle VPS)

Prosty rejestr PNG zgodny z launcherem Octra. Dzięki niemu znajomi na **publicznych serwerach** widzą skiny offline / z biblioteki (wszyscy muszą grać przez Octrę z tym samym URL).

## 1. Przygotowanie VPS

```bash
sudo apt update && sudo apt install -y python3 caddy

sudo useradd --system --home /opt/octra-skin-registry --shell /usr/sbin/nologin octra || true
sudo mkdir -p /opt/octra-skin-registry /var/lib/octra-skins
sudo cp server.py /opt/octra-skin-registry/
sudo chmod +x /opt/octra-skin-registry/server.py
sudo chown -R octra:octra /opt/octra-skin-registry /var/lib/octra-skins
```

## 2. Klucz API (zalecane)

```bash
sudo tee /etc/octra-skins.env <<'EOF'
API_KEY=twoj-losowy-ciag-min-32-znakow
BIND=127.0.0.1
PORT=8787
DATA_DIR=/var/lib/octra-skins
EOF
sudo chmod 600 /etc/octra-skins.env
```

Wygeneruj klucz: `openssl rand -hex 24`

## 3. Systemd

```bash
sudo cp octra-skins.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now octra-skins
curl -s http://127.0.0.1:8787/health
```

Po starcie serwisu istnieje tylko `/var/lib/octra-skins`. Katalog **`by-uuid/`** powstaje dopiero przy **pierwszym udanym uploadzie** PNG — samo `health → ok` nie tworzy plików.

## 4. Caddy na porcie 80 (Oracle — zalecane)

Serwer Python zostaje na `127.0.0.1:8787`. Na zewnątrz wystawiasz tylko **port 80** przez Caddy.

```bash
sudo tee /etc/caddy/Caddyfile <<'EOF'
:80 {
    reverse_proxy 127.0.0.1:8787
}
EOF

sudo systemctl enable --now caddy
sudo systemctl reload caddy
curl -s http://127.0.0.1/health
```

**Oracle Cloud:** Ingress **TCP 80**, source `0.0.0.0/0` (w Security List subnetu instancji + NSG jeśli jest).

Test z PC: `curl http://TWOJE_IP/health` → `ok`

### HTTPS (opcjonalnie, gdy masz domenę)

1. DNS `A` → IP VPS.
2. Zamień Caddyfile na `skiny.twojadomena.pl { reverse_proxy 127.0.0.1:8787 }`
3. Otwórz też port **443** w Oracle.

## 5. Konfiguracja Octra

W **`src-tauri/src/config.rs`** (dla całej paczki / znajomych):

```rust
pub const LUMEN_SKINS_URL: &str = "http://92.5.186.6";
pub const LUMEN_SKINS_API_KEY: &str = "ten-sam-klucz-co-w-/etc/octra-skins.env";
```

Przebuduj launcher i rozdaj ten sam build znajomym — URL i klucz API są wbudowane w `config.rs`, bez ustawień w UI.

Upload z Octry następuje przy:

- **starce launchera** (sync wszystkich lokalnych skinów),
- **zapisie** skina offline w Szafie,
- **założeniu** skina z biblioteki (sam dodatek do biblioteki nie wysyła).

Sprawdź na VPS:

```bash
ls -la /var/lib/octra-skins/by-uuid/
sudo journalctl -u octra-skins -n 30 --no-pager
```

Test ręczny (zamień klucz i UUID):

```bash
curl -v -X PUT "http://127.0.0.1:8787/skins/00000000-0000-0000-0000-000000000001" \
  -H "Content-Type: image/png" \
  -H "X-Lumen-Model: classic" \
  -H "X-Lumen-Name: TestNick" \
  -H "X-Octra-Key: twoj-klucz-z-octra-skins.env" \
  --data-binary @/tmp/skin.png
```

Oczekiwane: `200` i plik w `by-uuid/`. Jeśli curl działa, a Octra nie — sprawdź wartości w `config.rs` i przebuduj launcher.

## 6. Znajomi

- Ten sam build Octry (URL i klucz API z `config.rs`).
- Gra przez **Octrę** (CustomSkinLoader instaluje się przy starcie gry).

## Protokół

| Metoda | Ścieżka | Opis |
|--------|---------|------|
| GET | `/health` | `ok` |
| GET | `/skins/{uuid}` | PNG skina |
| PUT | `/skins/{uuid}` | Zapis PNG, nagłówki `X-Lumen-Model`, `X-Lumen-Name`, opcjonalnie `X-Octra-Key` |
| GET | `/skins/MinecraftSkins/{nick}.png` | PNG po nicku (CustomSkinLoader) |

## Firewall — Oracle Ubuntu (iptables)

Na obrazach Ubuntu w **Oracle Cloud** często działa **iptables** (nie UFW), który przepuszcza tylko **SSH (22)**. Objaw: lokalnie `curl http://127.0.0.1/health` → `ok`, ale z internetu timeout.

```bash
sudo iptables -L INPUT -n -v --line-numbers
```

Dodaj port **80** przed regułą `REJECT` (numer może się różnić — sprawdź `--line-numbers`):

```bash
sudo iptables -I INPUT 6 -p tcp -m state --state NEW -m tcp --dport 80 -j ACCEPT
sudo iptables -L INPUT -n --line-numbers
```

Zapisz na stałe:

```bash
sudo apt install -y iptables-persistent
sudo netfilter-persistent save
```

Test z PC: `curl http://TWOJE_IP/health` → `ok`

**Restart instancji nie pomaga** — to nie Security List, tylko iptables na VM.

### UFW (opcjonalnie, po naprawie iptables)

UFW **nie naprawi** zewnętrznego timeoutu, jeśli winny jest iptables Oracle. Po działającym porcie 80 możesz utwardzić:

```bash
sudo apt install -y ufw
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 22/tcp
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable
```

Nie otwieraj **8787** — Python słucha tylko na `127.0.0.1`.

Serwer Python na `127.0.0.1:8787` — na zewnątrz tylko Caddy na porcie **80**.
