//! Oficjalny katalog skinów Mojang + operacje profilu (skin, peleryna).

use serde::{Deserialize, Serialize};

use crate::auth::{self, hyphenate_uuid};
use crate::error::{Error, Result};
use crate::paths::Dirs;

const MC_PROFILE: &str = "https://api.minecraftservices.com/minecraft/profile";
const MC_SKINS: &str = "https://api.minecraftservices.com/minecraft/profile/skins";
const MC_CAPES: &str = "https://api.minecraftservices.com/minecraft/profile/capes/active";
const TEXTURE_BASE: &str = "https://textures.minecraft.net/texture/";

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
enum McExpressionState {
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
    state: McExpressionState,
    pub url: String,
    #[serde(default, alias = "name")]
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McOwnedSkin {
    pub id: String,
    #[serde(default)]
    state: McExpressionState,
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

pub async fn fetch_profile(client: &reqwest::Client, access_token: &str) -> Result<McPlayerProfile> {
    let resp = client
        .get(MC_PROFILE)
        .bearer_auth(access_token)
        .header("Accept", "application/json")
        .header(
            "User-Agent",
            "OctraLauncher/1.0 (Minecraft profile; +https://github.com/octra)",
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
    let variant = if variant.eq_ignore_ascii_case("slim") {
        "slim"
    } else {
        "classic"
    };
    let url = format!("{TEXTURE_BASE}{texture_key}");
    let resp = client
        .post(MC_SKINS)
        .bearer_auth(access_token)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "variant": variant, "url": url }))
        .send()
        .await?;
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::msg(format!(
            "Minecraft odrzuciło zmianę skina: {}",
            body.trim()
        )));
    }
    fetch_profile(client, access_token).await
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
        .multipart(form)
        .send()
        .await?;
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::msg(format!(
            "Minecraft odrzuciło wgranie skina: {}",
            body.trim()
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
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "capeId": hyphenate_uuid(id) }))
            .send()
            .await?
    } else {
        client
            .delete(MC_CAPES)
            .bearer_auth(access_token)
            .send()
            .await?
    };
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::msg(format!(
            "Minecraft odrzuciło zmianę peleryny: {}",
            body.trim()
        )));
    }
    fetch_profile(client, access_token).await
}

pub async fn texture_png_base64(client: &reqwest::Client, texture_key: &str) -> Result<String> {
    use base64::Engine as _;
    let url = format!("{TEXTURE_BASE}{texture_key}");
    let bytes = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

pub async fn profile_for_account(
    client: &reqwest::Client,
    client_id: &str,
    dirs: &Dirs,
    account_uuid: &str,
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
        return Err(Error::msg("Profil Mojang wymaga konta Premium."));
    }
    let session = auth::session_for_account(client, client_id, dirs, &account).await?;
    let profile = fetch_profile(client, &session.access_token).await?;
    auth::auth_log(
        dirs,
        &format!(
            "profil Mojang {}: {} skinów, {} peleryn",
            profile.name,
            profile.skins.len(),
            profile.capes.len()
        ),
    );
    Ok(profile)
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
    equip_catalog_skin(client, &session.access_token, texture_key, variant).await
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
    upload_custom_skin(client, &session.access_token, png_bytes, variant).await
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
    set_active_cape(client, &session.access_token, cape_id).await
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
