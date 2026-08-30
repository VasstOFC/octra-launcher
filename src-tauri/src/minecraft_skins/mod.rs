//! Unified Minecraft skins API — adaptacja logiki Modrinth App (GPLv3) na storage Octry.

pub mod png_util;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::auth::{self, hyphenate_uuid};
use crate::error::{Error, Result};
use crate::mojang_skins::{self, CatalogGroup, McExpressionState, McPlayerProfile};
use crate::paths::Dirs;
use crate::skin_library::{self, SavePremiumLibraryReq, SkinLibraryEntryView};
use crate::skins::SkinModel;

const TEXTURE_BASE: &str = "https://textures.minecraft.net/texture/";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkinVariant {
    Classic,
    Slim,
    Unknown,
}

impl SkinVariant {
    fn from_model(model: &SkinModel) -> Self {
        match model {
            SkinModel::Slim => Self::Slim,
            SkinModel::Classic => Self::Classic,
        }
    }

    fn from_str_loose(s: &str) -> Self {
        if s.eq_ignore_ascii_case("slim") {
            Self::Slim
        } else if s.eq_ignore_ascii_case("classic") {
            Self::Classic
        } else {
            Self::Unknown
        }
    }

    pub fn as_api_str(&self) -> &'static str {
        match self {
            Self::Slim => "slim",
            Self::Classic | Self::Unknown => "classic",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkinSource {
    Default,
    CustomExternal,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    pub texture_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    pub variant: SkinVariant,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cape_id: Option<String>,
    pub texture: String,
    pub source: SkinSource,
    pub is_equipped: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cape {
    pub id: String,
    pub name: String,
    pub texture: String,
    pub is_equipped: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipSkinReq {
    pub uuid: String,
    pub skin: Skin,
    #[serde(default)]
    pub png: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCustomSkinReq {
    pub uuid: String,
    pub skin: Skin,
    pub variant: String,
    #[serde(default)]
    pub cape_id: Option<String>,
    #[serde(default)]
    pub png: Option<Vec<u8>>,
    #[serde(default)]
    pub replace_texture: bool,
}

fn texture_url_for_key(key: &str) -> String {
    format!("{TEXTURE_BASE}{key}")
}

fn library_entry_texture(entry: &SkinLibraryEntryView) -> String {
    if let Some(b64) = entry.png_base64.as_ref() {
        if b64.starts_with("data:") {
            return b64.clone();
        }
        return format!("data:image/png;base64,{b64}");
    }
    entry
        .texture_key
        .as_ref()
        .map(|k| texture_url_for_key(k))
        .unwrap_or_default()
}

fn library_to_skin(entry: &SkinLibraryEntryView) -> Skin {
    let texture_key = entry
        .texture_key
        .clone()
        .filter(|k| !k.is_empty())
        .unwrap_or_else(|| format!("local-{}", entry.id));
    Skin {
        texture_key,
        name: Some(entry.name.clone()),
        section: None,
        variant: SkinVariant::from_str_loose(&entry.model),
        cape_id: entry.cape_id.clone(),
        texture: library_entry_texture(entry),
        source: SkinSource::Custom,
        is_equipped: entry.is_active,
        library_id: Some(entry.id.clone()),
    }
}

fn catalog_to_skin(
    group: &CatalogGroup,
    item: &mojang_skins::CatalogSkin,
    equipped_key: Option<&str>,
) -> Skin {
    let variant = SkinVariant::from_str_loose(&item.variant);
    Skin {
        texture_key: item.texture_key.clone(),
        name: Some(item.name.clone()),
        section: Some(group.title.clone()),
        variant,
        cape_id: None,
        texture: texture_url_for_key(&item.texture_key),
        source: SkinSource::Default,
        is_equipped: equipped_key == Some(item.texture_key.as_str()),
        library_id: None,
    }
}

fn active_texture_key(
    profile: Option<&McPlayerProfile>,
    account_skin_url: Option<&str>,
) -> Option<String> {
    if let Some(profile) = profile {
        for skin in &profile.skins {
            if skin.state == McExpressionState::Active {
                if let Some(key) = skin.texture_key.clone().or_else(|| {
                    skin.url
                        .strip_prefix(TEXTURE_BASE)
                        .or_else(|| skin.url.strip_prefix("http://textures.minecraft.net/texture/"))
                        .map(str::to_string)
                }) {
                    return Some(key);
                }
            }
        }
    }
    account_skin_url.and_then(|url| {
        url.strip_prefix(TEXTURE_BASE)
            .or_else(|| url.strip_prefix("https://textures.minecraft.net/texture/"))
            .map(str::to_string)
    })
}

pub async fn get_available_capes(
    client: &reqwest::Client,
    client_id: &str,
    dirs: &Dirs,
    uuid: &str,
) -> Result<Vec<Cape>> {
    let profile = mojang_skins::profile_for_account(client, client_id, dirs, uuid).await?;
    Ok(profile
        .capes
        .into_iter()
        .map(|c| Cape {
            id: c.id,
            name: c.alias.unwrap_or_else(|| "Peleryna".into()),
            texture: c.url,
            is_equipped: c.state == McExpressionState::Active,
        })
        .collect())
}

pub async fn get_available_skins_offline(dirs: &Dirs, uuid: &str) -> Result<Vec<Skin>> {
    let library = skin_library::list_offline_library(dirs, uuid)?;
    let mut skins: Vec<Skin> = library.iter().map(library_to_skin).collect();
    let custom_keys: HashSet<String> = skins.iter().map(|s| s.texture_key.clone()).collect();

    let equipped_key = library.iter().find(|e| e.is_active).and_then(|e| {
        e.texture_key
            .clone()
            .filter(|k| !k.is_empty())
            .or_else(|| Some(format!("local-{}", e.id)))
    });

    for group in mojang_skins::catalog() {
        for item in &group.skins {
            if custom_keys.contains(&item.texture_key) {
                continue;
            }
            skins.push(catalog_to_skin(&group, item, equipped_key.as_deref()));
        }
    }

    if !skins.iter().any(|s| s.is_equipped) {
        if let Some(active) = library.iter().find(|e| e.is_active) {
            let key = active
                .texture_key
                .clone()
                .filter(|k| !k.is_empty())
                .unwrap_or_else(|| format!("local-{}", active.id));
            for s in &mut skins {
                s.is_equipped = s.texture_key == key;
            }
        }
    }

    Ok(skins)
}

pub async fn get_available_skins(
    client: &reqwest::Client,
    client_id: &str,
    dirs: &Dirs,
    uuid: &str,
) -> Result<Vec<Skin>> {
    let file = auth::load_accounts(dirs)?;
    let wanted = hyphenate_uuid(uuid);
    let account = file
        .accounts
        .iter()
        .find(|a| hyphenate_uuid(&a.uuid) == wanted)
        .ok_or_else(|| Error::msg("Nie znaleziono konta."))?;

    if account.is_offline() {
        return get_available_skins_offline(dirs, uuid).await;
    }

    let library = skin_library::list_premium_library(dirs, uuid)?;
    let mut skins: Vec<Skin> = library.iter().map(library_to_skin).collect();
    let custom_keys: HashSet<String> = skins.iter().map(|s| s.texture_key.clone()).collect();

    let account_skin = if let Some(account) = file
        .accounts
        .iter()
        .find(|a| hyphenate_uuid(&a.uuid) == wanted)
        .cloned()
    {
        auth::fetch_account_skin(client, client_id, dirs, &account, false)
            .await
            .ok()
    } else {
        None
    };
    let profile = mojang_skins::profile_for_account(client, client_id, dirs, uuid)
        .await
        .ok();

    let equipped_key = active_texture_key(
        profile.as_ref(),
        account_skin.as_ref().and_then(|s| s.texture_url.as_deref()),
    );

    if let Some(key) = equipped_key.as_ref() {
        if let Some(entry) = skins.iter_mut().find(|s| s.texture_key == *key) {
            entry.is_equipped = true;
            if let Some(p) = profile.as_ref() {
                if let Some(active) = p
                    .skins
                    .iter()
                    .find(|s| s.state == McExpressionState::Active)
                {
                    entry.variant = SkinVariant::from_str_loose(&active.variant);
                }
                if let Some(cape) = p
                    .capes
                    .iter()
                    .find(|c| c.state == McExpressionState::Active)
                {
                    entry.cape_id = Some(cape.id.clone());
                }
            }
        } else if let Some(p) = profile.as_ref() {
            let active = p
                .skins
                .iter()
                .find(|s| s.state == McExpressionState::Active);
            let variant = active
                .map(|s| SkinVariant::from_str_loose(&s.variant))
                .unwrap_or(SkinVariant::Unknown);
            let texture = active
                .map(|s| s.url.clone())
                .or_else(|| account_skin.as_ref().and_then(|s| s.texture_url.clone()))
                .unwrap_or_else(|| texture_url_for_key(key));
            let cape_id = p
                .capes
                .iter()
                .find(|c| c.state == McExpressionState::Active)
                .map(|c| c.id.clone());
            skins.push(Skin {
                texture_key: key.clone(),
                name: active
                    .and_then(|s| s.alias.clone())
                    .or(Some("Aktywny".into())),
                section: None,
                variant,
                cape_id,
                texture,
                source: SkinSource::CustomExternal,
                is_equipped: true,
                library_id: None,
            });
        }
    }

    for group in mojang_skins::catalog() {
        for item in &group.skins {
            if custom_keys.contains(&item.texture_key) {
                continue;
            }
            let mut skin = catalog_to_skin(&group, item, equipped_key.as_deref());
            if equipped_key.as_deref() == Some(item.texture_key.as_str()) {
                skin.is_equipped = true;
            }
            skins.push(skin);
        }
    }

    Ok(skins)
}

pub async fn equip_skin_offline(dirs: &Dirs, uuid: &str, skin: &Skin) -> Result<()> {
    if let Some(id) = skin.library_id.as_deref() {
        skin_library::equip_offline_library_entry(dirs, uuid, id)?;
        return Ok(());
    }
    let library = skin_library::list_offline_library(dirs, uuid)?;
    if let Some(entry) = library
        .iter()
        .find(|e| e.texture_key.as_deref() == Some(skin.texture_key.as_str()))
    {
        skin_library::equip_offline_library_entry(dirs, uuid, &entry.id)?;
        return Ok(());
    }
    Err(Error::msg("Nie znaleziono skina w bibliotece offline."))
}

pub async fn equip_skin(
    client: &reqwest::Client,
    client_id: &str,
    dirs: &Dirs,
    uuid: &str,
    skin: &Skin,
    png: Option<&[u8]>,
) -> Result<Option<McPlayerProfile>> {
    let file = auth::load_accounts(dirs)?;
    let wanted = hyphenate_uuid(uuid);
    let account = file
        .accounts
        .iter()
        .find(|a| hyphenate_uuid(&a.uuid) == wanted)
        .ok_or_else(|| Error::msg("Nie znaleziono konta."))?;

    if account.is_offline() {
        equip_skin_offline(dirs, uuid, skin).await?;
        return Ok(None);
    }

    let variant = skin.variant.as_api_str();
    let profile = if let Some(bytes) = png {
        let normalized = png_util::normalize_skin_texture_bytes(bytes)?;
        mojang_skins::upload_for_account(client, client_id, dirs, uuid, &normalized, variant).await?
    } else if skin.texture_key.starts_with("local-") {
        let id = skin
            .library_id
            .as_deref()
            .ok_or_else(|| Error::msg("Brak identyfikatora biblioteki."))?;
        let lib = skin_library::list_premium_library(dirs, uuid)?;
        let entry = lib
            .iter()
            .find(|e| e.id == id)
            .ok_or_else(|| Error::msg("Nie znaleziono skina w bibliotece."))?;
        let b64 = entry
            .png_base64
            .as_ref()
            .ok_or_else(|| Error::msg("Brak pliku PNG dla tego skina."))?;
        let raw = b64.trim_start_matches("data:image/png;base64,");
        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(raw)
            .map_err(|e| Error::msg(format!("Błąd dekodowania PNG: {e}")))?;
        let normalized = png_util::normalize_skin_texture_bytes(&bytes)?;
        mojang_skins::upload_for_account(client, client_id, dirs, uuid, &normalized, variant)
            .await?
    } else {
        let bytes = mojang_skins::texture_png_bytes(client, &skin.texture_key).await?;
        let normalized = png_util::normalize_skin_texture_bytes(&bytes)?;
        mojang_skins::upload_for_account(client, client_id, dirs, uuid, &normalized, variant).await?
    };

    let profile = mojang_skins::sync_cape_for_account(
        client,
        client_id,
        dirs,
        uuid,
        skin.cape_id.as_deref(),
        &profile,
    )
    .await?;

    Ok(Some(profile))
}

pub fn save_custom_skin(
    dirs: &Dirs,
    uuid: &str,
    skin: &Skin,
    variant: &str,
    cape_id: Option<&str>,
    png: Option<&[u8]>,
    replace_texture: bool,
) -> Result<SkinLibraryEntryView> {
    let file = auth::load_accounts(dirs)?;
    let wanted = hyphenate_uuid(uuid);
    let account = file
        .accounts
        .iter()
        .find(|a| hyphenate_uuid(&a.uuid) == wanted)
        .ok_or_else(|| Error::msg("Nie znaleziono konta."))?;

    let png_bytes = if replace_texture {
        png.map(png_util::normalize_skin_texture_bytes)
            .transpose()?
    } else {
        png.map(|p| p.to_vec())
    };

    let req = SavePremiumLibraryReq {
        name: skin.name.clone().unwrap_or_else(|| "Profil".into()),
        variant: variant.to_string(),
        source: if png_bytes.is_some() {
            "upload".into()
        } else if skin.source == SkinSource::Default {
            "catalog".into()
        } else {
            "mojang".into()
        },
        texture_key: if skin.texture_key.starts_with("local-") {
            None
        } else {
            Some(skin.texture_key.clone())
        },
        cape_id: cape_id.map(str::to_string),
        png: png_bytes,
        mark_active: false,
    };

    if account.is_offline() {
        let png = req.png.as_ref().ok_or_else(|| Error::msg("Brak PNG."))?;
        return skin_library::add_offline_library_entry(
            dirs,
            uuid,
            png,
            &req.variant,
            &req.name,
        );
    }

    skin_library::save_premium_library_entry(dirs, uuid, req)
}

pub fn remove_custom_skin(dirs: &Dirs, uuid: &str, skin: &Skin) -> Result<()> {
    if let Some(id) = skin.library_id.as_deref() {
        let file = auth::load_accounts(dirs)?;
        let wanted = hyphenate_uuid(uuid);
        let offline = file
            .accounts
            .iter()
            .find(|a| hyphenate_uuid(&a.uuid) == wanted)
            .map(|a| a.is_offline())
            .unwrap_or(false);
        if offline {
            return skin_library::delete_offline_library_entry(dirs, uuid, id);
        }
        return skin_library::delete_premium_library_entry(dirs, uuid, id);
    }
    Err(Error::msg("Ten skin nie jest zapisany w bibliotece."))
}

pub async fn normalize_skin_texture_input(
    client: &reqwest::Client,
    texture: TextureInput,
) -> Result<Vec<u8>> {
    match texture {
        TextureInput::Bytes(bytes) => png_util::normalize_skin_texture_bytes(&bytes),
        TextureInput::Url(url) => {
            if url.starts_with("data:image/png;base64,") {
                let b64 = url.trim_start_matches("data:image/png;base64,");
                use base64::Engine as _;
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(b64)
                    .map_err(|e| Error::msg(format!("Błąd base64: {e}")))?;
                return png_util::normalize_skin_texture_bytes(&bytes);
            }
            let resp = client.get(&url).send().await?;
            let bytes = resp.bytes().await?;
            png_util::normalize_skin_texture_bytes(&bytes)
        }
    }
}

pub enum TextureInput {
    Bytes(Vec<u8>),
    Url(String),
}
