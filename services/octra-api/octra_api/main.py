from __future__ import annotations

import base64
import hashlib
import json
import os
import re
import time
import urllib.error
import urllib.request
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Annotated, Any, Literal, Optional

from fastapi import Depends, FastAPI, Header, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse, PlainTextResponse, Response
from pydantic import BaseModel, Field
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.types import ASGIApp

from .auth_util import (
	create_access_token,
	decode_access_token,
	hash_password,
	jwt_secret,
	norm_uuid,
	offline_player_uuid,
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
# Public base used in Yggdrasil profile texture URLs (authlib-injector skinDomains).
# Keep in sync with packages/app-lib/src/nervia.rs SKINS_URL until HTTPS cutover.
DEFAULT_PUBLIC_BASE = os.environ.get("PUBLIC_BASE_URL", "http://92.5.186.6").strip().rstrip("/")

app = FastAPI(title="Octra API", version="1.8.0")
PRESENCE_TTL = timedelta(seconds=60)
CHAT_MAX_LEN = 1000
CHAT_MIN_INTERVAL = timedelta(seconds=2)
app.add_middleware(
	CORSMiddleware,
	allow_origins=["*"],
	allow_methods=["*"],
	allow_headers=["*"],
)

db = Database(DB_PATH)


class AuthlibInjectorLocationMiddleware(BaseHTTPMiddleware):
	"""Advertise the Yggdrasil API root for authlib-injector ALI discovery."""

	def __init__(self, app: ASGIApp, public_base: str) -> None:
		super().__init__(app)
		self.public_base = public_base.rstrip("/")

	async def dispatch(self, request: Request, call_next):  # type: ignore[no-untyped-def]
		response = await call_next(request)
		response.headers.setdefault(
			"X-Authlib-Injector-API-Location", f"{self.public_base}/"
		)
		return response


app.add_middleware(AuthlibInjectorLocationMiddleware, public_base=DEFAULT_PUBLIC_BASE)


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
	join_address: Optional[str] = None
	pack_project_id: Optional[str] = None
	pack_version_id: Optional[str] = None
	last_seen: Optional[str] = None


class PresenceBody(BaseModel):
	status: Literal["launcher", "ingame", "offline"]
	instance_name: Optional[str] = None
	join_address: Optional[str] = None
	pack_project_id: Optional[str] = None
	pack_version_id: Optional[str] = None


class ChatMember(BaseModel):
	id: int
	minecraft_nick: str
	profile_uuid: str


class ChatChannel(BaseModel):
	id: int
	kind: Literal["dm", "group"]
	name: Optional[str] = None
	created_at: str
	last_body: Optional[str] = None
	last_at: Optional[str] = None
	last_id: Optional[int] = None
	last_read_id: int = 0
	unread_count: int = 0
	members: list[ChatMember] = Field(default_factory=list)


class ChatReaction(BaseModel):
	emoji: str
	count: int
	user_ids: list[int] = Field(default_factory=list)


class ChatMessage(BaseModel):
	id: int
	channel_id: int
	user_id: int
	minecraft_nick: str
	body: str
	created_at: str
	pinned: bool = False
	deleted: bool = False
	reactions: list[ChatReaction] = Field(default_factory=list)


class ChatPostBody(BaseModel):
	text: str = Field(min_length=1, max_length=CHAT_MAX_LEN)


class ChatDmBody(BaseModel):
	user_id: int


class ChatGroupBody(BaseModel):
	name: str = Field(min_length=1, max_length=64)
	member_ids: list[int] = Field(default_factory=list)


class ChatGroupMembersBody(BaseModel):
	member_ids: list[int] = Field(default_factory=list)


class ChatReadBody(BaseModel):
	last_read_id: int = 0


class ChatReactionBody(BaseModel):
	emoji: str = Field(min_length=1, max_length=16)


class ChatPinBody(BaseModel):
	pinned: bool = True


class SharedServer(BaseModel):
	id: int
	name: str
	address: str
	created_by: int
	created_by_nick: Optional[str] = None
	created_at: str


class SharedServerBody(BaseModel):
	name: str = Field(min_length=1, max_length=64)
	address: str = Field(min_length=1, max_length=255)


ALLOWED_REACTION_EMOJIS = {"👍", "❤️", "😂", "🔥", "🎉", "👀"}


def chat_message_from_row(row: dict) -> ChatMessage:
	return ChatMessage(
		id=int(row["id"]),
		channel_id=int(row["channel_id"]),
		user_id=int(row["user_id"]),
		minecraft_nick=row["minecraft_nick"],
		body="" if row.get("deleted_at") else row["body"],
		created_at=row["created_at"],
		pinned=bool(row.get("pinned")),
		deleted=bool(row.get("deleted_at")),
		reactions=[
			ChatReaction(
				emoji=r["emoji"],
				count=int(r["count"]),
				user_ids=[int(uid) for uid in r.get("user_ids") or []],
			)
			for r in row.get("reactions") or []
		],
	)


def chat_channel_from_row(row: dict) -> ChatChannel:
	last_id = row.get("last_id")
	last_read_id = int(row.get("last_read_id") or 0)
	unread = 0
	if last_id is not None and int(last_id) > last_read_id:
		unread = 1
	return ChatChannel(
		id=int(row["id"]),
		kind=row["kind"],  # type: ignore[arg-type]
		name=row.get("name"),
		created_at=row["created_at"],
		last_body=row.get("last_body"),
		last_at=row.get("last_at"),
		last_id=int(last_id) if last_id is not None else None,
		last_read_id=last_read_id,
		unread_count=unread,
		members=[
			ChatMember(
				id=int(m["id"]),
				minecraft_nick=m["minecraft_nick"],
				profile_uuid=m["profile_uuid"],
			)
			for m in row.get("members") or []
		],
	)


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


def effective_presence(
	row: dict,
) -> tuple[str, Optional[str], Optional[str], Optional[str], Optional[str], Optional[str]]:
	last_seen = row.get("last_seen") or None
	raw = (row.get("presence_status") or "offline").strip().lower()
	instance = row.get("presence_instance") or None
	join_address = row.get("presence_join_address") or None
	pack_project_id = row.get("presence_pack_project_id") or None
	pack_version_id = row.get("presence_pack_version_id") or None
	seen_at = parse_iso(last_seen)
	stale = seen_at is None or datetime.now(timezone.utc) - seen_at > PRESENCE_TTL
	if stale or raw not in ("launcher", "ingame"):
		return "offline", None, None, None, None, last_seen
	if raw != "ingame":
		return raw, None, None, None, None, last_seen
	return raw, instance, join_address, pack_project_id, pack_version_id, last_seen


def community_member_from_row(row: dict) -> CommunityMember:
	presence, instance_name, join_address, pack_project_id, pack_version_id, last_seen = (
		effective_presence(row)
	)
	return CommunityMember(
		id=int(row["id"]),
		minecraft_nick=row["minecraft_nick"],
		profile_uuid=row["profile_uuid"],
		account_type=row.get("account_type") or "offline",
		created_at=row["created_at"],
		presence=presence,  # type: ignore[arg-type]
		instance_name=instance_name,
		join_address=join_address,
		pack_project_id=pack_project_id,
		pack_version_id=pack_version_id,
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


def normalize_stored_model(model: Optional[str]) -> str:
	"""Persist Steve/Alex as Yggdrasil values: `default` or `slim`."""
	raw = str(model or "").strip().lower()
	return "slim" if raw == "slim" else "default"


def yggdrasil_texture_model(model: Optional[str]) -> str:
	"""Authlib / Mojang texture metadata: `default` (Steve) or `slim` (Alex)."""
	return normalize_stored_model(model)


def write_skin(uuid: str, png: bytes, model: str, name: str) -> None:
	by_uuid = DATA_DIR / "by-uuid"
	by_name = DATA_DIR / "by-name"
	by_uuid.mkdir(parents=True, exist_ok=True)
	by_name.mkdir(parents=True, exist_ok=True)

	(by_uuid / f"{uuid}.png").write_bytes(png)
	meta = {"model": normalize_stored_model(model), "name": name or ""}
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
	db.ensure_user_in_everyone(int(user["id"]))
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
	join_address = (body.join_address or "").strip() or None
	pack_project_id = (body.pack_project_id or "").strip() or None
	pack_version_id = (body.pack_version_id or "").strip() or None
	if body.status != "ingame":
		instance = None
		join_address = None
		pack_project_id = None
		pack_version_id = None
	elif join_address and len(join_address) > 255:
		raise HTTPException(status_code=400, detail="join_address too long")
	db.set_presence(
		int(user["id"]),
		body.status,
		instance,
		join_address,
		pack_project_id,
		pack_version_id,
		utcnow_iso(),
	)
	return JSONResponse({"ok": True})


@app.get("/api/v1/chat/channels", response_model=list[ChatChannel])
async def chat_channels(
	user: Annotated[dict, Depends(bearer_user)],
) -> list[ChatChannel]:
	db.ensure_user_in_everyone(int(user["id"]))
	rows = db.list_channels_for_user(int(user["id"]))
	return [chat_channel_from_row(row) for row in rows]


@app.post("/api/v1/chat/channels/dm", response_model=ChatChannel)
async def chat_open_dm(
	body: ChatDmBody,
	user: Annotated[dict, Depends(bearer_user)],
) -> ChatChannel:
	target = db.get_user_by_id(int(body.user_id))
	if not target:
		raise HTTPException(status_code=404, detail="user not found")
	try:
		channel = db.get_or_create_dm(int(user["id"]), int(body.user_id), utcnow_iso())
	except ValueError as exc:
		raise HTTPException(status_code=400, detail=str(exc)) from exc
	members = db.channel_member_rows(int(channel["id"]))
	channel["members"] = members
	channel["last_read_id"] = 0
	channel["unread_count"] = 0
	return chat_channel_from_row(channel)


@app.post("/api/v1/chat/channels/group", response_model=ChatChannel)
async def chat_create_group(
	body: ChatGroupBody,
	user: Annotated[dict, Depends(bearer_user)],
) -> ChatChannel:
	name = body.name.strip()
	if not name:
		raise HTTPException(status_code=400, detail="name required")
	member_ids: list[int] = []
	for raw_id in body.member_ids:
		uid = int(raw_id)
		if uid == int(user["id"]):
			continue
		if not db.get_user_by_id(uid):
			raise HTTPException(status_code=404, detail=f"user {uid} not found")
		member_ids.append(uid)
	channel = db.create_group(name, int(user["id"]), member_ids, utcnow_iso())
	members = db.channel_member_rows(int(channel["id"]))
	channel["members"] = members
	channel["last_read_id"] = 0
	channel["unread_count"] = 0
	return chat_channel_from_row(channel)


@app.post("/api/v1/chat/channels/{channel_id}/members", response_model=ChatChannel)
async def chat_add_group_members(
	channel_id: int,
	body: ChatGroupMembersBody,
	user: Annotated[dict, Depends(bearer_user)],
) -> ChatChannel:
	channel = db.get_channel(channel_id)
	if not channel or channel.get("kind") != "group":
		raise HTTPException(status_code=404, detail="group not found")
	if channel.get("name") == "Everyone":
		raise HTTPException(status_code=400, detail="cannot edit Everyone membership")
	if not db.is_channel_member(channel_id, int(user["id"])):
		raise HTTPException(status_code=403, detail="not a channel member")
	member_ids: list[int] = []
	for raw_id in body.member_ids:
		uid = int(raw_id)
		if not db.get_user_by_id(uid):
			raise HTTPException(status_code=404, detail=f"user {uid} not found")
		member_ids.append(uid)
	db.add_group_members(channel_id, member_ids)
	rows = db.list_channels_for_user(int(user["id"]))
	updated = next((r for r in rows if int(r["id"]) == channel_id), None)
	if not updated:
		raise HTTPException(status_code=404, detail="channel not found")
	return chat_channel_from_row(updated)


@app.post("/api/v1/chat/channels/{channel_id}/read")
async def chat_mark_read(
	channel_id: int,
	body: ChatReadBody,
	user: Annotated[dict, Depends(bearer_user)],
) -> JSONResponse:
	if not db.is_channel_member(channel_id, int(user["id"])):
		raise HTTPException(status_code=403, detail="not a channel member")
	db.mark_channel_read(channel_id, int(user["id"]), int(body.last_read_id))
	return JSONResponse({"ok": True})


@app.get("/api/v1/chat/channels/{channel_id}/messages", response_model=list[ChatMessage])
async def chat_channel_messages(
	channel_id: int,
	user: Annotated[dict, Depends(bearer_user)],
	after_id: int = 0,
) -> list[ChatMessage]:
	if not db.is_channel_member(channel_id, int(user["id"])):
		raise HTTPException(status_code=403, detail="not a channel member")
	rows = db.list_chat_messages(channel_id, after_id=max(0, after_id), limit=80)
	return [chat_message_from_row(row) for row in rows]


@app.post("/api/v1/chat/channels/{channel_id}/messages", response_model=ChatMessage)
async def chat_channel_post(
	channel_id: int,
	body: ChatPostBody,
	user: Annotated[dict, Depends(bearer_user)],
) -> ChatMessage:
	if not db.is_channel_member(channel_id, int(user["id"])):
		raise HTTPException(status_code=403, detail="not a channel member")
	text = body.text.strip()
	if not text:
		raise HTTPException(status_code=400, detail="empty message")
	user_id = int(user["id"])
	last_at = parse_iso(db.last_chat_message_at(user_id, channel_id))
	if last_at is not None and datetime.now(timezone.utc) - last_at < CHAT_MIN_INTERVAL:
		raise HTTPException(status_code=429, detail="slow down")
	row = db.add_chat_message(
		channel_id,
		user_id,
		user["minecraft_nick"],
		text,
		utcnow_iso(),
	)
	db.mark_channel_read(channel_id, user_id, int(row["id"]))
	return chat_message_from_row(row)


@app.delete("/api/v1/chat/messages/{message_id}", response_model=ChatMessage)
async def chat_delete_message(
	message_id: int,
	user: Annotated[dict, Depends(bearer_user)],
) -> ChatMessage:
	message = db.get_chat_message(message_id)
	if not message or message.get("deleted_at"):
		raise HTTPException(status_code=404, detail="message not found")
	if int(message["user_id"]) != int(user["id"]):
		raise HTTPException(status_code=403, detail="not your message")
	if not db.is_channel_member(int(message["channel_id"]), int(user["id"])):
		raise HTTPException(status_code=403, detail="not a channel member")
	updated = db.soft_delete_message(message_id, utcnow_iso())
	if not updated:
		raise HTTPException(status_code=404, detail="message not found")
	return chat_message_from_row(updated)


@app.post("/api/v1/chat/messages/{message_id}/pin", response_model=ChatMessage)
async def chat_pin_message(
	message_id: int,
	body: ChatPinBody,
	user: Annotated[dict, Depends(bearer_user)],
) -> ChatMessage:
	message = db.get_chat_message(message_id)
	if not message or message.get("deleted_at"):
		raise HTTPException(status_code=404, detail="message not found")
	if not db.is_channel_member(int(message["channel_id"]), int(user["id"])):
		raise HTTPException(status_code=403, detail="not a channel member")
	updated = db.set_message_pinned(message_id, body.pinned)
	if not updated:
		raise HTTPException(status_code=404, detail="message not found")
	return chat_message_from_row(updated)


@app.post("/api/v1/chat/messages/{message_id}/reactions", response_model=ChatMessage)
async def chat_react_message(
	message_id: int,
	body: ChatReactionBody,
	user: Annotated[dict, Depends(bearer_user)],
) -> ChatMessage:
	emoji = body.emoji.strip()
	if emoji not in ALLOWED_REACTION_EMOJIS:
		raise HTTPException(status_code=400, detail="unsupported emoji")
	message = db.get_chat_message(message_id)
	if not message or message.get("deleted_at"):
		raise HTTPException(status_code=404, detail="message not found")
	if not db.is_channel_member(int(message["channel_id"]), int(user["id"])):
		raise HTTPException(status_code=403, detail="not a channel member")
	updated = db.toggle_reaction(message_id, int(user["id"]), emoji, utcnow_iso())
	if not updated:
		raise HTTPException(status_code=404, detail="message not found")
	return chat_message_from_row(updated)


@app.get("/api/v1/servers", response_model=list[SharedServer])
async def list_shared_servers(
	user: Annotated[dict, Depends(bearer_user)],
) -> list[SharedServer]:
	_ = user
	return [
		SharedServer(
			id=int(row["id"]),
			name=row["name"],
			address=row["address"],
			created_by=int(row["created_by"]),
			created_by_nick=row.get("created_by_nick"),
			created_at=row["created_at"],
		)
		for row in db.list_shared_servers()
	]


@app.post("/api/v1/servers", response_model=SharedServer)
async def add_shared_server(
	body: SharedServerBody,
	user: Annotated[dict, Depends(bearer_user)],
) -> SharedServer:
	name = body.name.strip()
	address = body.address.strip()
	if not name or not address:
		raise HTTPException(status_code=400, detail="name and address required")
	try:
		row = db.add_shared_server(name, address, int(user["id"]), utcnow_iso())
	except Exception as exc:
		raise HTTPException(status_code=409, detail="server address already exists") from exc
	return SharedServer(
		id=int(row["id"]),
		name=row["name"],
		address=row["address"],
		created_by=int(row["created_by"]),
		created_by_nick=row.get("created_by_nick"),
		created_at=row["created_at"],
	)


@app.delete("/api/v1/servers/{server_id}")
async def delete_shared_server(
	server_id: int,
	user: Annotated[dict, Depends(bearer_user)],
) -> JSONResponse:
	_ = user
	if not db.delete_shared_server(server_id):
		raise HTTPException(status_code=404, detail="server not found")
	return JSONResponse({"ok": True})


# Legacy global chat endpoints kept as thin wrappers onto the Everyone group.
@app.get("/api/v1/chat", response_model=list[ChatMessage])
async def chat_list_legacy(
	user: Annotated[dict, Depends(bearer_user)],
	after_id: int = 0,
) -> list[ChatMessage]:
	db.ensure_user_in_everyone(int(user["id"]))
	channels = db.list_channels_for_user(int(user["id"]))
	everyone = next(
		(c for c in channels if c.get("kind") == "group" and c.get("name") == "Everyone"),
		None,
	)
	if not everyone:
		return []
	return await chat_channel_messages(int(everyone["id"]), user, after_id)


@app.post("/api/v1/chat", response_model=ChatMessage)
async def chat_post_legacy(
	body: ChatPostBody,
	user: Annotated[dict, Depends(bearer_user)],
) -> ChatMessage:
	db.ensure_user_in_everyone(int(user["id"]))
	channels = db.list_channels_for_user(int(user["id"]))
	everyone = next(
		(c for c in channels if c.get("kind") == "group" and c.get("name") == "Everyone"),
		None,
	)
	if not everyone:
		raise HTTPException(status_code=500, detail="everyone channel missing")
	return await chat_channel_post(int(everyone["id"]), body, user)


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
		"X-Lumen-Model": normalize_stored_model(meta.get("model")),
		"X-Lumen-Name": str(meta.get("name", "")),
	}
	return Response(content=png_path.read_bytes(), media_type="image/png", headers=headers)


@app.api_route("/skins/{uuid}", methods=["PUT", "POST"])
async def put_skin_by_uuid(
	uuid: str,
	request: Request,
	authorization: Annotated[Optional[str], Header()] = None,
	x_octra_key: Annotated[Optional[str], Header()] = None,
	x_lumen_model: Annotated[Optional[str], Header()] = "default",
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

	model = normalize_stored_model(x_lumen_model)
	write_skin(normalized, body, model, skin_name)
	return JSONResponse({"uuid": normalized, "ok": True})


@app.api_route("/skins/MinecraftSkins/{nick}.png", methods=["GET", "HEAD"])
async def get_skin_by_nick(nick: str, request: Request) -> Response:
	nick = nick.removesuffix(".png")
	if not re.fullmatch(r"^[a-zA-Z0-9_]{1,16}$", nick):
		raise HTTPException(status_code=400, detail="bad nick")
	resolved = resolve_by_nick(nick)
	if not resolved:
		raise HTTPException(status_code=404, detail="not found")
	_, png, meta = resolved
	headers = {
		"X-Lumen-Model": normalize_stored_model(meta.get("model")),
		"X-Lumen-Name": str(meta.get("name", nick)),
		"Content-Length": str(len(png)),
	}
	# MineSkin probes with HEAD before downloading — must not 405.
	if request.method == "HEAD":
		return Response(status_code=200, media_type="image/png", headers=headers)
	return Response(content=png, media_type="image/png", headers=headers)


# ---------------------------------------------------------------------------
# authlib-injector / Yggdrasil API (shared remote root for Octra launcher)
# Paths mirror packages/app-lib/src/octra_skins.rs local dispatch_ygg.
# ---------------------------------------------------------------------------


def public_base(request: Optional[Request] = None) -> str:
	env = os.environ.get("PUBLIC_BASE_URL", "").strip().rstrip("/")
	if env:
		return env
	if request is not None:
		host = (request.headers.get("host") or "").strip()
		if host and not host.startswith("127.") and "localhost" not in host.lower():
			fwd = (request.headers.get("x-forwarded-proto") or "").strip()
			scheme = fwd or request.url.scheme or "http"
			return f"{scheme}://{host}".rstrip("/")
	return DEFAULT_PUBLIC_BASE


def public_host(base: str) -> str:
	without = base
	for prefix in ("https://", "http://"):
		if without.startswith(prefix):
			without = without[len(prefix) :]
			break
	return without.split("/")[0].split(":")[0]


def plain_uuid(uuid_hyphen: str) -> str:
	return uuid_hyphen.replace("-", "").lower()


def legacy_skin_url(base: str, name: str) -> str:
	return f"{base.rstrip('/')}/skins/MinecraftSkins/{name}.png"


def resolve_player_from_registry(key: str) -> Optional[dict[str, Any]]:
	"""Look up a registered Octra skin by UUID or nick.

	Returns dict with uuid (hyphenated), name, model — or None.
	"""
	raw = (key or "").strip()
	if not raw:
		return None

	normalized = norm_uuid(raw)
	if normalized:
		png_path = DATA_DIR / "by-uuid" / f"{normalized}.png"
		if png_path.is_file():
			meta = read_meta(normalized)
			name = str(meta.get("name") or "").strip()
			if not name:
				user = db.get_user_by_profile_uuid(normalized)
				if user:
					name = user["minecraft_nick"]
			if not name:
				name = "Player"
			return {
				"uuid": normalized,
				"name": name,
				"model": normalize_stored_model(meta.get("model")),
			}
		user = db.get_user_by_profile_uuid(normalized)
		if user:
			nick = user["minecraft_nick"]
			resolved = resolve_by_nick(nick)
			if resolved:
				uuid_r, _png, meta = resolved
				return {
					"uuid": uuid_r if "-" in uuid_r else (norm_uuid(uuid_r) or normalized),
					"name": str(meta.get("name") or nick),
					"model": normalize_stored_model(meta.get("model")),
				}
			return {
				"uuid": normalized,
				"name": nick,
				"model": "default",
			}

	if re.fullmatch(r"^[a-zA-Z0-9_]{1,16}$", raw):
		resolved = resolve_by_nick(raw)
		if resolved:
			uuid_r, _png, meta = resolved
			uuid_h = uuid_r if "-" in str(uuid_r) else (norm_uuid(str(uuid_r)) or str(uuid_r))
			return {
				"uuid": uuid_h,
				"name": str(meta.get("name") or raw),
				"model": normalize_stored_model(meta.get("model")),
			}
		user = db.get_user_by_minecraft_nick(raw)
		if user:
			return {
				"uuid": user["profile_uuid"],
				"name": user["minecraft_nick"],
				"model": "default",
			}

	return None


def is_offline_uuid(uuid_key: str) -> bool:
	"""UUID v3 = offline-player namespace; v4 = Mojang premium."""
	compact = (norm_uuid(uuid_key) or uuid_key).replace("-", "").lower()
	return len(compact) == 32 and compact[12] == "3"


def registry_png_exists(player: dict[str, Any]) -> bool:
	uuid_h = player["uuid"]
	name = str(player.get("name") or "")
	if (DATA_DIR / "by-uuid" / f"{uuid_h}.png").is_file():
		return True
	if name and (DATA_DIR / "by-name" / f"{name.lower()}.png").is_file():
		return True
	return False


def profile_json(player: dict[str, Any], base: str) -> dict[str, Any]:
	"""Unsigned Octra registry profile (offline / custom skins).

	Texture host must be listed in `skinDomains`. Model is Yggdrasil
	`default` (Steve) or `slim` (Alex) — never `classic`.
	"""
	uuid_h = player["uuid"]
	name = player["name"]
	model = yggdrasil_texture_model(player.get("model"))
	skin_obj: dict[str, Any] = {
		"url": legacy_skin_url(base, name),
		"metadata": {"model": model},
	}
	textures = {
		"timestamp": int(time.time() * 1000),
		"profileId": plain_uuid(uuid_h),
		"profileName": name,
		"textures": {"SKIN": skin_obj},
	}
	value = base64.b64encode(json.dumps(textures, separators=(",", ":")).encode()).decode()
	return {
		"id": plain_uuid(uuid_h),
		"name": name,
		"properties": [{"name": "textures", "value": value}],
	}


def empty_profile(uuid_key: str, name: str = "Player") -> dict[str, Any]:
	normalized = norm_uuid(uuid_key) or norm_uuid(offline_player_uuid(uuid_key))
	if not normalized:
		normalized = offline_player_uuid(name if name != "Player" else uuid_key)
		normalized = norm_uuid(normalized) or normalized
	return {
		"id": plain_uuid(normalized),
		"name": name,
		"properties": [],
	}


def fetch_mojang_profile(uuid_plain: str) -> Optional[dict[str, Any]]:
	"""Signed Mojang session profile (keep Mojang signature for authlib)."""
	uid = uuid_plain.replace("-", "").lower()
	if len(uid) != 32:
		return None
	url = f"https://sessionserver.mojang.com/session/minecraft/profile/{uid}?unsigned=false"
	req = urllib.request.Request(url, headers={"User-Agent": "Octra-API/1.8"})
	try:
		with urllib.request.urlopen(req, timeout=6) as resp:
			if resp.status != 200:
				return None
			data = json.loads(resp.read().decode())
			if isinstance(data, dict) and "id" in data:
				return data
	except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError, json.JSONDecodeError, OSError):
		return None
	return None


def fetch_mojang_uuid_for_name(name: str) -> Optional[str]:
	if not re.fullmatch(r"^[a-zA-Z0-9_]{1,16}$", name):
		return None
	url = f"https://api.mojang.com/users/profiles/minecraft/{name}"
	req = urllib.request.Request(url, headers={"User-Agent": "Octra-API/1.8"})
	try:
		with urllib.request.urlopen(req, timeout=6) as resp:
			if resp.status != 200:
				return None
			data = json.loads(resp.read().decode())
			uid = data.get("id") if isinstance(data, dict) else None
			return uid if isinstance(uid, str) else None
	except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError, json.JSONDecodeError, OSError):
		return None


def resolve_authlib_profile(key: str, base: str) -> Optional[dict[str, Any]]:
	"""Single resolution path for authlib-injector clients.

	1. Premium UUID → Mojang signed profile only (never Octra PNG shadow).
	2. Offline / registry nick with PNG → Octra profile_json.
	3. Else Mojang by name / UUID fallthrough.
	"""
	plain = key.replace("-", "")
	normalized = norm_uuid(key)

	# Premium UUID: Mojang wins. Never serve local registry over it.
	if normalized and not is_offline_uuid(normalized):
		mojang = fetch_mojang_profile(plain_uuid(normalized))
		if mojang:
			return mojang

	player = resolve_player_from_registry(key)
	if player:
		if not is_offline_uuid(player["uuid"]):
			mojang = fetch_mojang_profile(plain_uuid(player["uuid"]))
			if mojang:
				return mojang
		if registry_png_exists(player):
			return profile_json(player, base)

	if normalized:
		mojang = fetch_mojang_profile(plain_uuid(normalized))
		if mojang:
			return mojang

	if re.fullmatch(r"^[a-zA-Z0-9_]{1,16}$", (key or "").strip()):
		mojang_id = fetch_mojang_uuid_for_name(key.strip())
		if mojang_id:
			mojang = fetch_mojang_profile(mojang_id)
			if mojang:
				return mojang

	if player:
		return empty_profile(player["uuid"], player["name"])
	return None


def ygg_index(base: str) -> dict[str, Any]:
	host = public_host(base)
	domains = [host, "127.0.0.1", "localhost", "textures.minecraft.net", ".minecraft.net"]
	seen: set[str] = set()
	skin_domains: list[str] = []
	for d in domains:
		if d and d not in seen:
			seen.add(d)
			skin_domains.append(d)
	return {
		"meta": {
			"serverName": "Octra",
			"implementationName": "octra-yggdrasil",
			"implementationVersion": app.version,
			"feature.non_email_login": True,
		},
		"skinDomains": skin_domains,
	}


@app.get("/")
@app.get("/index.json")
async def ygg_root(request: Request) -> JSONResponse:
	base = public_base(request)
	return JSONResponse(ygg_index(base))


@app.get("/sessionserver/session/minecraft/profile/{profile_id}")
async def ygg_profile(profile_id: str, request: Request) -> Response:
	base = public_base(request)
	resolved = resolve_authlib_profile(profile_id, base)
	if resolved:
		return JSONResponse(resolved)
	return JSONResponse(empty_profile(profile_id))


@app.get("/sessionserver/session/minecraft/hasJoined")
async def ygg_has_joined(
	request: Request,
	username: str = "",
	serverId: str = "",  # noqa: N803 — Mojang query param name
) -> Response:
	_ = serverId
	base = public_base(request)
	resolved = resolve_authlib_profile(username, base)
	if resolved:
		return JSONResponse(resolved)
	return Response(status_code=204)


@app.post("/sessionserver/session/minecraft/join")
async def ygg_join() -> Response:
	# Offline / custom Ygg: join is a no-op stub (same as local hub).
	return Response(status_code=204)


@app.post("/api/profiles/minecraft")
async def ygg_profiles_minecraft(request: Request) -> JSONResponse:
	try:
		names = await request.json()
	except Exception:
		names = []
	if not isinstance(names, list):
		names = []
	out: list[dict[str, str]] = []
	for raw in names:
		if not isinstance(raw, str):
			continue
		player = resolve_player_from_registry(raw)
		if player:
			out.append({"id": plain_uuid(player["uuid"]), "name": player["name"]})
			continue
		mojang_id = fetch_mojang_uuid_for_name(raw)
		if mojang_id:
			out.append({"id": mojang_id.replace("-", "").lower(), "name": raw})
	return JSONResponse(out)


@app.post("/authserver/authenticate")
@app.post("/authserver/refresh")
@app.post("/authserver/validate")
async def ygg_auth_stub(request: Request) -> JSONResponse:
	name = "Player"
	try:
		body = await request.json()
		if isinstance(body, dict):
			username = body.get("username")
			if isinstance(username, str) and username.strip():
				name = username.strip()
	except Exception:
		pass
	player = resolve_player_from_registry(name)
	if player:
		pid = plain_uuid(player["uuid"])
		pname = player["name"]
	else:
		pid = plain_uuid(offline_player_uuid(name))
		pname = name
	return JSONResponse(
		{
			"accessToken": "0",
			"clientToken": "octra",
			"selectedProfile": {"id": pid, "name": pname},
			"availableProfiles": [{"id": pid, "name": pname}],
		}
	)


@app.get("/textures/{texture_hash}")
async def ygg_texture_by_hash(texture_hash: str) -> Response:
	"""Optional hash-addressed textures (sha256 of PNG), if ever embedded that way."""
	h = texture_hash.strip().lower()
	if not re.fullmatch(r"^[a-f0-9]{64}$", h):
		raise HTTPException(status_code=400, detail="bad hash")
	# Scan by-uuid for matching hash (small private registry).
	by_uuid = DATA_DIR / "by-uuid"
	if by_uuid.is_dir():
		for path in by_uuid.glob("*.png"):
			try:
				data = path.read_bytes()
			except OSError:
				continue
			if hashlib.sha256(data).hexdigest() == h:
				return Response(content=data, media_type="image/png")
	raise HTTPException(status_code=404, detail="not found")
