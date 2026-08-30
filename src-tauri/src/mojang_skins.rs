//! Oficjalny katalog skinów Mojang + operacje profilu (skin, peleryna).

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::auth::{self, hyphenate_uuid};
use crate::error::{Error, Result};
use crate::paths::Dirs;

const MC_PROFILE: &str = "https://api.minecraftservices.com/minecraft/profile";
const MC_SKINS: &str = "https://api.minecraftservices.com/minecraft/profile/skins";
const MC_CAPES: &str = "https://api.minecraftservices.com/minecraft/profile/capes/active";
const TEXTURE_BASE: &str = "https://textures.minecraft.net/texture/";
const MINECRAFT_USER_AGENT: &str =
    "OctraLauncher/1.0 (Minecraft profile; +https://github.com/octra)";

fn format_mojang_error(context: &str, body: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(msg) = value.get("errorMessage").and_then(|v| v.as_str()) {
            return format!("{context}: {msg}");
        }
        if let Some(detail) = value.get("detail").and_then(|v| v.as_str()) {
            return format!("{context}: {detail}");
        }
        if let Some(title) = value.get("title").and_then(|v| v.as_str()) {
            return format!("{context}: {title}");
        }
    }
    let trimmed = body.trim();
    if trimmed.is_empty() {
        context.to_string()
    } else {
        format!("{context}: {trimmed}")
    }
}

pub fn active_cape_id(profile: &McPlayerProfile) -> Option<String> {
    profile
        .capes
        .iter()
        .find(|c| c.state == McExpressionState::Active)
        .map(|c| hyphenate_uuid(&c.id))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSkin {
    pub id: String,
    pub name: String,
    pub texture_key: String,
    pub variant: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogGroup {
    pub id: String,
    pub title: String,
    pub skins: Vec<CatalogSkin>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub(crate) enum McExpressionState {
    Active,
    Inactive,
    #[default]
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McCape {
    pub id: String,
    #[serde(default)]
    pub(crate) state: McExpressionState,
    pub url: String,
    #[serde(default, alias = "name")]
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McOwnedSkin {
    pub id: String,
    #[serde(default)]
    pub(crate) state: McExpressionState,
    pub url: String,
    pub variant: String,
    #[serde(default, alias = "name")]
    pub alias: Option<String>,
    #[serde(default, rename = "textureKey")]
    pub texture_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McPlayerProfile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub skins: Vec<McOwnedSkin>,
    #[serde(default)]
    pub capes: Vec<McCape>,
}

pub fn catalog() -> Vec<CatalogGroup> {
    serde_json::from_str(include_str!("../resources/mojang_skins.json")).unwrap_or_default()
}

fn bundled_texture_base64(texture_key: &str) -> Option<String> {
    static BUNDLED: OnceLock<HashMap<String, String>> = OnceLock::new();
    BUNDLED
        .get_or_init(|| {
            serde_json::from_str(include_str!("../resources/catalog_bundled_textures.json"))
                .unwrap_or_default()
        })
        .get(texture_key)
        .cloned()
}

pub async fn fetch_profile(client: &reqwest::Client, access_token: &str) -> Result<McPlayerProfile> {
    let resp = client
        .get(MC_PROFILE)
        .bearer_auth(access_token)
        .header("Accept", "application/json")
        .header(
            "User-Agent",
            MINECRAFT_USER_AGENT,
        )
        .send()
        .await?;
    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        return Err(Error::msg(format!(
            "Nie udało się pobrać profilu Minecraft (HTTP {}): {}",
            status.as_u16(),
            text.trim()
        )));
    }
    let mut profile: McPlayerProfile = serde_json::from_str(&text).map_err(|e| {
        Error::msg(format!(
            "Nie udało się odczytać profilu Minecraft: {e} (odpowiedź: {})",
            text.chars().take(240).collect::<String>()
        ))
    })?;
    for skin in &mut profile.skins {
        skin.texture_key = skin.url.strip_prefix(TEXTURE_BASE).map(str::to_string);
    }
    for cape in &mut profile.capes {
        if cape.url.starts_with("http://") {
            cape.url = cape.url.replacen("http://", "https://", 1);
        }
    }
    Ok(profile)
}

pub async fn equip_catalog_skin(
    client: &reqwest::Client,
    access_token: &str,
    texture_key: &str,
    variant: &str,
) -> Result<McPlayerProfile> {
    let bytes = texture_png_bytes(client, texture_key).await?;
    upload_custom_skin(client, access_token, &bytes, variant).await
}

pub async fn upload_custom_skin(
    client: &reqwest::Client,
    access_token: &str,
    png_bytes: &[u8],
    variant: &str,
) -> Result<McPlayerProfile> {
    let variant = if variant.eq_ignore_ascii_case("slim") {
        "slim"
    } else {
        "classic"
    };
    let part = reqwest::multipart::Part::bytes(png_bytes.to_vec())
        .file_name("skin.png")
        .mime_str("image/png")
        .map_err(|e| Error::msg(format!("Nie udało się przygotować pliku skina: {e}")))?;
    let form = reqwest::multipart::Form::new()
        .text("variant", variant.to_string())
        .part("file", part);
    let resp = client
        .post(MC_SKINS)
        .bearer_auth(access_token)
        .header("Accept", "application/json")
        .header("User-Agent", MINECRAFT_USER_AGENT)
        .multipart(form)
        .send()
        .await?;
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::msg(format_mojang_error(
            "Minecraft odrzuciło wgranie skina",
            &body,
        )));
    }
    fetch_profile(client, access_token).await
}

pub async fn set_active_cape(
    client: &reqwest::Client,
    access_token: &str,
    cape_id: Option<&str>,
) -> Result<McPlayerProfile> {
    let resp = if let Some(id) = cape_id.filter(|s| !s.trim().is_empty()) {
        client
            .put(MC_CAPES)
            .bearer_auth(access_token)
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Accept", "application/json")
            .header("User-Agent", MINECRAFT_USER_AGENT)
            .json(&serde_json::json!({ "capeId": hyphenate_uuid(id) }))
            .send()
            .await?
    } else {
        client
            .delete(MC_CAPES)
            .bearer_auth(access_token)
            .header("Accept", "application/json")
            .header("User-Agent", MINECRAFT_USER_AGENT)
            .send()
            .await?
    };
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::msg(format_mojang_error(
            "Minecraft odrzuciło zmianę peleryny",
            &body,
        )));
    }
    fetch_profile(client, access_token).await
}

pub async fn texture_png_bytes(client: &reqwest::Client, texture_key: &str) -> Result<Vec<u8>> {
    use base64::Engine as _;
    let b64 = texture_png_base64(client, texture_key).await?;
    base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| Error::msg(format!("Błąd dekodowania tekstury skina: {e}")))
}

pub async fn texture_png_base64(client: &reqwest::Client, texture_key: &str) -> Result<String> {
    use base64::Engine as _;
    let url = format!("{TEXTURE_BASE}{texture_key}");
    if let Ok(resp) = client.get(&url).send().await {
        if resp.status().is_success() {
            if let Ok(bytes) = resp.bytes().await {
                return Ok(base64::engine::general_purpose::STANDARD.encode(bytes));
            }
        }
    }
    bundled_texture_base64(texture_key).ok_or_else(|| {
        Error::msg(format!(
            "Nie udało się pobrać tekstury skina ({}).",
            &texture_key[..texture_key.len().min(12)]
        ))
    })
}

pub async fn profile_for_account(
    client: &reqwest::Client,
    client_id: &str,
    dirs: &Dirs,
    account_uuid: &str,
) -> Result<McPlayerProfile> {
    profile_for_account_with_refresh(client, client_id, dirs, account_uuid, false).await
}

pub async fn profile_for_account_with_refresh(
    client: &reqwest::Client,
    client_id: &str,
    dirs: &Dirs,
    account_uuid: &str,
    refresh: bool,
) -> Result<McPlayerProfile> {
    crate::mojang_cache::global()
        .profile_for_account(client, client_id, dirs, account_uuid, refresh)
        .await
}

pub async fn equip_for_account(
    client: &reqwest::Client,
    client_id: &str,
    dirs: &Dirs,
    account_uuid: &str,
    texture_key: &str,
    variant: &str,
) -> Result<McPlayerProfile> {
    let file = auth::load_accounts(dirs)?;
    let wanted = hyphenate_uuid(account_uuid);
    let account = file
        .accounts
        .iter()
        .find(|a| hyphenate_uuid(&a.uuid) == wanted)
        .ok_or_else(|| Error::msg("Nie znaleziono konta."))?
        .clone();
    if account.is_offline() {
        return Err(Error::msg("Skiny Mojang wymagają konta Premium."));
    }
    let session = auth::session_for_account(client, client_id, dirs, &account).await?;
    let profile = equip_catalog_skin(client, &session.access_token, texture_key, variant).await?;
    crate::mojang_cache::global().set_profile(account_uuid, profile.clone());
    Ok(profile)
}

pub async fn upload_for_account(
    client: &reqwest::Client,
    client_id: &str,
    dirs: &Dirs,
    account_uuid: &str,
    png_bytes: &[u8],
    variant: &str,
) -> Result<McPlayerProfile> {
    let file = auth::load_accounts(dirs)?;
    let wanted = hyphenate_uuid(account_uuid);
    let account = file
        .accounts
        .iter()
        .find(|a| hyphenate_uuid(&a.uuid) == wanted)
        .ok_or_else(|| Error::msg("Nie znaleziono konta."))?
        .clone();
    if account.is_offline() {
        return Err(Error::msg("Wgrywanie skinów Mojang wymaga konta Premium."));
    }
    let session = auth::session_for_account(client, client_id, dirs, &account).await?;
    let profile = upload_custom_skin(client, &session.access_token, png_bytes, variant).await?;
    crate::mojang_cache::global().set_profile(account_uuid, profile.clone());
    Ok(profile)
}

pub async fn set_cape_for_account(
    client: &reqwest::Client,
    client_id: &str,
    dirs: &Dirs,
    account_uuid: &str,
    cape_id: Option<&str>,
) -> Result<McPlayerProfile> {
    let file = auth::load_accounts(dirs)?;
    let wanted = hyphenate_uuid(account_uuid);
    let account = file
        .accounts
        .iter()
        .find(|a| hyphenate_uuid(&a.uuid) == wanted)
        .ok_or_else(|| Error::msg("Nie znaleziono konta."))?
        .clone();
    if account.is_offline() {
        return Err(Error::msg("Peleryny Mojang wymagają konta Premium."));
    }
    let session = auth::session_for_account(client, client_id, dirs, &account).await?;
    let profile = set_active_cape(client, &session.access_token, cape_id).await?;
    crate::mojang_cache::global().set_profile(account_uuid, profile.clone());
    Ok(profile)
}

pub async fn sync_cape_for_account(
    client: &reqwest::Client,
    client_id: &str,
    dirs: &Dirs,
    account_uuid: &str,
    target_cape_id: Option<&str>,
    profile_after_skin: &McPlayerProfile,
) -> Result<McPlayerProfile> {
    let current = active_cape_id(profile_after_skin);
    let target = target_cape_id
        .filter(|s| !s.trim().is_empty())
        .map(hyphenate_uuid);

    if current == target {
        return Ok(profile_after_skin.clone());
    }

    match target {
        Some(ref id) => set_cape_for_account(client, client_id, dirs, account_uuid, Some(id)).await,
        None => {
            if current.is_some() {
                set_cape_for_account(client, client_id, dirs, account_uuid, None).await
            } else {
                Ok(profile_after_skin.clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::McPlayerProfile;

    #[test]
    fn parses_profile_with_capes() {
        let json = r#"{
          "id": "e08354ca63304d2cb2ff6bedad30bc70",
          "name": "Vasst",
          "skins": [{
            "id": "8c94945e-d0b4-4df8-97d1-d8d397624f93",
            "state": "ACTIVE",
            "url": "http://textures.minecraft.net/texture/83e283ab33558baa2cd0184d2e85f090c795a797bdbcb2cc47230c27f23fe9b1",
            "textureKey": "83e283ab33558baa2cd0184d2e85f090c795a797bdbcb2cc47230c27f23fe9b1",
            "variant": "CLASSIC"
          }],
          "capes": [{
            "id": "b059d1c0-5c3c-4e3c-8c3d-2e3e3e3e3e3e",
            "state": "ACTIVE",
            "url": "http://textures.minecraft.net/texture/cd9d82ab17fd92022dbd4a86cde4c382a7540e117fae7b9a2853658505a80625",
            "alias": "Migrator"
          }]
        }"#;
        let profile: McPlayerProfile = serde_json::from_str(json).unwrap();
        assert_eq!(profile.capes.len(), 1);
        assert_eq!(profile.capes[0].alias.as_deref(), Some("Migrator"));
    }
}
