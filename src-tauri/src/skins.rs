//! Lokalne skiny kont offline Lumen (PNG + model). Konta Microsoft zostają przy Mojang.

use std::path::{Path, PathBuf};

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::auth::{self, hyphenate_uuid, plain_uuid};
use crate::error::{Error, Result};
use crate::paths::Dirs;

pub const AUTHLIB_VERSION: &str = "1.2.8";
pub const AUTHLIB_SHA256: &str = "9c7f4343e6c82034958ffb48c14a2cb0c85928be7283103ce17da00c6d5a7b10";
const AUTHLIB_URL: &str =
    "https://authlib-injector.yushi.moe/artifact/56/authlib-injector-1.2.8.jar";
const AUTHLIB_FALLBACK: &str =
    "https://github.com/yushijinhun/authlib-injector/releases/download/v1.2.8/authlib-injector-1.2.8.jar";
const AUTHLIB_LATEST: &str = "https://authlib-injector.yushi.moe/artifact/latest.json";
/// Wbudowany jar — fallback gdy pobieranie z sieci się nie uda (offline skiny wymagają javaagenta).
const AUTHLIB_EMBEDDED: &[u8] = include_bytes!("../resources/authlib-injector-1.2.8.jar");
const MAX_PNG: usize = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkinModel {
    Classic,
    Slim,
}

impl SkinModel {
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "slim" | "alex" => Ok(Self::Slim),
            "classic" | "default" | "steve" | "" => Ok(Self::Classic),
            _ => Err(Error::msg("Model skina: classic albo slim.")),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Slim => "slim",
        }
    }

    pub fn ygg_metadata(&self) -> Option<&'static str> {
        match self {
            Self::Slim => Some("slim"),
            Self::Classic => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinMeta {
    pub model: SkinModel,
    pub uploaded_at: String,
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineSkin {
    pub uuid: String,
    pub model: String,
    pub png_base64: Option<String>,
    pub uploaded_at: Option<String>,
    pub has_custom: bool,
}

pub fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex::encode(h.finalize())
}

pub fn png_dimensions(data: &[u8]) -> Result<(u32, u32)> {
    if data.len() < 24 || data.get(0..8) != Some(b"\x89PNG\r\n\x1a\n") {
        return Err(Error::msg("To nie jest plik PNG."));
    }
    if data.get(12..16) != Some(b"IHDR") {
        return Err(Error::msg("Niepoprawny PNG (brak IHDR)."));
    }
    let w = u32::from_be_bytes(data[16..20].try_into().unwrap());
    let h = u32::from_be_bytes(data[20..24].try_into().unwrap());
    Ok((w, h))
}

pub fn validate_skin_png(data: &[u8]) -> Result<(u32, u32)> {
    if data.len() > MAX_PNG {
        return Err(Error::msg("Skin jest za duży (max 1 MB)."));
    }
    let (w, h) = png_dimensions(data)?;
    let ok = matches!((w, h), (64, 32) | (64, 64) | (64, 128) | (128, 64) | (128, 128))
        || (w >= 64 && h >= 32 && w % 64 == 0 && h % 32 == 0 && w <= 128 && h <= 128);
    if !ok {
        return Err(Error::msg(
            "Skin Minecraft: PNG 64×64 (albo 64×32 / 64×128). Inny rozmiar nie zadziała.",
        ));
    }
    Ok((w, h))
}

fn safe_uuid(uuid: &str) -> Result<String> {
    let id = hyphenate_uuid(uuid);
    let plain = plain_uuid(&id);
    if plain.len() != 32 || !plain.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::msg("Niepoprawne UUID."));
    }
    Ok(id)
}

pub fn skin_png_path(dirs: &Dirs, uuid: &str) -> Result<PathBuf> {
    Ok(dirs.skins_dir().join(format!("{}.png", safe_uuid(uuid)?)))
}

pub fn skin_meta_path(dirs: &Dirs, uuid: &str) -> Result<PathBuf> {
    Ok(dirs.skins_dir().join(format!("{}.json", safe_uuid(uuid)?)))
}

pub fn cache_png_path(dirs: &Dirs, uuid: &str) -> Result<PathBuf> {
    Ok(dirs
        .skins_cache_dir()
        .join(format!("{}.png", safe_uuid(uuid)?)))
}

pub fn cache_meta_path(dirs: &Dirs, uuid: &str) -> Result<PathBuf> {
    Ok(dirs
        .skins_cache_dir()
        .join(format!("{}.json", safe_uuid(uuid)?)))
}

pub fn load_meta(path: &Path) -> Option<SkinMeta> {
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn require_offline_account(dirs: &Dirs, uuid: &str) -> Result<auth::Account> {
    let id = safe_uuid(uuid)?;
    let file = auth::load_accounts(dirs)?;
    let acc = file
        .accounts
        .iter()
        .find(|a| plain_uuid(&a.uuid) == plain_uuid(&id))
        .cloned()
        .ok_or_else(|| Error::msg("Nie znaleziono konta."))?;
    if !acc.is_offline() {
        return Err(Error::msg(
            "Konta Microsoft używają oficjalnego skina Mojang — Lumen go nie podmienia.",
        ));
    }
    Ok(acc)
}

pub fn get_offline_skin(dirs: &Dirs, uuid: &str) -> Result<OfflineSkin> {
    let acc = require_offline_account(dirs, uuid)?;
    let png_path = skin_png_path(dirs, &acc.uuid)?;
    let meta = load_meta(&skin_meta_path(dirs, &acc.uuid)?);
    if png_path.exists() {
        let bytes = std::fs::read(&png_path)?;
        let model = meta
            .as_ref()
            .map(|m| m.model.as_str().to_string())
            .unwrap_or_else(|| "classic".into());
        return Ok(OfflineSkin {
            uuid: acc.uuid,
            model,
            png_base64: Some(base64::engine::general_purpose::STANDARD.encode(&bytes)),
            uploaded_at: meta.map(|m| m.uploaded_at),
            has_custom: true,
        });
    }
    Ok(OfflineSkin {
        uuid: acc.uuid,
        model: "classic".into(),
        png_base64: None,
        uploaded_at: None,
        has_custom: false,
    })
}

pub fn save_offline_skin(
    dirs: &Dirs,
    uuid: &str,
    png: &[u8],
    model: &str,
) -> Result<(OfflineSkin, Vec<u8>, String)> {
    let acc = require_offline_account(dirs, uuid)?;
    validate_skin_png(png)?;
    let model = SkinModel::parse(model)?;
    dirs.ensure()?;
    let hash = sha256_hex(png);
    let meta = SkinMeta {
        model: model.clone(),
        uploaded_at: chrono::Utc::now().to_rfc3339(),
        sha256: hash.clone(),
        name: acc.name.clone(),
        source: "local".into(),
    };
    std::fs::write(skin_png_path(dirs, &acc.uuid)?, png)?;
    std::fs::write(
        skin_meta_path(dirs, &acc.uuid)?,
        serde_json::to_string_pretty(&meta)?,
    )?;
    let info = OfflineSkin {
        uuid: acc.uuid,
        model: model.as_str().into(),
        png_base64: Some(base64::engine::general_purpose::STANDARD.encode(png)),
        uploaded_at: Some(meta.uploaded_at),
        has_custom: true,
    };
    Ok((info, png.to_vec(), hash))
}

pub fn set_offline_skin_model(dirs: &Dirs, uuid: &str, model: &str) -> Result<OfflineSkin> {
    let acc = require_offline_account(dirs, uuid)?;
    let model = SkinModel::parse(model)?;
    let png_path = skin_png_path(dirs, &acc.uuid)?;
    if !png_path.exists() {
        return get_offline_skin(dirs, &acc.uuid);
    }
    let bytes = std::fs::read(&png_path)?;
    let mut meta = load_meta(&skin_meta_path(dirs, &acc.uuid)?).unwrap_or(SkinMeta {
        model: model.clone(),
        uploaded_at: chrono::Utc::now().to_rfc3339(),
        sha256: sha256_hex(&bytes),
        name: acc.name.clone(),
        source: "local".into(),
    });
    meta.model = model;
    meta.name = acc.name.clone();
    std::fs::write(
        skin_meta_path(dirs, &acc.uuid)?,
        serde_json::to_string_pretty(&meta)?,
    )?;
    get_offline_skin(dirs, &acc.uuid)
}

pub fn reset_offline_skin(dirs: &Dirs, uuid: &str) -> Result<()> {
    let acc = require_offline_account(dirs, uuid)?;
    let png = skin_png_path(dirs, &acc.uuid)?;
    let meta = skin_meta_path(dirs, &acc.uuid)?;
    let _ = std::fs::remove_file(png);
    let _ = std::fs::remove_file(meta);
    Ok(())
}

#[derive(Clone, Debug)]
pub struct StoredSkin {
    pub uuid: String,
    pub name: String,
    pub model: SkinModel,
    pub sha256: String,
    pub png: Vec<u8>,
}

pub fn load_local_skin(dirs: &Dirs, uuid: &str) -> Option<StoredSkin> {
    let id = safe_uuid(uuid).ok()?;
    let png_path = skin_png_path(dirs, &id).ok()?;
    if !png_path.exists() {
        return None;
    }
    let png = std::fs::read(png_path).ok()?;
    let meta = load_meta(&skin_meta_path(dirs, &id).ok()?);
    Some(StoredSkin {
        uuid: id,
        name: meta
            .as_ref()
            .map(|m| m.name.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_default(),
        model: meta
            .as_ref()
            .map(|m| m.model.clone())
            .unwrap_or(SkinModel::Classic),
        sha256: meta
            .as_ref()
            .map(|m| m.sha256.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| sha256_hex(&png)),
        png,
    })
}

pub fn load_cached_skin(dirs: &Dirs, uuid: &str) -> Option<StoredSkin> {
    let id = safe_uuid(uuid).ok()?;
    let png_path = cache_png_path(dirs, &id).ok()?;
    if !png_path.exists() {
        return None;
    }
    let png = std::fs::read(png_path).ok()?;
    let meta = load_meta(&cache_meta_path(dirs, &id).ok()?);
    Some(StoredSkin {
        uuid: id,
        name: meta
            .as_ref()
            .map(|m| m.name.clone())
            .unwrap_or_default(),
        model: meta
            .as_ref()
            .map(|m| m.model.clone())
            .unwrap_or(SkinModel::Classic),
        sha256: meta
            .as_ref()
            .map(|m| m.sha256.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| sha256_hex(&png)),
        png,
    })
}

pub fn write_cached_skin(dirs: &Dirs, skin: &StoredSkin) -> Result<()> {
    dirs.ensure()?;
    std::fs::write(cache_png_path(dirs, &skin.uuid)?, &skin.png)?;
    let meta = SkinMeta {
        model: skin.model.clone(),
        uploaded_at: chrono::Utc::now().to_rfc3339(),
        sha256: skin.sha256.clone(),
        name: skin.name.clone(),
        source: "cache".into(),
    };
    std::fs::write(
        cache_meta_path(dirs, &skin.uuid)?,
        serde_json::to_string_pretty(&meta)?,
    )?;
    Ok(())
}

/// Zapis skina z rejestru LAN (`PUT /skins/{uuid}`) — bez wymogu konta offline.
pub fn store_registry_skin(
    dirs: &Dirs,
    uuid: &str,
    png: &[u8],
    model: &str,
    name: &str,
) -> Result<StoredSkin> {
    let id = safe_uuid(uuid)?;
    validate_skin_png(png)?;
    let model = SkinModel::parse(model)?;
    dirs.ensure()?;
    let hash = sha256_hex(png);
    let display = if name.trim().is_empty() {
        load_meta(&skin_meta_path(dirs, &id)?)
            .map(|m| m.name)
            .unwrap_or_default()
    } else {
        name.trim().to_string()
    };
    let meta = SkinMeta {
        model: model.clone(),
        uploaded_at: chrono::Utc::now().to_rfc3339(),
        sha256: hash.clone(),
        name: display.clone(),
        source: "registry".into(),
    };
    std::fs::write(skin_png_path(dirs, &id)?, png)?;
    std::fs::write(
        skin_meta_path(dirs, &id)?,
        serde_json::to_string_pretty(&meta)?,
    )?;
    Ok(StoredSkin {
        uuid: id,
        name: display,
        model,
        sha256: hash,
        png: png.to_vec(),
    })
}

/// Skin zapisany w Szafie dla konta offline o podanym nicku (bez lookupu Mojang).
pub fn local_skin_by_username(dirs: &Dirs, name: &str) -> Option<StoredSkin> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let file = auth::load_accounts(dirs).ok()?;
    let account = file.accounts.iter().find(|a| {
        a.is_offline() && a.name.eq_ignore_ascii_case(name)
    })?;
    let mut skin = load_local_skin(dirs, &account.uuid)?;
    if skin.name.is_empty() {
        skin.name = account.name.clone();
    }
    Some(skin)
}

pub fn list_local_custom_skins(dirs: &Dirs) -> Vec<StoredSkin> {
    let Ok(file) = auth::load_accounts(dirs) else {
        return Vec::new();
    };
    file.accounts
        .into_iter()
        .filter(|a| a.is_offline())
        .filter_map(|a| {
            let mut s = load_local_skin(dirs, &a.uuid)?;
            if s.name.is_empty() {
                s.name = a.name;
            }
            Some(s)
        })
        .collect()
}

/// Wszystkie skiny do ogłoszenia w LAN gossip (offline, cache, premium aliasy).
pub fn list_gossip_skins(dirs: &Dirs) -> Vec<StoredSkin> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    let mut push = |skin: StoredSkin| {
        if skin.sha256.is_empty() || !seen.insert(skin.sha256.clone()) {
            return;
        }
        out.push(skin);
    };
    for s in list_local_custom_skins(dirs) {
        push(s);
    }
    for dir in [dirs.skins_dir(), dirs.skins_cache_dir()] {
        let Ok(rd) = std::fs::read_dir(dir) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("png") {
                continue;
            }
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let id = hyphenate_uuid(stem);
            if let Some(s) = load_local_skin(dirs, &id).or_else(|| load_cached_skin(dirs, &id)) {
                push(s);
            }
        }
    }
    out
}

pub fn index_texture_bytes(dirs: &Dirs) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for dir in [dirs.skins_dir(), dirs.skins_cache_dir()] {
        let Ok(rd) = std::fs::read_dir(dir) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("png") {
                continue;
            }
            if let Ok(bytes) = std::fs::read(&p) {
                out.push((sha256_hex(&bytes), bytes));
            }
        }
    }
    out
}

fn write_authlib_notice(dir: &Path) {
    let notice = dir.join("AUTHLIB-INJECTOR-NOTICE.txt");
    let _ = std::fs::write(
        notice,
        format!(
            "authlib-injector {AUTHLIB_VERSION} (AGPL-3.0) — yushijinhun\n\
         Źródło i licencja: https://github.com/yushijinhun/authlib-injector\n\
         Lumen pobiera jar przy pierwszym uruchomieniu gry; nie jest to ely.by.\n"
        ),
    );
}

#[derive(Deserialize)]
struct AuthlibLatest {
    download_url: Option<String>,
    checksums: Option<AuthlibSums>,
}

#[derive(Deserialize)]
struct AuthlibSums {
    sha256: Option<String>,
}

pub fn authlib_jar_path(dirs: &Dirs) -> PathBuf {
    dirs.meta_dir().join("authlib-injector.jar")
}

fn install_embedded_authlib(dirs: &Dirs) -> Result<PathBuf> {
    let hash = sha256_hex(AUTHLIB_EMBEDDED);
    if !hash.eq_ignore_ascii_case(AUTHLIB_SHA256) {
        return Err(Error::msg("Wbudowany authlib-injector ma niepoprawny skrót SHA-256."));
    }
    let dest = authlib_jar_path(dirs);
    std::fs::write(&dest, AUTHLIB_EMBEDDED)?;
    Ok(dest)
}

fn authlib_jar_valid(path: &Path) -> bool {
    std::fs::read(path)
        .ok()
        .is_some_and(|buf| {
            sha256_hex(&buf).eq_ignore_ascii_case(AUTHLIB_SHA256) || buf.len() > 50_000
        })
}

pub async fn ensure_authlib_injector(client: &reqwest::Client, dirs: &Dirs) -> Result<PathBuf> {
    dirs.ensure()?;
    write_authlib_notice(&dirs.meta_dir());
    let dest = authlib_jar_path(dirs);
    if dest.exists() && authlib_jar_valid(&dest) {
        return Ok(dest);
    }

    let mut url = AUTHLIB_URL.to_string();
    let mut expect = AUTHLIB_SHA256.to_string();
    if let Ok(resp) = client.get(AUTHLIB_LATEST).send().await {
        if let Ok(info) = resp.json::<AuthlibLatest>().await {
            if let Some(u) = info.download_url {
                url = u;
            }
            if let Some(s) = info.checksums.and_then(|c| c.sha256) {
                expect = s;
            }
        }
    }

    let mut last_err = None;
    for u in [url.as_str(), AUTHLIB_FALLBACK] {
        match crate::download::download_file(client, u, &dest, None, None, None).await {
            Ok(()) => {
                if let Ok(buf) = std::fs::read(&dest) {
                    let hash = sha256_hex(&buf);
                    if !expect.is_empty() && !hash.eq_ignore_ascii_case(&expect) {
                        let _ = std::fs::remove_file(&dest);
                        last_err = Some(Error::msg("Suma SHA-256 authlib-injector się nie zgadza."));
                        continue;
                    }
                    return Ok(dest);
                }
            }
            Err(e) => last_err = Some(e),
        }
    }

    install_embedded_authlib(dirs).or_else(|embedded_err| {
        Err(last_err.unwrap_or(embedded_err))
    })
}
