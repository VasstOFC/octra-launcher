//! CustomSkinLoader (Fabric) — skiny innych graczy na Open-to-LAN / offline (MC 1.21+).
//! authlib-injector 1.2.8 nie ładuje skinów innych graczy w multiplayerze vanilla 1.21+.

use std::path::Path;

use serde::Deserialize;
use serde_json::json;

use crate::error::{Error, Result};
use crate::instances::{Instance, Loader};
use crate::install;
use crate::loaders::fabric;
use crate::paths::Dirs;
use crate::skins;

const CSL_SLUG: &str = "customskinloader";
const CSL_MARKER: &str = "CustomSkinLoader_Fabric";

#[derive(Debug, Deserialize)]
struct ModrinthVersion {
    #[serde(default)]
    files: Vec<ModrinthFile>,
}

#[derive(Debug, Deserialize)]
struct ModrinthFile {
    filename: String,
    url: String,
    #[serde(default, rename = "primary")]
    is_primary: bool,
}

/// Przygotuj Fabric + CustomSkinLoader. Dla vanilla tymczasowo przełącza `version_id` na profil Fabric.
pub async fn ensure_skin_support(
    client: &reqwest::Client,
    dirs: &Dirs,
    inst: &mut Instance,
    ygg_root: &str,
) -> Result<()> {
    let game_version = inst.game_version.trim();
    if game_version.is_empty() {
        return Ok(());
    }
    let game_dir = dirs.game_dir(&inst.id);
    std::fs::create_dir_all(&game_dir)?;

    if inst.loader == Loader::Vanilla {
        let fabric_vid = ensure_fabric_profile(client, dirs, game_version).await?;
        inst.version_id = fabric_vid;
        inst.loader = Loader::Fabric;
    }

    if !matches!(inst.loader, Loader::Fabric | Loader::Quilt) {
        return Ok(());
    }

    ensure_custom_skin_loader_jar(client, dirs, &game_dir, game_version).await?;
    write_csl_config(&game_dir, ygg_root)?;
    sync_username_skin_files(&game_dir, dirs)?;
    Ok(())
}

async fn ensure_fabric_profile(
    client: &reqwest::Client,
    dirs: &Dirs,
    game_version: &str,
) -> Result<String> {
    let loaders = fabric::list_loaders(client, game_version).await?;
    let loader_version = loaders
        .into_iter()
        .next()
        .ok_or_else(|| Error::msg(format!("Brak loadera Fabric dla Minecraft {game_version}.")))?;
    let version_id = format!("{game_version}-fabric-{loader_version}");
    if dirs.version_json(&version_id).exists() {
        return Ok(version_id);
    }
    eprintln!(
        "Lumen skins: profil Fabric {loader_version} dla {game_version} (skin multiplayer)…"
    );
    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{game_version}/{loader_version}/profile/json"
    );
    let dest = dirs.version_json(&version_id);
    std::fs::create_dir_all(dirs.version_dir(&version_id))?;
    crate::download::download_file(client, &url, &dest, None, None, None).await?;
    let raw = std::fs::read_to_string(&dest)?;
    let mut profile: serde_json::Value = serde_json::from_str(&raw)?;
    if profile.get("id").and_then(|v| v.as_str()) != Some(version_id.as_str()) {
        profile["id"] = serde_json::Value::String(version_id.clone());
        std::fs::write(&dest, serde_json::to_string_pretty(&profile)?)?;
    }
    let meta: crate::meta::VersionMeta = serde_json::from_value(profile)?;
    download_fabric_libraries(client, dirs, &meta).await?;
    Ok(version_id)
}

async fn download_fabric_libraries(
    client: &reqwest::Client,
    dirs: &Dirs,
    meta: &crate::meta::VersionMeta,
) -> Result<()> {
    for lib in &meta.libraries {
        for f in install::library_files(dirs, lib) {
            if f.path.exists() {
                continue;
            }
            if !f.url.is_empty() {
                if let Some(parent) = f.path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                crate::download::download_file(client, &f.url, &f.path, None, None, None).await?;
            }
        }
    }
    Ok(())
}

async fn ensure_custom_skin_loader_jar(
    client: &reqwest::Client,
    dirs: &Dirs,
    game_dir: &Path,
    game_version: &str,
) -> Result<()> {
    let mods = game_dir.join("mods");
    std::fs::create_dir_all(&mods)?;
    if mods
        .read_dir()
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| {
            e.path()
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| {
                    n.contains(CSL_MARKER) || n.to_ascii_lowercase().contains("customskinloader")
                })
        })
    {
        return Ok(());
    }

    let url = format!(
        "https://api.modrinth.com/v2/project/{CSL_SLUG}/version?game_versions=[\"{game_version}\"]&loaders=[\"fabric\"]"
    );
    let resp = client
        .get(&url)
        .header("User-Agent", "OctraLauncher/0.1 (skin-loader)")
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(Error::msg(format!(
            "Modrinth CustomSkinLoader HTTP {}",
            resp.status()
        )));
    }
    let versions: Vec<ModrinthVersion> = resp.json().await?;
    let version = versions
        .into_iter()
        .find(|v| !v.files.is_empty())
        .ok_or_else(|| Error::msg(format!("Brak CustomSkinLoader dla Minecraft {game_version}.")))?;
    let file = version
        .files
        .iter()
        .find(|f| f.is_primary)
        .or_else(|| version.files.first())
        .ok_or_else(|| Error::msg("CustomSkinLoader: brak pliku w wersji Modrinth."))?;

    let cache = dirs.cache.join("mods");
    std::fs::create_dir_all(&cache)?;
    let cached = cache.join(&file.filename);
    if !cached.exists() {
        eprintln!("Lumen skins: pobieranie CustomSkinLoader…");
        crate::download::download_file(client, &file.url, &cached, None, None, None).await?;
    }
    let dest = mods.join(&file.filename);
    if !dest.exists() {
        std::fs::copy(&cached, &dest)?;
    }
    Ok(())
}

fn write_csl_config(game_dir: &Path, ygg_root: &str) -> Result<()> {
    let root = ygg_root.trim_end_matches('/');
    let legacy_root = format!("{root}/skins/MinecraftSkins/");
    let cfg_dir = game_dir.join("CustomSkinLoader");
    std::fs::create_dir_all(&cfg_dir)?;
    let cfg = json!({
        "version": "15.0",
        "enable": true,
        "loadlist": [
            {
                "name": "LumenLocal",
                "type": "Legacy",
                "checkPNG": false,
                "skin": "LocalSkin/skins/{USERNAME}.png",
                "model": "auto"
            },
            {
                "name": "Lumen",
                "type": "Legacy",
                "root": legacy_root
            },
            {
                "name": "Mojang",
                "type": "MojangAPI"
            }
        ]
    });
    std::fs::write(
        cfg_dir.join("CustomSkinLoader.json"),
        serde_json::to_string_pretty(&cfg)?,
    )?;
    std::fs::write(
        cfg_dir.join("skinurls.txt"),
        format!("{legacy_root}*.png\n"),
    )?;
    Ok(())
}

/// Kopiuje skiny offline lockera jako `LocalSkin/skins/{username}.png` dla CustomSkinLoader.
pub fn sync_username_skin_files(game_dir: &Path, dirs: &Dirs) -> Result<()> {
    let skin_dir = game_dir
        .join("CustomSkinLoader")
        .join("LocalSkin")
        .join("skins");
    std::fs::create_dir_all(&skin_dir)?;
    for skin in skins::list_local_custom_skins(dirs) {
        if skin.name.is_empty() {
            continue;
        }
        let dest = skin_dir.join(format!("{}.png", skin.name));
        eprintln!(
            "Lumen skins: sync offline skin {} -> {}",
            skin.name,
            dest.display()
        );
        std::fs::write(&dest, &skin.png)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csl_config_has_lumen_legacy() {
        let dir = std::env::temp_dir().join("octra-csl-test");
        let _ = std::fs::remove_dir_all(&dir);
        write_csl_config(&dir, "http://127.0.0.1:63078").unwrap();
        let raw =
            std::fs::read_to_string(dir.join("CustomSkinLoader/CustomSkinLoader.json")).unwrap();
        assert!(raw.contains("Lumen"));
        assert!(raw.contains("MinecraftSkins"));
        assert!(raw.contains("LumenLocal"));
        assert!(raw.contains("\"type\": \"Legacy\""));
        assert!(raw.contains("LocalSkin/skins/{USERNAME}.png"));
        assert!(!raw.contains("\"type\": \"Local\""));

        let skinurls =
            std::fs::read_to_string(dir.join("CustomSkinLoader/skinurls.txt")).unwrap();
        assert!(skinurls.contains("http://127.0.0.1:63078/skins/MinecraftSkins/*.png"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
