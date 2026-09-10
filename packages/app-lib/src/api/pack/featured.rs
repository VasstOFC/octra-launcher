use crate::event::LoadingBarType;
use crate::event::emit::{emit_loading, init_loading};
use crate::state::State;
use crate::util::fetch::{
	INSECURE_NO_TIMEOUT_REQWEST_CLIENT, INSECURE_REQWEST_CLIENT,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

const ZIP_LOCAL_FILE_HEADER: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];
const MIN_PACK_BYTES: u64 = 64 * 1024;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeaturedPackInfo {
	pub enabled: bool,
	pub title: String,
	pub blurb: String,
}

#[derive(Clone, Debug)]
struct FeaturedPackConfig {
	enabled: bool,
	title: String,
	blurb: String,
	url: String,
	version: String,
	cache_name: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct FeaturedPackRemoteConfig {
	enabled: Option<bool>,
	title: Option<String>,
	blurb: Option<String>,
	url: Option<String>,
	version: Option<String>,
	#[serde(alias = "cacheName")]
	cache_name: Option<String>,
}

fn default_config() -> FeaturedPackConfig {
	FeaturedPackConfig {
		enabled: true,
		title: crate::nervia::FEATURED_PACK_TITLE.to_string(),
		blurb: crate::nervia::FEATURED_PACK_BLURB.to_string(),
		url: crate::nervia::FEATURED_PACK_URL.to_string(),
		version: crate::nervia::FEATURED_PACK_VERSION.to_string(),
		cache_name: crate::nervia::FEATURED_PACK_CACHE_NAME.to_string(),
	}
}

static REMOTE_CONFIG: OnceLock<Mutex<Option<FeaturedPackConfig>>> = OnceLock::new();

fn remote_slot() -> &'static Mutex<Option<FeaturedPackConfig>> {
	REMOTE_CONFIG.get_or_init(|| Mutex::new(None))
}

async fn load_featured_pack_config() -> FeaturedPackConfig {
	{
		let guard = remote_slot().lock().await;
		if let Some(cached) = guard.as_ref() {
			return cached.clone();
		}
	}

	let mut resolved = default_config();
	let url = format!(
		"{}{}",
		crate::nervia::skins_url(),
		crate::nervia::FEATURED_PACK_CONFIG_PATH
	);

	match INSECURE_REQWEST_CLIENT
		.get(&url)
		.timeout(Duration::from_secs(4))
		.send()
		.await
	{
		Ok(response) if response.status().is_success() => {
			match response.json::<FeaturedPackRemoteConfig>().await {
				Ok(remote) => {
					if let Some(enabled) = remote.enabled {
						resolved.enabled = enabled;
					}
					if let Some(title) = remote.title.filter(|s| !s.trim().is_empty()) {
						resolved.title = title;
					}
					if let Some(blurb) = remote.blurb.filter(|s| !s.trim().is_empty()) {
						resolved.blurb = blurb;
					}
					if let Some(pack_url) = remote.url.filter(|s| !s.trim().is_empty()) {
						resolved.url = pack_url;
					}
					if let Some(version) = remote.version.filter(|s| !s.trim().is_empty()) {
						resolved.version = version;
					}
					if let Some(cache_name) =
						remote.cache_name.filter(|s| !s.trim().is_empty())
					{
						resolved.cache_name = cache_name;
					}
					tracing::info!("featured pack: loaded remote config from {url}");
				}
				Err(error) => tracing::warn!(
					"featured pack: invalid remote config at {url}: {error}"
				),
			}
		}
		Ok(response) => tracing::debug!(
			"featured pack: remote config {url} → HTTP {}",
			response.status()
		),
		Err(error) => tracing::debug!("featured pack: remote config fetch failed: {error}"),
	}

	let mut guard = remote_slot().lock().await;
	*guard = Some(resolved.clone());
	resolved
}

pub async fn featured_pack_info() -> FeaturedPackInfo {
	let config = load_featured_pack_config().await;
	FeaturedPackInfo {
		enabled: config.enabled,
		title: config.title,
		blurb: config.blurb,
	}
}

pub async fn resolve_featured_pack_path() -> crate::Result<PathBuf> {
	let config = load_featured_pack_config().await;
	if !config.enabled {
		return Err(crate::ErrorKind::InputError(
			"featured pack is disabled by remote config".to_string(),
		)
		.into());
	}

	let state = State::get().await?;
	let cache_dir = state.directories.caches_dir().join("featured-packs");
	tokio::fs::create_dir_all(&cache_dir).await?;

	let cache_path = cache_dir.join(format!("{}-{}", config.version, config.cache_name));
	if is_usable_mrpack(&cache_path).await {
		return Ok(cache_path);
	}

	for candidate in local_pack_candidates(&state.directories) {
		if is_usable_mrpack(&candidate).await {
			tokio::fs::copy(&candidate, &cache_path).await?;
			return Ok(cache_path);
		}
	}

	let url = config.url.trim();
	if !url.is_empty() {
		download_pack_url(url, &cache_path, &config.title).await?;
		if is_usable_mrpack(&cache_path).await {
			return Ok(cache_path);
		}
		let _ = tokio::fs::remove_file(&cache_path).await;
	}

	Err(crate::ErrorKind::InputError(format!(
		"featured pack is missing. drop `{pack}` into `{packs}` or host it at {url}",
		pack = crate::nervia::FEATURED_PACK,
		packs = state
			.directories
			.settings_dir
			.join("packs")
			.display(),
		url = if url.is_empty() {
			"(no download url configured)"
		} else {
			url
		},
	))
	.into())
}

/// Download an arbitrary `.mrpack` URL into the launcher cache and return its path.
pub async fn cache_mrpack_from_url(url: &str) -> crate::Result<PathBuf> {
	let url = url.trim();
	if !(url.starts_with("http://") || url.starts_with("https://")) {
		return Err(crate::ErrorKind::InputError(
			"modpack url must start with http:// or https://".to_string(),
		)
		.into());
	}
	if !url.to_ascii_lowercase().contains(".mrpack") {
		return Err(crate::ErrorKind::InputError(
			"url must point to a .mrpack file".to_string(),
		)
		.into());
	}

	let state = State::get().await?;
	let cache_dir = state.directories.caches_dir().join("octra-chat-packs");
	tokio::fs::create_dir_all(&cache_dir).await?;

	let hash = {
		let digest = Sha256::digest(url.as_bytes());
		digest
			.iter()
			.take(8)
			.map(|b| format!("{b:02x}"))
			.collect::<String>()
	};
	let cache_path = cache_dir.join(format!("{hash}.mrpack"));
	if is_usable_mrpack(&cache_path).await {
		return Ok(cache_path);
	}

	download_pack_url(url, &cache_path, "Modpack").await?;
	if !is_usable_mrpack(&cache_path).await {
		let _ = tokio::fs::remove_file(&cache_path).await;
		return Err(crate::ErrorKind::InputError(
			"downloaded file is not a valid .mrpack".to_string(),
		)
		.into());
	}
	Ok(cache_path)
}

fn local_pack_candidates(dirs: &crate::state::DirectoryInfo) -> Vec<PathBuf> {
	let relative = PathBuf::from(crate::nervia::FEATURED_PACK);
	let filename = Path::new(crate::nervia::FEATURED_PACK)
		.file_name()
		.map(PathBuf::from)
		.unwrap_or_else(|| {
			PathBuf::from(crate::nervia::FEATURED_PACK_CACHE_NAME)
		});

	let mut candidates = vec![
		dirs.config_dir.join(&relative),
		dirs.settings_dir.join(&relative),
		dirs.config_dir.join("packs").join(&filename),
		dirs.settings_dir.join("packs").join(&filename),
		dirs.settings_dir
			.join("packs")
			.join(crate::nervia::FEATURED_PACK_CACHE_NAME),
	];

	if let Ok(exe) = std::env::current_exe() {
		if let Some(parent) = exe.parent() {
			candidates.push(parent.join(&relative));
			candidates.push(parent.join("packs").join(&filename));
		}
	}

	candidates.sort();
	candidates.dedup();
	candidates
}

async fn is_usable_mrpack(path: &Path) -> bool {
	let Ok(metadata) = tokio::fs::metadata(path).await else {
		return false;
	};
	if !metadata.is_file() || metadata.len() < MIN_PACK_BYTES {
		return false;
	}

	let Ok(mut file) = tokio::fs::File::open(path).await else {
		return false;
	};

	let mut header = [0_u8; 4];
	tokio::io::AsyncReadExt::read_exact(&mut file, &mut header)
		.await
		.ok()
		.is_some_and(|_| header == ZIP_LOCAL_FILE_HEADER)
}

async fn download_pack_url(
	url: &str,
	dest: &Path,
	pack_name: &str,
) -> crate::Result<()> {
	if let Some(parent) = dest.parent() {
		tokio::fs::create_dir_all(parent).await?;
	}

	let loading_bar = init_loading(
		LoadingBarType::PackImport {
			pack_name: pack_name.to_string(),
		},
		100.0,
		"Downloading featured pack",
	)
	.await
	.ok();

	let tmp = dest.with_extension("mrpack.part");
	let result = download_pack_url_inner(url, &tmp, loading_bar.as_ref()).await;
	match result {
		Ok(()) => {
			tokio::fs::rename(&tmp, dest).await?;
			Ok(())
		}
		Err(err) => {
			let _ = tokio::fs::remove_file(&tmp).await;
			Err(err)
		}
	}
}

async fn download_pack_url_inner(
	url: &str,
	dest: &Path,
	loading_bar: Option<&crate::event::LoadingBarId>,
) -> crate::Result<()> {
	let response = INSECURE_NO_TIMEOUT_REQWEST_CLIENT
		.get(url)
		.send()
		.await
		.map_err(|err| {
			crate::ErrorKind::InputError(format!(
				"failed to download featured pack from {url}: {err}"
			))
		})?;

	if !response.status().is_success() {
		return Err(crate::ErrorKind::InputError(format!(
			"featured pack download from {url} returned {}",
			response.status()
		))
		.into());
	}

	let total_size = response.content_length().unwrap_or(0);
	let mut stream = response.bytes_stream();
	let mut file = tokio::fs::File::create(dest).await?;
	let mut downloaded = 0_u64;
	let mut last_emitted = 0.0_f64;

	while let Some(chunk) = stream.next().await {
		let chunk = chunk.map_err(|err| {
			crate::ErrorKind::InputError(format!(
				"featured pack download from {url} was interrupted: {err}"
			))
		})?;
		file.write_all(&chunk).await?;
		downloaded += chunk.len() as u64;

		if let Some(bar) = loading_bar {
			let fraction = if total_size > 0 {
				(downloaded as f64 / total_size as f64) * 100.0
			} else {
				last_emitted + 0.1
			};
			let increment = (fraction - last_emitted).max(0.0);
			if increment >= 1.0 || total_size == 0 {
				let _ = emit_loading(bar, increment, None);
				last_emitted += increment;
			}
		}
	}

	file.flush().await?;

	if let Some(bar) = loading_bar {
		let remaining = (100.0 - last_emitted).max(0.0);
		if remaining > 0.0 {
			let _ =
				emit_loading(bar, remaining, Some("Downloaded featured pack"));
		}
	}

	Ok(())
}
