//! Import profili z Prism Launcher i MultiMC.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use walkdir::WalkDir;

use crate::error::{Error, Result};
use crate::instances::{self, CreateInstance, Instance, Loader};
use crate::paths::Dirs;
use crate::settings::Settings;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherInstanceHit {
    pub path: String,
    pub name: String,
    pub game_version: String,
    pub loader: Loader,
    pub loader_version: Option<String>,
    pub source: String,
}

#[derive(Debug, Deserialize)]
struct MmcPack {
    #[serde(default)]
    components: Vec<MmcComponent>,
}

#[derive(Debug, Deserialize)]
struct MmcComponent {
    #[serde(default)]
    uid: String,
    #[serde(default)]
    version: String,
}

fn prism_root() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("PrismLauncher").join("instances"))
}

fn multimc_roots() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(data) = dirs::data_dir() {
        out.push(data.join("MultiMC").join("instances"));
        out.push(data.join("multimc").join("instances"));
    }
    if let Some(home) = dirs::home_dir() {
        out.push(home.join("MultiMC").join("instances"));
    }
    out
}

fn parse_mmc_pack(path: &Path) -> Result<(String, Loader, Option<String>)> {
    let raw = std::fs::read_to_string(path)?;
    let pack: MmcPack = serde_json::from_str(&raw)?;
    let mut game_version = String::new();
    let mut loader = Loader::Vanilla;
    let mut loader_version = None;

    for c in &pack.components {
        let uid = c.uid.to_lowercase();
        if uid == "net.minecraft" {
            game_version = c.version.clone();
        } else if uid.contains("fabric") {
            loader = Loader::Fabric;
            loader_version = Some(c.version.clone());
        } else if uid.contains("quilt") {
            loader = Loader::Quilt;
            loader_version = Some(c.version.clone());
        } else if uid.contains("neoforge") {
            loader = Loader::Neoforge;
            loader_version = Some(c.version.clone());
        } else if uid.contains("forge") || uid.contains("minecraftforge") {
            loader = Loader::Forge;
            loader_version = Some(c.version.clone());
        }
    }

    if game_version.is_empty() {
        return Err(Error::msg("Brak wersji Minecraft w mmc-pack.json."));
    }
    Ok((game_version, loader, loader_version))
}

fn instance_name_from_dir(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Import")
        .to_string()
}

fn scan_instances_dir(root: &Path, source: &str) -> Result<Vec<LauncherInstanceHit>> {
    let mut out = Vec::new();
    if !root.is_dir() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let pack = dir.join("mmc-pack.json");
        if !pack.exists() {
            continue;
        }
        let Ok((game_version, loader, loader_version)) = parse_mmc_pack(&pack) else {
            continue;
        };
        let name = instance_name_from_dir(&dir);
        out.push(LauncherInstanceHit {
            path: dir.to_string_lossy().to_string(),
            name,
            game_version,
            loader,
            loader_version,
            source: source.to_string(),
        });
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

pub fn scan_prism_instances() -> Result<Vec<LauncherInstanceHit>> {
    let Some(root) = prism_root() else {
        return Ok(Vec::new());
    };
    scan_instances_dir(&root, "prism")
}

pub fn scan_multimc_instances(root: Option<&Path>) -> Result<Vec<LauncherInstanceHit>> {
    if let Some(r) = root {
        return scan_instances_dir(r, "multimc");
    }
    let mut all = Vec::new();
    for r in multimc_roots() {
        all.extend(scan_instances_dir(&r, "multimc")?);
    }
    Ok(all)
}

pub fn import_launcher_instance(
    dirs: &Dirs,
    settings: &Settings,
    path: &str,
    source: &str,
) -> Result<Instance> {
    let dir = PathBuf::from(path);
    if !dir.is_dir() {
        return Err(Error::msg("Folder profilu nie istnieje."));
    }
    let pack_path = dir.join("mmc-pack.json");
    if !pack_path.exists() {
        return Err(Error::msg("Brak mmc-pack.json — to nie profil Prism/MultiMC."));
    }
    let (game_version, loader, loader_version) = parse_mmc_pack(&pack_path)?;
    let name = instance_name_from_dir(&dir);
    let inst = instances::create(
        dirs,
        settings,
        CreateInstance {
            name: format!("{name} ({source})"),
            game_version,
            loader,
            loader_version,
            memory_max_mb: None,
        },
    )?;

    let dst = dirs.game_dir(&inst.id);
    copy_game_files(&dir, &dst)?;
    Ok(inst)
}

fn copy_game_files(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for sub in ["mods", "config", "resourcepacks", "shaderpacks", "saves", "datapacks"] {
        let from = src.join(sub);
        if from.is_dir() {
            copy_dir_recursive(&from, &dst.join(sub))?;
        }
    }
    Ok(())
}

fn copy_dir_recursive(from: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in WalkDir::new(from).min_depth(1).into_iter().filter_map(|e| e.ok()) {
        let rel = entry.path().strip_prefix(from).unwrap_or(entry.path());
        let target = to.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let _ = std::fs::copy(entry.path(), &target);
        }
    }
    Ok(())
}
