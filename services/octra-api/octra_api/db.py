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
	presence_join_address TEXT,
	last_seen TEXT
);

CREATE TABLE IF NOT EXISTS chat_channels (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	kind TEXT NOT NULL,
	name TEXT,
	dm_key TEXT UNIQUE,
	created_by INTEGER,
	created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS chat_channel_members (
	channel_id INTEGER NOT NULL,
	user_id INTEGER NOT NULL,
	PRIMARY KEY (channel_id, user_id)
);

CREATE TABLE IF NOT EXISTS chat_messages (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	channel_id INTEGER NOT NULL DEFAULT 0,
	user_id INTEGER NOT NULL,
	minecraft_nick TEXT NOT NULL,
	body TEXT NOT NULL,
	created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_users_minecraft_nick ON users(minecraft_nick);
CREATE INDEX IF NOT EXISTS idx_chat_channel_members_user ON chat_channel_members(user_id);
"""

USER_PUBLIC_SELECT = (
	"SELECT id, username, minecraft_nick, profile_uuid, account_type, created_at, "
	"presence_status, presence_instance, presence_join_address, last_seen FROM users"
)
USER_AUTH_SELECT = (
	"SELECT id, username, password_hash, minecraft_nick, profile_uuid, account_type, created_at, "
	"presence_status, presence_instance, presence_join_address, last_seen FROM users"
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
			if "presence_join_address" not in cols:
				conn.execute("ALTER TABLE users ADD COLUMN presence_join_address TEXT")
			if "last_seen" not in cols:
				conn.execute("ALTER TABLE users ADD COLUMN last_seen TEXT")
			conn.executescript(
				"""
				CREATE TABLE IF NOT EXISTS chat_channels (
					id INTEGER PRIMARY KEY AUTOINCREMENT,
					kind TEXT NOT NULL,
					name TEXT,
					dm_key TEXT UNIQUE,
					created_by INTEGER,
					created_at TEXT NOT NULL
				);
				CREATE TABLE IF NOT EXISTS chat_channel_members (
					channel_id INTEGER NOT NULL,
					user_id INTEGER NOT NULL,
					PRIMARY KEY (channel_id, user_id)
				);
				CREATE TABLE IF NOT EXISTS chat_messages (
					id INTEGER PRIMARY KEY AUTOINCREMENT,
					channel_id INTEGER NOT NULL DEFAULT 0,
					user_id INTEGER NOT NULL,
					minecraft_nick TEXT NOT NULL,
					body TEXT NOT NULL,
					created_at TEXT NOT NULL
				);
				CREATE INDEX IF NOT EXISTS idx_chat_channel_members_user ON chat_channel_members(user_id);
				"""
			)
			msg_cols = {
				row["name"] if isinstance(row, sqlite3.Row) else row[1]
				for row in conn.execute("PRAGMA table_info(chat_messages)").fetchall()
			}
			if "channel_id" not in msg_cols:
				conn.execute(
					"ALTER TABLE chat_messages ADD COLUMN channel_id INTEGER NOT NULL DEFAULT 0"
				)
			conn.execute(
				"CREATE INDEX IF NOT EXISTS idx_chat_messages_channel ON chat_messages(channel_id, id)"
			)
			msg_cols = {
				row["name"] if isinstance(row, sqlite3.Row) else row[1]
				for row in conn.execute("PRAGMA table_info(chat_messages)").fetchall()
			}
			if "pinned" not in msg_cols:
				conn.execute(
					"ALTER TABLE chat_messages ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0"
				)
			if "deleted_at" not in msg_cols:
				conn.execute("ALTER TABLE chat_messages ADD COLUMN deleted_at TEXT")
			conn.executescript(
				"""
				CREATE TABLE IF NOT EXISTS chat_reactions (
					message_id INTEGER NOT NULL,
					user_id INTEGER NOT NULL,
					emoji TEXT NOT NULL,
					created_at TEXT NOT NULL,
					PRIMARY KEY (message_id, user_id, emoji)
				);
				CREATE TABLE IF NOT EXISTS chat_channel_reads (
					channel_id INTEGER NOT NULL,
					user_id INTEGER NOT NULL,
					last_read_id INTEGER NOT NULL DEFAULT 0,
					PRIMARY KEY (channel_id, user_id)
				);
				CREATE TABLE IF NOT EXISTS shared_servers (
					id INTEGER PRIMARY KEY AUTOINCREMENT,
					name TEXT NOT NULL,
					address TEXT NOT NULL,
					created_by INTEGER NOT NULL,
					created_at TEXT NOT NULL,
					UNIQUE(address)
				);
				CREATE INDEX IF NOT EXISTS idx_chat_reactions_message ON chat_reactions(message_id);
				"""
			)
			self._ensure_everyone_channel(conn)

	def _ensure_everyone_channel(self, conn: sqlite3.Connection) -> None:
		row = conn.execute(
			"SELECT id FROM chat_channels WHERE kind = 'group' AND name = 'Everyone' LIMIT 1"
		).fetchone()
		if row:
			channel_id = int(row["id"])
		else:
			cur = conn.execute(
				"""
				INSERT INTO chat_channels (kind, name, dm_key, created_by, created_at)
				VALUES ('group', 'Everyone', NULL, NULL, datetime('now'))
				"""
			)
			channel_id = int(cur.lastrowid)
		user_ids = [
			int(r["id"]) for r in conn.execute("SELECT id FROM users").fetchall()
		]
		for uid in user_ids:
			conn.execute(
				"""
				INSERT OR IGNORE INTO chat_channel_members (channel_id, user_id)
				VALUES (?, ?)
				""",
				(channel_id, uid),
			)
		conn.execute(
			"""
			UPDATE chat_messages SET channel_id = ?
			WHERE channel_id = 0 OR channel_id IS NULL
			""",
			(channel_id,),
		)

	def ensure_user_in_everyone(self, user_id: int) -> None:
		with self._connect() as conn:
			self._ensure_everyone_channel(conn)
			row = conn.execute(
				"SELECT id FROM chat_channels WHERE kind = 'group' AND name = 'Everyone' LIMIT 1"
			).fetchone()
			if not row:
				return
			conn.execute(
				"""
				INSERT OR IGNORE INTO chat_channel_members (channel_id, user_id)
				VALUES (?, ?)
				""",
				(int(row["id"]), user_id),
			)

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
		join_address: Optional[str],
		last_seen: str,
	) -> None:
		with self._connect() as conn:
			conn.execute(
				"""
				UPDATE users
				SET presence_status = ?, presence_instance = ?, presence_join_address = ?, last_seen = ?
				WHERE id = ?
				""",
				(status, instance_name, join_address, last_seen, user_id),
			)

	def list_channels_for_user(self, user_id: int) -> list[dict[str, Any]]:
		with self._connect() as conn:
			self._ensure_everyone_channel(conn)
			rows = conn.execute(
				"""
				SELECT
					c.id,
					c.kind,
					c.name,
					c.created_at,
					(
						SELECT body FROM chat_messages m
						WHERE m.channel_id = c.id AND m.deleted_at IS NULL
						ORDER BY m.id DESC LIMIT 1
					) AS last_body,
					(
						SELECT created_at FROM chat_messages m
						WHERE m.channel_id = c.id AND m.deleted_at IS NULL
						ORDER BY m.id DESC LIMIT 1
					) AS last_at,
					(
						SELECT m.id FROM chat_messages m
						WHERE m.channel_id = c.id AND m.deleted_at IS NULL
						ORDER BY m.id DESC LIMIT 1
					) AS last_id,
					COALESCE(
						(
							SELECT last_read_id FROM chat_channel_reads r
							WHERE r.channel_id = c.id AND r.user_id = ?
						),
						0
					) AS last_read_id
				FROM chat_channels c
				INNER JOIN chat_channel_members cm ON cm.channel_id = c.id
				WHERE cm.user_id = ?
				ORDER BY COALESCE(last_at, c.created_at) DESC, c.id DESC
				""",
				(user_id, user_id),
			).fetchall()
			channels: list[dict[str, Any]] = []
			for row in rows:
				channel = dict(row)
				members = conn.execute(
					"""
					SELECT u.id, u.minecraft_nick, u.profile_uuid
					FROM chat_channel_members cm
					INNER JOIN users u ON u.id = cm.user_id
					WHERE cm.channel_id = ?
					ORDER BY u.minecraft_nick COLLATE NOCASE
					""",
					(int(channel["id"]),),
				).fetchall()
				channel["members"] = [dict(m) for m in members]
				channels.append(channel)
			return channels

	def get_channel(self, channel_id: int) -> Optional[dict[str, Any]]:
		with self._connect() as conn:
			row = conn.execute(
				"SELECT id, kind, name, dm_key, created_by, created_at FROM chat_channels WHERE id = ?",
				(channel_id,),
			).fetchone()
			return dict(row) if row else None

	def is_channel_member(self, channel_id: int, user_id: int) -> bool:
		with self._connect() as conn:
			row = conn.execute(
				"""
				SELECT 1 FROM chat_channel_members
				WHERE channel_id = ? AND user_id = ?
				""",
				(channel_id, user_id),
			).fetchone()
			return row is not None

	def get_or_create_dm(self, user_a: int, user_b: int, created_at: str) -> dict[str, Any]:
		if user_a == user_b:
			raise ValueError("cannot dm yourself")
		lo, hi = (user_a, user_b) if user_a < user_b else (user_b, user_a)
		dm_key = f"{lo}:{hi}"
		with self._connect() as conn:
			row = conn.execute(
				"SELECT id, kind, name, dm_key, created_by, created_at FROM chat_channels WHERE dm_key = ?",
				(dm_key,),
			).fetchone()
			if row:
				return dict(row)
			cur = conn.execute(
				"""
				INSERT INTO chat_channels (kind, name, dm_key, created_by, created_at)
				VALUES ('dm', NULL, ?, ?, ?)
				""",
				(dm_key, user_a, created_at),
			)
			channel_id = int(cur.lastrowid)
			for uid in (lo, hi):
				conn.execute(
					"""
					INSERT OR IGNORE INTO chat_channel_members (channel_id, user_id)
					VALUES (?, ?)
					""",
					(channel_id, uid),
				)
			row = conn.execute(
				"SELECT id, kind, name, dm_key, created_by, created_at FROM chat_channels WHERE id = ?",
				(channel_id,),
			).fetchone()
			return dict(row)

	def create_group(
		self,
		name: str,
		creator_id: int,
		member_ids: list[int],
		created_at: str,
	) -> dict[str, Any]:
		members = sorted({creator_id, *member_ids})
		with self._connect() as conn:
			cur = conn.execute(
				"""
				INSERT INTO chat_channels (kind, name, dm_key, created_by, created_at)
				VALUES ('group', ?, NULL, ?, ?)
				""",
				(name, creator_id, created_at),
			)
			channel_id = int(cur.lastrowid)
			for uid in members:
				conn.execute(
					"""
					INSERT OR IGNORE INTO chat_channel_members (channel_id, user_id)
					VALUES (?, ?)
					""",
					(channel_id, uid),
				)
			row = conn.execute(
				"SELECT id, kind, name, dm_key, created_by, created_at FROM chat_channels WHERE id = ?",
				(channel_id,),
			).fetchone()
			return dict(row)

	def channel_member_rows(self, channel_id: int) -> list[dict[str, Any]]:
		with self._connect() as conn:
			rows = conn.execute(
				"""
				SELECT u.id, u.minecraft_nick, u.profile_uuid
				FROM chat_channel_members cm
				INNER JOIN users u ON u.id = cm.user_id
				WHERE cm.channel_id = ?
				ORDER BY u.minecraft_nick COLLATE NOCASE
				""",
				(channel_id,),
			).fetchall()
			return [dict(r) for r in rows]

	def list_chat_messages(
		self,
		channel_id: int,
		after_id: int = 0,
		limit: int = 80,
	) -> list[dict[str, Any]]:
		limit = max(1, min(limit, 200))
		with self._connect() as conn:
			if after_id > 0:
				rows = conn.execute(
					"""
					SELECT id, channel_id, user_id, minecraft_nick, body, created_at,
						COALESCE(pinned, 0) AS pinned, deleted_at
					FROM chat_messages
					WHERE channel_id = ? AND id > ?
					ORDER BY id ASC
					LIMIT ?
					""",
					(channel_id, after_id, limit),
				).fetchall()
			else:
				rows = conn.execute(
					"""
					SELECT id, channel_id, user_id, minecraft_nick, body, created_at,
						pinned, deleted_at
					FROM (
						SELECT id, channel_id, user_id, minecraft_nick, body, created_at,
							COALESCE(pinned, 0) AS pinned, deleted_at
						FROM chat_messages
						WHERE channel_id = ?
						ORDER BY id DESC
						LIMIT ?
					)
					ORDER BY id ASC
					""",
					(channel_id, limit),
				).fetchall()
			messages: list[dict[str, Any]] = []
			for row in rows:
				message = dict(row)
				if message.get("deleted_at"):
					message["body"] = ""
				message["reactions"] = self._reactions_for_message(conn, int(message["id"]))
				messages.append(message)
			return messages

	def _reactions_for_message(
		self, conn: sqlite3.Connection, message_id: int
	) -> list[dict[str, Any]]:
		rows = conn.execute(
			"""
			SELECT emoji, COUNT(*) AS count,
				GROUP_CONCAT(user_id) AS user_ids
			FROM chat_reactions
			WHERE message_id = ?
			GROUP BY emoji
			ORDER BY emoji
			""",
			(message_id,),
		).fetchall()
		out: list[dict[str, Any]] = []
		for row in rows:
			ids_raw = row["user_ids"] or ""
			user_ids = [int(x) for x in str(ids_raw).split(",") if x]
			out.append(
				{
					"emoji": row["emoji"],
					"count": int(row["count"]),
					"user_ids": user_ids,
				}
			)
		return out

	def get_chat_message(self, message_id: int) -> Optional[dict[str, Any]]:
		with self._connect() as conn:
			row = conn.execute(
				"""
				SELECT id, channel_id, user_id, minecraft_nick, body, created_at,
					COALESCE(pinned, 0) AS pinned, deleted_at
				FROM chat_messages WHERE id = ?
				""",
				(message_id,),
			).fetchone()
			if not row:
				return None
			message = dict(row)
			message["reactions"] = self._reactions_for_message(conn, int(message["id"]))
			return message

	def soft_delete_message(self, message_id: int, deleted_at: str) -> Optional[dict[str, Any]]:
		with self._connect() as conn:
			conn.execute(
				"""
				UPDATE chat_messages
				SET deleted_at = ?, body = '', pinned = 0
				WHERE id = ? AND deleted_at IS NULL
				""",
				(deleted_at, message_id),
			)
			conn.execute("DELETE FROM chat_reactions WHERE message_id = ?", (message_id,))
		return self.get_chat_message(message_id)

	def set_message_pinned(self, message_id: int, pinned: bool) -> Optional[dict[str, Any]]:
		with self._connect() as conn:
			conn.execute(
				"""
				UPDATE chat_messages SET pinned = ?
				WHERE id = ? AND deleted_at IS NULL
				""",
				(1 if pinned else 0, message_id),
			)
		return self.get_chat_message(message_id)

	def toggle_reaction(
		self,
		message_id: int,
		user_id: int,
		emoji: str,
		created_at: str,
	) -> Optional[dict[str, Any]]:
		with self._connect() as conn:
			existing = conn.execute(
				"""
				SELECT 1 FROM chat_reactions
				WHERE message_id = ? AND user_id = ? AND emoji = ?
				""",
				(message_id, user_id, emoji),
			).fetchone()
			if existing:
				conn.execute(
					"""
					DELETE FROM chat_reactions
					WHERE message_id = ? AND user_id = ? AND emoji = ?
					""",
					(message_id, user_id, emoji),
				)
			else:
				conn.execute(
					"""
					INSERT INTO chat_reactions (message_id, user_id, emoji, created_at)
					VALUES (?, ?, ?, ?)
					""",
					(message_id, user_id, emoji, created_at),
				)
		return self.get_chat_message(message_id)

	def mark_channel_read(self, channel_id: int, user_id: int, last_read_id: int) -> None:
		last_read_id = max(0, int(last_read_id))
		with self._connect() as conn:
			row = conn.execute(
				"""
				SELECT last_read_id FROM chat_channel_reads
				WHERE channel_id = ? AND user_id = ?
				""",
				(channel_id, user_id),
			).fetchone()
			current = int(row["last_read_id"]) if row else 0
			next_id = max(current, last_read_id)
			conn.execute(
				"""
				INSERT INTO chat_channel_reads (channel_id, user_id, last_read_id)
				VALUES (?, ?, ?)
				ON CONFLICT(channel_id, user_id) DO UPDATE SET last_read_id = excluded.last_read_id
				""",
				(channel_id, user_id, next_id),
			)

	def add_group_members(self, channel_id: int, member_ids: list[int]) -> None:
		with self._connect() as conn:
			for uid in member_ids:
				conn.execute(
					"""
					INSERT OR IGNORE INTO chat_channel_members (channel_id, user_id)
					VALUES (?, ?)
					""",
					(channel_id, uid),
				)

	def list_shared_servers(self) -> list[dict[str, Any]]:
		with self._connect() as conn:
			rows = conn.execute(
				"""
				SELECT s.id, s.name, s.address, s.created_by, s.created_at,
					u.minecraft_nick AS created_by_nick
				FROM shared_servers s
				LEFT JOIN users u ON u.id = s.created_by
				ORDER BY s.name COLLATE NOCASE, s.id
				"""
			).fetchall()
			return [dict(row) for row in rows]

	def add_shared_server(
		self,
		name: str,
		address: str,
		created_by: int,
		created_at: str,
	) -> dict[str, Any]:
		with self._connect() as conn:
			cur = conn.execute(
				"""
				INSERT INTO shared_servers (name, address, created_by, created_at)
				VALUES (?, ?, ?, ?)
				""",
				(name, address, created_by, created_at),
			)
			row = conn.execute(
				"""
				SELECT s.id, s.name, s.address, s.created_by, s.created_at,
					u.minecraft_nick AS created_by_nick
				FROM shared_servers s
				LEFT JOIN users u ON u.id = s.created_by
				WHERE s.id = ?
				""",
				(cur.lastrowid,),
			).fetchone()
			return dict(row)

	def delete_shared_server(self, server_id: int) -> bool:
		with self._connect() as conn:
			cur = conn.execute("DELETE FROM shared_servers WHERE id = ?", (server_id,))
			return cur.rowcount > 0

	def last_chat_message_at(self, user_id: int, channel_id: int) -> Optional[str]:
		with self._connect() as conn:
			row = conn.execute(
				"""
				SELECT created_at FROM chat_messages
				WHERE user_id = ? AND channel_id = ? AND deleted_at IS NULL
				ORDER BY id DESC
				LIMIT 1
				""",
				(user_id, channel_id),
			).fetchone()
			return str(row["created_at"]) if row else None

	def add_chat_message(
		self,
		channel_id: int,
		user_id: int,
		minecraft_nick: str,
		body: str,
		created_at: str,
	) -> dict[str, Any]:
		with self._connect() as conn:
			cur = conn.execute(
				"""
				INSERT INTO chat_messages (channel_id, user_id, minecraft_nick, body, created_at)
				VALUES (?, ?, ?, ?, ?)
				""",
				(channel_id, user_id, minecraft_nick, body, created_at),
			)
			row = conn.execute(
				"""
				SELECT id, channel_id, user_id, minecraft_nick, body, created_at,
					COALESCE(pinned, 0) AS pinned, deleted_at
				FROM chat_messages WHERE id = ?
				""",
				(cur.lastrowid,),
			).fetchone()
			message = dict(row)
			message["reactions"] = []
			return message

