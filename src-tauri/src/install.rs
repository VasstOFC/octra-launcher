use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Semaphore;

use crate::download::{download_file, download_json, ByteProgress};
use crate::error::{Error, Result};
use crate::meta::{
    self, rules_allow, AssetIndex, Features, Library, ManifestVersion, VersionManifest, VersionMeta,
};
use crate::paths::Dirs;

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const RESOURCES: &str = "https://resources.download.minecraft.net";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
    pub instance_id: String,
    pub stage: String,
    pub current: u64,
    pub total: u64,
    pub file: Option<String>,
    pub message: String,
}

pub fn emit_progress(app: &AppHandle, p: InstallProgress) {
    if let Some(state) = app.try_state::<crate::AppState>() {
        if !p.message.is_empty() && p.stage != "done" {
            state.discord.set_installing(&p.message);
        }
    }
    let _ = app.emit("install-progress", p);
}

/// Clear the installing overlay after a failed install so the UI is not stuck.
pub fn emit_install_cleared(app: &AppHandle) {
    emit_progress(
        app,
        InstallProgress {
            instance_id: String::new(),
            stage: "done".into(),
            current: 0,
            total: 0,
            file: None,
            message: String::new(),
        },
    );
    let _ = app.emit("install-finished", ());
}

pub async fn fetch_manifest(client: &reqwest::Client, dirs: &Dirs) -> Result<VersionManifest> {
    let cache = dirs.cache.join("version_manifest_v2.json");
    match download_json::<VersionManifest>(client, MANIFEST_URL).await {
        Ok(m) => {
            let _ = std::fs::write(&cache, serde_json::to_vec_pretty(&m)?);
            Ok(m)
        }
        Err(e) => {
            if cache.exists() {
                Ok(serde_json::from_str(&std::fs::read_to_string(cache)?)?)
            } else {
                Err(e)
            }
        }
    }
}

pub async fn load_or_fetch_version_json(
    client: &reqwest::Client,
    dirs: &Dirs,
    id: &str,
    url: Option<&str>,
    sha1: Option<&str>,
) -> Result<VersionMeta> {
    let path = dirs.version_json(id);
    if path.exists() {
        return Ok(serde_json::from_str(&std::fs::read_to_string(&path)?)?);
    }
    let url = match url {
        Some(u) => u.to_string(),
        None => {
            let manifest = fetch_manifest(client, dirs).await?;
            manifest
                .versions
                .iter()
                .find(|v| v.id == id)
                .map(|v| v.url.clone())
                .ok_or_else(|| Error::msg(format!("Nie znaleziono wersji {id} w manifeście Mojang.")))?
        }
    };
    std::fs::create_dir_all(dirs.version_dir(id))?;
    download_file(client, &url, &path, sha1, None, None).await?;
    Ok(serde_json::from_str(&std::fs::read_to_string(&path)?)?)
}

pub async fn resolve_version(
    client: &reqwest::Client,
    dirs: &Dirs,
    id: &str,
) -> Result<VersionMeta> {
    let overlay = load_or_fetch_version_json(client, dirs, id, None, None).await?;
    if let Some(parent) = overlay.inherits_from.clone() {
        let base = Box::pin(resolve_version(client, dirs, &parent)).await?;
        Ok(meta::merge_versions(base, overlay))
    } else {
        Ok(overlay)
    }
}

pub struct LibraryFile {
    pub path: std::path::PathBuf,
    pub url: String,
    pub sha1: Option<String>,
    pub size: Option<u64>,
    pub native: bool,
    pub exclude: Vec<String>,
    pub name: String,
}

pub fn library_files(dirs: &Dirs, lib: &Library) -> Vec<LibraryFile> {
    let mut files = Vec::new();
    if !rules_allow(&lib.rules, &Features::default()) {
        return files;
    }
    let artifact = lib.downloads.as_ref().and_then(|d| d.artifact.as_ref());
    let artifact_url = artifact
        .and_then(|a| a.url.as_ref())
        .filter(|u| !u.is_empty());
    if let (Some(artifact), Some(url)) = (artifact, artifact_url) {
        let rel = artifact
            .path
            .clone()
            .unwrap_or_else(|| meta::maven_path(&lib.name));
        files.push(LibraryFile {
            path: dirs.library_file(&rel),
            url: url.clone(),
            sha1: artifact.sha1.clone(),
            size: artifact.size,
            native: meta::is_native_library(lib) && lib.natives.is_none(),
            exclude: lib.extract.as_ref().map(|e| e.exclude.clone()).unwrap_or_default(),
            name: lib.name.clone(),
        });
    } else if let Some(base) = &lib.url {
        let rel = meta::maven_path(&lib.name);
        let url = format!("{}/{}", base.trim_end_matches('/'), rel);
        files.push(LibraryFile {
            path: dirs.library_file(&rel),
            url,
            sha1: None,
            size: None,
            native: meta::is_native_library(lib),
            exclude: Vec::new(),
            name: lib.name.clone(),
        });
    } else if lib.downloads.is_none() || artifact.is_some() {
        // Forge/Quilt coords without a usable download URL — pick a Maven by group.
        let rel = meta::maven_path(&lib.name);
        let url = default_maven_url(&lib.name, &rel);
        files.push(LibraryFile {
            path: dirs.library_file(&rel),
            url,
            sha1: None,
            size: None,
            native: meta::is_native_library(lib),
            exclude: Vec::new(),
            name: lib.name.clone(),
        });
    }

    if let Some(classifier) = meta::native_classifier(lib) {
        if let Some(art) = lib
            .downloads
            .as_ref()
            .and_then(|d| d.classifiers.as_ref())
            .and_then(|c| c.get(&classifier))
        {
            if let Some(url) = art.url.as_ref().filter(|u| !u.is_empty()) {
                let rel = art
                    .path
                    .clone()
                    .unwrap_or_else(|| meta::maven_path(&format!("{}:{classifier}", lib.name)));
                files.push(LibraryFile {
                    path: dirs.library_file(&rel),
                    url: url.clone(),
                    sha1: art.sha1.clone(),
                    size: art.size,
                    native: true,
                    exclude: lib.extract.as_ref().map(|e| e.exclude.clone()).unwrap_or_default(),
                    name: format!("{}:{classifier}", lib.name),
                });
            }
        }
    }
    files
}

fn default_maven_url(name: &str, rel: &str) -> String {
    let base = if name.starts_with("net.minecraftforge") || name.starts_with("de.oceanlabs") {
        "https://maven.minecraftforge.net"
    } else if name.starts_with("net.neoforged") {
        "https://maven.neoforged.net/releases"
    } else if name.starts_with("org.quiltmc") {
        "https://maven.quiltmc.org/repository/release"
    } else if name.starts_with("net.fabricmc") || name.starts_with("net.fabricmc.fabric-api") {
        "https://maven.fabricmc.net"
    } else {
        "https://libraries.minecraft.net"
    };
    format!("{base}/{rel}")
}

pub async fn download_libraries(
    client: &reqwest::Client,
    app: &AppHandle,
    instance_id: &str,
    files: Vec<LibraryFile>,
    cancel: &tokio_util::sync::CancellationToken,
) -> Result<()> {
    let total: u64 = files.iter().map(|f| f.size.unwrap_or(1)).sum::<u64>().max(1);
    let progress = ByteProgress::new();
    progress.total.store(total, Ordering::Relaxed);
    let sem = Arc::new(Semaphore::new(24));
    let mut tasks = Vec::new();
    let files = Arc::new(files);
    for (idx, _) in files.iter().enumerate() {
        if cancel.is_cancelled() {
            return Err(Error::msg("Instalacja anulowana."));
        }
        let permit = sem.clone().acquire_owned().await.unwrap();
        let client = client.clone();
        let files = files.clone();
        let progress = progress.clone();
        let cancel = cancel.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = permit;
            if cancel.is_cancelled() {
                return Err(Error::msg("Instalacja anulowana."));
            }
            let f = &files[idx];
            download_file(
                &client,
                &f.url,
                &f.path,
                f.sha1.as_deref(),
                f.size,
                Some(&progress),
            )
            .await
            .map_err(|e| Error::msg(format!("{}: {e}", f.name)))
        }));
    }

    let app = app.clone();
    let instance_id = instance_id.to_string();
    let progress2 = progress.clone();
    let nfiles = files.len() as u64;
    let ticker = async {
        loop {
            tokio::time::sleep(Duration::from_millis(250)).await;
            emit_progress(
                &app,
                InstallProgress {
                    instance_id: instance_id.clone(),
                    stage: "libraries".into(),
                    current: progress2.current.load(Ordering::Relaxed),
                    total,
                    file: None,
                    message: format!("Biblioteki ({nfiles} plików)"),
                },
            );
        }
    };
    tokio::select! {
        _ = ticker => unreachable!(),
        res = futures::future::try_join_all(tasks) => {
            for r in res.map_err(|e| Error::msg(e.to_string()))? {
                r?;
            }
        }
    }
    Ok(())
}

pub async fn install_assets(
    client: &reqwest::Client,
    app: &AppHandle,
    dirs: &Dirs,
    instance_id: &str,
    meta: &VersionMeta,
    cancel: &tokio_util::sync::CancellationToken,
) -> Result<()> {
    let Some(index_ref) = &meta.asset_index else {
        return Ok(());
    };
    let index_path = dirs.assets.join("indexes").join(format!("{}.json", index_ref.id));
    if let Some(url) = &index_ref.url {
        download_file(
            client,
            url,
            &index_path,
            index_ref.sha1.as_deref(),
            index_ref.size,
            None,
        )
        .await?;
    }
    if !index_path.exists() {
        return Ok(());
    }
    let index: AssetIndex = serde_json::from_str(&std::fs::read_to_string(&index_path)?)?;
    let objects: Vec<(String, meta::AssetObject)> = index.objects.into_iter().collect();
    let total = objects.len() as u64;
    emit_progress(
        app,
        InstallProgress {
            instance_id: instance_id.into(),
            stage: "assets".into(),
            current: 0,
            total,
            file: None,
            message: format!("Zasoby gry ({total})"),
        },
    );
    let sem = Arc::new(Semaphore::new(48));
    let done = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let mut tasks = Vec::new();
    for (name, obj) in objects {
        if cancel.is_cancelled() {
            return Err(Error::msg("Instalacja anulowana."));
        }
        let permit = sem.clone().acquire_owned().await.unwrap();
        let client = client.clone();
        let hash = obj.hash.clone();
        let dest = dirs
            .assets
            .join("objects")
            .join(&hash[..2.min(hash.len())])
            .join(&hash);
        let url = format!("{RESOURCES}/{}/{}", &hash[..2.min(hash.len())], hash);
        let size = obj.size;
        let cancel = cancel.clone();
        let done = done.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = permit;
            if cancel.is_cancelled() {
                return Err(Error::msg("Instalacja anulowana."));
            }
            let _ = name;
            download_file(&client, &url, &dest, Some(&hash), Some(size), None).await?;
            done.fetch_add(1, Ordering::Relaxed);
            Ok::<_, Error>(())
        }));
    }
    let app = app.clone();
    let instance_id = instance_id.to_string();
    let done2 = done.clone();
    let ticker = async {
        loop {
            tokio::time::sleep(Duration::from_millis(300)).await;
            let c = done2.load(Ordering::Relaxed);
            emit_progress(
                &app,
                InstallProgress {
                    instance_id: instance_id.clone(),
                    stage: "assets".into(),
                    current: c,
                    total,
                    file: None,
                    message: format!("Zasoby gry ({c}/{total})"),
                },
            );
        }
    };
    tokio::select! {
        _ = ticker => unreachable!(),
        res = futures::future::try_join_all(tasks) => {
            for r in res.map_err(|e| Error::msg(e.to_string()))? {
                r?;
            }
        }
    }
    Ok(())
}

pub async fn install_vanilla(
    client: &reqwest::Client,
    app: &AppHandle,
    dirs: &Dirs,
    instance_id: &str,
    game_version: &str,
    cancel: &tokio_util::sync::CancellationToken,
) -> Result<VersionMeta> {
    emit_progress(
        app,
        InstallProgress {
            instance_id: instance_id.into(),
            stage: "meta".into(),
            current: 0,
            total: 1,
            file: None,
            message: format!("Pobieranie metadanych {game_version}"),
        },
    );
    let manifest = fetch_manifest(client, dirs).await?;
    let listed: Option<&ManifestVersion> = manifest.versions.iter().find(|v| v.id == game_version);
    let meta = load_or_fetch_version_json(
        client,
        dirs,
        game_version,
        listed.map(|v| v.url.as_str()),
        listed.and_then(|v| v.sha1.as_deref()),
    )
    .await?;
    if let Some(client_dl) = meta.downloads.as_ref().and_then(|d| d.client.as_ref()) {
        if let Some(url) = &client_dl.url {
            emit_progress(
                app,
                InstallProgress {
                    instance_id: instance_id.into(),
                    stage: "client".into(),
                    current: 0,
                    total: client_dl.size.unwrap_or(1),
                    file: Some(format!("{game_version}.jar")),
                    message: "Klient Minecraft".into(),
                },
            );
            download_file(
                client,
                url,
                &dirs.version_jar(game_version),
                client_dl.sha1.as_deref(),
                client_dl.size,
                None,
            )
            .await?;
        }
    }
    if let Some(log) = meta
        .logging
        .as_ref()
        .and_then(|l| l.client.as_ref())
        .and_then(|c| c.file.as_ref())
    {
        if let (Some(url), Some(id)) = (&log.url, &log.id) {
            let path = dirs.assets.join("log_configs").join(id);
            let _ = download_file(client, url, &path, log.sha1.as_deref(), log.size, None).await;
        }
    }
    let mut files = Vec::new();
    for lib in &meta.libraries {
        files.extend(library_files(dirs, lib));
    }
    download_libraries(client, app, instance_id, files, cancel).await?;
    install_assets(client, app, dirs, instance_id, &meta, cancel).await?;
    Ok(meta)
}
