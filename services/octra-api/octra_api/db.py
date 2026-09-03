from __future__ import annotations

import sqlite3
from pathlib import Path
from typing import Any, Optional

SCHEMA = """
CREATE TABLE IF NOT EXISTS users (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	username TEXT NOT NULL COLLATE NOCASE UNIQUE,
	password_hash TEXT NOT NULL,
	minecraft_nick TEXT NOT NULL COLLATE NOCASE UNIQUE,
	profile_uuid TEXT NOT NULL UNIQUE,
	account_type TEXT NOT NULL DEFAULT 'offline',
	created_at TEXT NOT NULL DEFAULT (datetime('now')),
	presence_status TEXT NOT NULL DEFAULT 'offline',
	presence_instance TEXT,
	last_seen TEXT
);

CREATE INDEX IF NOT EXISTS idx_users_minecraft_nick ON users(minecraft_nick);
"""

USER_PUBLIC_SELECT = (
	"SELECT id, username, minecraft_nick, profile_uuid, account_type, created_at, "
	"presence_status, presence_instance, last_seen FROM users"
)
USER_AUTH_SELECT = (
	"SELECT id, username, password_hash, minecraft_nick, profile_uuid, account_type, created_at, "
	"presence_status, presence_instance, last_seen FROM users"
)


class Database:
	def __init__(self, path: Path) -> None:
		self.path = path
		self.path.parent.mkdir(parents=True, exist_ok=True)
		self._init()

	def _connect(self) -> sqlite3.Connection:
		conn = sqlite3.connect(self.path, check_same_thread=False)
		conn.row_factory = sqlite3.Row
		return conn

	def _init(self) -> None:
		with self._connect() as conn:
			conn.executescript(SCHEMA)
			cols = {
				row["name"] if isinstance(row, sqlite3.Row) else row[1]
				for row in conn.execute("PRAGMA table_info(users)").fetchall()
			}
			if "account_type" not in cols:
				conn.execute(
					"ALTER TABLE users ADD COLUMN account_type TEXT NOT NULL DEFAULT 'offline'"
				)
			if "presence_status" not in cols:
				conn.execute(
					"ALTER TABLE users ADD COLUMN presence_status TEXT NOT NULL DEFAULT 'offline'"
				)
			if "presence_instance" not in cols:
				conn.execute("ALTER TABLE users ADD COLUMN presence_instance TEXT")
			if "last_seen" not in cols:
				conn.execute("ALTER TABLE users ADD COLUMN last_seen TEXT")

	def create_user(
		self,
		username: str,
		password_hash: str,
		minecraft_nick: str,
		profile_uuid: str,
		account_type: str = "offline",
	) -> dict[str, Any]:
		with self._connect() as conn:
			cur = conn.execute(
				"""
				INSERT INTO users (username, password_hash, minecraft_nick, profile_uuid, account_type)
				VALUES (?, ?, ?, ?, ?)
				""",
				(username, password_hash, minecraft_nick, profile_uuid, account_type),
			)
			user_id = cur.lastrowid
			row = conn.execute(
				f"{USER_PUBLIC_SELECT} WHERE id = ?",
				(user_id,),
			).fetchone()
			return dict(row)

	def get_user_by_username(self, username: str) -> Optional[dict[str, Any]]:
		with self._connect() as conn:
			row = conn.execute(
				f"{USER_AUTH_SELECT} WHERE username = ? COLLATE NOCASE",
				(username,),
			).fetchone()
			return dict(row) if row else None

	def get_user_by_id(self, user_id: int) -> Optional[dict[str, Any]]:
		with self._connect() as conn:
			row = conn.execute(
				f"{USER_PUBLIC_SELECT} WHERE id = ?",
				(user_id,),
			).fetchone()
			return dict(row) if row else None

	def get_user_by_minecraft_nick(self, nick: str) -> Optional[dict[str, Any]]:
		with self._connect() as conn:
			row = conn.execute(
				f"{USER_AUTH_SELECT} WHERE minecraft_nick = ? COLLATE NOCASE",
				(nick,),
			).fetchone()
			return dict(row) if row else None

	def get_user_by_profile_uuid(self, profile_uuid: str) -> Optional[dict[str, Any]]:
		with self._connect() as conn:
			row = conn.execute(
				f"{USER_PUBLIC_SELECT} WHERE lower(profile_uuid) = lower(?)",
				(profile_uuid,),
			).fetchone()
			return dict(row) if row else None

	def list_users_except(self, user_id: int) -> list[dict[str, Any]]:
		with self._connect() as conn:
			rows = conn.execute(
				f"{USER_PUBLIC_SELECT} WHERE id != ? ORDER BY minecraft_nick COLLATE NOCASE",
				(user_id,),
			).fetchall()
			return [dict(row) for row in rows]

	def set_presence(
		self,
		user_id: int,
		status: str,
		instance_name: Optional[str],
		last_seen: str,
	) -> None:
		with self._connect() as conn:
			conn.execute(
				"""
				UPDATE users
				SET presence_status = ?, presence_instance = ?, last_seen = ?
				WHERE id = ?
				""",
				(status, instance_name, last_seen, user_id),
			)
