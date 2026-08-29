use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::icon;
use crate::paths::Dirs;
use crate::settings::Settings;

const DISABLED_SUFFIX: &str = ".disabled";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Loader {
    Vanilla,
    Fabric,
    Quilt,
    Forge,
    Neoforge,
}

impl Loader {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
            Self::Forge => "forge",
            Self::Neoforge => "neoforge",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub game_version: String,
    pub loader: Loader,
    #[serde(default)]
    pub loader_version: Option<String>,
    pub version_id: String,
    pub created_at: String,
    #[serde(default)]
    pub last_played: Option<String>,
    pub memory_max_mb: u32,
    pub memory_min_mb: u32,
    #[serde(default)]
    pub java_path: Option<String>,
    #[serde(default)]
    pub java_args: String,
    #[serde(default)]
    pub join_server: String,
    #[serde(default)]
    pub icon_color: String,
    #[serde(default)]
    pub icon_symbol: String,
    /// Plik ikony względem katalogu instancji, np. `icon.png`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_path: Option<String>,
    /// Plik tapety względem katalogu instancji, np. `wallpaper.jpg`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallpaper_path: Option<String>,
    /// Akcent LED z logo (hex). Pusty = licz z `icon_color`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub led_color: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub led_color_2: String,
    #[serde(default)]
    pub play_time_secs: u64,
    /// Slug Modrinth, `"mrpack"` albo `"curseforge"` przy imporcie pliku. Brak = zwykła instancja.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linked_pack: Option<String>,
    /// `true` dopóki użytkownik nie odłączy paczki — wtedy można edytować zawartość.
    #[serde(default)]
    pub pack_locked: bool,
    #[serde(default)]
    pub custom_java: bool,
    /// Brak pola w starym JSON = instancja miała własny RAM.
    #[serde(default = "default_true")]
    pub custom_memory: bool,
    #[serde(default)]
    pub custom_java_args: bool,
    #[serde(default)]
    pub custom_env: bool,
    #[serde(default)]
    pub custom_window: bool,
    #[serde(default)]
    pub custom_hooks: bool,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default = "default_window_width")]
    pub window_width: u32,
    #[serde(default = "default_window_height")]
    pub window_height: u32,
    #[serde(default)]
    pub env_vars: String,
    #[serde(default)]
    pub pre_launch: String,
    #[serde(default)]
    pub wrapper: String,
    #[serde(default)]
    pub post_exit: String,
}

impl Instance {
    pub fn memory_max(&self, settings: &Settings) -> u32 {
        if self.custom_memory {
            self.memory_max_mb.max(512)
        } else {
            settings.memory_max_mb.max(512)
        }
    }

    pub fn memory_min(&self, settings: &Settings) -> u32 {
        if self.custom_memory {
            self.memory_min_mb.max(256)
        } else {
            settings.memory_min_mb.max(256)
        }
    }

    pub fn extra_java_args(&self, settings: &Settings) -> String {
        if self.custom_java_args {
            self.java_args.clone()
        } else if !self.java_args.trim().is_empty() {
            self.java_args.clone()
        } else {
            settings.default_java_args.clone()
        }
    }

    pub fn env_vars_text(&self, settings: &Settings) -> String {
        if self.custom_env {
            self.env_vars.clone()
        } else {
            settings.default_env_vars.clone()
        }
    }

    pub fn window_size(&self, settings: &Settings) -> (u32, u32, bool) {
        if self.custom_window {
            (
                self.window_width.max(1),
                self.window_height.max(1),
                self.fullscreen,
            )
        } else {
            (
                settings.default_window_width.max(1),
                settings.default_window_height.max(1),
                settings.default_fullscreen,
            )
        }
    }
}

fn default_true() -> bool {
    true
}
fn default_window_width() -> u32 {
    854
}
fn default_window_height() -> u32 {
    480
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstance {
    pub name: String,
    pub game_version: String,
    pub loader: Loader,
    #[serde(default)]
    pub loader_version: Option<String>,
    #[serde(default)]
    pub memory_max_mb: Option<u32>,
}

pub fn list(dirs: &Dirs) -> Result<Vec<Instance>> {
    let mut items = Vec::new();
    if !dirs.instances.exists() {
        return Ok(items);
    }
    for entry in std::fs::read_dir(&dirs.instances)? {
        let entry = entry?;
        let file = entry.path().join("instance.json");
        if file.exists() {
            let raw = std::fs::read_to_string(&file)?;
            if let Ok(inst) = serde_json::from_str::<Instance>(&raw) {
                items.push(inst);
            }
        }
    }
    items.sort_by(|a, b| b.last_played.cmp(&a.last_played).then(b.created_at.cmp(&a.created_at)));
    Ok(items)
}

pub fn get(dirs: &Dirs, id: &str) -> Result<Instance> {
    let file = dirs.instance_dir(id).join("instance.json");
    if !file.exists() {
        return Err(Error::msg("Nie znaleziono instancji."));
    }
    Ok(serde_json::from_str(&std::fs::read_to_string(file)?)?)
}

pub fn save(dirs: &Dirs, inst: &Instance) -> Result<()> {
    let dir = dirs.instance_dir(&inst.id);
    std::fs::create_dir_all(dirs.game_dir(&inst.id))?;
    std::fs::create_dir_all(dirs.instance_logs(&inst.id))?;
    std::fs::write(dir.join("instance.json"), serde_json::to_string_pretty(inst)?)?;
    Ok(())
}

pub fn create(dirs: &Dirs, settings: &Settings, req: CreateInstance) -> Result<Instance> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err(Error::msg("Podaj nazwę instancji."));
    }
    if req.game_version.trim().is_empty() {
        return Err(Error::msg("Wybierz wersję Minecraft."));
    }
    if req.loader != Loader::Vanilla && req.loader_version.as_ref().map(|s| s.is_empty()).unwrap_or(true)
    {
        return Err(Error::msg("Wybierz wersję loadera."));
    }
    let id = Uuid::new_v4().to_string();
    let version_id = compute_version_id(&req);
    let inst = Instance {
        id,
        name: name.to_string(),
        game_version: req.game_version,
        loader: req.loader,
        loader_version: req.loader_version,
        version_id,
        created_at: Utc::now().to_rfc3339(),
        last_played: None,
        memory_max_mb: req.memory_max_mb.unwrap_or(settings.memory_max_mb),
        memory_min_mb: settings.memory_min_mb,
        java_path: None,
        java_args: String::new(),
        join_server: String::new(),
        icon_color: String::new(),
        icon_symbol: String::new(),
        icon_path: None,
        wallpaper_path: None,
        led_color: String::new(),
        led_color_2: String::new(),
        play_time_secs: 0,
        linked_pack: None,
        pack_locked: false,
        custom_java: false,
        custom_memory: req.memory_max_mb.is_some(),
        custom_java_args: false,
        custom_env: false,
        custom_window: false,
        custom_hooks: false,
        fullscreen: settings.default_fullscreen,
        window_width: settings.default_window_width.max(1),
        window_height: settings.default_window_height.max(1),
        env_vars: String::new(),
        pre_launch: String::new(),
        wrapper: String::new(),
        post_exit: String::new(),
    };
    save(dirs, &inst)?;
    Ok(inst)
}

pub fn compute_version_id(req: &CreateInstance) -> String {
    match req.loader {
        Loader::Vanilla => req.game_version.clone(),
        Loader::Fabric => format!(
            "{}-fabric-{}",
            req.game_version,
            req.loader_version.as_deref().unwrap_or("unknown")
        ),
        Loader::Quilt => format!(
            "{}-quilt-{}",
            req.game_version,
            req.loader_version.as_deref().unwrap_or("unknown")
        ),
        Loader::Forge => format!(
            "{}-forge-{}",
            req.game_version,
            req.loader_version.as_deref().unwrap_or("unknown")
        ),
        Loader::Neoforge => format!(
            "neoforge-{}",
            req.loader_version.as_deref().unwrap_or("unknown")
        ),
    }
}

pub fn delete(dirs: &Dirs, id: &str) -> Result<()> {
    let dir = dirs.instance_dir(id);
    if dir.exists() {
        std::fs::remove_dir_all(dir)?;
    }
    Ok(())
}

pub fn touch_played(dirs: &Dirs, id: &str) -> Result<Instance> {
    let mut inst = get(dirs, id)?;
    inst.last_played = Some(Utc::now().to_rfc3339());
    save(dirs, &inst)?;
    Ok(inst)
}

pub fn add_play_time(dirs: &Dirs, id: &str, secs: u64) -> Result<Instance> {
    let mut inst = get(dirs, id)?;
    inst.play_time_secs = inst.play_time_secs.saturating_add(secs);
    save(dirs, &inst)?;
    Ok(inst)
}

pub fn duplicate(dirs: &Dirs, id: &str) -> Result<Instance> {
    let src = get(dirs, id)?;
    let src_dir = dirs.instance_dir(id);
    if !src_dir.exists() {
        return Err(Error::msg("Nie znaleziono instancji."));
    }
    let new_id = Uuid::new_v4().to_string();
    let dest_dir = dirs.instance_dir(&new_id);
    if dest_dir.exists() {
        return Err(Error::msg("Nie udało się utworzyć kopii — ID zajęte."));
    }
    copy_dir(&src_dir, &dest_dir)?;
    let mut copy = src;
    copy.id = new_id;
    copy.name = unique_copy_name(dirs, &copy.name)?;
    copy.created_at = Utc::now().to_rfc3339();
    copy.last_played = None;
    copy.play_time_secs = 0;
    save(dirs, &copy)?;
    Ok(copy)
}

fn unique_copy_name(dirs: &Dirs, name: &str) -> Result<String> {
    unique_labeled_name(dirs, name, "kopia")
}

pub fn unique_instance_name(dirs: &Dirs, name: &str) -> Result<String> {
    let existing: Vec<String> = list(dirs)?.into_iter().map(|i| i.name).collect();
    Ok(unique_among(&existing, name))
}

fn unique_labeled_name(dirs: &Dirs, name: &str, label: &str) -> Result<String> {
    let existing: Vec<String> = list(dirs)?.into_iter().map(|i| i.name).collect();
    Ok(unique_labeled_among(&existing, name, label))
}

fn unique_among(existing: &[String], name: &str) -> String {
    if !existing.iter().any(|n| n.eq_ignore_ascii_case(name)) {
        return name.to_string();
    }
    for n in 2..1000 {
        let candidate = format!("{name} ({n})");
        if !existing.iter().any(|e| e.eq_ignore_ascii_case(&candidate)) {
            return candidate;
        }
    }
    format!("{name} ({})", Uuid::new_v4().simple())
}

fn unique_world_folder(existing: &[String], folder: &str) -> String {
    if !existing.iter().any(|n| n.eq_ignore_ascii_case(folder)) {
        return folder.to_string();
    }
    unique_labeled_among(existing, folder, "kopia")
}

fn unique_labeled_among(existing: &[String], name: &str, label: &str) -> String {
    let base = format!("{name} ({label})");
    if !existing.iter().any(|n| n.eq_ignore_ascii_case(&base)) {
        return base;
    }
    for n in 2..1000 {
        let candidate = format!("{name} ({label} {n})");
        if !existing.iter().any(|e| e.eq_ignore_ascii_case(&candidate)) {
            return candidate;
        }
    }
    format!("{name} ({label} {})", Uuid::new_v4().simple())
}

pub(crate) fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in walkdir::WalkDir::new(src).into_iter().flatten() {
        let path = entry.path();
        let rel = match path.strip_prefix(src) {
            Ok(r) if !r.as_os_str().is_empty() => r,
            _ => continue,
        };
        let dest = dst.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(path, &dest)?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldEntry {
    pub name: String,
    pub folder: String,
    pub size: u64,
}

pub fn list_worlds(dirs: &Dirs, id: &str) -> Result<Vec<WorldEntry>> {
    let _ = get(dirs, id)?;
    let saves = dirs.game_dir(id).join("saves");
    let mut out = Vec::new();
    if !saves.exists() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(&saves)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let Some(folder) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if folder.starts_with('.') {
            continue;
        }
        out.push(WorldEntry {
            name: folder.clone(),
            folder,
            size: dir_size(&entry.path()),
        });
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

fn dir_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

pub fn delete_world(dirs: &Dirs, id: &str, folder: &str) -> Result<Vec<WorldEntry>> {
    let _ = get(dirs, id)?;
    let name = safe_file_name(folder)?;
    let path = dirs.game_dir(id).join("saves").join(name);
    if !path.is_dir() {
        return Err(Error::msg("Nie znaleziono świata."));
    }
    std::fs::remove_dir_all(path)?;
    list_worlds(dirs, id)
}

pub fn world_dir(dirs: &Dirs, id: &str, folder: &str) -> Result<PathBuf> {
    let _ = get(dirs, id)?;
    let name = safe_file_name(folder)?;
    let path = dirs.game_dir(id).join("saves").join(name);
    if !path.is_dir() {
        return Err(Error::msg("Nie znaleziono świata."));
    }
    Ok(path)
}

pub fn copy_world(dirs: &Dirs, from_id: &str, folder: &str, to_id: &str) -> Result<Vec<WorldEntry>> {
    if from_id == to_id {
        return Err(Error::msg("Wybierz inną instancję docelową."));
    }
    let src = world_dir(dirs, from_id, folder)?;
    let _ = get(dirs, to_id)?;
    let dest_saves = dirs.game_dir(to_id).join("saves");
    std::fs::create_dir_all(&dest_saves)?;
    let existing: Vec<String> = list_worlds(dirs, to_id)?
        .into_iter()
        .map(|w| w.folder)
        .collect();
    let dest_folder = unique_world_folder(&existing, folder);
    let dest_name = safe_file_name(&dest_folder)?;
    copy_dir(&src, &dest_saves.join(dest_name))?;
    list_worlds(dirs, from_id)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrashReport {
    pub name: String,
    pub size: u64,
    pub modified: Option<String>,
}

pub fn list_crash_reports(dirs: &Dirs, id: &str) -> Result<Vec<CrashReport>> {
    let _ = get(dirs, id)?;
    let dir = dirs.game_dir(id).join("crash-reports");
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_string) else {
            continue;
        };
        let meta = entry.metadata().ok();
        let modified = meta.as_ref().and_then(|m| m.modified().ok()).map(|t| {
            chrono::DateTime::<Utc>::from(t).to_rfc3339()
        });
        out.push(CrashReport {
            name,
            size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
            modified,
        });
    }
    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(out)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotEntry {
    pub name: String,
    pub size: u64,
    pub modified: Option<String>,
}

pub fn list_screenshots(dirs: &Dirs, id: &str) -> Result<Vec<ScreenshotEntry>> {
    let _ = get(dirs, id)?;
    let dir = dirs.game_dir(id).join("screenshots");
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_string) else {
            continue;
        };
        let lower = name.to_ascii_lowercase();
        if !(lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg")) {
            continue;
        }
        let meta = entry.metadata().ok();
        let modified = meta.as_ref().and_then(|m| m.modified().ok()).map(|t| {
            chrono::DateTime::<Utc>::from(t).to_rfc3339()
        });
        out.push(ScreenshotEntry {
            name,
            size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
            modified,
        });
    }
    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(out)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalScreenshotEntry {
    pub instance_id: String,
    pub instance_name: String,
    pub name: String,
    pub size: u64,
    pub modified: Option<String>,
}

pub fn list_all_screenshots(dirs: &Dirs) -> Result<Vec<GlobalScreenshotEntry>> {
    let mut out = Vec::new();
    for inst in list(dirs)? {
        for shot in list_screenshots(dirs, &inst.id)? {
            out.push(GlobalScreenshotEntry {
                instance_id: inst.id.clone(),
                instance_name: inst.name.clone(),
                name: shot.name,
                size: shot.size,
                modified: shot.modified,
            });
        }
    }
    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(out)
}

pub fn read_screenshot_path(
    dirs: &Dirs,
    id: &str,
    name: &str,
    full: bool,
) -> Result<String> {
    let _ = get(dirs, id)?;
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(Error::msg("Nieprawidłowa nazwa pliku."));
    }
    let path = dirs.game_dir(id).join("screenshots").join(name);
    if !path.is_file() {
        return Err(Error::msg("Zrzut ekranu nie istnieje."));
    }
    if full {
        return Ok(crate::thumbs::path_to_string(path));
    }
    let thumb = crate::thumbs::ensure_thumb(dirs, &path, 480)?;
    Ok(crate::thumbs::path_to_string(thumb))
}

pub fn game_subdir(dirs: &Dirs, id: &str, folder: &str) -> Result<PathBuf> {
    let _ = get(dirs, id)?;
    let allowed = [
        "config",
        "crash-reports",
        "logs",
        "mods",
        "resourcepacks",
        "shaderpacks",
        "datapacks",
        "screenshots",
        "saves",
    ];
    if !allowed.contains(&folder) {
        return Err(Error::msg("Ten folder nie jest dostępny."));
    }
    let path = dirs.game_dir(id).join(folder);
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn is_installed(dirs: &Dirs, inst: &Instance) -> bool {
    let json = dirs.version_json(&inst.version_id);
    if !json.exists() {
        return false;
    }
    // Vanilla / forge client jar may live under the inherited vanilla id.
    if inst.loader == Loader::Vanilla {
        return dirs.version_jar(&inst.game_version).exists();
    }
    true
}

pub fn unlink_pack(dirs: &Dirs, id: &str) -> Result<Instance> {
    let mut inst = get(dirs, id)?;
    inst.pack_locked = false;
    save(dirs, &inst)?;
    Ok(inst)
}

/// Nadpisuje wersję gry i loader z paczki, bez zmiany id, nazwy ani światów.
pub fn apply_pack_profile(inst: &mut Instance, req: &CreateInstance) {
    inst.game_version = req.game_version.clone();
    inst.loader = req.loader;
    inst.loader_version = req.loader_version.clone();
    inst.version_id = compute_version_id(req);
    inst.pack_locked = true;
}

/// Usuwa `mods/`, żeby stara wersja paczki nie mieszała jarów z nową.
pub fn clear_mods_dir(dirs: &Dirs, id: &str) -> Result<()> {
    let _ = get(dirs, id)?;
    let mods = dirs.game_dir(id).join("mods");
    if mods.exists() {
        std::fs::remove_dir_all(&mods)?;
    }
    std::fs::create_dir_all(&mods)?;
    Ok(())
}

/// Zachowaj powiązanie z paczką przy zapisie ustawień z UI.
pub fn merge_pack_link(existing: &Instance, incoming: &mut Instance) {
    incoming.linked_pack = existing.linked_pack.clone();
    incoming.pack_locked = existing.pack_locked;
}

/// Pusty `iconPath` z edytora glifów czyści logo i usuwa plik. Brak pola (None) zachowuje plik.
/// Niektóre warstwy IPC gubią `""` i wysyłają `null` — wtedy path wracałby z `existing`
/// i logo paczki zasłaniało glif. Zapis ikony idzie przez `apply_glyph_icon` / `apply_file_icon`.
pub fn merge_icon(dirs: &Dirs, existing: &Instance, incoming: &mut Instance) {
    match incoming.icon_path.as_deref().map(str::trim) {
        Some("") => {
            incoming.icon_path = None;
            icon::remove_installed_icons(dirs, &existing.id);
            if incoming.led_color.trim().is_empty() {
                incoming.led_color.clear();
                incoming.led_color_2.clear();
            }
        }
        Some(_) => {}
        None => {
            incoming.icon_path = existing.icon_path.clone();
            if incoming.led_color.trim().is_empty() {
                incoming.led_color = existing.led_color.clone();
            }
            if incoming.led_color_2.trim().is_empty() {
                incoming.led_color_2 = existing.led_color_2.clone();
            }
        }
    }
}

fn icon_edit_error() -> Error {
    Error::msg("Ta instancja pochodzi z paczki. Odłącz ją od oryginału, żeby zmienić ikonę.")
}

/// Zastępuje logo paczki kolorem i symbolem: kasuje `icon.*` i czyści `iconPath`.
pub fn apply_glyph_icon(dirs: &Dirs, id: &str, color: &str, symbol: &str) -> Result<Instance> {
    let mut inst = get(dirs, id)?;
    if inst.pack_locked {
        return Err(icon_edit_error());
    }
    icon::remove_installed_icons(dirs, &inst.id);
    inst.icon_path = None;
    inst.icon_color = color.trim().to_string();
    inst.icon_symbol = symbol.trim().to_string();
    inst.led_color.clear();
    inst.led_color_2.clear();
    save(dirs, &inst)?;
    icon::remove_installed_icons(dirs, &inst.id);
    inst.icon_path = None;
    save(dirs, &inst)?;
    Ok(inst)
}

/// Zapisuje nowy obraz jako `icon.*`, kasuje poprzednie pliki i ustawia `iconPath`.
pub fn apply_file_icon(dirs: &Dirs, id: &str, path: &Path) -> Result<Instance> {
    let mut inst = get(dirs, id)?;
    if inst.pack_locked {
        return Err(icon_edit_error());
    }
    if !path.is_file() {
        return Err(Error::msg("Nie znaleziono pliku obrazu."));
    }
    let bytes = std::fs::read(path)?;
    if bytes.is_empty() {
        return Err(Error::msg("Plik obrazu jest pusty."));
    }
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(Error::msg("Obraz jest za duży (maks. 2 MB)."));
    }
    icon::install_icon_bytes(dirs, &mut inst, &bytes)?;
    if inst.icon_path.as_deref().map(str::trim).filter(|s| !s.is_empty()).is_none() {
        return Err(Error::msg("Nie udało się zapisać ikony. Wybierz PNG, JPEG, WebP albo GIF."));
    }
    save(dirs, &inst)?;
    Ok(inst)
}

const MAX_WALLPAPER_BYTES: usize = 4 * 1024 * 1024;

fn wallpaper_abs_path(dirs: &Dirs, inst: &Instance) -> Option<PathBuf> {
    let rel = inst.wallpaper_path.as_deref()?.trim();
    if rel.is_empty() || rel.contains("..") || rel.contains('/') || rel.contains('\\') {
        return None;
    }
    let path = dirs.instance_dir(&inst.id).join(rel);
    path.is_file().then_some(path)
}

fn sniff_image_ext(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        "png"
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        "jpg"
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "webp"
    } else {
        "jpg"
    }
}

fn remove_other_wallpapers(dir: &Path, keep: &str) {
    for name in ["wallpaper.png", "wallpaper.jpg", "wallpaper.jpeg", "wallpaper.webp"] {
        if name != keep {
            let p = dir.join(name);
            if p.is_file() {
                let _ = std::fs::remove_file(p);
            }
        }
    }
}

/// Zapisuje tapetę profilu z pliku obrazu.
pub fn apply_profile_wallpaper(dirs: &Dirs, id: &str, path: &Path) -> Result<Instance> {
    let mut inst = get(dirs, id)?;
    if !path.is_file() {
        return Err(Error::msg("Nie znaleziono pliku obrazu."));
    }
    let bytes = std::fs::read(path)?;
    if bytes.is_empty() {
        return Err(Error::msg("Plik obrazu jest pusty."));
    }
    if bytes.len() > MAX_WALLPAPER_BYTES {
        return Err(Error::msg("Tapeta jest za duża (maks. 4 MB)."));
    }
    let ext = sniff_image_ext(&bytes);
    let name = format!("wallpaper.{ext}");
    let dir = dirs.instance_dir(&inst.id);
    std::fs::create_dir_all(&dir)?;
    let dest = dir.join(&name);
    std::fs::write(&dest, &bytes)?;
    remove_other_wallpapers(&dir, &name);
    inst.wallpaper_path = Some(name);
    save(dirs, &inst)?;
    Ok(inst)
}

pub fn read_wallpaper_thumb_path(dirs: &Dirs, inst: &Instance) -> Option<String> {
    let path = wallpaper_abs_path(dirs, inst)?;
    let thumb = crate::thumbs::ensure_thumb(dirs, &path, 800).ok()?;
    Some(crate::thumbs::path_to_string(thumb))
}

fn resolved_icon_path<'a>(existing: &'a Instance, incoming: &'a Instance) -> Option<&'a str> {
    match incoming.icon_path.as_deref().map(str::trim) {
        Some("") => None,
        Some(path) => Some(path),
        None => existing
            .icon_path
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty()),
    }
}

/// Czy zapis z UI zmieniłby ikonę (kolor, symbol, plik albo akcent LED).
pub fn icon_would_change(existing: &Instance, incoming: &Instance) -> bool {
    let existing_path = existing
        .icon_path
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    resolved_icon_path(existing, incoming) != existing_path
        || incoming.icon_color.trim() != existing.icon_color.trim()
        || incoming.icon_symbol.trim() != existing.icon_symbol.trim()
        || incoming.led_color.trim() != existing.led_color.trim()
        || incoming.led_color_2.trim() != existing.led_color_2.trim()
}

pub fn ensure_icon_unlocked(existing: &Instance, incoming: &Instance) -> Result<()> {
    if existing.pack_locked && icon_would_change(existing, incoming) {
        return Err(Error::msg(
            "Ta instancja pochodzi z paczki. Odłącz ją od oryginału, żeby zmienić ikonę.",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContentKind {
    Mods,
    Resourcepacks,
    Shaderpacks,
    Datapacks,
}

impl ContentKind {
    pub fn dir_name(self) -> &'static str {
        match self {
            Self::Mods => "mods",
            Self::Resourcepacks => "resourcepacks",
            Self::Shaderpacks => "shaderpacks",
            Self::Datapacks => "datapacks",
        }
    }

    pub fn from_modrinth_type(project_type: &str) -> Result<Self> {
        match project_type {
            "mod" => Ok(Self::Mods),
            "shader" => Ok(Self::Shaderpacks),
            "resourcepack" => Ok(Self::Resourcepacks),
            "datapack" => Ok(Self::Datapacks),
            _ => Err(Error::msg(
                "Ten typ projektu nie pasuje do zawartości instancji.",
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentFile {
    pub name: String,
    pub display_name: String,
    pub enabled: bool,
    pub kind: ContentKind,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ContentIndex {
    #[serde(default)]
    files: HashMap<String, ContentMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ContentMeta {
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    project_id: Option<String>,
}

fn content_index_path(dirs: &Dirs, id: &str) -> PathBuf {
    dirs.instance_dir(id).join("content-index.json")
}

fn index_key(kind: ContentKind, filename: &str) -> String {
    format!("{}/{}", kind.dir_name(), display_name(filename))
}

fn load_content_index(dirs: &Dirs, id: &str) -> ContentIndex {
    std::fs::read_to_string(content_index_path(dirs, id))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_content_index(dirs: &Dirs, id: &str, index: &ContentIndex) -> Result<()> {
    let path = content_index_path(dirs, id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(index)?)?;
    Ok(())
}

pub fn record_content_meta(
    dirs: &Dirs,
    id: &str,
    kind: ContentKind,
    filename: &str,
    slug: Option<String>,
    project_id: Option<String>,
) {
    let slug = slug.filter(|s| !s.trim().is_empty());
    let project_id = project_id.filter(|s| !s.trim().is_empty());
    if slug.is_none() && project_id.is_none() {
        return;
    }
    let mut index = load_content_index(dirs, id);
    index.files.insert(
        index_key(kind, filename),
        ContentMeta { slug, project_id },
    );
    let _ = save_content_index(dirs, id, &index);
}

fn forget_content_meta(dirs: &Dirs, id: &str, kind: ContentKind, filename: &str) {
    let mut index = load_content_index(dirs, id);
    if index.files.remove(&index_key(kind, filename)).is_some() {
        let _ = save_content_index(dirs, id, &index);
    }
}

fn pack_edit_error() -> Error {
    Error::msg("Ta instancja pochodzi z paczki. Odłącz ją od oryginału, żeby zmieniać zawartość.")
}

pub(crate) fn ensure_unlocked(inst: &Instance) -> Result<()> {
    if inst.pack_locked {
        return Err(pack_edit_error());
    }
    Ok(())
}

fn safe_file_name(name: &str) -> Result<&str> {
    let path = Path::new(name);
    if name.is_empty() || name.len() > 255 || path.is_absolute() {
        return Err(Error::msg("Niepoprawna nazwa pliku."));
    }
    let mut parts = path.components();
    match (parts.next(), parts.next()) {
        (Some(Component::Normal(os)), None) if os.to_str() == Some(name) => Ok(name),
        _ => Err(Error::msg("Niepoprawna nazwa pliku.")),
    }
}

fn kind_dir(dirs: &Dirs, id: &str, kind: ContentKind) -> PathBuf {
    dirs.game_dir(id).join(kind.dir_name())
}

fn content_file_path(dirs: &Dirs, id: &str, kind: ContentKind, name: &str) -> Result<PathBuf> {
    let name = safe_file_name(name)?;
    Ok(kind_dir(dirs, id, kind).join(name))
}

pub(crate) fn prepare_content_dest(
    dirs: &Dirs,
    id: &str,
    kind: ContentKind,
    filename: &str,
) -> Result<PathBuf> {
    let dest = content_file_path(dirs, id, kind, filename)?;
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(dest)
}

pub(crate) fn content_file_present(
    dirs: &Dirs,
    id: &str,
    kind: ContentKind,
    filename: &str,
) -> bool {
    if content_file_path(dirs, id, kind, filename)
        .map(|p| p.is_file())
        .unwrap_or(false)
    {
        return true;
    }
    let disabled = format!("{filename}{DISABLED_SUFFIX}");
    content_file_path(dirs, id, kind, &disabled)
        .map(|p| p.is_file())
        .unwrap_or(false)
}

fn is_disabled_name(name: &str) -> bool {
    name.len() > DISABLED_SUFFIX.len() && name.to_ascii_lowercase().ends_with(DISABLED_SUFFIX)
}

fn display_name(name: &str) -> String {
    if is_disabled_name(name) {
        name[..name.len() - DISABLED_SUFFIX.len()].to_string()
    } else {
        name.to_string()
    }
}

pub fn list_content(dirs: &Dirs, id: &str) -> Result<Vec<ContentFile>> {
    let _inst = get(dirs, id)?;
    let index = load_content_index(dirs, id);
    let mut out = Vec::new();
    for kind in [
        ContentKind::Mods,
        ContentKind::Resourcepacks,
        ContentKind::Shaderpacks,
        ContentKind::Datapacks,
    ] {
        let dir = kind_dir(dirs, id, kind);
        if !dir.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_string) else {
                continue;
            };
            if safe_file_name(&name).is_err() {
                continue;
            }
            let enabled = !is_disabled_name(&name);
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            let meta = index.files.get(&index_key(kind, &name));
            out.push(ContentFile {
                display_name: display_name(&name),
                name,
                enabled,
                kind,
                size,
                slug: meta.and_then(|m| m.slug.clone()),
                project_id: meta.and_then(|m| m.project_id.clone()),
            });
        }
    }
    out.sort_by(|a, b| {
        a.kind
            .dir_name()
            .cmp(b.kind.dir_name())
            .then_with(|| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()))
    });
    Ok(out)
}

pub fn toggle_content(dirs: &Dirs, id: &str, kind: ContentKind, name: &str) -> Result<Vec<ContentFile>> {
    let inst = get(dirs, id)?;
    ensure_unlocked(&inst)?;
    let src = content_file_path(dirs, id, kind, name)?;
    if !src.is_file() {
        return Err(Error::msg("Nie znaleziono pliku."));
    }
    let dest_name = if is_disabled_name(name) {
        display_name(name)
    } else {
        format!("{name}{DISABLED_SUFFIX}")
    };
    let dest = content_file_path(dirs, id, kind, &dest_name)?;
    if dest.exists() {
        return Err(Error::msg("Nie można przełączyć — plik docelowy już istnieje."));
    }
    std::fs::rename(&src, &dest)?;
    list_content(dirs, id)
}

pub fn delete_content(dirs: &Dirs, id: &str, kind: ContentKind, name: &str) -> Result<Vec<ContentFile>> {
    let inst = get(dirs, id)?;
    ensure_unlocked(&inst)?;
    let path = content_file_path(dirs, id, kind, name)?;
    if !path.is_file() {
        return Err(Error::msg("Nie znaleziono pliku."));
    }
    std::fs::remove_file(&path)?;
    forget_content_meta(dirs, id, kind, name);
    list_content(dirs, id)
}

fn unique_dest(dir: &Path, filename: &str) -> Result<PathBuf> {
    let name = safe_file_name(filename)?;
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("plik");
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    let first = dir.join(name);
    if !first.exists() {
        return Ok(first);
    }
    for n in 2..200 {
        let candidate = dir.join(format!("{stem}-{n}{ext}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(Error::msg("Zbyt wiele plików o tej samej nazwie."))
}

pub fn import_local_content(
    dirs: &Dirs,
    id: &str,
    kind: ContentKind,
    source: &Path,
) -> Result<Vec<ContentFile>> {
    let inst = get(dirs, id)?;
    ensure_unlocked(&inst)?;
    if !source.is_file() {
        return Err(Error::msg("Nie znaleziono pliku do skopiowania."));
    }
    let filename = source
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::msg("Niepoprawna nazwa pliku."))?;
    let lower = filename.to_ascii_lowercase();
    match kind {
        ContentKind::Mods if !(lower.ends_with(".jar") || lower.ends_with(".zip")) => {
            return Err(Error::msg("Do folderu mods wrzuć plik .jar albo .zip."));
        }
        ContentKind::Resourcepacks
        | ContentKind::Shaderpacks
        | ContentKind::Datapacks
            if !lower.ends_with(".zip") && !lower.ends_with(".jar") =>
        {
            return Err(Error::msg("Ten typ zawartości przyjmuje .zip (albo .jar)."));
        }
        _ => {}
    }
    let dir = kind_dir(dirs, id, kind);
    std::fs::create_dir_all(&dir)?;
    let dest = unique_dest(&dir, filename)?;
    std::fs::copy(source, &dest)?;
    list_content(dirs, id)
}

#[cfg(test)]
pub(crate) fn test_instance(id: &str) -> Instance {
    Instance {
        id: id.to_string(),
        name: "Test".into(),
        game_version: "1.21".into(),
        loader: Loader::Vanilla,
        loader_version: None,
        version_id: "1.21".into(),
        created_at: "2026-01-01T00:00:00Z".into(),
        last_played: None,
        memory_max_mb: 4096,
        memory_min_mb: 512,
        java_path: None,
        java_args: String::new(),
        join_server: String::new(),
        icon_color: String::new(),
        icon_symbol: String::new(),
        icon_path: None,
        wallpaper_path: None,
        led_color: String::new(),
        led_color_2: String::new(),
        play_time_secs: 0,
        linked_pack: None,
        pack_locked: false,
        custom_java: false,
        custom_memory: false,
        custom_java_args: false,
        custom_env: false,
        custom_window: false,
        custom_hooks: false,
        fullscreen: false,
        window_width: 854,
        window_height: 480,
        env_vars: String::new(),
        pre_launch: String::new(),
        wrapper: String::new(),
        post_exit: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icon;
    use uuid::Uuid;

    #[test]
    fn icon_would_change_ignores_kept_path() {
        let mut existing = test_instance("a");
        existing.icon_path = Some("icon.png".into());
        existing.icon_color = "#aabbcc".into();
        let mut incoming = existing.clone();
        incoming.icon_path = None;
        assert!(!icon_would_change(&existing, &incoming));
    }

    #[test]
    fn icon_would_change_detects_clear_and_color() {
        let mut existing = test_instance("a");
        existing.icon_path = Some("icon.png".into());
        existing.pack_locked = true;
        let mut incoming = existing.clone();
        incoming.icon_path = Some(String::new());
        assert!(icon_would_change(&existing, &incoming));
        incoming.icon_path = existing.icon_path.clone();
        incoming.icon_color = "#ffffff".into();
        assert!(icon_would_change(&existing, &incoming));
    }

    #[test]
    fn ensure_icon_unlocked_blocks_pack() {
        let mut existing = test_instance("a");
        existing.pack_locked = true;
        existing.icon_path = Some("icon.png".into());
        let mut incoming = existing.clone();
        incoming.icon_path = Some(String::new());
        assert!(ensure_icon_unlocked(&existing, &incoming).is_err());
        incoming.icon_path = existing.icon_path.clone();
        assert!(ensure_icon_unlocked(&existing, &incoming).is_ok());
    }

    #[test]
    fn merge_icon_deletes_file_when_cleared() {
        let root = std::env::temp_dir().join(format!("lumen-icon-{}", Uuid::new_v4()));
        let dirs = Dirs::from_root(root.clone());
        let id = "inst-1";
        std::fs::create_dir_all(dirs.instance_dir(id)).unwrap();
        let file = dirs.instance_dir(id).join("icon.png");
        std::fs::write(&file, [0x89, b'P', b'N', b'G', 0, 0, 0, 0]).unwrap();
        let mut existing = test_instance(id);
        existing.icon_path = Some("icon.png".into());
        let mut incoming = existing.clone();
        incoming.icon_path = Some(String::new());
        merge_icon(&dirs, &existing, &mut incoming);
        assert!(incoming.icon_path.is_none());
        assert!(!file.exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_glyph_icon_deletes_pack_file_and_clears_path() {
        let root = std::env::temp_dir().join(format!("lumen-glyph-{}", Uuid::new_v4()));
        let dirs = Dirs::from_root(root.clone());
        let id = "inst-glyph";
        std::fs::create_dir_all(dirs.instance_dir(id)).unwrap();
        let file = dirs.instance_dir(id).join("icon.png");
        std::fs::write(&file, [0x89, b'P', b'N', b'G', 0, 0, 0, 0]).unwrap();
        let mut inst = test_instance(id);
        inst.icon_path = Some("icon.png".into());
        inst.led_color = "#ff0000".into();
        save(&dirs, &inst).unwrap();
        let next = apply_glyph_icon(&dirs, id, "#aabbcc", "craft").unwrap();
        assert!(next.icon_path.is_none());
        assert!(!file.exists());
        assert_eq!(next.icon_color, "#aabbcc");
        assert_eq!(next.icon_symbol, "craft");
        assert!(next.led_color.is_empty());
        let loaded = get(&dirs, id).unwrap();
        assert!(loaded.icon_path.is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_glyph_icon_blocked_when_locked() {
        let root = std::env::temp_dir().join(format!("lumen-glyph-lock-{}", Uuid::new_v4()));
        let dirs = Dirs::from_root(root.clone());
        let mut inst = test_instance("locked");
        inst.pack_locked = true;
        inst.icon_path = Some("icon.png".into());
        save(&dirs, &inst).unwrap();
        assert!(apply_glyph_icon(&dirs, "locked", "#fff", "grass").is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn unique_instance_name_adds_number() {
        let root = std::env::temp_dir().join(format!("lumen-uname-{}", Uuid::new_v4()));
        let dirs = Dirs::from_root(root.clone());
        let mut a = test_instance("a");
        a.name = "ATM9".into();
        save(&dirs, &a).unwrap();
        assert_eq!(unique_instance_name(&dirs, "ATM9").unwrap(), "ATM9 (2)");
        assert_eq!(unique_instance_name(&dirs, "Inna").unwrap(), "Inna");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn copy_world_uniquifies_folder() {
        let root = std::env::temp_dir().join(format!("lumen-world-{}", Uuid::new_v4()));
        let dirs = Dirs::from_root(root.clone());
        let src = test_instance("src");
        let dst = test_instance("dst");
        save(&dirs, &src).unwrap();
        save(&dirs, &dst).unwrap();
        let saves_src = dirs.game_dir("src").join("saves").join("world");
        std::fs::create_dir_all(&saves_src).unwrap();
        std::fs::write(saves_src.join("level.dat"), b"x").unwrap();
        let saves_dst = dirs.game_dir("dst").join("saves").join("world");
        std::fs::create_dir_all(&saves_dst).unwrap();
        std::fs::write(saves_dst.join("level.dat"), b"y").unwrap();
        copy_world(&dirs, "src", "world", "dst").unwrap();
        assert!(dirs.game_dir("dst").join("saves").join("world (kopia)").is_dir());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_pack_profile_updates_loader_keeps_identity() {
        let mut inst = test_instance("keep");
        inst.name = "Moja paczka".into();
        inst.linked_pack = Some("fabulously-optimized".into());
        inst.play_time_secs = 99;
        apply_pack_profile(
            &mut inst,
            &CreateInstance {
                name: "Inna nazwa".into(),
                game_version: "1.21.8".into(),
                loader: Loader::Fabric,
                loader_version: Some("0.16.14".into()),
                memory_max_mb: None,
            },
        );
        assert_eq!(inst.id, "keep");
        assert_eq!(inst.name, "Moja paczka");
        assert_eq!(inst.play_time_secs, 99);
        assert_eq!(inst.game_version, "1.21.8");
        assert_eq!(inst.loader, Loader::Fabric);
        assert_eq!(inst.loader_version.as_deref(), Some("0.16.14"));
        assert_eq!(inst.version_id, "1.21.8-fabric-0.16.14");
        assert!(inst.pack_locked);
    }

    #[test]
    fn clear_mods_dir_removes_jars_keeps_saves() {
        let root = std::env::temp_dir().join(format!("lumen-resync-{}", Uuid::new_v4()));
        let dirs = Dirs::from_root(root.clone());
        let inst = test_instance("rs");
        save(&dirs, &inst).unwrap();
        let mods = dirs.game_dir("rs").join("mods");
        let saves = dirs.game_dir("rs").join("saves").join("world");
        std::fs::create_dir_all(&mods).unwrap();
        std::fs::create_dir_all(&saves).unwrap();
        std::fs::write(mods.join("old.jar"), b"x").unwrap();
        std::fs::write(saves.join("level.dat"), b"y").unwrap();
        clear_mods_dir(&dirs, "rs").unwrap();
        assert!(mods.is_dir());
        assert!(!mods.join("old.jar").exists());
        assert!(saves.join("level.dat").is_file());
        let _ = std::fs::remove_dir_all(&root);
    }
}
