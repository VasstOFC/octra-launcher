use serde::Deserialize;

use crate::download::{download_json, download_file};
use crate::error::Result;
use crate::install::{self, emit_progress, InstallProgress};
use crate::paths::Dirs;

#[derive(Debug, Deserialize)]
struct FabricLoaderListItem {
    loader: FabricLoaderInfo,
}

#[derive(Debug, Deserialize)]
struct FabricLoaderInfo {
    version: String,
    #[serde(default)]
    stable: bool,
}

pub async fn list_loaders(client: &reqwest::Client, game: &str) -> Result<Vec<String>> {
    let url = format!("https://meta.fabricmc.net/v2/versions/loader/{game}");
    let entries: Vec<FabricLoaderListItem> = match download_json(client, &url).await {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };
    let mut versions: Vec<String> = entries
        .iter()
        .filter(|e| e.loader.stable)
        .map(|e| e.loader.version.clone())
        .collect();
    if versions.is_empty() {
        versions = entries.into_iter().map(|e| e.loader.version).collect();
    }
    Ok(versions)
}

pub async fn install_fabric(
    client: &reqwest::Client,
    app: &tauri::AppHandle,
    dirs: &Dirs,
    instance_id: &str,
    game_version: &str,
    loader_version: &str,
    cancel: &tokio_util::sync::CancellationToken,
) -> Result<String> {
    install::install_vanilla(client, app, dirs, instance_id, game_version, cancel).await?;
    let version_id = format!("{game_version}-fabric-{loader_version}");
    emit_progress(
        app,
        InstallProgress {
            instance_id: instance_id.into(),
            stage: "loader".into(),
            current: 0,
            total: 1,
            file: None,
            message: format!("Profil Fabric {loader_version}"),
        },
    );
    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{game_version}/{loader_version}/profile/json"
    );
    let dest = dirs.version_json(&version_id);
    std::fs::create_dir_all(dirs.version_dir(&version_id))?;
    download_file(client, &url, &dest, None, None, None).await?;
    let raw = std::fs::read_to_string(&dest)?;
    let mut json: serde_json::Value = serde_json::from_str(&raw)?;
    if json.get("id").and_then(|v| v.as_str()) != Some(version_id.as_str()) {
        json["id"] = serde_json::Value::String(version_id.clone());
        std::fs::write(&dest, serde_json::to_string_pretty(&json)?)?;
    }
    let meta: crate::meta::VersionMeta = serde_json::from_value(json)?;
    let mut files = Vec::new();
    for lib in &meta.libraries {
        files.extend(install::library_files(dirs, lib));
    }
    install::download_libraries(client, app, instance_id, files, cancel).await?;
    Ok(version_id)
}
