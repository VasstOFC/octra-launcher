//! Octra skins via authlib-injector + a client-side Fabric overlay mod.
//!
//! - Every launch gets `-javaagent:authlib-injector.jar=<ygg_root>`.
//! - Fabric / Quilt: `-Dfabric.addMods=<octra-client-skins.jar>` (not copied into packs).
//! - Vanilla 1.20.1: overlay Fabric loader so the client mod can load.
//! - Legacy PNG URLs (`/skins/MinecraftSkins/{nick}.png`) remain for SkinsRestorer.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use daedalus::modded::LoaderVersion;
use md5::Md5;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::nervia;
use crate::state::{
	Credentials, DirectoryInfo, MinecraftSkinVariant, ModLoader, State,
};
use crate::util::fetch::INSECURE_REQWEST_CLIENT;
use crate::util::io;

pub const OFFLINE_REFRESH_TOKEN: &str = "octra-offline";

const CLIENT_SKINS_EPHEMERAL: &str = ".octra-client-skins.jar";
const CLIENT_SKINS_1_20_1: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/octra-client-skins-1.20.1.jar"
));

const AUTHLIB_VERSION: &str = "1.2.8";
const AUTHLIB_SHA256: &str =
	"9c7f4343e6c82034958ffb48c14a2cb0c85928be7283103ce17da00c6d5a7b10";
const AUTHLIB_URL: &str =
	"https://authlib-injector.yushi.moe/artifact/56/authlib-injector-1.2.8.jar";
const AUTHLIB_FALLBACK: &str = "https://github.com/yushijinhun/authlib-injector/releases/download/v1.2.8/authlib-injector-1.2.8.jar";

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
	if name.is_empty() || name.len() > 16 || name.contains(char::is_whitespace) {
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

/// Shared remote Yggdrasil / authlib-injector API root.
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

fn ygg_model(variant: MinecraftSkinVariant) -> &'static str {
	match variant {
		MinecraftSkinVariant::Slim => "slim",
		_ => "default",
	}
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

pub async fn load_equipped_png(uuid: Uuid) -> Option<Vec<u8>> {
	let state = State::get().await.ok()?;
	let path = skin_dir(&state.directories).join(format!("{uuid}.png"));
	let bytes = tokio::fs::read(path).await.ok()?;
	(bytes.len() >= 8).then_some(bytes)
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
	if png.is_empty() {
		return Ok(());
	}

	let name = credentials.offline_profile.name.clone();
	if publish_to_registry(credentials, variant, png).await {
		if verify_registry_skin(&name).await {
			tracing::info!(
				"Octra skins: {name} live at {}",
				registry_skin_url(&name)
			);
		} else {
			tracing::warn!(
				"Octra skins: {name} uploaded but GET {} failed",
				registry_skin_url(&name)
			);
		}
	} else {
		tracing::warn!(
			"Octra skins: failed to publish {name} to {}",
			nervia::skins_url()
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

/// Public legacy skin URL (authlib textures + SkinsRestorer).
pub fn registry_skin_url(name: &str) -> String {
	format!("{}/skins/MinecraftSkins/{name}.png", nervia::skins_url())
}

pub async fn verify_registry_skin(name: &str) -> bool {
	INSECURE_REQWEST_CLIENT
		.get(registry_skin_url(name))
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
	let model = ygg_model(variant);
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

	for method in [reqwest::Method::PUT, reqwest::Method::POST] {
		match send(method.clone()).await {
			Ok(resp) if resp.status().is_success() => {
				tracing::info!("Octra skins: published {name} ({uuid})");
				return true;
			}
			Ok(resp) => tracing::warn!(
				"Octra skins: {method} {name} → HTTP {}",
				resp.status()
			),
			Err(error) => {
				tracing::warn!("Octra skins: {method} {name} → {error}")
			}
		}
	}
	false
}

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

/// Download authlib-injector once; periodically re-publish equipped skins.
pub async fn ensure_runtime() -> crate::Result<()> {
	if RUNTIME_STARTED.swap(true, Ordering::SeqCst) {
		return Ok(());
	}
	let _ = authlib_jar_path().await?;
	tracing::info!("Octra skins: Yggdrasil root {}", ygg_root());
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
		let Ok(resp) = INSECURE_REQWEST_CLIENT
			.get(url)
			.timeout(Duration::from_secs(30))
			.send()
			.await
		else {
			continue;
		};
		if !resp.status().is_success() {
			continue;
		}
		let Ok(bytes) = resp.bytes().await else {
			continue;
		};
		if sha256_hex(&bytes) != AUTHLIB_SHA256 {
			tracing::warn!("Octra skins: authlib SHA-256 mismatch from {url}");
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

/// Overlay Fabric onto vanilla when we have a client-skins jar for this MC version.
pub async fn overlay_fabric_if_vanilla(
	game_version: &str,
	loader: ModLoader,
	loader_version: Option<LoaderVersion>,
) -> (ModLoader, Option<LoaderVersion>) {
	if loader != ModLoader::Vanilla || !supports_client_skins(game_version) {
		return (loader, loader_version);
	}
	match crate::launcher::get_loader_version_from_profile(
		game_version,
		ModLoader::Fabric,
		Some("stable"),
	)
	.await
	{
		Ok(Some(fabric)) => {
			tracing::info!(
				"Octra skins: overlaying Fabric {} on vanilla {game_version}",
				fabric.id
			);
			(ModLoader::Fabric, Some(fabric))
		}
		Ok(None) => {
			tracing::debug!("Octra skins: no Fabric loader for {game_version}");
			(loader, loader_version)
		}
		Err(error) => {
			tracing::warn!("Octra skins: Fabric overlay failed: {error}");
			(loader, loader_version)
		}
	}
}

/// Before Minecraft starts: publish equipped skin, inject authlib + client skins mod.
pub async fn prepare_launch(
	instance_path: &Path,
	game_version: &str,
	loader: ModLoader,
	credentials: &Credentials,
	java_args: &mut Vec<String>,
) -> crate::Result<()> {
	if let Err(error) = ensure_runtime().await {
		tracing::warn!("Octra skins: runtime {error}");
	}

	if let Some(png) = load_equipped_png(credentials.offline_profile.id).await {
		let variant = load_equipped(credentials.offline_profile.id)
			.await
			.map(|equipped| equipped.variant)
			.unwrap_or(MinecraftSkinVariant::Classic);
		if publish_to_registry(credentials, variant, &png).await {
			tracing::info!(
				"Octra skins: published {} before launch",
				credentials.offline_profile.name
			);
		}
	}

	let jar = authlib_jar_path().await?;
	let jar = dunce::canonicalize(&jar).unwrap_or(jar);
	let root = ygg_root();
	java_args.insert(0, format!("-javaagent:{}={}", jar.display(), root));
	java_args.insert(1, "-Dauthlibinjector.side=client".to_string());
	java_args.insert(2, format!("-Doctra.skins.url={root}"));
	tracing::info!(
		"Octra skins: authlib-injector → {root} ({})",
		credentials.offline_profile.name
	);

	if let Err(error) =
		inject_client_skins(instance_path, game_version, loader, java_args)
			.await
	{
		tracing::warn!("Octra skins: client mod inject: {error}");
	}
	Ok(())
}

pub async fn cleanup_ephemeral_client_skins(instance_path: &Path) {
	let ephemeral = ephemeral_client_skins_path(instance_path);
	if ephemeral.exists() {
		let _ = tokio::fs::remove_file(ephemeral).await;
	}
}

fn supports_client_skins(game_version: &str) -> bool {
	game_version == "1.20.1"
}

fn client_skins_bytes(game_version: &str) -> Option<&'static [u8]> {
	match game_version {
		"1.20.1" => Some(CLIENT_SKINS_1_20_1),
		_ => None,
	}
}

fn ephemeral_client_skins_path(instance_path: &Path) -> PathBuf {
	instance_path.join("mods").join(CLIENT_SKINS_EPHEMERAL)
}

async fn client_skins_jar_path(game_version: &str) -> crate::Result<PathBuf> {
	let bytes = client_skins_bytes(game_version).ok_or_else(|| {
		crate::ErrorKind::OtherError(format!(
			"no octra-client-skins for Minecraft {game_version}"
		))
	})?;
	let state = State::get().await?;
	let dest = state
		.directories
		.metadata_dir()
		.join(format!("octra-client-skins-{game_version}.jar"));
	if dest.exists()
		&& let Ok(existing) = tokio::fs::read(&dest).await
		&& existing.as_slice() == bytes
	{
		return Ok(dest);
	}
	io::create_dir_all(state.directories.metadata_dir()).await?;
	io::write(&dest, bytes).await?;
	Ok(dest)
}

async fn inject_client_skins(
	_instance_path: &Path,
	game_version: &str,
	loader: ModLoader,
	java_args: &mut Vec<String>,
) -> crate::Result<()> {
	if !supports_client_skins(game_version) {
		tracing::debug!(
			"Octra skins: no client mod for {game_version} yet"
		);
		return Ok(());
	}
	let jar = client_skins_jar_path(game_version).await?;
	let jar = dunce::canonicalize(&jar).unwrap_or(jar);
	match loader {
		ModLoader::Fabric | ModLoader::Quilt => {
			java_args.push(format!("-Dfabric.addMods={}", jar.display()));
			tracing::info!(
				"Octra skins: injected client mod via fabric.addMods ({game_version})"
			);
			Ok(())
		}
		ModLoader::Forge | ModLoader::NeoForge | ModLoader::Vanilla => {
			tracing::info!(
				"Octra skins: client mod skipped for {} {game_version} (Fabric/Quilt only for now)",
				loader.as_str()
			);
			Ok(())
		}
	}
}
