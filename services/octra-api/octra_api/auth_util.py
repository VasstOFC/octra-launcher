from __future__ import annotations

import hashlib
import os
import re
import uuid
from datetime import datetime, timedelta, timezone
from typing import Any, Optional

import bcrypt
import jwt

USERNAME_RE = re.compile(r"^[A-Za-z0-9_]{3,24}$")
NICK_RE = re.compile(r"^[a-zA-Z0-9_]{1,16}$")


def offline_player_uuid(name: str) -> str:
	digest = hashlib.md5(f"OfflinePlayer:{name}".encode()).digest()
	b = bytearray(digest)
	b[6] = (b[6] & 0x0F) | 0x30
	b[8] = (b[8] & 0x3F) | 0x80
	return str(uuid.UUID(bytes=bytes(b)))


def norm_uuid(raw: str) -> Optional[str]:
	s = raw.strip().replace("-", "").lower()
	if len(s) != 32:
		return None
	return f"{s[0:8]}-{s[8:12]}-{s[12:16]}-{s[16:20]}-{s[20:32]}"


def hash_password(password: str) -> str:
	return bcrypt.hashpw(password.encode(), bcrypt.gensalt(rounds=12)).decode()


def verify_password(password: str, password_hash: str) -> bool:
	return bcrypt.checkpw(password.encode(), password_hash.encode())


def validate_username(username: str) -> str:
	username = username.strip()
	if not USERNAME_RE.fullmatch(username):
		raise ValueError("nazwa użytkownika: 3–24 znaki, litery, cyfry, podkreślenie")
	return username


def validate_password(password: str) -> None:
	if len(password) < 8:
		raise ValueError("hasło musi mieć co najmniej 8 znaków")


def validate_minecraft_nick(nick: str) -> str:
	nick = nick.strip()
	if not NICK_RE.fullmatch(nick):
		raise ValueError("nick minecraft: 1–16 znaków, litery, cyfry, podkreślenie")
	return nick


def create_access_token(user_id: int, secret: str, days: int = 30) -> str:
	now = datetime.now(timezone.utc)
	payload = {
		"sub": str(user_id),
		"iat": now,
		"exp": now + timedelta(days=days),
	}
	return jwt.encode(payload, secret, algorithm="HS256")


def decode_access_token(token: str, secret: str) -> dict[str, Any]:
	return jwt.decode(token, secret, algorithms=["HS256"])


def jwt_secret() -> str:
	secret = os.environ.get("JWT_SECRET", "").strip()
	if not secret:
		raise RuntimeError("JWT_SECRET is not configured")
	return secret
