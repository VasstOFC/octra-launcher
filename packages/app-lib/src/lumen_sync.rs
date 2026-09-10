//! Import instances and the multiplayer server list from the previous Octra Launcher.
//!
//! Instance game directories (`minecraft/`) are linked into this app's `profiles/` folder
//! so worlds, mods, and configs stay shared. The server list is read from the same
//! `servers.json` file both launchers use.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::state::{
    CreateInstance, EditInstance, InstanceLaunchOverridesPatch, InstanceLink,
    MemorySettings, ModLoader, State,
};
use crate::util::io;

const OCTRA_DATA_DIR: &str = ".octralauncher";
const OCTRA_DATA_DIR_DEV: &str = ".octralauncher-dev";
const IMPORT_STATE_FILE: &str = "octra-import-state.json";
const OCTRA_INSTANCE_PREFIX: &str = "octra-";

#[derive(Debug, Default, Serialize, Deserialize)]
struct ImportState {
    #[serde(default)]
    known_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OctraServerEntry {
    pub name: String,
    pub address: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct OctraServerListFile {
    #[serde(default)]
    servers: Vec<OctraServerEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OctraInstanceFile {
    id: String,
    name: String,
    game_version: String,
    #[serde(default)]
    loader: Option<String>,
    #[serde(default)]
    loader_version: Option<String>,
    #[serde(default)]
    last_played: Option<String>,
    #[serde(default)]
    memory_max_mb: Option<u32>,
    #[serde(default)]
    custom_memory: Option<bool>,
    #[serde(default)]
    play_time_secs: Option<u64>,
}

pub fn octra_launcher_dir() -> Option<PathBuf> {
    let data = dirs::data_dir()?;
    let primary = data.join(OCTRA_DATA_DIR);
    if primary.join("instances").is_dir() {
        return Some(primary);
    }
    let dev = data.join(OCTRA_DATA_DIR_DEV);
    if dev.join("instances").is_dir() {
        return Some(dev);
    }
    if primary.exists() {
        return Some(primary);
    }
    None
}

pub fn list_octra_servers() -> Vec<OctraServerEntry> {
    let Some(dir) = octra_launcher_dir() else {
        return Vec::new();
    };
    let path = dir.join("servers.json");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let parsed: OctraServerListFile =
        serde_json::from_str(&raw).unwrap_or_default();
    parsed
        .servers
        .into_iter()
        .filter(|s| !s.address.trim().is_empty())
        .collect()
}

pub fn remove_octra_server(address: &str) -> crate::Result<bool> {
    let trimmed = address.trim();
    if trimmed.is_empty() {
        return Ok(false);
    }
    let Some(dir) = octra_launcher_dir() else {
        return Ok(false);
    };
    let path = dir.join("servers.json");
    let raw = match std::fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(_) => return Ok(false),
    };
    let mut parsed: OctraServerListFile =
        serde_json::from_str(&raw).unwrap_or_default();
    let before = parsed.servers.len();
    let wanted = trimmed.to_ascii_lowercase();
    parsed.servers.retain(|server| {
        server.address.trim().to_ascii_lowercase() != wanted
    });
    if parsed.servers.len() == before {
        return Ok(false);
    }
    let encoded = serde_json::to_vec_pretty(&parsed).map_err(|error| {
        crate::ErrorKind::OtherError(format!(
            "failed to serialize octra servers.json: {error}"
        ))
    })?;
    std::fs::write(&path, encoded).map_err(|error| {
        crate::ErrorKind::FSError(format!(
            "failed to write octra servers.json: {error}"
        ))
    })?;
    Ok(true)
}

impl Default for OctraServerListFile {
    fn default() -> Self {
        Self {
            servers: Vec::new(),
        }
    }
}

pub async fn sync_from_octra_launcher() -> crate::Result<()> {
    let Some(octra_dir) = octra_launcher_dir() else {
        info!(
            "Octra Launcher data directory not found; skipping instance sync"
        );
        return Ok(());
    };

    import_octra_instances(&octra_dir).await
}

async fn import_octra_instances(octra_dir: &Path) -> crate::Result<()> {
    let instances_root = octra_dir.join("instances");
    if !instances_root.is_dir() {
        return Ok(());
    }

    let state = State::get().await?;
    let existing = crate::state::list_instances(&state.pool).await?;
    let existing_paths: Vec<String> =
        existing.iter().map(|m| m.instance.path.clone()).collect();
    let existing_names: Vec<String> = existing
        .iter()
        .map(|m| m.instance.name.to_lowercase())
        .collect();

    let mut known_ids = load_import_state(&state).await;
    for path in &existing_paths {
        if let Some(id) = path.strip_prefix(OCTRA_INSTANCE_PREFIX) {
            known_ids.insert(id.to_string());
        }
    }
    let _ = save_import_state(&state, &known_ids).await;

    let mut dir = match io::read_dir(&instances_root).await {
        Ok(dir) => dir,
        Err(error) => {
            warn!("Could not read Octra instances folder: {error}");
            return Ok(());
        }
    };

    let mut imported = 0u32;
    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        match import_one_instance(
            &path,
            &state,
            &existing_paths,
            &existing_names,
            &mut known_ids,
        )
        .await
        {
            Ok(true) => {
                imported += 1;
                let _ = save_import_state(&state, &known_ids).await;
            }
            Ok(false) => {}
            Err(error) => {
                warn!(
                    "Failed to import Octra instance from {}: {error}",
                    path.display()
                );
            }
        }
    }

    if imported > 0 {
        info!("Imported {imported} instance(s) from Octra Launcher");
        if let Err(error) =
            crate::onboarding_checklist::mark_created_instance().await
        {
            warn!("Failed to mark onboarding instance creation: {error}");
        }
    }

    Ok(())
}

pub async fn remember_removed_octra_instance(instance_path: &str) {
    let Some(id) = instance_path.strip_prefix(OCTRA_INSTANCE_PREFIX) else {
        return;
    };
    let Ok(state) = State::get().await else {
        return;
    };
    let mut known_ids = load_import_state(&state).await;
    if known_ids.insert(id.to_string()) {
        let _ = save_import_state(&state, &known_ids).await;
    }
}

pub async fn remove_instance_directory(path: &Path) -> crate::Result<()> {
    if path.exists() {
        remove_profile_placeholder(path).await?;
    }
    Ok(())
}

async fn load_import_state(state: &State) -> HashSet<String> {
    let path = state.directories.settings_dir.join(IMPORT_STATE_FILE);
    let Ok(raw) = io::read(&path).await else {
        return HashSet::new();
    };
    serde_json::from_slice::<ImportState>(&raw)
        .map(|state| state.known_ids.into_iter().collect())
        .unwrap_or_default()
}

async fn save_import_state(
    state: &State,
    ids: &HashSet<String>,
) -> crate::Result<()> {
    let path = state.directories.settings_dir.join(IMPORT_STATE_FILE);
    let mut known_ids: Vec<String> = ids.iter().cloned().collect();
    known_ids.sort();
    io::write(
        &path,
        serde_json::to_vec_pretty(&ImportState { known_ids })?,
    )
    .await?;
    Ok(())
}

async fn import_one_instance(
    instance_dir: &Path,
    state: &State,
    existing_paths: &[String],
    existing_names: &[String],
    known_ids: &mut HashSet<String>,
) -> crate::Result<bool> {
    let meta_path = instance_dir.join("instance.json");
    if !meta_path.exists() {
        return Ok(false);
    }
    let raw = io::read(&meta_path).await?;
    let meta: OctraInstanceFile = serde_json::from_slice(&raw)?;

    let linked_path = format!("{OCTRA_INSTANCE_PREFIX}{}", meta.id);
    if existing_paths.iter().any(|p| p == &linked_path) {
        known_ids.insert(meta.id.clone());
        return Ok(false);
    }
    if known_ids.contains(&meta.id) {
        let leftover = state.directories.instances_dir().join(&linked_path);
        if leftover.exists() {
            let _ = remove_profile_placeholder(&leftover).await;
        }
        return Ok(false);
    }
    if existing_names
        .iter()
        .any(|n| n == &meta.name.to_lowercase())
    {
        return Ok(false);
    }

    let leftover = state.directories.instances_dir().join(&linked_path);
    if leftover.exists() {
        remove_profile_placeholder(&leftover).await?;
    }

    let minecraft_dir = instance_dir.join("minecraft");
    if !minecraft_dir.is_dir() {
        warn!(
            "Octra instance '{}' has no minecraft folder; skipping",
            meta.name
        );
        return Ok(false);
    }

    let loader =
        ModLoader::from_string(meta.loader.as_deref().unwrap_or("vanilla"));

    let created = crate::state::create_instance(
        CreateInstance {
            name: meta.name.clone(),
            path: Some(linked_path.clone()),
            game_version: meta.game_version.clone(),
            loader,
            loader_version: meta.loader_version.clone(),
            icon_path: None,
            icon_config: None,
            link: InstanceLink::Unmanaged,
        },
        state,
    )
    .await?;

    if let Err(error) =
        replace_profile_with_link(&created.path, &minecraft_dir, state).await
    {
        let _ = crate::state::remove_instance(&created.id, state).await;
        return Err(error);
    }

    crate::state::instances::watcher::watch_instance_folder(
        &created.id,
        &created.path,
        &state.file_watcher,
        &state.directories,
    )
    .await;

    let wallpaper = instance_dir.join("wallpaper.png");
    let icon_path = if wallpaper.is_file() {
        match crate::api::instance::cache_icon_from_path(&wallpaper, state)
            .await
        {
            Ok(cached) => Some(cached.to_string_lossy().into_owned()),
            Err(error) => {
                warn!(
                    "Could not cache icon for Octra instance '{}': {error}",
                    meta.name
                );
                None
            }
        }
    } else {
        None
    };

    let last_played = meta.last_played.as_deref().and_then(|value| {
        DateTime::parse_from_rfc3339(value)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    });

    let memory = if meta.custom_memory.unwrap_or(false) {
        meta.memory_max_mb.map(|maximum| MemorySettings { maximum })
    } else {
        None
    };

    if icon_path.is_some()
        || last_played.is_some()
        || meta.play_time_secs.is_some()
        || memory.is_some()
    {
        if let Err(error) = crate::state::edit_instance(
            &created.id,
            EditInstance {
                icon_path: icon_path.map(Some),
                last_played: last_played.map(Some),
                submitted_time_played: meta.play_time_secs,
                launch_overrides: memory.map(|mem| {
                    InstanceLaunchOverridesPatch {
                        memory: Some(Some(mem)),
                        ..Default::default()
                    }
                }),
                ..Default::default()
            },
            &state.pool,
        )
        .await
        {
            warn!("Failed to apply Octra metadata to '{}': {error}", meta.name);
        }
    }

    let _ = emit_instance(&created.id, InstancePayloadType::Created).await;
    known_ids.insert(meta.id.clone());
    info!(
        "Linked Octra instance '{}' ({}) -> {}",
        meta.name, meta.id, created.path
    );
    Ok(true)
}

async fn replace_profile_with_link(
    instance_path: &str,
    minecraft_dir: &Path,
    state: &State,
) -> crate::Result<()> {
    let profile = state.directories.instances_dir().join(instance_path);
    if profile.exists() {
        remove_profile_placeholder(&profile).await?;
    }

    let target = io::canonicalize(minecraft_dir)?;
    let link = profile.clone();
    tokio::task::spawn_blocking(move || create_directory_link(&link, &target))
        .await
        .map_err(|e| {
            crate::ErrorKind::FSError(format!(
                "failed to create instance link: {e}"
            ))
        })?
        .map_err(|e| {
            crate::ErrorKind::FSError(format!(
                "failed to link Octra minecraft folder: {e}"
            ))
        })?;

    Ok(())
}

/// Removes a Theseus profile placeholder. Junctions are unlinked without
/// touching the Octra `minecraft/` target; real directories are deleted.
async fn remove_profile_placeholder(path: &Path) -> crate::Result<()> {
    if is_reparse_point(path) {
        io::remove_dir(path).await.map_err(|e| {
            crate::ErrorKind::FSError(format!(
                "could not unlink instance folder {}: {e}",
                path.display()
            ))
        })?;
    } else {
        io::remove_dir_all(path).await.map_err(|e| {
            crate::ErrorKind::FSError(format!(
                "could not replace instance folder {}: {e}",
                path.display()
            ))
        })?;
    }
    Ok(())
}

fn is_reparse_point(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        std::fs::symlink_metadata(path)
            .map(|meta| {
                (meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT) != 0
            })
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        std::fs::symlink_metadata(path)
            .map(|meta| meta.file_type().is_symlink())
            .unwrap_or(false)
    }
}

fn create_directory_link(link: &Path, target: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        let status = std::process::Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/J")
            .arg(link.as_os_str())
            .arg(target.as_os_str())
            .status()?;
        if !status.success() {
            return Err(std::io::Error::other(format!(
                "mklink /J failed for {} -> {}",
                link.display(),
                target.display()
            )));
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(target, link)
    }
}
