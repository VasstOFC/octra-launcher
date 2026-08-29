//! Import folderu instancji aplikacji CurseForge (`minecraftinstance.json`).
//! Nie mylić z paczką ZIP (`manifest.json`) — to gotowy folder z modami na dysku.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;

use crate::error::{Error, Result};
use crate::icon;
use crate::instances::{self, CreateInstance, Instance, Loader};
use crate::paths::Dirs;
use crate::settings::Settings;

const COPY_DIRS: &[&str] = &[
    "mods",
    "resourcepacks",
    "shaderpacks",
    "datapacks",
    "saves",
    "config",
    "defaultconfigs",
    "kubejs",
    "scripts",
    "patchouli_books",
];

const COPY_FILES: &[&str] = &[
    "options.txt",
    "optionsof.txt",
    "servers.dat",
    "icon.png",
];

const SKIP_TOP: &[&str] = &["libraries", "versions", "natives", ".curseclient"];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeInstanceHit {
    pub path: String,
    pub name: String,
    pub game_version: String,
    pub loader: Loader,
    pub loader_version: Option<String>,
}

pub fn default_instances_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let path = home.join("curseforge").join("minecraft").join("Instances");
    path.is_dir().then_some(path)
}

pub fn scan(root: Option<&Path>) -> Result<Vec<CurseForgeInstanceHit>> {
    let dir = match root {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => default_instances_dir().ok_or_else(|| {
            Error::msg(
                "Nie znaleziono folderu instancji CurseForge. Wskaż katalog Instances albo sam folder instancji.",
            )
        })?,
    };
    if !dir.is_dir() {
        return Err(Error::msg("Wybrana ścieżka nie jest folderem."));
    }
    if instance_json(&dir).is_some() {
        return match parse_hit(&dir) {
            Ok(hit) => Ok(vec![hit]),
            Err(e) => Err(e),
        };
    }
    let mut hits = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path();
        if instance_json(&path).is_none() {
            continue;
        }
        if let Ok(hit) = parse_hit(&path) {
            hits.push(hit);
        }
    }
    hits.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(hits)
}

pub fn import_folder(dirs: &Dirs, settings: &Settings, path: &Path) -> Result<Instance> {
    if !path.is_dir() {
        return Err(Error::msg("Wskaż folder instancji CurseForge."));
    }
    let json_path = instance_json(path).ok_or_else(|| {
        Error::msg("W tym folderze nie ma minecraftinstance.json — to nie instancja aplikacji CurseForge.")
    })?;
    let raw = std::fs::read_to_string(&json_path)?;
    let value: Value = serde_json::from_str(&raw)?;
    let parsed = parse_value(&value, path)?;
    if parsed.loader != Loader::Vanilla
        && parsed
            .loader_version
            .as_ref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(true)
    {
        return Err(Error::msg(
            "Nie udało się odczytać wersji loadera z instancji CurseForge.",
        ));
    }
    let name = instances::unique_instance_name(dirs, &parsed.name)?;
    let req = CreateInstance {
        name,
        game_version: parsed.game_version,
        loader: parsed.loader,
        loader_version: parsed.loader_version,
        memory_max_mb: None,
    };
    let mut inst = instances::create(dirs, settings, req)?;
    let src_root = content_root(path, &value);
    let dest = dirs.game_dir(&inst.id);
    std::fs::create_dir_all(&dest)?;
    copy_instance_content(&src_root, &dest)?;
    if src_root != path {
        copy_named_file(path, &dest, "icon.png")?;
    }
    if icon::adopt_from_game_dir(dirs, &mut inst).unwrap_or(false) {
        instances::save(dirs, &inst)?;
    }
    Ok(inst)
}

fn instance_json(dir: &Path) -> Option<PathBuf> {
    for name in ["minecraftinstance.json", "MinecraftInstance.json"] {
        let p = dir.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn parse_hit(dir: &Path) -> Result<CurseForgeInstanceHit> {
    let json_path = instance_json(dir).ok_or_else(|| Error::msg("Brak minecraftinstance.json."))?;
    let raw = std::fs::read_to_string(json_path)?;
    let value: Value = serde_json::from_str(&raw)?;
    let parsed = parse_value(&value, dir)?;
    Ok(CurseForgeInstanceHit {
        path: dir.to_string_lossy().into_owned(),
        name: parsed.name,
        game_version: parsed.game_version,
        loader: parsed.loader,
        loader_version: parsed.loader_version,
    })
}

struct ParsedInstance {
    name: String,
    game_version: String,
    loader: Loader,
    loader_version: Option<String>,
}

fn parse_value(v: &Value, dir: &Path) -> Result<ParsedInstance> {
    let folder_name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Instancja CurseForge")
        .to_string();
    let name = first_str(v, &["name", "instanceName", "displayName"])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&folder_name)
        .to_string();

    let gd = v.get("loader");
    let base = v.get("baseModLoader");

    let game_version = first_str(v, &["gameVersion", "minecraftVersion", "mcVersion"])
        .or_else(|| json_str(base, "minecraftVersion"))
        .or_else(|| json_str(gd, "mcVersion"))
        .or_else(|| json_str(gd, "minecraftVersion"))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::msg("W instancji CurseForge brak wersji Minecraft."))?
        .to_string();

    let (loader, loader_version) = parse_loader_fields(v, &game_version);
    Ok(ParsedInstance {
        name,
        game_version,
        loader,
        loader_version,
    })
}

fn parse_loader_fields(v: &Value, mc: &str) -> (Loader, Option<String>) {
    if let Some(gd) = v.get("loader") {
        if let Some(kind) = json_str(Some(gd), "loaderType")
            .or_else(|| json_str(Some(gd), "type"))
            .or_else(|| json_str(Some(gd), "modLoader"))
        {
            let loader = loader_from_name(kind);
            let ver = json_str(Some(gd), "loaderVersion")
                .or_else(|| json_str(Some(gd), "version"))
                .map(|s| strip_loader_version(s, mc));
            let ver = ver.filter(|s| !s.is_empty());
            if loader != Loader::Vanilla {
                return (loader, ver);
            }
        }
    }

    let base = v.get("baseModLoader");
    let loader = json_value(base, "modLoader")
        .map(loader_from_json)
        .or_else(|| json_str(base, "name").map(loader_from_name))
        .unwrap_or(Loader::Vanilla);

    let ver = json_str(base, "forgeVersion")
        .or_else(|| json_str(base, "version"))
        .or_else(|| json_str(base, "name"))
        .map(|s| strip_loader_version(s, mc))
        .filter(|s| !s.is_empty());

    if loader == Loader::Vanilla {
        if let Some(name) = json_str(base, "name") {
            let (parsed, pv) = loader_from_prefixed(name, mc);
            if parsed != Loader::Vanilla {
                return (parsed, pv.or(ver));
            }
        }
        return (Loader::Vanilla, None);
    }
    (loader, ver)
}

fn loader_from_json(v: &Value) -> Loader {
    if let Some(n) = v.as_i64() {
        return match n {
            1 => Loader::Forge,
            4 => Loader::Fabric,
            5 => Loader::Quilt,
            6 => Loader::Neoforge,
            _ => Loader::Vanilla,
        };
    }
    if let Some(s) = v.as_str() {
        return loader_from_name(s);
    }
    Loader::Vanilla
}

fn loader_from_name(s: &str) -> Loader {
    let lower = s.trim().to_ascii_lowercase();
    if lower.contains("neoforge") || lower == "neo" {
        Loader::Neoforge
    } else if lower.contains("fabric") {
        Loader::Fabric
    } else if lower.contains("quilt") {
        Loader::Quilt
    } else if lower.contains("forge") {
        Loader::Forge
    } else {
        Loader::Vanilla
    }
}

fn loader_from_prefixed(id: &str, mc: &str) -> (Loader, Option<String>) {
    let id = id.trim();
    let lower = id.to_ascii_lowercase();
    let pairs: [(&str, Loader); 6] = [
        ("neoforge-", Loader::Neoforge),
        ("forge-", Loader::Forge),
        ("fabric-loader-", Loader::Fabric),
        ("quilt-loader-", Loader::Quilt),
        ("fabric-", Loader::Fabric),
        ("quilt-", Loader::Quilt),
    ];
    for (prefix, loader) in pairs {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let ver = id.get(prefix.len()..).unwrap_or(rest).trim();
            let ver = strip_loader_version(ver, mc);
            if ver.is_empty() {
                return (loader, None);
            }
            return (loader, Some(ver));
        }
    }
    (Loader::Vanilla, None)
}

pub(crate) fn strip_loader_version(raw: &str, mc: &str) -> String {
    let mut s = raw.trim().to_string();
    let lower = s.to_ascii_lowercase();
    for prefix in [
        "fabric-loader-",
        "quilt-loader-",
        "neoforge-",
        "forge-",
        "fabric-",
        "quilt-",
        "neo-",
    ] {
        if lower.starts_with(prefix) {
            s = s[prefix.len()..].trim().to_string();
            break;
        }
    }
    if !mc.is_empty() {
        let p = format!("{mc}-");
        if s.starts_with(&p) {
            s = s[p.len()..].to_string();
        }
    }
    s.trim().to_string()
}

fn content_root(instance_dir: &Path, json: &Value) -> PathBuf {
    if let Some(p) = first_str(json, &["installPath", "gameDir", "minecraftPath"]) {
        let p = PathBuf::from(p);
        if p.is_dir() {
            return prefer_minecraft_subdir(&p);
        }
    }
    prefer_minecraft_subdir(instance_dir)
}

fn prefer_minecraft_subdir(dir: &Path) -> PathBuf {
    let mc = dir.join("minecraft");
    if mc.is_dir()
        && (mc.join("mods").is_dir()
            || mc.join("saves").is_dir()
            || mc.join("options.txt").is_file())
    {
        return mc;
    }
    dir.to_path_buf()
}

fn copy_instance_content(src: &Path, dest: &Path) -> Result<()> {
    for name in COPY_DIRS {
        if SKIP_TOP.contains(name) {
            continue;
        }
        let from = src.join(name);
        if from.is_dir() {
            instances::copy_dir(&from, &dest.join(name))?;
        }
    }
    for name in COPY_FILES {
        copy_named_file(src, dest, name)?;
    }
    Ok(())
}

fn copy_named_file(src: &Path, dest: &Path, name: &str) -> Result<()> {
    let from = src.join(name);
    if from.is_file() {
        std::fs::create_dir_all(dest)?;
        std::fs::copy(&from, dest.join(name))?;
    }
    Ok(())
}

fn first_str<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a str> {
    for k in keys {
        if let Some(s) = v.get(*k).and_then(|x| x.as_str()) {
            if !s.trim().is_empty() {
                return Some(s);
            }
        }
    }
    None
}

fn json_str<'a>(obj: Option<&'a Value>, key: &str) -> Option<&'a str> {
    obj.and_then(|v| v.get(key)).and_then(|x| x.as_str())
}

fn json_value<'a>(obj: Option<&'a Value>, key: &str) -> Option<&'a Value> {
    obj.and_then(|v| v.get(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_loader_prefixes() {
        assert_eq!(strip_loader_version("fabric-loader-0.15.0", "1.20.1"), "0.15.0");
        assert_eq!(strip_loader_version("forge-47.2.0", "1.20.1"), "47.2.0");
        assert_eq!(strip_loader_version("1.20.1-47.2.0", "1.20.1"), "47.2.0");
        assert_eq!(strip_loader_version("0.16.9", "1.21"), "0.16.9");
    }

    #[test]
    fn parses_curseforge_app_json() {
        let v: Value = serde_json::from_str(
            r#"{
                "name": "ATM9",
                "gameVersion": "1.20.1",
                "baseModLoader": {
                    "forgeVersion": "47.2.20",
                    "name": "forge-47.2.20",
                    "minecraftVersion": "1.20.1",
                    "modLoader": 1
                }
            }"#,
        )
        .unwrap();
        let dir = Path::new("/tmp/ATM9");
        let p = parse_value(&v, dir).unwrap();
        assert_eq!(p.name, "ATM9");
        assert_eq!(p.game_version, "1.20.1");
        assert_eq!(p.loader, Loader::Forge);
        assert_eq!(p.loader_version.as_deref(), Some("47.2.20"));
    }

    #[test]
    fn parses_modloader_enum_and_strings() {
        let fabric: Value = serde_json::json!({
            "name": "Fab",
            "gameVersion": "1.21",
            "baseModLoader": { "modLoader": 4, "forgeVersion": "fabric-loader-0.16.0" }
        });
        let p = parse_value(&fabric, Path::new("/x/Fab")).unwrap();
        assert_eq!(p.loader, Loader::Fabric);
        assert_eq!(p.loader_version.as_deref(), Some("0.16.0"));

        let neo: Value = serde_json::json!({
            "name": "Neo",
            "baseModLoader": {
                "minecraftVersion": "1.21.1",
                "modLoader": "NeoForge",
                "name": "neoforge-21.1.66"
            }
        });
        let p = parse_value(&neo, Path::new("/x/Neo")).unwrap();
        assert_eq!(p.game_version, "1.21.1");
        assert_eq!(p.loader, Loader::Neoforge);
        assert_eq!(p.loader_version.as_deref(), Some("21.1.66"));

        let quilt: Value = serde_json::json!({
            "name": "Q",
            "gameVersion": "1.20.4",
            "baseModLoader": { "modLoader": 5, "forgeVersion": "0.26.1" }
        });
        let p = parse_value(&quilt, Path::new("/x/Q")).unwrap();
        assert_eq!(p.loader, Loader::Quilt);
        assert_eq!(p.loader_version.as_deref(), Some("0.26.1"));
    }

    #[test]
    fn parses_gdlauncher_shape() {
        let v: Value = serde_json::json!({
            "name": "GD Pack",
            "loader": {
                "loaderType": "fabric",
                "loaderVersion": "fabric-loader-0.15.11",
                "mcVersion": "1.20.4"
            }
        });
        let p = parse_value(&v, Path::new("/x/gd")).unwrap();
        assert_eq!(p.game_version, "1.20.4");
        assert_eq!(p.loader, Loader::Fabric);
        assert_eq!(p.loader_version.as_deref(), Some("0.15.11"));
    }
}
