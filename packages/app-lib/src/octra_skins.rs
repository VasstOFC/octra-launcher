//! Offline accounts, authlib-injector, and the Octra skin VPS.
//!
//! Friends see offline skins via authlib-injector pointed at the shared remote
//! Yggdrasil root ([`crate::nervia::skins_url`]) — no CustomSkinLoader and no
//! Fabric overlay solely for skins. Texture URLs in profile JSON point at
//! `/skins/MinecraftSkins/{nick}.png` on the same host.
//!
//! SkinsRestorer on dedicated servers can still use the same legacy URL.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use daedalus::modded::LoaderVersion;
use md5::Md5;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::data::ModLoader;
use crate::nervia;
use crate::state::{Credentials, DirectoryInfo, MinecraftSkinVariant, State};
use crate::util::fetch::INSECURE_REQWEST_CLIENT;
use crate::util::io;

pub const OFFLINE_REFRESH_TOKEN: &str = "octra-offline";

const AUTHLIB_VERSION: &str = "1.2.8";
const AUTHLIB_SHA256: &str =
	"9c7f4343e6c82034958ffb48c14a2cb0c85928be7283103ce17da00c6d5a7b10";
const AUTHLIB_URL: &str =
	"https://authlib-injector.yushi.moe/artifact/56/authlib-injector-1.2.8.jar";
const AUTHLIB_FALLBACK: &str = "https://github.com/yushijinhun/authlib-injector/releases/download/v1.2.8/authlib-injector-1.2.8.jar";

/// Leftover Forge/NeoForge CSL hardlink from older Octra builds — cleaned on exit.
const OCTRA_CSL_EPHEMERAL_MOD: &str = ".octra-customskinloader.jar";

static RUNTIME_STARTED: AtomicBool = AtomicBool::new(false);

/// UUID v3 matching Java `UUID.nameUUIDFromBytes("OfflinePlayer:" + name)`.
pub fn offline_player_uuid(name: &str) -> Uuid {
	let mut hasher = Md5::new();
	hasher.update(format!("OfflinePlayer:{name}").as_bytes());
	let mut bytes = hasher.finalize();
	bytes[6] = (bytes[6] & 0x0f) | 0x30;
	bytes[8] = (bytes[8] & 0x3f) | 0x80;
	Uuid::from_bytes(bytes.into())
}

pub fn validate_offline_name(name: &str) -> crate::Result<String> {
	let name = name.trim();
	if name.is_empty() || name.len() > 16 || name.contains(char::is_whitespace)
	{
		return Err(crate::ErrorKind::InputError(
			"nick offline: 1–16 znaków, bez spacji".to_string(),
		)
		.into());
	}
	if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
		return Err(crate::ErrorKind::InputError(
			"nick może zawierać tylko litery, cyfry i podkreślenie".to_string(),
		)
		.into());
	}
	Ok(name.to_string())
}

/// Shared remote Yggdrasil / authlib-injector API root (same host as skin registry).
pub fn ygg_root() -> String {
	nervia::skins_url().trim_end_matches('/').to_string()
}

fn sha256_hex(data: &[u8]) -> String {
	Sha256::digest(data)
		.iter()
		.map(|b| format!("{b:02x}"))
		.collect()
}

fn hyphenated_uuid(id: &Uuid) -> String {
	id.as_hyphenated().to_string()
}

fn skin_dir(directories: &DirectoryInfo) -> PathBuf {
	directories.settings_dir.join("octra-skins")
}

#[derive(Serialize, Deserialize)]
struct EquippedRecord {
	texture_key: String,
	variant: MinecraftSkinVariant,
}

pub struct EquippedSkin {
	pub texture_key: String,
	pub variant: MinecraftSkinVariant,
}

pub async fn load_equipped(uuid: Uuid) -> Option<EquippedSkin> {
	let state = State::get().await.ok()?;
	let path = skin_dir(&state.directories).join(format!("{uuid}.json"));
	let raw = tokio::fs::read_to_string(path).await.ok()?;
	let record: EquippedRecord = serde_json::from_str(&raw).ok()?;
	Some(EquippedSkin {
		texture_key: record.texture_key,
		variant: record.variant,
	})
}

pub async fn save_equipped(
	credentials: &Credentials,
	texture_key: &str,
	variant: MinecraftSkinVariant,
	png: &[u8],
) -> crate::Result<()> {
	let state = State::get().await?;
	let dir = skin_dir(&state.directories);
	io::create_dir_all(&dir).await?;
	let uuid = credentials.offline_profile.id;
	let record = EquippedRecord {
		texture_key: texture_key.to_string(),
		variant,
	};
	io::write(
		dir.join(format!("{uuid}.json")),
		serde_json::to_vec_pretty(&record)?,
	)
	.await?;
	let owned_png;
	let png = if png.is_empty() {
		owned_png = load_equipped_png(uuid).await.unwrap_or_default();
		owned_png.as_slice()
	} else {
		png
	};
	if !png.is_empty() {
		io::write(dir.join(format!("{uuid}.png")), png).await?;
	}
	let published = publish_to_registry(credentials, variant, png).await;
	let name = credentials.offline_profile.name.clone();
	if png.is_empty() {
		return Ok(());
	}
	if !published {
		tracing::warn!(
			"nie udało się opublikować skina dla {name} na serwerze Octra ({})",
			nervia::skins_url()
		);
	} else if !verify_registry_skin(&name).await {
		tracing::warn!(
			"skin dla {name} wysłany, ale {} nie odpowiada — znajomi mogą nie widzieć skina",
			registry_skin_url(&name)
		);
	} else {
		tracing::info!(
			"skin dla {name} dostępny przez authlib + chmurę Octra ({})",
			registry_skin_url(&name)
		);
	}
	Ok(())
}

pub async fn clear_equipped(credentials: &Credentials) -> crate::Result<()> {
	let state = State::get().await?;
	let dir = skin_dir(&state.directories);
	let uuid = credentials.offline_profile.id;
	let _ = tokio::fs::remove_file(dir.join(format!("{uuid}.json"))).await;
	let _ = tokio::fs::remove_file(dir.join(format!("{uuid}.png"))).await;
	Ok(())
}

fn registry_legacy_url(name: &str) -> String {
	format!("{}/skins/MinecraftSkins/{name}.png", nervia::skins_url())
}

/// Public legacy skin URL (authlib profile textures + optional SkinsRestorer).
pub fn registry_skin_url(name: &str) -> String {
	registry_legacy_url(name)
}

pub async fn verify_registry_skin(name: &str) -> bool {
	let url = registry_legacy_url(name);
	INSECURE_REQWEST_CLIENT
		.get(&url)
		.timeout(Duration::from_secs(8))
		.send()
		.await
		.map(|response| response.status().is_success())
		.unwrap_or(false)
}

pub async fn publish_to_registry(
	credentials: &Credentials,
	variant: MinecraftSkinVariant,
	png: &[u8],
) -> bool {
	if png.is_empty() {
		return false;
	}
	let model = match variant {
		MinecraftSkinVariant::Slim => "slim",
		_ => "classic",
	};
	let uuid = hyphenated_uuid(&credentials.offline_profile.id);
	let name = &credentials.offline_profile.name;
	let url = format!("{}/skins/{uuid}", nervia::skins_url());
	let bearer = crate::octra_accounts::bearer_token().await;
	let send = |method: reqwest::Method| {
		let mut request = INSECURE_REQWEST_CLIENT
			.request(method, &url)
			.header(reqwest::header::CONTENT_TYPE, "image/png")
			.header("X-Lumen-Model", model)
			.header("X-Lumen-Name", name)
			.timeout(Duration::from_secs(12))
			.body(png.to_vec());
		if let Some(token) = bearer.as_deref() {
			request = request.header(
				reqwest::header::AUTHORIZATION,
				format!("Bearer {token}"),
			);
		} else {
			request = request.header("X-Octra-Key", nervia::SKINS_API_KEY);
		}
		request.send()
	};
	match send(reqwest::Method::PUT).await {
		Ok(resp) if resp.status().is_success() => {
			tracing::info!(
				"published skin for {name} to octra registry ({uuid})"
			);
			true
		}
		Ok(resp) => {
			tracing::warn!(
				"octra skin registry PUT for {name} failed: HTTP {}",
				resp.status()
			);
			match send(reqwest::Method::POST).await {
				Ok(resp) if resp.status().is_success() => {
					tracing::info!(
						"published skin for {name} to octra registry via POST ({uuid})"
					);
					true
				}
				Ok(resp) => {
					tracing::warn!(
						"octra skin registry POST for {name} failed: HTTP {}",
						resp.status()
					);
					false
				}
				Err(error) => {
					tracing::warn!(
						"octra skin registry POST for {name} failed: {error}"
					);
					false
				}
			}
		}
		Err(error) => {
			tracing::warn!(
				"octra skin registry PUT for {name} failed: {error}"
			);
			match send(reqwest::Method::POST).await {
				Ok(resp) if resp.status().is_success() => {
					tracing::info!(
						"published skin for {name} to octra registry via POST ({uuid})"
					);
					true
				}
				Ok(resp) => {
					tracing::warn!(
						"octra skin registry POST for {name} failed: HTTP {}",
						resp.status()
					);
					false
				}
				Err(error) => {
					tracing::warn!(
						"octra skin registry POST for {name} failed: {error}"
					);
					false
				}
			}
		}
	}
}

/// Re-uploads equipped skins for every saved Minecraft account (startup + retry).
pub async fn sync_all_equipped_skins() {
	let Ok(state) = State::get().await else {
		return;
	};
	let Ok(accounts) = Credentials::get_all(&state.pool).await else {
		return;
	};
	for entry in accounts.iter() {
		let credentials = entry.value();
		let Some(png) = load_equipped_png(credentials.offline_profile.id).await
		else {
			continue;
		};
		let variant = load_equipped(credentials.offline_profile.id)
			.await
			.map(|equipped| equipped.variant)
			.unwrap_or(MinecraftSkinVariant::Classic);
		publish_to_registry(&credentials, variant, &png).await;
	}
}

pub async fn load_equipped_png(uuid: Uuid) -> Option<Vec<u8>> {
	let state = State::get().await.ok()?;
	let path = skin_dir(&state.directories).join(format!("{uuid}.png"));
	let bytes = tokio::fs::read(path).await.ok()?;
	if bytes.len() < 8 {
		return None;
	}
	Some(bytes)
}

/// Starts background skin sync to the VPS registry (no local Yggdrasil hub).
pub async fn ensure_runtime() -> crate::Result<()> {
	if RUNTIME_STARTED.swap(true, Ordering::SeqCst) {
		return Ok(());
	}
	tracing::info!(
		"Octra skins: authlib-injector → remote Yggdrasil {}",
		ygg_root()
	);
	tokio::spawn(async {
		sync_all_equipped_skins().await;
		loop {
			tokio::time::sleep(Duration::from_secs(15 * 60)).await;
			sync_all_equipped_skins().await;
		}
	});
	Ok(())
}

async fn authlib_jar_path() -> crate::Result<PathBuf> {
	let state = State::get().await?;
	let dest = state
		.directories
		.metadata_dir()
		.join(format!("authlib-injector-{AUTHLIB_VERSION}.jar"));
	if dest.exists()
		&& let Ok(bytes) = tokio::fs::read(&dest).await
		&& sha256_hex(&bytes) == AUTHLIB_SHA256
	{
		return Ok(dest);
	}
	io::create_dir_all(state.directories.metadata_dir()).await?;
	for url in [AUTHLIB_URL, AUTHLIB_FALLBACK] {
		let resp = INSECURE_REQWEST_CLIENT
			.get(url)
			.timeout(Duration::from_secs(30))
			.send()
			.await;
		let Ok(resp) = resp else {
			continue;
		};
		if !resp.status().is_success() {
			continue;
		}
		let Ok(bytes) = resp.bytes().await else {
			continue;
		};
		if sha256_hex(&bytes) != AUTHLIB_SHA256 {
			tracing::warn!("authlib-injector SHA-256 mismatch from {url}");
			continue;
		}
		io::write(&dest, &bytes).await?;
		return Ok(dest);
	}
	Err(crate::ErrorKind::OtherError(
		"could not download authlib-injector".to_string(),
	)
	.into())
}

/// Previously overlaid Fabric on vanilla solely for CustomSkinLoader.
/// Skins now use authlib-injector only — loader is left unchanged.
pub async fn overlay_fabric_if_vanilla(
	_game_version: &str,
	loader: ModLoader,
	loader_version: Option<LoaderVersion>,
) -> (ModLoader, Option<LoaderVersion>) {
	(loader, loader_version)
}

pub async fn prepare_launch(
	instance_path: &Path,
	_game_version: &str,
	_loader: ModLoader,
	credentials: &Credentials,
	java_args: &mut Vec<String>,
) -> crate::Result<()> {
	if let Err(e) = ensure_runtime().await {
		tracing::warn!("Octra skins runtime: {e}");
	}

	// Publish equipped skin so friends' authlib clients can resolve it on the VPS.
	if let Some(png) = load_equipped_png(credentials.offline_profile.id).await {
		let variant = load_equipped(credentials.offline_profile.id)
			.await
			.map(|e| e.variant)
			.unwrap_or(MinecraftSkinVariant::Classic);
		let published =
			publish_to_registry(credentials, variant, &png).await;
		if published {
			tracing::info!(
				"Octra skins: published {} before launch",
				credentials.offline_profile.name
			);
		}
	}

	// Always inject authlib-injector for Octra launches. The remote Yggdrasil
	// falls through to Mojang for unknown (premium) profiles so online skins
	// keep working; offline registry skins resolve from the VPS.
	match authlib_jar_path().await {
		Ok(jar) => {
			let jar = dunce::canonicalize(&jar).unwrap_or(jar);
			let root = ygg_root();
			java_args.insert(
				0,
				format!("-javaagent:{}={}", jar.display(), root),
			);
			tracing::info!(
				"Octra skins: authlib-injector → {root} (vanilla-compatible, no CSL)"
			);
		}
		Err(e) => {
			tracing::warn!(
				"Octra skins: authlib-injector unavailable ({e}) — friends may not see offline skins"
			);
		}
	}

	// Best-effort cleanup of leftover CSL artifacts from older Octra versions.
	remove_legacy_csl_from_mods(instance_path).await;
	cleanup_ephemeral_csl(instance_path).await;

	Ok(())
}

/// Removes ephemeral Forge/NeoForge CSL hardlink after Minecraft exits (legacy).
pub async fn cleanup_ephemeral_csl(instance_path: &Path) {
	let ephemeral = instance_path.join("mods").join(OCTRA_CSL_EPHEMERAL_MOD);
	if ephemeral.exists() {
		let _ = tokio::fs::remove_file(ephemeral).await;
	}
}

async fn remove_legacy_csl_from_mods(instance_path: &Path) {
	let mods = instance_path.join("mods");
	let Ok(mut dir) = tokio::fs::read_dir(&mods).await else {
		return;
	};
	while let Ok(Some(entry)) = dir.next_entry().await {
		let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
		if name.contains("customskinloader") {
			let _ = tokio::fs::remove_file(entry.path()).await;
			tracing::info!(
				"Octra skins: removed leftover CustomSkinLoader {}",
				entry.file_name().to_string_lossy()
			);
		}
	}
}
