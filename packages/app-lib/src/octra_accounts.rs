//! Octra cloud account (registration, login, JWT for skin uploads).

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::nervia;
use crate::state::State;
use crate::util::fetch::INSECURE_REQWEST_CLIENT;

const TOKEN_KEY: &str = "octra_account_token";
const USERNAME_KEY: &str = "octra_account_username";
const MINECRAFT_NICK_KEY: &str = "octra_account_minecraft_nick";
const PROFILE_UUID_KEY: &str = "octra_account_profile_uuid";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OctraAccountSession {
	pub token: String,
	pub username: String,
	pub minecraft_nick: String,
	pub profile_uuid: String,
}

#[derive(Deserialize)]
struct AuthResponse {
	token: String,
	username: String,
	minecraft_nick: String,
	profile_uuid: String,
}

pub async fn session() -> crate::Result<Option<OctraAccountSession>> {
	let state = State::get().await?;
	let token = get_metadata(&state, TOKEN_KEY).await?;
	let username = get_metadata(&state, USERNAME_KEY).await?;
	let minecraft_nick = get_metadata(&state, MINECRAFT_NICK_KEY).await?;
	let profile_uuid = get_metadata(&state, PROFILE_UUID_KEY).await?;
	match (token, username, minecraft_nick, profile_uuid) {
		(Some(token), Some(username), Some(minecraft_nick), Some(profile_uuid))
			if !token.is_empty() =>
		{
			Ok(Some(OctraAccountSession {
				token,
				username,
				minecraft_nick,
				profile_uuid,
			}))
		}
		_ => Ok(None),
	}
}

pub async fn register(
	username: &str,
	password: &str,
	minecraft_nick: &str,
) -> crate::Result<OctraAccountSession> {
	let body = serde_json::json!({
		"username": username,
		"password": password,
		"minecraft_nick": minecraft_nick,
	});
	let response = post_auth("/api/v1/auth/register", &body).await?;
	save_session(&response).await?;
	ensure_offline_profile(&response.minecraft_nick).await?;
	Ok(response)
}

pub async fn login(username: &str, password: &str) -> crate::Result<OctraAccountSession> {
	let body = serde_json::json!({
		"username": username,
		"password": password,
	});
	let response = post_auth("/api/v1/auth/login", &body).await?;
	save_session(&response).await?;
	ensure_offline_profile(&response.minecraft_nick).await?;
	Ok(response)
}

pub async fn logout() -> crate::Result<()> {
	let state = State::get().await?;
	for key in [TOKEN_KEY, USERNAME_KEY, MINECRAFT_NICK_KEY, PROFILE_UUID_KEY] {
		sqlx::query!("DELETE FROM app_metadata WHERE key = ?", key)
			.execute(&state.pool)
			.await?;
	}
	Ok(())
}

pub async fn bearer_token() -> Option<String> {
	session().await.ok().flatten().map(|s| s.token)
}

async fn post_auth(path: &str, body: &serde_json::Value) -> crate::Result<OctraAccountSession> {
	let url = format!("{}{}", nervia::SKINS_URL.trim_end_matches('/'), path);
	let response = INSECURE_REQWEST_CLIENT
		.post(&url)
		.json(body)
		.timeout(Duration::from_secs(15))
		.send()
		.await
		.map_err(|e| {
			crate::ErrorKind::OtherError(format!("octra account request failed: {e}"))
		})?;

	let status = response.status();
	let text = response.text().await.unwrap_or_default();
	if !status.is_success() {
		let detail = serde_json::from_str::<serde_json::Value>(&text)
			.ok()
			.and_then(|v| v.get("detail").and_then(|d| d.as_str()).map(ToOwned::to_owned))
			.unwrap_or(text);
		return Err(crate::ErrorKind::OtherError(detail).into());
	}

	let parsed: AuthResponse = serde_json::from_str(&text).map_err(|e| {
		crate::ErrorKind::OtherError(format!("octra account response parse failed: {e}"))
	})?;
	Ok(OctraAccountSession {
		token: parsed.token,
		username: parsed.username,
		minecraft_nick: parsed.minecraft_nick,
		profile_uuid: parsed.profile_uuid,
	})
}

async fn save_session(session: &OctraAccountSession) -> crate::Result<()> {
	let state = State::get().await?;
	set_metadata(&state, TOKEN_KEY, &session.token).await?;
	set_metadata(&state, USERNAME_KEY, &session.username).await?;
	set_metadata(&state, MINECRAFT_NICK_KEY, &session.minecraft_nick).await?;
	set_metadata(&state, PROFILE_UUID_KEY, &session.profile_uuid).await?;
	Ok(())
}

async fn ensure_offline_profile(minecraft_nick: &str) -> crate::Result<()> {
	let uuid = crate::octra_skins::offline_player_uuid(minecraft_nick);
	let state = State::get().await?;
	let users = Credentials::get_all(&state.pool).await?;
	if !users.contains_key(&uuid) {
		crate::api::minecraft_auth::login_offline(minecraft_nick).await?;
	}
	crate::api::minecraft_auth::set_default_user(uuid).await?;
	Ok(())
}

async fn get_metadata(state: &State, key: &str) -> crate::Result<Option<String>> {
	let row = sqlx::query_scalar!(
		r#"SELECT value FROM app_metadata WHERE key = ?"#,
		key
	)
	.fetch_optional(&state.pool)
	.await?;
	Ok(row)
}

async fn set_metadata(state: &State, key: &str, value: &str) -> crate::Result<()> {
	sqlx::query!(
		r#"
		INSERT INTO app_metadata (key, value, updated_at)
		VALUES (?, ?, unixepoch())
		ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = unixepoch()
		"#,
		key,
		value
	)
	.execute(&state.pool)
	.await?;
	Ok(())
}

use crate::state::Credentials;
