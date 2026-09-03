//! Octra cloud account (registration, login, JWT for skin uploads).
//!
//! Registration ("skin passport") binds to the launcher's active Minecraft
//! Credentials (nick + profile UUID). Octra login username equals that nick.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::nervia;
use crate::state::{Credentials, State};
use crate::util::fetch::INSECURE_REQWEST_CLIENT;

const TOKEN_KEY: &str = "octra_account_token";
const USERNAME_KEY: &str = "octra_account_username";
const MINECRAFT_NICK_KEY: &str = "octra_account_minecraft_nick";
const PROFILE_UUID_KEY: &str = "octra_account_profile_uuid";
const ACCOUNT_TYPE_KEY: &str = "octra_account_account_type";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OctraAccountSession {
	pub token: String,
	pub username: String,
	pub minecraft_nick: String,
	pub profile_uuid: String,
	#[serde(default = "default_account_type")]
	pub account_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OctraCommunityMember {
	pub id: i64,
	pub minecraft_nick: String,
	pub profile_uuid: String,
	#[serde(default = "default_account_type")]
	pub account_type: String,
	pub created_at: String,
	pub avatar_url: String,
	#[serde(default = "default_presence")]
	pub presence: String,
	#[serde(default)]
	pub instance_name: Option<String>,
	#[serde(default)]
	pub last_seen: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OctraCommunitySnapshot {
	pub connected: bool,
	pub members: Vec<OctraCommunityMember>,
}

fn default_account_type() -> String {
	"offline".to_string()
}

fn default_presence() -> String {
	"offline".to_string()
}

#[derive(Deserialize)]
struct AuthResponse {
	token: String,
	username: String,
	minecraft_nick: String,
	profile_uuid: String,
	#[serde(default = "default_account_type")]
	account_type: String,
}

#[derive(Deserialize)]
struct CommunityMemberApi {
	id: i64,
	minecraft_nick: String,
	profile_uuid: String,
	#[serde(default = "default_account_type")]
	account_type: String,
	created_at: String,
	#[serde(default = "default_presence")]
	presence: String,
	#[serde(default)]
	instance_name: Option<String>,
	#[serde(default)]
	last_seen: Option<String>,
}

pub async fn session() -> crate::Result<Option<OctraAccountSession>> {
	let state = State::get().await?;
	let token = get_metadata(&state, TOKEN_KEY).await?;
	let username = get_metadata(&state, USERNAME_KEY).await?;
	let minecraft_nick = get_metadata(&state, MINECRAFT_NICK_KEY).await?;
	let profile_uuid = get_metadata(&state, PROFILE_UUID_KEY).await?;
	let account_type = get_metadata(&state, ACCOUNT_TYPE_KEY)
		.await?
		.unwrap_or_else(default_account_type);
	match (token, username, minecraft_nick, profile_uuid) {
		(Some(token), Some(username), Some(minecraft_nick), Some(profile_uuid))
			if !token.is_empty() =>
		{
			Ok(Some(OctraAccountSession {
				token,
				username,
				minecraft_nick,
				profile_uuid,
				account_type,
			}))
		}
		_ => Ok(None),
	}
}

/// Register an Octra account linked to the default Minecraft Credentials.
/// Does not create a Minecraft account — one must already be signed in.
pub async fn register(password: &str) -> crate::Result<OctraAccountSession> {
	let state = State::get().await?;
	let credentials = Credentials::get_default_credential(&state.pool)
		.await?
		.ok_or_else(|| {
			crate::ErrorKind::OtherError(
				"Add a Microsoft or offline Minecraft account before creating an Octra account"
					.to_string(),
			)
		})?;

	let minecraft_nick = credentials.offline_profile.name.clone();
	let profile_uuid = credentials.offline_profile.id.to_string();
	let account_type = if credentials.is_offline() {
		"offline"
	} else {
		"premium"
	};

	let body = serde_json::json!({
		"password": password,
		"minecraft_nick": minecraft_nick,
		"profile_uuid": profile_uuid,
		"account_type": account_type,
	});
	let response = post_auth("/api/v1/auth/register", &body).await?;
	save_session(&response).await?;
	let _ = sync_presence().await;
	Ok(response)
}

pub async fn login(username: &str, password: &str) -> crate::Result<OctraAccountSession> {
	let body = serde_json::json!({
		"username": username,
		"password": password,
	});
	let response = post_auth("/api/v1/auth/login", &body).await?;
	save_session(&response).await?;
	let _ = sync_presence().await;
	Ok(response)
}

pub async fn logout() -> crate::Result<()> {
	let _ = publish_presence("offline", None).await;
	let state = State::get().await?;
	for key in [
		TOKEN_KEY,
		USERNAME_KEY,
		MINECRAFT_NICK_KEY,
		PROFILE_UUID_KEY,
		ACCOUNT_TYPE_KEY,
	] {
		sqlx::query!("DELETE FROM app_metadata WHERE key = ?", key)
			.execute(&state.pool)
			.await?;
	}
	Ok(())
}

pub async fn community() -> crate::Result<OctraCommunitySnapshot> {
	let Some(session) = session().await? else {
		return Ok(OctraCommunitySnapshot {
			connected: false,
			members: Vec::new(),
		});
	};

	let url = format!("{}/api/v1/community", nervia::skins_url());
	let response = match INSECURE_REQWEST_CLIENT
		.get(&url)
		.header("Authorization", format!("Bearer {}", session.token))
		.timeout(Duration::from_secs(15))
		.send()
		.await
	{
		Ok(response) => response,
		Err(error) => {
			tracing::warn!("octra community request failed: {error}");
			return Ok(OctraCommunitySnapshot {
				connected: false,
				members: Vec::new(),
			});
		}
	};

	let status = response.status();
	let text = response.text().await.unwrap_or_default();
	if !status.is_success() {
		tracing::warn!("octra community HTTP {status}: {text}");
		return Ok(OctraCommunitySnapshot {
			connected: false,
			members: Vec::new(),
		});
	}

	let parsed: Vec<CommunityMemberApi> = match serde_json::from_str(&text) {
		Ok(parsed) => parsed,
		Err(error) => {
			tracing::warn!("octra community response parse failed: {error}");
			return Ok(OctraCommunitySnapshot {
				connected: false,
				members: Vec::new(),
			});
		}
	};
	let base = nervia::skins_url();
	Ok(OctraCommunitySnapshot {
		connected: true,
		members: parsed
			.into_iter()
			.map(|member| {
				let avatar_url = format!("{}/skins/{}", base, member.profile_uuid);
				OctraCommunityMember {
					id: member.id,
					minecraft_nick: member.minecraft_nick,
					profile_uuid: member.profile_uuid,
					account_type: member.account_type,
					created_at: member.created_at,
					avatar_url,
					presence: member.presence,
					instance_name: member.instance_name,
					last_seen: member.last_seen,
				}
			})
			.collect(),
	})
}

pub async fn publish_presence(
	status: &str,
	instance_name: Option<&str>,
) -> crate::Result<()> {
	let Some(session) = session().await? else {
		return Ok(());
	};

	let url = format!("{}/api/v1/presence", nervia::skins_url());
	let body = serde_json::json!({
		"status": status,
		"instance_name": instance_name,
	});
	let response = INSECURE_REQWEST_CLIENT
		.post(&url)
		.header("Authorization", format!("Bearer {}", session.token))
		.json(&body)
		.timeout(Duration::from_secs(8))
		.send()
		.await
		.map_err(|e| {
			crate::ErrorKind::OtherError(format!("octra presence request failed: {e}"))
		})?;
	if !response.status().is_success() {
		let text = response.text().await.unwrap_or_default();
		return Err(crate::ErrorKind::OtherError(format!(
			"octra presence failed: {text}"
		))
		.into());
	}
	Ok(())
}

pub async fn sync_presence() -> crate::Result<()> {
	if session().await?.is_none() {
		return Ok(());
	}
	let state = State::get().await?;
	let processes = state.process_manager.get_all();
	if let Some(process) = processes.first() {
		publish_presence("ingame", Some(&process.instance_name)).await
	} else {
		publish_presence("launcher", None).await
	}
}

pub fn spawn_presence_heartbeat() {
	tokio::spawn(async {
		loop {
			if let Err(error) = sync_presence().await {
				tracing::debug!("octra presence heartbeat: {error}");
			}
			tokio::time::sleep(Duration::from_secs(20)).await;
		}
	});
}

pub async fn bearer_token() -> Option<String> {
	session().await.ok().flatten().map(|s| s.token)
}

async fn post_auth(path: &str, body: &serde_json::Value) -> crate::Result<OctraAccountSession> {
	let url = format!("{}{}", nervia::skins_url(), path);
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
		account_type: parsed.account_type,
	})
}

async fn save_session(session: &OctraAccountSession) -> crate::Result<()> {
	let state = State::get().await?;
	set_metadata(&state, TOKEN_KEY, &session.token).await?;
	set_metadata(&state, USERNAME_KEY, &session.username).await?;
	set_metadata(&state, MINECRAFT_NICK_KEY, &session.minecraft_nick).await?;
	set_metadata(&state, PROFILE_UUID_KEY, &session.profile_uuid).await?;
	set_metadata(&state, ACCOUNT_TYPE_KEY, &session.account_type).await?;
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
