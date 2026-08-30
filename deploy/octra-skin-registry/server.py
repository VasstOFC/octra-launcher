#!/usr/bin/env python3
"""
Octra / Lumen skin registry — protokół zgodny z launcherem Octra.

  GET  /health
  GET  /skins/{uuid}              → PNG (+ nagłówki X-Lumen-*)
  PUT  /skins/{uuid}              → zapis PNG (nagłówek X-Octra-Key jeśli ustawiony API_KEY)
  POST /skins/{uuid}              → jak PUT
  GET  /skins/MinecraftSkins/{nick}.png  → PNG po nicku (CustomSkinLoader Legacy)

Zmienne środowiska:
  PORT=8787
  DATA_DIR=/var/lib/octra-skins
  API_KEY=...          (opcjonalnie; bez tego zapis jest otwarty — tylko LAN!)
  MAX_BODY=1048576
"""

from __future__ import annotations

import json
import os
import re
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Optional
from urllib.parse import unquote

PORT = int(os.environ.get("PORT", "8787"))
DATA_DIR = Path(os.environ.get("DATA_DIR", "/var/lib/octra-skins"))
API_KEY = os.environ.get("API_KEY", "").strip()
MAX_BODY = int(os.environ.get("MAX_BODY", str(1024 * 1024)))

UUID_HYPH = re.compile(
    r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
)
UUID_PLAIN = re.compile(r"^[0-9a-fA-F]{32}$")
NICK_RE = re.compile(r"^[a-zA-Z0-9_]{1,16}$")


def norm_uuid(raw: str) -> Optional[str]:
    s = raw.strip()
    if UUID_HYPH.fullmatch(s):
        return s.lower()
    if UUID_PLAIN.fullmatch(s):
        s = s.lower()
        return f"{s[0:8]}-{s[8:12]}-{s[12:16]}-{s[16:20]}-{s[20:32]}"
    return None


def png_path(uuid: str) -> Path:
    return DATA_DIR / "by-uuid" / f"{uuid}.png"


def meta_path(uuid: str) -> Path:
    return DATA_DIR / "by-uuid" / f"{uuid}.json"


def nick_path(nick: str) -> Path:
    return DATA_DIR / "by-name" / f"{nick.lower()}.json"


def read_meta(uuid: str) -> dict:
    p = meta_path(uuid)
    if not p.exists():
        return {}
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}


def write_skin(uuid: str, png: bytes, model: str, name: str) -> None:
    by_uuid = DATA_DIR / "by-uuid"
    by_name = DATA_DIR / "by-name"
    by_uuid.mkdir(parents=True, exist_ok=True)
    by_name.mkdir(parents=True, exist_ok=True)

    png_path(uuid).write_bytes(png)
    meta = {"model": model or "classic", "name": name or ""}
    meta_path(uuid).write_text(json.dumps(meta, ensure_ascii=False), encoding="utf-8")

    nick = (name or "").strip()
    if nick and NICK_RE.fullmatch(nick):
        nick_path(nick).write_text(
            json.dumps({"uuid": uuid, "model": meta["model"], "name": nick}, ensure_ascii=False),
            encoding="utf-8",
        )
        # Kopia PNG pod nickiem — niektóre klienty szukają po ścieżce Legacy.
        (DATA_DIR / "by-name" / f"{nick.lower()}.png").write_bytes(png)


def resolve_by_nick(nick: str) -> Optional[tuple[str, bytes, dict]]:
    key = nick_path(nick)
    if not key.exists():
        return None
    try:
        ref = json.loads(key.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    uuid = ref.get("uuid")
    if not isinstance(uuid, str):
        return None
    p = png_path(uuid)
    if not p.exists():
        p = DATA_DIR / "by-name" / f"{nick.lower()}.png"
    if not p.exists():
        return None
    meta = read_meta(uuid)
    if not meta.get("name"):
        meta["name"] = nick
    return uuid, p.read_bytes(), meta


class Handler(BaseHTTPRequestHandler):
    server_version = "OctraSkinRegistry/1.0"

    def log_message(self, fmt: str, *args) -> None:
        sys.stderr.write("%s - %s\n" % (self.address_string(), fmt % args))

    def _cors(self) -> None:
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, PUT, POST, OPTIONS")
        self.send_header(
            "Access-Control-Allow-Headers",
            "Content-Type, X-Lumen-Model, X-Lumen-Name, X-Octra-Key",
        )

    def _send(self, code: int, body: bytes, ctype: str, extra: Optional[dict] = None) -> None:
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self._cors()
        if extra:
            for k, v in extra.items():
                self.send_header(k, v)
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(body)

    def do_OPTIONS(self) -> None:
        self.send_response(204)
        self._cors()
        self.end_headers()

    def do_GET(self) -> None:
        path = unquote(self.path.split("?", 1)[0])
        if path in ("/", "/health"):
            self._send(200, b"ok", "text/plain; charset=utf-8")
            return

        parts = [p for p in path.strip("/").split("/") if p]
        if len(parts) == 2 and parts[0] == "skins":
            uuid = norm_uuid(parts[1])
            if uuid and png_path(uuid).exists():
                meta = read_meta(uuid)
                extra = {
                    "X-Lumen-Model": str(meta.get("model", "classic")),
                    "X-Lumen-Name": str(meta.get("name", "")),
                }
                self._send(200, png_path(uuid).read_bytes(), "image/png", extra)
                return
            self._send(404, b"not found", "text/plain; charset=utf-8")
            return

        if (
            len(parts) == 3
            and parts[0] == "skins"
            and parts[1] == "MinecraftSkins"
            and parts[2].lower().endswith(".png")
        ):
            nick = parts[2][:-4]
            if not NICK_RE.fullmatch(nick):
                self._send(400, b"bad nick", "text/plain; charset=utf-8")
                return
            resolved = resolve_by_nick(nick)
            if not resolved:
                self._send(404, b"not found", "text/plain; charset=utf-8")
                return
            _, png, meta = resolved
            extra = {
                "X-Lumen-Model": str(meta.get("model", "classic")),
                "X-Lumen-Name": str(meta.get("name", nick)),
            }
            self._send(200, png, "image/png", extra)
            return

        self._send(404, b"not found", "text/plain; charset=utf-8")

    def do_PUT(self) -> None:
        self._store()

    def do_POST(self) -> None:
        self._store()

    def _store(self) -> None:
        if API_KEY:
            key = self.headers.get("X-Octra-Key", "")
            if key != API_KEY:
                self._send(401, b"unauthorized", "text/plain; charset=utf-8")
                return

        path = unquote(self.path.split("?", 1)[0])
        parts = [p for p in path.strip("/").split("/") if p]
        if len(parts) != 2 or parts[0] != "skins":
            self._send(404, b"not found", "text/plain; charset=utf-8")
            return

        uuid = norm_uuid(parts[1])
        if not uuid:
            self._send(400, b"bad uuid", "text/plain; charset=utf-8")
            return

        length = int(self.headers.get("Content-Length", "0") or "0")
        if length <= 0 or length > MAX_BODY:
            self._send(400, b"bad body", "text/plain; charset=utf-8")
            return

        body = self.rfile.read(length)
        ctype = (self.headers.get("Content-Type") or "").lower()

        if "json" in ctype or (body[:1] == b"{"):
            try:
                info = json.loads(body.decode("utf-8"))
            except (UnicodeDecodeError, json.JSONDecodeError):
                self._send(400, b"bad json", "text/plain; charset=utf-8")
                return
            self._send(400, b"use direct png upload", "text/plain; charset=utf-8")
            return

        if not body.startswith(b"\x89PNG\r\n\x1a\n"):
            self._send(400, b"not png", "text/plain; charset=utf-8")
            return

        model = self.headers.get("X-Lumen-Model", "classic") or "classic"
        name = self.headers.get("X-Lumen-Name", "") or ""
        try:
            write_skin(uuid, body, model, name)
        except OSError as e:
            self._send(500, str(e).encode(), "text/plain; charset=utf-8")
            return

        resp = json.dumps({"uuid": uuid, "ok": True}).encode()
        self._send(200, resp, "application/json; charset=utf-8")


def main() -> None:
    DATA_DIR.mkdir(parents=True, exist_ok=True)
    bind = os.environ.get("BIND", "127.0.0.1")
    httpd = ThreadingHTTPServer((bind, PORT), Handler)
    print(f"Octra skin registry on http://{bind}:{PORT}  data={DATA_DIR}", flush=True)
    if API_KEY:
        print("Write protection: API_KEY enabled", flush=True)
    else:
        print("WARNING: API_KEY not set — anyone can upload skins!", flush=True)
    httpd.serve_forever()


if __name__ == "__main__":
    main()
