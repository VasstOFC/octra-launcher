//! Offline accounts, CustomSkinLoader, authlib-injector, and the Octra skin VPS.
//!
//! Other players on 1.21+ see skins via CustomSkinLoader + `http://92.5.186.6`.
//! SkinsRestorer uses the same legacy URL: `/skins/MinecraftSkins/{nick}.png`.
//!
//! CustomSkinLoader is injected at launch (not copied into instance modpacks):
//! - Fabric / Quilt: `-Dfabric.addMods=<launcher cache>/CustomSkinLoader.jar`
//! - Forge / NeoForge: ephemeral hardlink in `mods/.octra-customskinloader.jar` (removed on exit)

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use daedalus::modded::LoaderVersion;
use md5::Md5;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
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

const CSL_SLUG: &str = "customskinloader";
const OCTRA_CSL_CACHE_DIR: &str = "octra-csl";
const OCTRA_CSL_EPHEMERAL_MOD: &str = ".octra-customskinloader.jar";

static HUB: OnceLock<SkinHub> = OnceLock::new();

#[derive(Clone)]
struct StoredPlayerSkin {
	uuid: Uuid,
	name: String,
	model: String,
	png: Vec<u8>,
}

struct SkinHub {
    ygg_port: AtomicU16,
    players: Mutex<HashMap<String, StoredPlayerSkin>>,
    textures: Mutex<HashMap<String, Vec<u8>>>,
    started: tokio::sync::Mutex<bool>,
}

impl SkinHub {
    fn new() -> Self {
        Self {
            ygg_port: AtomicU16::new(0),
            players: Mutex::new(HashMap::new()),
            textures: Mutex::new(HashMap::new()),
            started: tokio::sync::Mutex::new(false),
        }
    }
}

fn hub() -> &'static SkinHub {
    HUB.get_or_init(SkinHub::new)
}

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

pub fn ygg_root() -> Option<String> {
    let port = hub().ygg_port.load(Ordering::Relaxed);
    if port == 0 {
        None
    } else {
        Some(format!("http://127.0.0.1:{port}"))
    }
}

fn sha256_hex(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn plain_uuid(id: &Uuid) -> String {
    id.as_simple().to_string()
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
    if !png.is_empty() {
        io::write(dir.join(format!("{uuid}.png")), png).await?;
    }
    register_player(credentials, variant, png);
    let published = publish_to_registry(credentials, variant, png).await;
    let name = credentials.offline_profile.name.clone();
    if png.is_empty() {
        return Ok(());
    }
    if !published {
        tracing::warn!(
            "nie udało się opublikować skina dla {name} na serwerze Octra ({})",
            nervia::SKINS_URL
        );
    } else if !verify_registry_skin(&name).await {
        tracing::warn!(
            "skin dla {name} wysłany, ale {} nie odpowiada — znajomi mogą nie widzieć skina",
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

fn register_player(
    credentials: &Credentials,
    variant: MinecraftSkinVariant,
    png: &[u8],
) {
    if png.is_empty() {
        return;
    }
    let model = match variant {
        MinecraftSkinVariant::Slim => "slim",
        _ => "classic",
    };
    let sha = sha256_hex(png);
    let stored = StoredPlayerSkin {
        uuid: credentials.offline_profile.id,
        name: credentials.offline_profile.name.clone(),
        model: model.to_string(),
        png: png.to_vec(),
    };
    let hub = hub();
    hub.textures.lock().insert(sha, png.to_vec());
    let mut players = hub.players.lock();
    players.insert(plain_uuid(&stored.uuid), stored.clone());
    players.insert(stored.name.to_ascii_lowercase(), stored);
}

fn registry_legacy_url(name: &str) -> String {
    format!(
        "{}/skins/MinecraftSkins/{name}.png",
        nervia::SKINS_URL.trim_end_matches('/')
    )
}

/// Public URL friends' CustomSkinLoader uses to fetch this player's skin.
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
    let url =
        format!("{}/skins/{uuid}", nervia::SKINS_URL.trim_end_matches('/'));
    let send = |method: reqwest::Method| {
        INSECURE_REQWEST_CLIENT
            .request(method, &url)
            .header(reqwest::header::CONTENT_TYPE, "image/png")
            .header("X-Lumen-Model", model)
            .header("X-Lumen-Name", name)
            .header("X-Octra-Key", nervia::SKINS_API_KEY)
            .timeout(Duration::from_secs(12))
            .body(png.to_vec())
            .send()
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
            tracing::warn!("octra skin registry PUT for {name} failed: {error}");
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
        register_player(&credentials, variant, &png);
        publish_to_registry(&credentials, variant, &png).await;
    }
}

async fn load_equipped_png(uuid: Uuid) -> Option<Vec<u8>> {
    let state = State::get().await.ok()?;
    let path = skin_dir(&state.directories).join(format!("{uuid}.png"));
    tokio::fs::read(path).await.ok()
}

pub async fn ensure_runtime() -> crate::Result<()> {
    let hub = hub();
    let mut started = hub.started.lock().await;
    if *started {
        return Ok(());
    }
    start_yggdrasil(hub).await?;
    *started = true;
    tokio::spawn(async {
        sync_all_equipped_skins().await;
        loop {
            tokio::time::sleep(Duration::from_secs(15 * 60)).await;
            sync_all_equipped_skins().await;
        }
    });
    Ok(())
}

async fn start_yggdrasil(hub: &'static SkinHub) -> crate::Result<()> {
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .map_err(|e| {
            crate::ErrorKind::OtherError(format!(
                "failed to bind octra yggdrasil: {e}"
            ))
        })?;
    let port = listener
        .local_addr()
        .map_err(|e| {
            crate::ErrorKind::OtherError(format!(
                "failed to read octra yggdrasil port: {e}"
            ))
        })?
        .port();
    hub.ygg_port.store(port, Ordering::Relaxed);
    tracing::info!("Octra yggdrasil listening on 127.0.0.1:{port}");
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                continue;
            };
            tokio::spawn(async move {
                if let Err(e) = serve_ygg(&mut stream).await {
                    tracing::debug!("octra yggdrasil connection: {e}");
                }
            });
        }
    });
    Ok(())
}

async fn serve_ygg(stream: &mut tokio::net::TcpStream) -> std::io::Result<()> {
    let mut buf = vec![0u8; 16_384];
    let n = stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }
    let req = String::from_utf8_lossy(&buf[..n]);
    let mut lines = req.split("\r\n");
    let first = lines.next().unwrap_or("");
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let path = parts.next().unwrap_or("/");
    let (path, query) = path.split_once('?').unwrap_or((path, ""));
    let body = req.split("\r\n\r\n").nth(1).unwrap_or("");
    let response = dispatch_ygg(method, path, query, body).await;
    stream.write_all(&response).await?;
    Ok(())
}

async fn dispatch_ygg(
    method: &str,
    path: &str,
    query: &str,
    body: &str,
) -> Vec<u8> {
    let segs: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    if method == "OPTIONS" {
        return http_empty(204, "No Content");
    }

    if method == "GET" && (segs.is_empty() || segs == ["index.json"]) {
        return http_json(
            200,
            "OK",
            json!({
                "meta": {
                    "serverName": "Octra",
                    "implementationName": "octra-yggdrasil",
                    "implementationVersion": env!("CARGO_PKG_VERSION"),
                    "feature.non_email_login": true
                },
                "skinDomains": [
                    "127.0.0.1",
                    "localhost",
                    "92.5.186.6"
                ],
            }),
        );
    }

    if method == "GET"
        && segs.len() >= 3
        && segs[0] == "skins"
        && segs[1] == "MinecraftSkins"
    {
        let raw = segs[2].trim_end_matches(".png");
        if let Some(png) = skin_png_for_name(raw).await {
            return http_png(png);
        }
        return http_empty(404, "Not Found");
    }

    if method == "GET" && segs.len() == 2 && segs[0] == "textures" {
        let hash = segs[1].to_ascii_lowercase();
        if let Some(bytes) = hub().textures.lock().get(&hash).cloned() {
            return http_png(bytes);
        }
        return http_empty(404, "Not Found");
    }

    if method == "GET"
        && segs.len() >= 4
        && segs[0] == "sessionserver"
        && segs[1] == "session"
        && segs[2] == "minecraft"
        && segs[3] == "profile"
    {
        let id = segs.get(4).copied().unwrap_or("");
        return player_profile_response(id);
    }

    if method == "GET"
        && segs.len() >= 4
        && segs[0] == "sessionserver"
        && segs[1] == "session"
        && segs[2] == "minecraft"
        && segs[3] == "hasJoined"
    {
        let username = query
            .split('&')
            .find_map(|pair| pair.strip_prefix("username="))
            .unwrap_or("");
        if let Some(skin) = find_player(username) {
            return http_json(200, "OK", profile_json(&skin));
        }
        return http_empty(204, "No Content");
    }

    if method == "POST"
        && segs.len() >= 4
        && segs[0] == "sessionserver"
        && segs[1] == "session"
        && segs[2] == "minecraft"
        && segs[3] == "join"
    {
        return http_empty(204, "No Content");
    }

    if method == "POST" && segs.ends_with(&["api", "profiles", "minecraft"]) {
        let names: Vec<String> = serde_json::from_str(body).unwrap_or_default();
        let list: Vec<serde_json::Value> = names
            .into_iter()
            .filter_map(|name| {
                find_player(&name).map(|skin| {
                    json!({
                        "id": plain_uuid(&skin.uuid),
                        "name": skin.name,
                    })
                })
            })
            .collect();
        return http_json(200, "OK", json!(list));
    }

    if method == "POST"
        && (segs.ends_with(&["authserver", "authenticate"])
            || segs.ends_with(&["authserver", "refresh"])
            || segs.ends_with(&["authserver", "validate"]))
    {
        let name = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| {
                v.get("username")
                    .and_then(|n| n.as_str())
                    .map(ToOwned::to_owned)
            })
            .or_else(|| {
                hub().players.lock().values().next().map(|p| p.name.clone())
            })
            .unwrap_or_else(|| "Player".into());
        let skin = find_player(&name);
        let id = skin
            .as_ref()
            .map(|s| plain_uuid(&s.uuid))
            .unwrap_or_else(|| plain_uuid(&offline_player_uuid(&name)));
        return http_json(
            200,
            "OK",
            json!({
                "accessToken": "0",
                "clientToken": "octra",
                "selectedProfile": { "id": id, "name": name },
                "availableProfiles": [{ "id": id, "name": name }]
            }),
        );
    }

    http_empty(404, "Not Found")
}

fn find_player(key: &str) -> Option<StoredPlayerSkin> {
    let hub = hub();
    let players = hub.players.lock();
    let plain = key.replace('-', "").to_ascii_lowercase();
    players
        .get(&plain)
        .or_else(|| players.get(&key.to_ascii_lowercase()))
        .cloned()
}

fn player_profile_response(id: &str) -> Vec<u8> {
    if let Some(skin) = find_player(id) {
        return http_json(200, "OK", profile_json(&skin));
    }
    let uuid = Uuid::parse_str(&id.replace('-', ""))
        .or_else(|_| Uuid::parse_str(id))
        .unwrap_or_else(|_| offline_player_uuid(id));
    http_json(
        200,
        "OK",
        json!({
            "id": plain_uuid(&uuid),
            "name": "Player",
            "properties": []
        }),
    )
}

fn profile_json(skin: &StoredPlayerSkin) -> serde_json::Value {
    let mut skin_obj = json!({
        "url": registry_legacy_url(&skin.name),
    });
    if skin.model == "slim" {
        skin_obj["metadata"] = json!({ "model": "slim" });
    }
    let textures = json!({
        "timestamp": chrono::Utc::now().timestamp_millis(),
        "profileId": plain_uuid(&skin.uuid),
        "profileName": skin.name,
        "textures": { "SKIN": skin_obj }
    });
    let value = BASE64_STANDARD.encode(textures.to_string().as_bytes());
    json!({
        "id": plain_uuid(&skin.uuid),
        "name": skin.name,
        "properties": [{ "name": "textures", "value": value }]
    })
}

async fn skin_png_for_name(name: &str) -> Option<Vec<u8>> {
    if let Some(skin) = find_player(name) {
        return Some(skin.png);
    }
    let url = format!(
        "{}/skins/MinecraftSkins/{name}.png",
        nervia::SKINS_URL.trim_end_matches('/')
    );
    let resp = INSECURE_REQWEST_CLIENT
        .get(&url)
        .timeout(Duration::from_secs(8))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.bytes().await.ok().map(|b| b.to_vec())
}

fn http_json(status: u16, reason: &str, body: serde_json::Value) -> Vec<u8> {
    let body = body.to_string();
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        body.len()
    );
    let mut out = header.into_bytes();
    out.extend_from_slice(body.as_bytes());
    out
}

fn http_png(png: Vec<u8>) -> Vec<u8> {
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        png.len()
    );
    let mut out = header.into_bytes();
    out.extend_from_slice(&png);
    out
}

fn http_empty(status: u16, reason: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Length: 0\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n"
    )
    .into_bytes()
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

pub async fn overlay_fabric_if_vanilla(
    game_version: &str,
    loader: ModLoader,
    loader_version: Option<LoaderVersion>,
) -> (ModLoader, Option<LoaderVersion>) {
    if loader != ModLoader::Vanilla {
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
                "Octra skins: overlaying Fabric {} on vanilla {game_version} for CustomSkinLoader",
                fabric.id
            );
            (ModLoader::Fabric, Some(fabric))
        }
        Ok(None) => {
            tracing::debug!("Octra skins: no Fabric loader for {game_version}");
            (loader, loader_version)
        }
        Err(e) => {
            tracing::warn!("Octra skins: Fabric overlay failed: {e}");
            (loader, loader_version)
        }
    }
}

pub async fn prepare_launch(
    instance_path: &Path,
    game_version: &str,
    loader: ModLoader,
    credentials: &Credentials,
    java_args: &mut Vec<String>,
) -> crate::Result<()> {
    if let Err(e) = ensure_runtime().await {
        tracing::warn!("Octra skins runtime: {e}");
    }

    if let Some(png) = load_equipped_png(credentials.offline_profile.id).await {
        let variant = load_equipped(credentials.offline_profile.id)
            .await
            .map(|e| e.variant)
            .unwrap_or(MinecraftSkinVariant::Classic);
        register_player(credentials, variant, &png);
        publish_to_registry(credentials, variant, &png).await;
        write_local_csl_skin(
            instance_path,
            &credentials.offline_profile.name,
            &png,
        )
        .await?;
    }

    if let (Ok(jar), Some(root)) = (authlib_jar_path().await, ygg_root()) {
        let jar = dunce::canonicalize(&jar).unwrap_or(jar);
        java_args.insert(
            0,
            format!(
                "-javaagent:{}={}",
                jar.display(),
                root.trim_end_matches('/')
            ),
        );
    }

    if matches!(
        loader,
        ModLoader::Fabric
            | ModLoader::Quilt
            | ModLoader::Forge
            | ModLoader::NeoForge
    ) {
        remove_legacy_csl_from_mods(instance_path).await;
        match resolve_custom_skin_loader_jar(game_version, loader).await {
            Ok(jar) => {
                if let Err(e) = inject_custom_skin_loader(
                    instance_path,
                    &jar,
                    loader,
                    java_args,
                )
                .await
                {
                    tracing::warn!("Octra skins: CustomSkinLoader inject: {e}");
                } else {
                    tracing::info!(
                        "Octra skins: injected CustomSkinLoader for {game_version} ({})",
                        loader.as_str()
                    );
                }
            }
            Err(e) => tracing::warn!("Octra skins: CustomSkinLoader: {e}"),
        }
        write_csl_config(instance_path)?;
        if let Err(e) = sync_username_skin_files(instance_path).await {
            tracing::warn!("Octra skins: sync local skin files: {e}");
        }
    } else {
        tracing::warn!(
            "Octra skins: instancja vanilla bez Fabric — CustomSkinLoader nie zostanie wstrzyknięty; znajomi nie zobaczą skina w multiplayerze"
        );
    }

    Ok(())
}

/// Removes ephemeral Forge/NeoForge CSL hardlink after Minecraft exits.
pub async fn cleanup_ephemeral_csl(instance_path: &Path) {
    let ephemeral = ephemeral_csl_mod_path(instance_path);
    if ephemeral.exists() {
        let _ = tokio::fs::remove_file(ephemeral).await;
    }
}

fn ephemeral_csl_mod_path(instance_path: &Path) -> PathBuf {
    instance_path.join("mods").join(OCTRA_CSL_EPHEMERAL_MOD)
}

async fn remove_legacy_csl_from_mods(instance_path: &Path) {
    let mods = instance_path.join("mods");
    let Ok(mut dir) = tokio::fs::read_dir(&mods).await else {
        return;
    };
    while let Ok(Some(entry)) = dir.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if name.contains("customskinloader")
            && name != OCTRA_CSL_EPHEMERAL_MOD.to_ascii_lowercase()
        {
            let _ = tokio::fs::remove_file(entry.path()).await;
        }
    }
}

async fn inject_custom_skin_loader(
    instance_path: &Path,
    jar: &Path,
    loader: ModLoader,
    java_args: &mut Vec<String>,
) -> crate::Result<()> {
    let jar_path = dunce::canonicalize(jar).unwrap_or_else(|_| jar.to_path_buf());

    match loader {
        ModLoader::Fabric | ModLoader::Quilt => {
            java_args.push(format!(
                "-Dfabric.addMods={}",
                jar_path.display()
            ));
            Ok(())
        }
        ModLoader::Forge | ModLoader::NeoForge => {
            cleanup_ephemeral_csl(instance_path).await;
            let mods = instance_path.join("mods");
            io::create_dir_all(&mods).await?;
            let dest = ephemeral_csl_mod_path(instance_path);
            if dest.exists() {
                let _ = tokio::fs::remove_file(&dest).await;
            }
            match std::fs::hard_link(&jar_path, &dest) {
                Ok(()) => Ok(()),
                Err(_) => {
                    tokio::fs::copy(&jar_path, &dest).await?;
                    Ok(())
                }
            }
        }
        _ => Ok(()),
    }
}

async fn write_local_csl_skin(
    instance_path: &Path,
    name: &str,
    png: &[u8],
) -> crate::Result<()> {
    let dir = instance_path
        .join("CustomSkinLoader")
        .join("LocalSkin")
        .join("skins");
    io::create_dir_all(&dir).await?;
    io::write(dir.join(format!("{name}.png")), png).await?;
    Ok(())
}

fn write_csl_config(instance_path: &Path) -> crate::Result<()> {
    let cfg_dir = instance_path.join("CustomSkinLoader");
    std::fs::create_dir_all(&cfg_dir)?;
    let registry = nervia::SKINS_URL.trim_end_matches('/');
    let mut loadlist = vec![json!({
        "name": "OctraCloud",
        "type": "Legacy",
        "root": format!("{registry}/skins/MinecraftSkins/")
    })];
    if let Some(root) = ygg_root() {
        loadlist.push(json!({
            "name": "OctraYgg",
            "type": "Legacy",
            "root": format!("{}/skins/MinecraftSkins/", root.trim_end_matches('/'))
        }));
    }
    loadlist.push(json!({
        "name": "OctraLocal",
        "type": "Legacy",
        "checkPNG": false,
        "skin": "LocalSkin/skins/{USERNAME}.png",
        "model": "auto"
    }));
    loadlist.push(json!({
        "name": "Mojang",
        "type": "MojangAPI"
    }));
    let cfg = json!({
        "version": "15.0",
        "enable": true,
        "loadlist": loadlist,
    });
    std::fs::write(
        cfg_dir.join("CustomSkinLoader.json"),
        serde_json::to_string_pretty(&cfg)?,
    )?;
    let mut skinurls = format!("{registry}/skins/MinecraftSkins/*.png\n");
    if let Some(root) = ygg_root() {
        skinurls.push_str(&format!(
            "{}/skins/MinecraftSkins/*.png\n",
            root.trim_end_matches('/')
        ));
    }
    std::fs::write(cfg_dir.join("skinurls.txt"), skinurls)?;
    Ok(())
}

async fn sync_username_skin_files(instance_path: &Path) -> crate::Result<()> {
    let state = State::get().await?;
    let accounts = Credentials::get_all(&state.pool).await?;
    let dir = instance_path
        .join("CustomSkinLoader")
        .join("LocalSkin")
        .join("skins");
    io::create_dir_all(&dir).await?;
    for entry in accounts.iter() {
        let credentials = entry.value();
        let name = credentials.offline_profile.name.trim();
        if name.is_empty() {
            continue;
        }
        let Some(png) = load_equipped_png(credentials.offline_profile.id).await else {
            continue;
        };
        io::write(dir.join(format!("{name}.png")), &png).await?;
    }
    Ok(())
}

#[derive(Deserialize)]
struct ModrinthVersion {
    #[serde(default)]
    files: Vec<ModrinthFile>,
}

#[derive(Deserialize)]
struct ModrinthFile {
    filename: String,
    url: String,
    #[serde(default, rename = "primary")]
    is_primary: bool,
}

async fn resolve_custom_skin_loader_jar(
    game_version: &str,
    loader: ModLoader,
) -> crate::Result<PathBuf> {
    let loader_param = match loader {
        ModLoader::Quilt => "quilt",
        ModLoader::Forge => "forge",
        ModLoader::NeoForge => "neoforge",
        _ => "fabric",
    };
    let url = format!(
        "https://api.modrinth.com/v2/project/{CSL_SLUG}/version?game_versions=[\"{game_version}\"]&loaders=[\"{loader_param}\"]"
    );
    let resp = INSECURE_REQWEST_CLIENT
        .get(&url)
        .header("User-Agent", crate::launcher_user_agent())
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| {
            crate::ErrorKind::OtherError(format!(
                "CustomSkinLoader request failed: {e}"
            ))
        })?;
    if !resp.status().is_success() {
        return Err(crate::ErrorKind::OtherError(format!(
            "CustomSkinLoader HTTP {}",
            resp.status()
        ))
        .into());
    }
    let versions: Vec<ModrinthVersion> = resp.json().await.map_err(|e| {
        crate::ErrorKind::OtherError(format!("CustomSkinLoader JSON: {e}"))
    })?;
    let version = versions
        .into_iter()
        .find(|v| !v.files.is_empty())
        .ok_or_else(|| {
            crate::ErrorKind::OtherError(format!(
                "no CustomSkinLoader for Minecraft {game_version} ({loader_param})"
            ))
        })?;
    let file = version
        .files
        .iter()
        .find(|f| f.is_primary)
        .or_else(|| version.files.first())
        .ok_or_else(|| {
            crate::ErrorKind::OtherError(
                "CustomSkinLoader version has no files".to_string(),
            )
        })?;

    let state = State::get().await?;
    let cache = state
        .directories
        .caches_dir()
        .join(OCTRA_CSL_CACHE_DIR)
        .join(loader_param)
        .join(game_version);
    io::create_dir_all(&cache).await?;
    let cached = cache.join(&file.filename);
    if !cached.exists() {
        tracing::info!(
            "Octra skins: downloading CustomSkinLoader for {game_version} ({loader_param})"
        );
        let bytes = INSECURE_REQWEST_CLIENT
            .get(&file.url)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| {
                crate::ErrorKind::OtherError(format!(
                    "CustomSkinLoader download: {e}"
                ))
            })?
            .bytes()
            .await
            .map_err(|e| {
                crate::ErrorKind::OtherError(format!(
                    "CustomSkinLoader body: {e}"
                ))
            })?;
        io::write(&cached, &bytes).await?;
    }
    Ok(cached)
}
