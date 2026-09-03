from __future__ import annotations

import json
import os
import re
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Annotated, Literal, Optional

from fastapi import Depends, FastAPI, Header, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse, PlainTextResponse, Response
from pydantic import BaseModel, Field

from .auth_util import (
	create_access_token,
	decode_access_token,
	hash_password,
	jwt_secret,
	norm_uuid,
	validate_account_type,
	validate_login_name,
	validate_minecraft_nick,
	validate_password,
	verify_password,
)
from .db import Database

DATA_DIR = Path(os.environ.get("DATA_DIR", "/var/lib/octra-skins"))
DB_PATH = Path(os.environ.get("DATABASE_PATH", str(DATA_DIR / "octra.db")))
API_KEY = os.environ.get("API_KEY", "").strip()
MAX_BODY = int(os.environ.get("MAX_BODY", str(1024 * 1024)))

app = FastAPI(title="Octra API", version="1.3.0")
PRESENCE_TTL = timedelta(seconds=60)
app.add_middleware(
	CORSMiddleware,
	allow_origins=["*"],
	allow_methods=["*"],
	allow_headers=["*"],
)

db = Database(DB_PATH)


class RegisterBody(BaseModel):
	"""Passport registration: identity comes from the launcher Minecraft account.

	Login username is the Minecraft nick (nick-as-login). profile_uuid must be the
	real premium or offline UUID from the launcher — not recomputed on the server.

	Optional ``username`` is accepted for older clients but ignored when
	``minecraft_nick`` is present (nick-as-login wins).
	"""

	password: str
	minecraft_nick: Optional[str] = None
	profile_uuid: str
	account_type: Optional[Literal["premium", "offline"]] = "offline"
	# Legacy field from pre-passport clients; prefer minecraft_nick when set.
	username: Optional[str] = None


class LoginBody(BaseModel):
	username: str = Field(description="Minecraft nick (or legacy Octra username)")
	password: str


class SessionResponse(BaseModel):
	token: str
	username: str
	minecraft_nick: str
	profile_uuid: str
	account_type: str = "offline"


class CommunityMember(BaseModel):
	id: int
	minecraft_nick: str
	profile_uuid: str
	account_type: str = "offline"
	created_at: str
	presence: Literal["launcher", "ingame", "offline"] = "offline"
	instance_name: Optional[str] = None
	last_seen: Optional[str] = None


class PresenceBody(BaseModel):
	status: Literal["launcher", "ingame", "offline"]
	instance_name: Optional[str] = None


def utcnow_iso() -> str:
	return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def parse_iso(value: Optional[str]) -> Optional[datetime]:
	if not value:
		return None
	raw = value.strip()
	if raw.endswith("Z"):
		raw = raw[:-1] + "+00:00"
	elif "T" not in raw and " " in raw:
		raw = raw.replace(" ", "T", 1) + "+00:00"
	try:
		parsed = datetime.fromisoformat(raw)
	except ValueError:
		return None
	if parsed.tzinfo is None:
		parsed = parsed.replace(tzinfo=timezone.utc)
	return parsed


def effective_presence(row: dict) -> tuple[str, Optional[str], Optional[str]]:
	last_seen = row.get("last_seen") or None
	raw = (row.get("presence_status") or "offline").strip().lower()
	instance = row.get("presence_instance") or None
	seen_at = parse_iso(last_seen)
	stale = seen_at is None or datetime.now(timezone.utc) - seen_at > PRESENCE_TTL
	if stale or raw not in ("launcher", "ingame"):
		return "offline", None, last_seen
	if raw != "ingame":
		instance = None
	return raw, instance, last_seen


def community_member_from_row(row: dict) -> CommunityMember:
	presence, instance_name, last_seen = effective_presence(row)
	return CommunityMember(
		id=int(row["id"]),
		minecraft_nick=row["minecraft_nick"],
		profile_uuid=row["profile_uuid"],
		account_type=row.get("account_type") or "offline",
		created_at=row["created_at"],
		presence=presence,  # type: ignore[arg-type]
		instance_name=instance_name,
		last_seen=last_seen,
	)


def session_from_user(user: dict, token: str = "") -> SessionResponse:
	return SessionResponse(
		token=token,
		username=user["username"],
		minecraft_nick=user["minecraft_nick"],
		profile_uuid=user["profile_uuid"],
		account_type=user.get("account_type") or "offline",
	)


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
		validate_password(body.password)
		# Nick-as-login: prefer minecraft_nick; fall back to legacy username.
		raw_nick = (body.minecraft_nick or body.username or "").strip()
		if not raw_nick:
			raise ValueError("minecraft_nick jest wymagany")
		nick = validate_minecraft_nick(raw_nick)
		account_type = validate_account_type(body.account_type)
	except ValueError as exc:
		raise HTTPException(status_code=400, detail=str(exc)) from exc

	profile_uuid = norm_uuid(body.profile_uuid)
	if not profile_uuid:
		raise HTTPException(status_code=400, detail="nieprawidłowy profile_uuid")

	# Octra login username always equals the Minecraft nick.
	username = nick

	if db.get_user_by_username(username) or db.get_user_by_minecraft_nick(nick):
		raise HTTPException(status_code=409, detail="nick minecraft jest zajęty")
	if db.get_user_by_profile_uuid(profile_uuid):
		raise HTTPException(status_code=409, detail="ten profil minecraft jest już powiązany")

	password_hash = hash_password(body.password)
	user = db.create_user(username, password_hash, nick, profile_uuid, account_type)
	token = create_access_token(user["id"], jwt_secret())
	return session_from_user(user, token)


@app.post("/api/v1/auth/login", response_model=SessionResponse)
async def login(body: LoginBody) -> SessionResponse:
	try:
		login_name = validate_login_name(body.username)
	except ValueError as exc:
		raise HTTPException(status_code=400, detail=str(exc)) from exc

	user = db.get_user_by_username(login_name) or db.get_user_by_minecraft_nick(login_name)
	if not user or not verify_password(body.password, user["password_hash"]):
		raise HTTPException(status_code=401, detail="nieprawidłowa nazwa użytkownika lub hasło")

	token = create_access_token(user["id"], jwt_secret())
	return session_from_user(user, token)


@app.get("/api/v1/auth/me", response_model=SessionResponse)
async def me(user: Annotated[dict, Depends(bearer_user)]) -> SessionResponse:
	return session_from_user(user)


@app.get("/api/v1/community", response_model=list[CommunityMember])
async def community(
	user: Annotated[dict, Depends(bearer_user)],
) -> list[CommunityMember]:
	"""Every other Octra account — private launcher, no friend requests."""
	return [community_member_from_row(row) for row in db.list_users_except(int(user["id"]))]


@app.post("/api/v1/presence")
async def presence(
	body: PresenceBody,
	user: Annotated[dict, Depends(bearer_user)],
) -> JSONResponse:
	instance = (body.instance_name or "").strip() or None
	if body.status != "ingame":
		instance = None
	db.set_presence(int(user["id"]), body.status, instance, utcnow_iso())
	return JSONResponse({"ok": True})


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
			registered_nick = skin_user["minecraft_nick"]
			header_nick = (x_lumen_name or "").strip()
			if skin_user["profile_uuid"].lower() == normalized.lower():
				allowed = True
				skin_name = registered_nick
			elif header_nick and header_nick.lower() == registered_nick.lower():
				allowed = True
				skin_name = registered_nick
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
