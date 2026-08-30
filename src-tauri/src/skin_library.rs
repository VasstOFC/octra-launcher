//! Biblioteki skinów Szafy — wiele zapisanych profili na konto offline / premium.

use std::path::{Path, PathBuf};

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::{self, plain_uuid};
use crate::error::{Error, Result};
use crate::paths::Dirs;
use crate::skins::{self, SkinModel, sha256_hex, validate_skin_png};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinLibraryEntry {
    pub id: String,
    pub name: String,
    pub model: SkinModel,
    pub created_at: String,
    /// upload | catalog | mojang
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub png_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cape_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct SkinLibraryIndex {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    active_id: Option<String>,
    #[serde(default)]
    entries: Vec<SkinLibraryEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinLibraryEntryView {
    pub id: String,
    pub name: String,
    pub model: String,
    pub created_at: String,
    pub source: String,
    pub texture_key: Option<String>,
    pub cape_id: Option<String>,
    pub png_base64: Option<String>,
    pub is_active: bool,
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

fn offline_library_dir(dirs: &Dirs, uuid: &str) -> Result<PathBuf> {
    let id = skins::skin_png_path(dirs, uuid)?;
    let account_uuid = id
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::msg("Niepoprawne UUID."))?;
    Ok(dirs.skins_dir().join("offline-library").join(account_uuid))
}

fn premium_library_dir(dirs: &Dirs, uuid: &str) -> Result<PathBuf> {
    let id = skins::skin_png_path(dirs, uuid)?;
    let account_uuid = id
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::msg("Niepoprawne UUID."))?;
    Ok(dirs.skins_dir().join("premium-library").join(account_uuid))
}

fn index_path(base: &Path) -> PathBuf {
    base.join("index.json")
}

fn load_index(path: &Path) -> SkinLibraryIndex {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_index(path: &Path, index: &SkinLibraryIndex) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(index)?)?;
    Ok(())
}

fn entry_png_path(base: &Path, entry: &SkinLibraryEntry) -> Option<PathBuf> {
    entry
        .png_file
        .as_ref()
        .map(|f| base.join(f))
}

fn png_to_view(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    Some(base64::engine::general_purpose::STANDARD.encode(bytes))
}

fn entry_to_view(entry: &SkinLibraryEntry, base: &Path, active_id: Option<&str>) -> SkinLibraryEntryView {
    let png_base64 = entry_png_path(base, entry).and_then(|p| png_to_view(&p));
    SkinLibraryEntryView {
        id: entry.id.clone(),
        name: entry.name.clone(),
        model: entry.model.as_str().into(),
        created_at: entry.created_at.clone(),
        source: entry.source.clone(),
        texture_key: entry.texture_key.clone(),
        cape_id: entry.cape_id.clone(),
        png_base64,
        is_active: active_id == Some(entry.id.as_str()),
    }
}

fn require_offline(dirs: &Dirs, uuid: &str) -> Result<auth::Account> {
    skins::require_offline_account(dirs, uuid)
}

fn require_premium(dirs: &Dirs, uuid: &str) -> Result<auth::Account> {
    let file = auth::load_accounts(dirs)?;
    let id = plain_uuid(uuid);
    file.accounts
        .into_iter()
        .find(|a| plain_uuid(&a.uuid) == id && !a.is_offline())
        .ok_or_else(|| Error::msg("Nie znaleziono konta premium."))
}

// ── Offline ──────────────────────────────────────────────────────────────────

pub fn list_offline_library(dirs: &Dirs, uuid: &str) -> Result<Vec<SkinLibraryEntryView>> {
    let _acc = require_offline(dirs, uuid)?;
    let base = offline_library_dir(dirs, uuid)?;
    let idx_path = index_path(&base);
    let index = load_index(&idx_path);
    Ok(index
        .entries
        .iter()
        .map(|e| entry_to_view(e, &base, index.active_id.as_deref()))
        .collect())
}

pub fn add_offline_library_entry(
    dirs: &Dirs,
    uuid: &str,
    png: &[u8],
    model: &str,
    name: &str,
) -> Result<SkinLibraryEntryView> {
    let acc = require_offline(dirs, uuid)?;
    validate_skin_png(png)?;
    let model = SkinModel::parse(model)?;
    dirs.ensure()?;
    let base = offline_library_dir(dirs, uuid)?;
    std::fs::create_dir_all(&base)?;
    let idx_path = index_path(&base);
    let mut index = load_index(&idx_path);

    let id = new_id();
    let file_name = format!("{id}.png");
    std::fs::write(base.join(&file_name), png)?;

    let display = if name.trim().is_empty() {
        format!("Skin {}", index.entries.len() + 1)
    } else {
        name.trim().to_string()
    };

    let entry = SkinLibraryEntry {
        id: id.clone(),
        name: display,
        model,
        created_at: chrono::Utc::now().to_rfc3339(),
        source: "upload".into(),
        texture_key: None,
        png_file: Some(file_name),
        cape_id: None,
    };
    index.entries.push(entry.clone());
    save_index(&idx_path, &index)?;

    let _ = acc;
    Ok(entry_to_view(&entry, &base, index.active_id.as_deref()))
}

pub fn equip_offline_library_entry(
    dirs: &Dirs,
    uuid: &str,
    entry_id: &str,
) -> Result<skins::OfflineSkin> {
    let _acc = require_offline(dirs, uuid)?;
    let base = offline_library_dir(dirs, uuid)?;
    let idx_path = index_path(&base);
    let mut index = load_index(&idx_path);
    let entry = index
        .entries
        .iter()
        .find(|e| e.id == entry_id)
        .cloned()
        .ok_or_else(|| Error::msg("Nie znaleziono skina w bibliotece."))?;
    let png_path = entry_png_path(&base, &entry)
        .ok_or_else(|| Error::msg("Ten wpis nie ma pliku PNG."))?;
    let png = std::fs::read(&png_path)?;
    let (info, _, _) = skins::save_offline_skin(dirs, uuid, &png, entry.model.as_str())?;
    index.active_id = Some(entry_id.to_string());
    save_index(&idx_path, &index)?;
    Ok(info)
}

pub fn delete_offline_library_entry(dirs: &Dirs, uuid: &str, entry_id: &str) -> Result<()> {
    let _acc = require_offline(dirs, uuid)?;
    let base = offline_library_dir(dirs, uuid)?;
    let idx_path = index_path(&base);
    let mut index = load_index(&idx_path);
    let pos = index
        .entries
        .iter()
        .position(|e| e.id == entry_id)
        .ok_or_else(|| Error::msg("Nie znaleziono skina w bibliotece."))?;
    let entry = index.entries.remove(pos);
    if let Some(p) = entry_png_path(&base, &entry) {
        let _ = std::fs::remove_file(p);
    }
    if index.active_id.as_deref() == Some(entry_id) {
        index.active_id = None;
    }
    save_index(&idx_path, &index)?;
    Ok(())
}

pub fn set_offline_library_entry_model(
    dirs: &Dirs,
    uuid: &str,
    entry_id: &str,
    model: &str,
) -> Result<SkinLibraryEntryView> {
    let _acc = require_offline(dirs, uuid)?;
    let model = SkinModel::parse(model)?;
    let base = offline_library_dir(dirs, uuid)?;
    let idx_path = index_path(&base);
    let mut index = load_index(&idx_path);
    let entry = index
        .entries
        .iter_mut()
        .find(|e| e.id == entry_id)
        .ok_or_else(|| Error::msg("Nie znaleziono skina w bibliotece."))?;
    entry.model = model;
    let view = entry_to_view(entry, &base, index.active_id.as_deref());
    save_index(&idx_path, &index)?;
    Ok(view)
}

// ── Premium ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePremiumLibraryReq {
    pub name: String,
    pub variant: String,
    pub source: String,
    pub texture_key: Option<String>,
    pub cape_id: Option<String>,
    pub png: Option<Vec<u8>>,
    #[serde(default)]
    pub mark_active: bool,
}

pub fn list_premium_library(dirs: &Dirs, uuid: &str) -> Result<Vec<SkinLibraryEntryView>> {
    let _acc = require_premium(dirs, uuid)?;
    let base = premium_library_dir(dirs, uuid)?;
    let idx_path = index_path(&base);
    let index = load_index(&idx_path);
    Ok(index
        .entries
        .iter()
        .map(|e| entry_to_view(e, &base, index.active_id.as_deref()))
        .collect())
}

fn find_premium_skin_entry_id(
    base: &Path,
    index: &SkinLibraryIndex,
    texture_key: Option<&str>,
    variant: &str,
    png: Option<&[u8]>,
) -> Option<String> {
    let png_hash = png.map(sha256_hex);
    let target_variant = SkinModel::parse(variant).ok()?;
    for entry in &index.entries {
        if entry.model != target_variant {
            continue;
        }
        if let Some(key) = texture_key.filter(|k| !k.is_empty()) {
            if entry.texture_key.as_deref() == Some(key) {
                return Some(entry.id.clone());
            }
        }
        if let Some(hash) = png_hash.as_ref() {
            if let Some(p) = entry_png_path(base, entry) {
                if std::fs::read(&p)
                    .ok()
                    .map(|b| sha256_hex(&b) == *hash)
                    .unwrap_or(false)
                {
                    return Some(entry.id.clone());
                }
            }
        }
    }
    None
}

pub fn sync_premium_library_active(
    dirs: &Dirs,
    uuid: &str,
    texture_key: Option<&str>,
    variant: &str,
    png: Option<&[u8]>,
) -> Result<Vec<SkinLibraryEntryView>> {
    let _acc = require_premium(dirs, uuid)?;
    let base = premium_library_dir(dirs, uuid)?;
    let idx_path = index_path(&base);
    let mut index = load_index(&idx_path);
    if let Some(id) = find_premium_skin_entry_id(&base, &index, texture_key, variant, png) {
        index.active_id = Some(id);
        save_index(&idx_path, &index)?;
    }
    Ok(index
        .entries
        .iter()
        .map(|e| entry_to_view(e, &base, index.active_id.as_deref()))
        .collect())
}

pub fn set_premium_library_active(
    dirs: &Dirs,
    uuid: &str,
    entry_id: &str,
) -> Result<Vec<SkinLibraryEntryView>> {
    let _acc = require_premium(dirs, uuid)?;
    let base = premium_library_dir(dirs, uuid)?;
    let idx_path = index_path(&base);
    let mut index = load_index(&idx_path);
    if !index.entries.iter().any(|e| e.id == entry_id) {
        return Err(Error::msg("Nie znaleziono profilu w bibliotece."));
    }
    index.active_id = Some(entry_id.to_string());
    save_index(&idx_path, &index)?;
    Ok(index
        .entries
        .iter()
        .map(|e| entry_to_view(e, &base, index.active_id.as_deref()))
        .collect())
}

pub fn save_premium_library_entry(
    dirs: &Dirs,
    uuid: &str,
    req: SavePremiumLibraryReq,
) -> Result<SkinLibraryEntryView> {
    let _acc = require_premium(dirs, uuid)?;
    dirs.ensure()?;
    let base = premium_library_dir(dirs, uuid)?;
    std::fs::create_dir_all(&base)?;
    let idx_path = index_path(&base);
    let mut index = load_index(&idx_path);

    let model = SkinModel::parse(&req.variant)?;
    let id = new_id();
    let mut png_file = None;

    if let Some(png) = req.png.as_ref() {
        validate_skin_png(png)?;
        let file_name = format!("{id}.png");
        std::fs::write(base.join(&file_name), png)?;
        png_file = Some(file_name);
    } else if req.texture_key.as_ref().is_none_or(|k| k.is_empty()) {
        return Err(Error::msg("Brak skina do zapisania (PNG albo textureKey)."));
    }

    let display = if req.name.trim().is_empty() {
        format!("Profil {}", index.entries.len() + 1)
    } else {
        req.name.trim().to_string()
    };

    let entry = SkinLibraryEntry {
        id,
        name: display,
        model,
        created_at: chrono::Utc::now().to_rfc3339(),
        source: req.source,
        texture_key: req.texture_key.filter(|k| !k.is_empty()),
        png_file,
        cape_id: req.cape_id.filter(|c| !c.is_empty()),
    };
    index.entries.push(entry.clone());
    if req.mark_active {
        index.active_id = Some(entry.id.clone());
    }
    save_index(&idx_path, &index)?;
    Ok(entry_to_view(
        &entry,
        &base,
        index.active_id.as_deref(),
    ))
}

pub fn delete_premium_library_entry(dirs: &Dirs, uuid: &str, entry_id: &str) -> Result<()> {
    let _acc = require_premium(dirs, uuid)?;
    let base = premium_library_dir(dirs, uuid)?;
    let idx_path = index_path(&base);
    let mut index = load_index(&idx_path);
    let pos = index
        .entries
        .iter()
        .position(|e| e.id == entry_id)
        .ok_or_else(|| Error::msg("Nie znaleziono profilu w bibliotece."))?;
    let entry = index.entries.remove(pos);
    if let Some(p) = entry_png_path(&base, &entry) {
        let _ = std::fs::remove_file(p);
    }
    if index.active_id.as_deref() == Some(entry_id) {
        index.active_id = None;
    }
    save_index(&idx_path, &index)?;
    Ok(())
}

pub fn find_premium_library_duplicate(
    dirs: &Dirs,
    uuid: &str,
    texture_key: Option<&str>,
    variant: &str,
    cape_id: Option<&str>,
    png: Option<&[u8]>,
) -> Option<SkinLibraryEntryView> {
    let base = premium_library_dir(dirs, uuid).ok()?;
    let index = load_index(&index_path(&base));
    let active_id = index.active_id.as_deref();
    let png_hash = png.map(sha256_hex);
    let target_variant = SkinModel::parse(variant).ok()?;
    for entry in &index.entries {
        if entry.model != target_variant {
            continue;
        }
        let cape_match = entry.cape_id.as_deref() == cape_id;
        if let Some(key) = texture_key.filter(|k| !k.is_empty()) {
            if entry.texture_key.as_deref() == Some(key) && cape_match {
                return Some(entry_to_view(entry, &base, active_id));
            }
        }
        if let Some(hash) = png_hash.as_ref() {
            if let Some(p) = entry_png_path(&base, entry) {
                if std::fs::read(&p).ok().map(|b| sha256_hex(&b) == *hash).unwrap_or(false)
                    && cape_match
                {
                    return Some(entry_to_view(entry, &base, active_id));
                }
            }
        }
    }
    None
}
