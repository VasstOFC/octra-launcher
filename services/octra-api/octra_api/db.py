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
	created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_users_minecraft_nick ON users(minecraft_nick);
"""


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

	def create_user(
		self,
		username: str,
		password_hash: str,
		minecraft_nick: str,
		profile_uuid: str,
	) -> dict[str, Any]:
		with self._connect() as conn:
			cur = conn.execute(
				"""
				INSERT INTO users (username, password_hash, minecraft_nick, profile_uuid)
				VALUES (?, ?, ?, ?)
				""",
				(username, password_hash, minecraft_nick, profile_uuid),
			)
			user_id = cur.lastrowid
			row = conn.execute(
				"SELECT id, username, minecraft_nick, profile_uuid, created_at FROM users WHERE id = ?",
				(user_id,),
			).fetchone()
			return dict(row)

	def get_user_by_username(self, username: str) -> Optional[dict[str, Any]]:
		with self._connect() as conn:
			row = conn.execute(
				"SELECT * FROM users WHERE username = ? COLLATE NOCASE",
				(username,),
			).fetchone()
			return dict(row) if row else None

	def get_user_by_id(self, user_id: int) -> Optional[dict[str, Any]]:
		with self._connect() as conn:
			row = conn.execute(
				"SELECT id, username, minecraft_nick, profile_uuid, created_at FROM users WHERE id = ?",
				(user_id,),
			).fetchone()
			return dict(row) if row else None

	def get_user_by_minecraft_nick(self, nick: str) -> Optional[dict[str, Any]]:
		with self._connect() as conn:
			row = conn.execute(
				"SELECT id, username, minecraft_nick, profile_uuid, created_at FROM users WHERE minecraft_nick = ? COLLATE NOCASE",
				(nick,),
			).fetchone()
			return dict(row) if row else None
