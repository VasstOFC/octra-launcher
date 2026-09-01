from __future__ import annotations

import json
import os
import re
from pathlib import Path
from typing import Annotated, Optional

from fastapi import Depends, FastAPI, Header, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse, PlainTextResponse, Response
from pydantic import BaseModel

from .auth_util import (
	create_access_token,
	decode_access_token,
	hash_password,
	jwt_secret,
	norm_uuid,
	offline_player_uuid,
	validate_minecraft_nick,
	validate_password,
	validate_username,
	verify_password,
)
from .db import Database

DATA_DIR = Path(os.environ.get("DATA_DIR", "/var/lib/octra-skins"))
DB_PATH = Path(os.environ.get("DATABASE_PATH", str(DATA_DIR / "octra.db")))
API_KEY = os.environ.get("API_KEY", "").strip()
MAX_BODY = int(os.environ.get("MAX_BODY", str(1024 * 1024)))

app = FastAPI(title="Octra API", version="1.0.0")
app.add_middleware(
	CORSMiddleware,
	allow_origins=["*"],
	allow_methods=["*"],
	allow_headers=["*"],
)

db = Database(DB_PATH)


class RegisterBody(BaseModel):
	username: str
	password: str
	minecraft_nick: str


class LoginBody(BaseModel):
	username: str
	password: str


class SessionResponse(BaseModel):
	token: str
	username: str
	minecraft_nick: str
	profile_uuid: str


def write_skin(uuid: str, png: bytes, model: str, name: str) -> None:
	by_uuid = DATA_DIR / "by-uuid"
	by_name = DATA_DIR / "by-name"
	by_uuid.mkdir(parents=True, exist_ok=True)
	by_name.mkdir(parents=True, exist_ok=True)

	(by_uuid / f"{uuid}.png").write_bytes(png)
	meta = {"model": model or "classic", "name": name or ""}
	(by_uuid / f"{uuid}.json").write_text(
		json.dumps(meta, ensure_ascii=False), encoding="utf-8"
	)

	nick = (name or "").strip()
	if re.fullmatch(r"^[a-zA-Z0-9_]{1,16}$", nick):
		(by_name / f"{nick.lower()}.json").write_text(
			json.dumps({"uuid": uuid, "model": meta["model"], "name": nick}, ensure_ascii=False),
			encoding="utf-8",
		)
		(by_name / f"{nick.lower()}.png").write_bytes(png)


def read_meta(uuid: str) -> dict:
	path = DATA_DIR / "by-uuid" / f"{uuid}.json"
	if not path.exists():
		return {}
	try:
		return json.loads(path.read_text(encoding="utf-8"))
	except (OSError, json.JSONDecodeError):
		return {}


def resolve_by_nick(nick: str) -> Optional[tuple[str, bytes, dict]]:
	key = DATA_DIR / "by-name" / f"{nick.lower()}.json"
	if not key.exists():
		return None
	try:
		ref = json.loads(key.read_text(encoding="utf-8"))
	except (OSError, json.JSONDecodeError):
		return None
	uuid = ref.get("uuid")
	if not isinstance(uuid, str):
		return None
	png_path = DATA_DIR / "by-uuid" / f"{uuid}.png"
	if not png_path.exists():
		png_path = DATA_DIR / "by-name" / f"{nick.lower()}.png"
	if not png_path.exists():
		return None
	meta = read_meta(uuid)
	if not meta.get("name"):
		meta["name"] = nick
	return uuid, png_path.read_bytes(), meta


def bearer_user(authorization: Annotated[Optional[str], Header()] = None) -> dict:
	if not authorization or not authorization.lower().startswith("bearer "):
		raise HTTPException(status_code=401, detail="missing bearer token")
	token = authorization[7:].strip()
	try:
		payload = decode_access_token(token, jwt_secret())
	except Exception as exc:
		raise HTTPException(status_code=401, detail="invalid token") from exc
	user = db.get_user_by_id(int(payload["sub"]))
	if not user:
		raise HTTPException(status_code=401, detail="user not found")
	return user


def legacy_api_key_allowed(x_octra_key: Annotated[Optional[str], Header()] = None) -> bool:
	if not API_KEY:
		return False
	return x_octra_key == API_KEY


@app.get("/health")
async def health() -> PlainTextResponse:
	return PlainTextResponse("ok")


@app.post("/api/v1/auth/register", response_model=SessionResponse)
async def register(body: RegisterBody) -> SessionResponse:
	try:
		username = validate_username(body.username)
		validate_password(body.password)
		nick = validate_minecraft_nick(body.minecraft_nick)
	except ValueError as exc:
		raise HTTPException(status_code=400, detail=str(exc)) from exc

	if db.get_user_by_username(username):
		raise HTTPException(status_code=409, detail="nazwa użytkownika jest zajęta")
	if db.get_user_by_minecraft_nick(nick):
		raise HTTPException(status_code=409, detail="nick minecraft jest zajęty")

	profile_uuid = offline_player_uuid(nick)
	password_hash = hash_password(body.password)
	user = db.create_user(username, password_hash, nick, profile_uuid)
	token = create_access_token(user["id"], jwt_secret())
	return SessionResponse(
		token=token,
		username=user["username"],
		minecraft_nick=user["minecraft_nick"],
		profile_uuid=user["profile_uuid"],
	)


@app.post("/api/v1/auth/login", response_model=SessionResponse)
async def login(body: LoginBody) -> SessionResponse:
	try:
		username = validate_username(body.username)
	except ValueError as exc:
		raise HTTPException(status_code=400, detail=str(exc)) from exc

	user = db.get_user_by_username(username)
	if not user or not verify_password(body.password, user["password_hash"]):
		raise HTTPException(status_code=401, detail="nieprawidłowa nazwa użytkownika lub hasło")

	token = create_access_token(user["id"], jwt_secret())
	return SessionResponse(
		token=token,
		username=user["username"],
		minecraft_nick=user["minecraft_nick"],
		profile_uuid=user["profile_uuid"],
	)


@app.get("/api/v1/auth/me", response_model=SessionResponse)
async def me(user: Annotated[dict, Depends(bearer_user)]) -> SessionResponse:
	return SessionResponse(
		token="",
		username=user["username"],
		minecraft_nick=user["minecraft_nick"],
		profile_uuid=user["profile_uuid"],
	)


@app.get("/skins/{uuid}")
async def get_skin_by_uuid(uuid: str) -> Response:
	normalized = norm_uuid(uuid)
	if not normalized:
		raise HTTPException(status_code=400, detail="bad uuid")
	png_path = DATA_DIR / "by-uuid" / f"{normalized}.png"
	if not png_path.is_file():
		raise HTTPException(status_code=404, detail="not found")
	meta = read_meta(normalized)
	headers = {
		"X-Lumen-Model": str(meta.get("model", "classic")),
		"X-Lumen-Name": str(meta.get("name", "")),
	}
	return Response(content=png_path.read_bytes(), media_type="image/png", headers=headers)


@app.api_route("/skins/{uuid}", methods=["PUT", "POST"])
async def put_skin_by_uuid(
	uuid: str,
	request: Request,
	authorization: Annotated[Optional[str], Header()] = None,
	x_octra_key: Annotated[Optional[str], Header()] = None,
	x_lumen_model: Annotated[Optional[str], Header()] = "classic",
	x_lumen_name: Annotated[Optional[str], Header()] = None,
) -> JSONResponse:
	normalized = norm_uuid(uuid)
	if not normalized:
		raise HTTPException(status_code=400, detail="bad uuid")

	body = await request.body()
	if len(body) <= 0 or len(body) > MAX_BODY:
		raise HTTPException(status_code=400, detail="bad body")
	if not body.startswith(b"\x89PNG\r\n\x1a\n"):
		raise HTTPException(status_code=400, detail="not png")

	allowed = False
	skin_name = x_lumen_name or ""

	if authorization and authorization.lower().startswith("bearer "):
		try:
			skin_user = bearer_user(authorization)
			if skin_user["profile_uuid"].lower() == normalized.lower():
				allowed = True
				skin_name = skin_user["minecraft_nick"]
		except HTTPException:
			allowed = False

	if not allowed and legacy_api_key_allowed(x_octra_key):
		allowed = True

	if not allowed:
		raise HTTPException(status_code=401, detail="unauthorized")

	model = (x_lumen_model or "classic").strip() or "classic"
	write_skin(normalized, body, model, skin_name)
	return JSONResponse({"uuid": normalized, "ok": True})


@app.get("/skins/MinecraftSkins/{nick}.png")
async def get_skin_by_nick(nick: str) -> Response:
	nick = nick.removesuffix(".png")
	if not re.fullmatch(r"^[a-zA-Z0-9_]{1,16}$", nick):
		raise HTTPException(status_code=400, detail="bad nick")
	resolved = resolve_by_nick(nick)
	if not resolved:
		raise HTTPException(status_code=404, detail="not found")
	_, png, meta = resolved
	headers = {
		"X-Lumen-Model": str(meta.get("model", "classic")),
		"X-Lumen-Name": str(meta.get("name", nick)),
	}
	return Response(content=png, media_type="image/png", headers=headers)
