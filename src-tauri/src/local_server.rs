use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};
use tokio::sync::mpsc;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::download::{download_file, download_json};
use crate::error::{Error, Result};
use crate::instances::{self, Loader};
use crate::java;
use crate::loaders;
use crate::meta::{self, VersionMeta};
use crate::paths::Dirs;
use crate::servers;
use crate::settings::Settings;
use crate::AppState;

const JAR_NAME: &str = "server.jar";
const EULA_URL: &str = "https://aka.ms/MinecraftEULA";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LocalSoftware {
    Vanilla,
    Paper,
    Fabric,
}

impl LocalSoftware {
    pub fn label(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::Paper => "Paper",
            Self::Fabric => "Fabric",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalServerConfig {
    pub id: String,
    pub name: String,
    pub software: LocalSoftware,
    pub game_version: String,
    #[serde(default)]
    pub loader_version: Option<String>,
    #[serde(default = "default_mem")]
    pub memory_mb: u32,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_motd")]
    pub motd: String,
    #[serde(default)]
    pub online_mode: bool,
    #[serde(default = "default_max_players")]
    pub max_players: u32,
    #[serde(default = "default_difficulty")]
    pub difficulty: String,
    #[serde(default = "default_view")]
    pub view_distance: u32,
    #[serde(default)]
    pub eula_accepted: bool,
    #[serde(default)]
    pub source_instance_id: Option<String>,
    #[serde(default)]
    pub play_address: Option<String>,
    /// Empty / None = Lumen dobiera Javę przy starcie.
    #[serde(default)]
    pub java_path: Option<String>,
    #[serde(default)]
    pub required_java: Option<u32>,
    pub created_at: String,
}

fn default_mem() -> u32 {
    2048
}
fn default_port() -> u16 {
    25565
}
fn default_motd() -> String {
    "Serwer Octra".into()
}
fn default_max_players() -> u32 {
    10
}
fn default_difficulty() -> String {
    "easy".into()
}
fn default_view() -> u32 {
    10
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocalServer {
    pub name: String,
    pub game_version: String,
    pub software: LocalSoftware,
    #[serde(default)]
    pub loader_version: Option<String>,
    #[serde(default)]
    pub memory_mb: Option<u32>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub motd: Option<String>,
    #[serde(default)]
    pub online_mode: bool,
    #[serde(default)]
    pub max_players: Option<u32>,
    #[serde(default)]
    pub difficulty: Option<String>,
    #[serde(default)]
    pub view_distance: Option<u32>,
    pub eula_accepted: bool,
    #[serde(default)]
    pub source_instance_id: Option<String>,
    #[serde(default)]
    pub java_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocalServer {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub memory_mb: Option<u32>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub motd: Option<String>,
    #[serde(default)]
    pub online_mode: Option<bool>,
    #[serde(default)]
    pub max_players: Option<u32>,
    #[serde(default)]
    pub difficulty: Option<String>,
    #[serde(default)]
    pub view_distance: Option<u32>,
    /// Some("") = automatyczna; Some(path) = własna.
    #[serde(default)]
    pub java_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalServerInfo {
    pub id: String,
    pub name: String,
    pub software: LocalSoftware,
    pub game_version: String,
    pub loader_version: Option<String>,
    pub memory_mb: u32,
    pub port: u16,
    pub motd: String,
    pub online_mode: bool,
    pub max_players: u32,
    pub difficulty: String,
    pub view_distance: u32,
    pub eula_accepted: bool,
    pub source_instance_id: Option<String>,
    pub java_path: Option<String>,
    pub required_java: u32,
    pub jar_ready: bool,
    pub status: String,
    pub pid: Option<u32>,
    pub address: String,
    pub lan_ip: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalServerStatusEvent {
    pub server_id: String,
    pub name: String,
    pub status: String,
    pub port: u16,
    pub pid: Option<u32>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalServerLogEvent {
    pub server_id: String,
    pub line: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalServerProgressEvent {
    pub server_id: String,
    pub message: String,
}

pub struct LocalServerProc {
    pub pid: u32,
    pub cmd_tx: mpsc::UnboundedSender<String>,
    pub status: String,
}

#[derive(Deserialize)]
struct PaperVersionInfo {
    builds: Vec<u32>,
}

#[derive(Deserialize)]
struct PaperBuildInfo {
    downloads: PaperDownloads,
}

#[derive(Deserialize)]
struct PaperDownloads {
    application: PaperFile,
}

#[derive(Deserialize)]
struct PaperFile {
    name: String,
}

#[derive(Deserialize)]
struct FabricInstaller {
    version: String,
    #[serde(default)]
    stable: bool,
}

fn cfg_path(dir: &Path) -> PathBuf {
    dir.join("lumen.json")
}

fn jar_path(dir: &Path) -> PathBuf {
    dir.join(JAR_NAME)
}

fn props_path(dir: &Path) -> PathBuf {
    dir.join("server.properties")
}

fn eula_path(dir: &Path) -> PathBuf {
    dir.join("eula.txt")
}

fn log_path(dir: &Path) -> PathBuf {
    dir.join("logs").join("lumen-console.log")
}

fn backups_dir(dir: &Path) -> PathBuf {
    dir.join("backups")
}

fn load_config(dir: &Path) -> Result<LocalServerConfig> {
    let raw = std::fs::read_to_string(cfg_path(dir))?;
    Ok(serde_json::from_str(&raw)?)
}

fn save_config(dir: &Path, cfg: &LocalServerConfig) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(cfg_path(dir), serde_json::to_string_pretty(cfg)?)?;
    Ok(())
}

fn list_ids(dirs: &Dirs) -> Result<Vec<String>> {
    let root = dirs.servers_root();
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut ids = Vec::new();
    for entry in std::fs::read_dir(&root)?.flatten() {
        if entry.path().join("lumen.json").is_file() {
            if let Some(name) = entry.file_name().to_str() {
                ids.push(name.to_string());
            }
        }
    }
    ids.sort();
    Ok(ids)
}

fn runtime_status(state: &AppState, id: &str) -> (String, Option<u32>) {
    state
        .local_servers
        .lock()
        .get(id)
        .map(|p| (p.status.clone(), Some(p.pid)))
        .unwrap_or_else(|| ("stopped".into(), None))
}

fn lan_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let ip = socket.local_addr().ok()?.ip();
    if ip.is_loopback() || ip.is_unspecified() {
        return None;
    }
    Some(ip.to_string())
}

fn to_info(cfg: &LocalServerConfig, dir: &Path, state: &AppState) -> LocalServerInfo {
    let (status, pid) = runtime_status(state, &cfg.id);
    LocalServerInfo {
        id: cfg.id.clone(),
        name: cfg.name.clone(),
        software: cfg.software,
        game_version: cfg.game_version.clone(),
        loader_version: cfg.loader_version.clone(),
        memory_mb: cfg.memory_mb,
        port: cfg.port,
        motd: cfg.motd.clone(),
        online_mode: cfg.online_mode,
        max_players: cfg.max_players,
        difficulty: cfg.difficulty.clone(),
        view_distance: cfg.view_distance,
        eula_accepted: cfg.eula_accepted,
        source_instance_id: cfg.source_instance_id.clone(),
        java_path: cfg.java_path.clone(),
        required_java: cfg
            .required_java
            .unwrap_or_else(|| meta::required_java_for_id(&cfg.game_version)),
        jar_ready: jar_path(dir).is_file(),
        status,
        pid,
        address: format!("127.0.0.1:{}", cfg.port),
        lan_ip: lan_ip(),
        created_at: cfg.created_at.clone(),
    }
}

fn emit_status(app: &AppHandle, cfg: &LocalServerConfig, status: &str, pid: Option<u32>) {
    let _ = app.emit(
        "local-server-status",
        LocalServerStatusEvent {
            server_id: cfg.id.clone(),
            name: cfg.name.clone(),
            status: status.into(),
            port: cfg.port,
            pid,
        },
    );
}

fn emit_log(app: &AppHandle, id: &str, line: &str) {
    let _ = app.emit(
        "local-server-log",
        LocalServerLogEvent {
            server_id: id.to_string(),
            line: line.to_string(),
        },
    );
}

fn emit_progress(app: &AppHandle, id: &str, message: &str) {
    let _ = app.emit(
        "local-server-progress",
        LocalServerProgressEvent {
            server_id: id.to_string(),
            message: message.to_string(),
        },
    );
}

async fn required_java_for_game(
    client: &reqwest::Client,
    dirs: &Dirs,
    game: &str,
) -> u32 {
    match crate::install::load_or_fetch_version_json(client, dirs, game, None, None).await {
        Ok(m) => meta::required_java(&m),
        Err(_) => meta::required_java_for_id(game),
    }
}

fn resolve_server_java(
    dirs: &Dirs,
    settings: &Settings,
    cfg: &LocalServerConfig,
    required: u32,
) -> Result<java::JavaRuntime> {
    if let Some(path) = cfg.java_path.as_deref().map(str::trim).filter(|s| !s.is_empty())
    {
        let rt = java::probe_java(Path::new(path)).ok_or_else(|| {
            Error::msg("Wybrana Java serwera nie działa. Wskaż inną instalację.")
        })?;
        if rt.major < required {
            return Err(Error::msg(format!(
                "Wybrana Java {} jest za stara dla Minecraft {}. Wymagana jest Java {required} lub nowsza.",
                rt.major, cfg.game_version
            )));
        }
        return Ok(rt);
    }
    let runtimes = java::scan(dirs, settings);
    java::pick_compatible(&runtimes, required, settings).map_err(|_| {
        Error::msg(format!(
            "Brak Javy {required} dla Minecraft {}. Pobierz Temurin {required} w ustawieniach serwera albo w Ustawieniach launchera.",
            cfg.game_version
        ))
    })
}

fn looks_ready(line: &str) -> bool {
    let l = line.to_ascii_lowercase();
    l.contains("done (") || l.contains("for help, type") || l.contains("done!")
}

fn write_eula(dir: &Path) -> Result<()> {
    let body = format!(
        "# Zaakceptowano z launchera Lumen.\n# {EULA_URL}\neula=true\n"
    );
    std::fs::write(eula_path(dir), body)?;
    Ok(())
}

fn upsert_properties(path: &Path, updates: &[(&str, String)]) -> Result<()> {
    let mut lines: Vec<String> = if path.exists() {
        let raw = std::fs::read_to_string(path)?;
        raw.lines().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    };
    for (key, val) in updates {
        let prefix = format!("{key}=");
        let hashed = format!("#{key}=");
        if let Some(line) = lines
            .iter_mut()
            .find(|l| l.starts_with(&prefix) || l.starts_with(&hashed))
        {
            *line = format!("{key}={val}");
        } else {
            lines.push(format!("{key}={val}"));
        }
    }
    if lines.is_empty() {
        return Ok(());
    }
    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    std::fs::write(path, out)?;
    Ok(())
}

fn write_runtime_properties(dir: &Path, cfg: &LocalServerConfig) -> Result<()> {
    let mut updates = vec![
        ("server-port", cfg.port.to_string()),
        ("query.port", cfg.port.to_string()),
        ("motd", cfg.motd.replace('\n', "\\n")),
        (
            "online-mode",
            if cfg.online_mode {
                "true"
            } else {
                "false"
            }
            .into(),
        ),
        ("max-players", cfg.max_players.to_string()),
        ("difficulty", cfg.difficulty.clone()),
        ("view-distance", cfg.view_distance.to_string()),
        ("simulation-distance", cfg.view_distance.to_string()),
    ];
    if !cfg.online_mode {
        updates.push(("enforce-secure-profile", "false".into()));
    }
    upsert_properties(&props_path(dir), &updates)
}

fn copy_dir(src: &Path, dst: &Path) -> Result<usize> {
    if !src.exists() {
        return Ok(0);
    }
    std::fs::create_dir_all(dst)?;
    let mut n = 0;
    for entry in WalkDir::new(src).into_iter().flatten() {
        let rel = match entry.path().strip_prefix(src) {
            Ok(r) if !r.as_os_str().is_empty() => r.to_path_buf(),
            _ => continue,
        };
        let dest = dst.join(&rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest)?;
            continue;
        }
        if rel.file_name().and_then(|s| s.to_str()) == Some("session.lock") {
            continue;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(entry.path(), &dest)?;
        n += 1;
    }
    Ok(n)
}

fn copy_from_instance(dirs: &Dirs, inst_id: &str, dest: &Path) -> Result<(u32, u32)> {
    let game = dirs.game_dir(inst_id);
    let mods = copy_dir(&game.join("mods"), &dest.join("mods"))?;
    let cfg = copy_dir(&game.join("config"), &dest.join("config"))?;
    Ok((mods as u32, cfg as u32))
}

fn open_folder(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    #[cfg(windows)]
    {
        let mut cmd = std::process::Command::new("explorer");
        cmd.arg(path);
        crate::winhide::hide_std(&mut cmd);
        cmd.spawn()
            .map_err(|e| Error::msg(format!("Nie otwarto folderu: {e}")))?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        Err(Error::msg("Otwieranie folderu jest dostępne na Windows."))
    }
}

fn java_console_exe(java: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let name = java
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if name.eq_ignore_ascii_case("javaw.exe") {
            let exe = java.with_file_name("java.exe");
            if exe.is_file() {
                return exe;
            }
        }
    }
    java.to_path_buf()
}

fn spawn_server_process(
    java: &Path,
    args: &[String],
    cwd: &Path,
) -> Result<tokio::process::Child> {
    let spawn_with = |flags: Option<u32>| -> std::io::Result<tokio::process::Child> {
        let mut cmd = tokio::process::Command::new(java);
        cmd.args(args)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(false);
        #[cfg(windows)]
        if let Some(flags) = flags {
            if flags != 0 {
                cmd.creation_flags(flags);
            }
        }
        #[cfg(not(windows))]
        {
            let _ = flags;
        }
        cmd.spawn()
    };

    #[cfg(windows)]
    {
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        // Do not use CREATE_BREAKAWAY_FROM_JOB — WebView2 job returns OS error 5.
        let attempts = [
            Some(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW),
            Some(CREATE_NO_WINDOW),
            None,
        ];
        let mut last = None;
        for flags in attempts {
            match spawn_with(flags) {
                Ok(child) => return Ok(child),
                Err(e) if e.raw_os_error() == Some(5) => last = Some(e),
                Err(e) => {
                    return Err(Error::msg(format!("Nie udało się uruchomić serwera: {e}")));
                }
            }
        }
        Err(Error::msg(format!(
            "Nie udało się uruchomić serwera: {}",
            last.map(|e| e.to_string())
                .unwrap_or_else(|| "odmowa dostępu".into())
        )))
    }
    #[cfg(not(windows))]
    {
        spawn_with(None).map_err(|e| Error::msg(format!("Nie udało się uruchomić serwera: {e}")))
    }
}

fn kill_pid_tree(pid: u32) -> std::io::Result<bool> {
    #[cfg(windows)]
    {
        let mut kill = std::process::Command::new("taskkill");
        kill.args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        crate::winhide::hide_std(&mut kill);
        let status = kill.status()?;
        return Ok(status.success());
    }
    #[cfg(not(windows))]
    {
        let _ = pid;
        Ok(false)
    }
}

fn append_log_line(path: &Path, line: &str) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{line}");
    }
}

async fn fetch_vanilla_server_artifact(
    client: &reqwest::Client,
    dirs: &Dirs,
    game: &str,
) -> Result<(String, Option<String>, Option<u64>, u32)> {
    let meta: VersionMeta =
        crate::install::load_or_fetch_version_json(client, dirs, game, None, None).await?;
    let java = meta::required_java(&meta);
    let server = meta
        .downloads
        .as_ref()
        .and_then(|d| d.server.clone())
        .ok_or_else(|| {
            Error::msg(format!(
                "Brak oficjalnego server.jar dla Minecraft {game}."
            ))
        })?;
    let url = server
        .url
        .ok_or_else(|| Error::msg(format!("Manifest Mojang nie ma URL-a serwera dla {game}.")))?;
    Ok((url, server.sha1, server.size, java))
}

#[derive(Deserialize)]
struct PaperProject {
    versions: Vec<String>,
}

fn paper_missing(game: &str) -> Error {
    Error::msg(format!(
        "Paper nie ma jeszcze buildu dla Minecraft {game}. Wybierz Vanilla, Fabric albo inną wersję."
    ))
}

pub async fn list_paper_versions(client: &reqwest::Client) -> Result<Vec<String>> {
    let project: PaperProject =
        download_json(client, "https://api.papermc.io/v2/projects/paper").await?;
    Ok(project.versions)
}

pub async fn probe_software(
    client: &reqwest::Client,
    dirs: &Dirs,
    software: LocalSoftware,
    game: &str,
    loader_version: Option<&str>,
) -> Result<()> {
    match software {
        LocalSoftware::Paper => {
            let info_url = format!(
                "https://api.papermc.io/v2/projects/paper/versions/{game}"
            );
            let info: PaperVersionInfo = match download_json(client, &info_url).await {
                Ok(v) => v,
                Err(e) => {
                    let text = e.to_string();
                    if text.contains("404") {
                        return Err(paper_missing(game));
                    }
                    return Err(e);
                }
            };
            if info.builds.is_empty() {
                return Err(paper_missing(game));
            }
        }
        LocalSoftware::Vanilla => {
            let _ = fetch_vanilla_server_artifact(client, dirs, game).await?;
        }
        LocalSoftware::Fabric => {
            let loaders = loaders::fabric::list_loaders(client, game).await?;
            if loaders.is_empty() {
                return Err(Error::msg(format!(
                    "Brak loadera Fabric dla Minecraft {game}."
                )));
            }
            let _ = loader_version;
        }
    }
    Ok(())
}

async fn latest_fabric_installer(client: &reqwest::Client) -> Result<String> {
    let list: Vec<FabricInstaller> =
        download_json(client, "https://meta.fabricmc.net/v2/versions/installer").await?;
    list.iter()
        .find(|i| i.stable)
        .or_else(|| list.first())
        .map(|i| i.version.clone())
        .ok_or_else(|| Error::msg("Nie udało się pobrać listy instalatorów Fabric."))
}

async fn store_required_java(
    client: &reqwest::Client,
    dirs: &Dirs,
    cfg: &mut LocalServerConfig,
) -> u32 {
    let required = required_java_for_game(client, dirs, &cfg.game_version).await;
    cfg.required_java = Some(required);
    required
}

async fn install_software(
    app: &AppHandle,
    client: &reqwest::Client,
    dirs: &Dirs,
    cfg: &mut LocalServerConfig,
) -> Result<()> {
    let dir = dirs.local_server_dir(&cfg.id);
    std::fs::create_dir_all(&dir)?;
    let dest = jar_path(&dir);
    match cfg.software {
        LocalSoftware::Vanilla => {
            emit_progress(app, &cfg.id, "Pobieranie oficjalnego server.jar…");
            let (url, sha1, size, java) =
                fetch_vanilla_server_artifact(client, dirs, &cfg.game_version).await?;
            cfg.required_java = Some(java);
            download_file(client, &url, &dest, sha1.as_deref(), size, None).await?;
        }
        LocalSoftware::Paper => {
            emit_progress(
                app,
                &cfg.id,
                &format!("Szukanie buildu Paper dla {}…", cfg.game_version),
            );
            let info_url = format!(
                "https://api.papermc.io/v2/projects/paper/versions/{}",
                cfg.game_version
            );
            let info: PaperVersionInfo = match download_json(client, &info_url).await {
                Ok(v) => v,
                Err(e) => {
                    let text = e.to_string();
                    if text.contains("404") {
                        return Err(paper_missing(&cfg.game_version));
                    }
                    return Err(e);
                }
            };
            let build = info
                .builds
                .iter()
                .copied()
                .max()
                .ok_or_else(|| paper_missing(&cfg.game_version))?;
            let build_url = format!(
                "https://api.papermc.io/v2/projects/paper/versions/{}/builds/{build}",
                cfg.game_version
            );
            let build_info: PaperBuildInfo = download_json(client, &build_url).await?;
            let name = build_info.downloads.application.name;
            let url = format!(
                "https://api.papermc.io/v2/projects/paper/versions/{}/builds/{build}/downloads/{name}",
                cfg.game_version
            );
            emit_progress(app, &cfg.id, &format!("Pobieranie Paper #{build}…"));
            download_file(client, &url, &dest, None, None, None).await?;
            store_required_java(client, dirs, cfg).await;
        }
        LocalSoftware::Fabric => {
            let loader = match cfg.loader_version.clone() {
                Some(v) if !v.is_empty() => v,
                _ => {
                    let list =
                        loaders::fabric::list_loaders(client, &cfg.game_version).await?;
                    list.first()
                        .cloned()
                        .ok_or_else(|| {
                            Error::msg(format!(
                                "Brak loadera Fabric dla Minecraft {}.",
                                cfg.game_version
                            ))
                        })?
                }
            };
            cfg.loader_version = Some(loader.clone());
            let installer = latest_fabric_installer(client).await?;
            let url = format!(
                "https://meta.fabricmc.net/v2/versions/loader/{}/{loader}/{installer}/server/jar",
                cfg.game_version
            );
            emit_progress(
                app,
                &cfg.id,
                &format!("Pobieranie serwera Fabric {loader}…"),
            );
            download_file(client, &url, &dest, None, None, None).await?;
            store_required_java(client, dirs, cfg).await;
        }
    }
    emit_progress(app, &cfg.id, "Oprogramowanie serwera gotowe.");
    Ok(())
}

pub fn list(state: &AppState) -> Result<Vec<LocalServerInfo>> {
    let (_, dirs) = Settings::load()?;
    dirs.ensure()?;
    let mut out = Vec::new();
    for id in list_ids(&dirs)? {
        let dir = dirs.local_server_dir(&id);
        if let Ok(cfg) = load_config(&dir) {
            out.push(to_info(&cfg, &dir, state));
        }
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(out)
}

pub fn get(state: &AppState, id: &str) -> Result<LocalServerInfo> {
    let (_, dirs) = Settings::load()?;
    let dir = dirs.local_server_dir(id);
    let cfg = load_config(&dir)
        .map_err(|_| Error::msg("Nie znaleziono tego serwera lokalnego."))?;
    Ok(to_info(&cfg, &dir, state))
}

pub async fn create(app: AppHandle, req: CreateLocalServer) -> Result<LocalServerInfo> {
    if !req.eula_accepted {
        return Err(Error::msg(
            "Musisz zaakceptować EULA Mojang, żeby utworzyć serwer.",
        ));
    }
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(Error::msg("Podaj nazwę serwera."));
    }
    let game = req.game_version.trim().to_string();
    if game.is_empty() {
        return Err(Error::msg("Wybierz wersję Minecraft."));
    }
    let (settings, dirs) = Settings::load()?;
    dirs.ensure()?;
    let mut software = req.software;
    let mut loader_version = req.loader_version.clone();
    let mut game_version = game;
    if let Some(inst_id) = req.source_instance_id.as_deref() {
        let inst = instances::get(&dirs, inst_id)?;
        game_version = inst.game_version.clone();
        match inst.loader {
            Loader::Fabric => {
                software = LocalSoftware::Fabric;
                if loader_version.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
                    loader_version = inst.loader_version.clone();
                }
            }
            Loader::Quilt => {
                software = LocalSoftware::Fabric;
                loader_version = None;
            }
            _ => {}
        }
    }
    let client = app.state::<AppState>().http.clone();
    probe_software(
        &client,
        &dirs,
        software,
        &game_version,
        loader_version.as_deref(),
    )
    .await?;
    let id = Uuid::new_v4().to_string();
    let dir = dirs.local_server_dir(&id);
    let created: Result<LocalServerInfo> = async {
        std::fs::create_dir_all(&dir)?;
        std::fs::create_dir_all(dir.join("logs"))?;
        let mut cfg = LocalServerConfig {
            id: id.clone(),
            name,
            software,
            game_version,
            loader_version,
            memory_mb: req.memory_mb.unwrap_or(settings.memory_max_mb).clamp(512, 32768),
            port: req.port.unwrap_or(25565).max(1),
            motd: req
                .motd
                .unwrap_or_else(default_motd)
                .trim()
                .to_string(),
            online_mode: req.online_mode,
            max_players: req.max_players.unwrap_or(10).clamp(1, 200),
            difficulty: req.difficulty.unwrap_or_else(default_difficulty),
            view_distance: req.view_distance.unwrap_or(10).clamp(2, 32),
            eula_accepted: true,
            source_instance_id: req.source_instance_id.clone(),
            play_address: None,
            java_path: req
                .java_path
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            required_java: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        store_required_java(&client, &dirs, &mut cfg).await;
        if cfg.java_path.is_some() {
            let required = cfg.required_java.unwrap_or_else(|| {
                meta::required_java_for_id(&cfg.game_version)
            });
            resolve_server_java(&dirs, &settings, &cfg, required)?;
        }
        if cfg.motd.is_empty() {
            cfg.motd = default_motd();
        }
        write_eula(&dir)?;
        write_runtime_properties(&dir, &cfg)?;
        if let Some(inst_id) = cfg.source_instance_id.clone() {
            let (mods, configs) = copy_from_instance(&dirs, &inst_id, &dir)?;
            emit_progress(
                &app,
                &cfg.id,
                &format!("Skopiowano {mods} modów i {configs} plików konfiguracji."),
            );
        }
        save_config(&dir, &cfg)?;
        install_software(&app, &client, &dirs, &mut cfg).await?;
        save_config(&dir, &cfg)?;
        Ok(to_info(&cfg, &dir, app.state::<AppState>().inner()))
    }
    .await;
    match created {
        Ok(info) => Ok(info),
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dir);
            Err(e)
        }
    }
}

pub async fn install(app: AppHandle, id: String) -> Result<LocalServerInfo> {
    let (_, dirs) = Settings::load()?;
    let dir = dirs.local_server_dir(&id);
    let mut cfg = load_config(&dir)?;
    let client = app.state::<AppState>().http.clone();
    install_software(&app, &client, &dirs, &mut cfg).await?;
    save_config(&dir, &cfg)?;
    Ok(to_info(&cfg, &dir, app.state::<AppState>().inner()))
}

pub fn update(state: &AppState, id: &str, patch: UpdateLocalServer) -> Result<LocalServerInfo> {
    let (_, dirs) = Settings::load()?;
    let dir = dirs.local_server_dir(id);
    let mut cfg = load_config(&dir)?;
    if let Some(name) = patch.name {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(Error::msg("Nazwa nie może być pusta."));
        }
        cfg.name = name;
    }
    if let Some(m) = patch.memory_mb {
        cfg.memory_mb = m.clamp(512, 32768);
    }
    if let Some(p) = patch.port {
        cfg.port = p.max(1);
    }
    if let Some(motd) = patch.motd {
        let motd = motd.trim().to_string();
        cfg.motd = if motd.is_empty() { default_motd() } else { motd };
    }
    if let Some(v) = patch.online_mode {
        cfg.online_mode = v;
    }
    if let Some(v) = patch.max_players {
        cfg.max_players = v.clamp(1, 200);
    }
    if let Some(v) = patch.difficulty {
        cfg.difficulty = v;
    }
    if let Some(v) = patch.view_distance {
        cfg.view_distance = v.clamp(2, 32);
    }
    if let Some(p) = patch.java_path {
        let t = p.trim();
        if t.is_empty() {
            cfg.java_path = None;
        } else {
            let required = cfg
                .required_java
                .unwrap_or_else(|| meta::required_java_for_id(&cfg.game_version));
            let rt = java::probe_java(Path::new(t)).ok_or_else(|| {
                Error::msg("Wybrana Java nie działa. Wskaż inną instalację.")
            })?;
            if rt.major < required {
                return Err(Error::msg(format!(
                    "Wybrana Java {} jest za stara dla Minecraft {}. Wymagana jest Java {required} lub nowsza.",
                    rt.major, cfg.game_version
                )));
            }
            cfg.java_path = Some(t.to_string());
        }
    }
    write_runtime_properties(&dir, &cfg)?;
    save_config(&dir, &cfg)?;
    Ok(to_info(&cfg, &dir, state))
}

pub async fn delete(app: AppHandle, id: String) -> Result<()> {
    if app
        .state::<AppState>()
        .local_servers
        .lock()
        .contains_key(&id)
    {
        stop(app.clone(), id.clone()).await?;
        tokio::time::sleep(Duration::from_millis(400)).await;
    }
    let (_, dirs) = Settings::load()?;
    let dir = dirs.local_server_dir(&id);
    if let Ok(cfg) = load_config(&dir) {
        if let Some(addr) = cfg.play_address {
            let _ = servers::remove_address(&dirs, &addr);
        }
    }
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

pub async fn start(app: AppHandle, id: String) -> Result<LocalServerInfo> {
    {
        let state = app.state::<AppState>();
        if state.local_servers.lock().contains_key(&id) {
            return Err(Error::msg("Ten serwer już działa."));
        }
    }
    let (settings, dirs) = Settings::load()?;
    let dir = dirs.local_server_dir(&id);
    let mut cfg = load_config(&dir)?;
    if !cfg.eula_accepted {
        return Err(Error::msg("Ten serwer nie ma zaakceptowanego EULA."));
    }
    {
        let state = app.state::<AppState>();
        for other in list(state.inner())? {
            if other.id != id && other.port == cfg.port && other.status != "stopped" {
                return Err(Error::msg(format!(
                    "Port {} jest zajęty przez inny serwer Lumen ({}).",
                    cfg.port, other.name
                )));
            }
        }
    }
    if !jar_path(&dir).is_file() {
        let client = app.state::<AppState>().http.clone();
        install_software(&app, &client, &dirs, &mut cfg).await?;
        save_config(&dir, &cfg)?;
    }
    write_eula(&dir)?;
    write_runtime_properties(&dir, &cfg)?;
    std::fs::create_dir_all(dir.join("logs"))?;

    let client = app.state::<AppState>().http.clone();
    let required = store_required_java(&client, &dirs, &mut cfg).await;
    save_config(&dir, &cfg)?;
    let java_rt = resolve_server_java(&dirs, &settings, &cfg, required)?;
    let java_exe = java_console_exe(Path::new(&java_rt.path));
    let xmx = cfg.memory_mb.max(512);
    let xms = (xmx / 4).clamp(512, xmx);
    let args = vec![
        format!("-Xms{xms}M"),
        format!("-Xmx{xmx}M"),
        "-Dfile.encoding=UTF-8".into(),
        "-jar".into(),
        JAR_NAME.into(),
        "nogui".into(),
    ];

    emit_status(&app, &cfg, "starting", None);
    emit_log(
        &app,
        &cfg.id,
        &format!(
            "[Octra] Start {} {} · Java {} · {} MB · port {}",
            cfg.software.label(),
            cfg.game_version,
            java_rt.major,
            cfg.memory_mb,
            cfg.port
        ),
    );

    let mut child = spawn_server_process(&java_exe, &args, &dir)?;
    let pid = child
        .id()
        .ok_or_else(|| Error::msg("Serwer wystartował, ale nie ma PID."))?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| Error::msg("Brak stdin procesu serwera."))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| Error::msg("Brak stdout procesu serwera."))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| Error::msg("Brak stderr procesu serwera."))?;

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<String>();
    app.state::<AppState>().local_servers.lock().insert(
        cfg.id.clone(),
        LocalServerProc {
            pid,
            cmd_tx: cmd_tx.clone(),
            status: "starting".into(),
        },
    );
    emit_status(&app, &cfg, "starting", Some(pid));

    tokio::spawn(watch_server(
        app.clone(),
        cfg.clone(),
        child,
        stdin,
        stdout,
        stderr,
        cmd_rx,
        dirs.clone(),
        dir.clone(),
    ));

    Ok(to_info(&cfg, &dir, app.state::<AppState>().inner()))
}

async fn pump_lines<R>(
    app: AppHandle,
    id: String,
    log_file: PathBuf,
    ready: Arc<AtomicBool>,
    cfg: LocalServerConfig,
    dirs: Dirs,
    dir: PathBuf,
    reader: TokioBufReader<R>,
) where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        append_log_line(&log_file, &line);
        emit_log(&app, &id, &line);
        if !ready.load(Ordering::Relaxed) && looks_ready(&line) {
            ready.store(true, Ordering::Relaxed);
            if let Some(state) = app.try_state::<AppState>() {
                if let Some(rt) = state.local_servers.lock().get_mut(&id) {
                    rt.status = "running".into();
                }
            }
            let addr = format!("127.0.0.1:{}", cfg.port);
            let play_name = format!("{} (lokalny)", cfg.name);
            if let Some(old) = cfg.play_address.as_ref() {
                if old != &addr {
                    let _ = servers::remove_address(&dirs, old);
                }
            }
            let _ = servers::upsert(&dirs, &play_name, &addr);
            if let Ok(mut stored) = load_config(&dir) {
                stored.play_address = Some(addr);
                let _ = save_config(&dir, &stored);
            }
            let pid = app.try_state::<AppState>().and_then(|s| {
                s.local_servers.lock().get(&id).map(|p| p.pid)
            });
            emit_status(&app, &cfg, "running", pid);
        }
    }
}

async fn watch_server(
    app: AppHandle,
    cfg: LocalServerConfig,
    mut child: tokio::process::Child,
    mut stdin: tokio::process::ChildStdin,
    stdout: tokio::process::ChildStdout,
    stderr: tokio::process::ChildStderr,
    mut cmd_rx: mpsc::UnboundedReceiver<String>,
    dirs: Dirs,
    dir: PathBuf,
) {
    let log_file = log_path(&dir);
    let ready = Arc::new(AtomicBool::new(false));
    let id = cfg.id.clone();

    tokio::spawn(pump_lines(
        app.clone(),
        id.clone(),
        log_file.clone(),
        ready.clone(),
        cfg.clone(),
        dirs.clone(),
        dir.clone(),
        TokioBufReader::new(stdout),
    ));
    tokio::spawn(pump_lines(
        app.clone(),
        id.clone(),
        log_file,
        ready,
        cfg.clone(),
        dirs,
        dir,
        TokioBufReader::new(stderr),
    ));

    loop {
        tokio::select! {
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(line) => {
                        let mut payload = line;
                        if !payload.ends_with('\n') {
                            payload.push('\n');
                        }
                        if stdin.write_all(payload.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            status = child.wait() => {
                let code = status.ok().and_then(|s| s.code()).unwrap_or(-1);
                emit_log(&app, &id, &format!("[Octra] Proces zakończony (kod {code})."));
                break;
            }
        }
    }

    if let Some(state) = app.try_state::<AppState>() {
        state.local_servers.lock().remove(&id);
    }
    emit_status(&app, &cfg, "stopped", None);
}

pub async fn stop(app: AppHandle, id: String) -> Result<()> {
    let (tx, pid) = {
        let state = app.state::<AppState>();
        let map = state.local_servers.lock();
        match map.get(&id) {
            Some(p) => (p.cmd_tx.clone(), p.pid),
            None => return Ok(()),
        }
    };
    let _ = tx.send("stop".into());
    for _ in 0..40 {
        tokio::time::sleep(Duration::from_millis(300)).await;
        if !app
            .state::<AppState>()
            .local_servers
            .lock()
            .contains_key(&id)
        {
            return Ok(());
        }
    }
    let _ = kill_pid_tree(pid);
    tokio::time::sleep(Duration::from_millis(400)).await;
    app.state::<AppState>().local_servers.lock().remove(&id);
    if let Ok(info) = get(app.state::<AppState>().inner(), &id) {
        let cfg = LocalServerConfig {
            id: info.id,
            name: info.name,
            software: info.software,
            game_version: info.game_version,
            loader_version: info.loader_version,
            memory_mb: info.memory_mb,
            port: info.port,
            motd: info.motd,
            online_mode: info.online_mode,
            max_players: info.max_players,
            difficulty: info.difficulty,
            view_distance: info.view_distance,
            eula_accepted: info.eula_accepted,
            source_instance_id: info.source_instance_id,
            play_address: None,
            java_path: info.java_path,
            required_java: Some(info.required_java),
            created_at: info.created_at,
        };
        emit_status(&app, &cfg, "stopped", None);
    }
    Ok(())
}

pub fn send_command(state: &AppState, id: &str, command: &str) -> Result<()> {
    let cmd = command.trim();
    if cmd.is_empty() {
        return Err(Error::msg("Wpisz komendę."));
    }
    let map = state.local_servers.lock();
    let proc = map
        .get(id)
        .ok_or_else(|| Error::msg("Serwer nie jest uruchomiony."))?;
    proc.cmd_tx
        .send(cmd.to_string())
        .map_err(|_| Error::msg("Nie udało się wysłać komendy — proces już nie działa."))?;
    Ok(())
}

pub fn read_log(id: &str) -> Result<String> {
    let (_, dirs) = Settings::load()?;
    let path = log_path(&dirs.local_server_dir(id));
    if !path.exists() {
        return Ok(String::new());
    }
    let file = std::fs::File::open(&path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map_while(std::result::Result::ok).collect();
    let start = lines.len().saturating_sub(400);
    Ok(lines[start..].join("\n"))
}

pub fn open_dir(id: &str) -> Result<()> {
    let (_, dirs) = Settings::load()?;
    let dir = dirs.local_server_dir(id);
    if !dir.exists() {
        return Err(Error::msg("Folder serwera nie istnieje."));
    }
    open_folder(&dir)
}

pub fn open_backups(id: &str) -> Result<()> {
    let (_, dirs) = Settings::load()?;
    let dir = backups_dir(&dirs.local_server_dir(id));
    open_folder(&dir)
}

pub fn open_properties(id: &str) -> Result<()> {
    let (_, dirs) = Settings::load()?;
    let path = props_path(&dirs.local_server_dir(id));
    if !path.exists() {
        return Err(Error::msg("Brak pliku server.properties — uruchom serwer raz."));
    }
    #[cfg(windows)]
    {
        let mut cmd = std::process::Command::new("notepad");
        cmd.arg(&path);
        crate::winhide::hide_std(&mut cmd);
        cmd.spawn()
            .map_err(|e| Error::msg(format!("Nie otwarto pliku: {e}")))?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        Err(Error::msg("Otwieranie pliku jest dostępne na Windows."))
    }
}

fn zip_tree(zip: &mut zip::ZipWriter<std::fs::File>, src: &Path, prefix: &str) -> Result<()> {
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    if !src.exists() {
        return Ok(());
    }
    for entry in WalkDir::new(src).into_iter().flatten() {
        let rel = match entry.path().strip_prefix(src) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let name = if rel.as_os_str().is_empty() {
            prefix.trim_end_matches('/').to_string()
        } else {
            format!("{prefix}{}", rel.to_string_lossy().replace('\\', "/"))
        };
        if entry.file_type().is_dir() {
            zip.add_directory(format!("{name}/"), options)?;
            continue;
        }
        if rel.file_name().and_then(|s| s.to_str()) == Some("session.lock") {
            continue;
        }
        zip.start_file(&name, options)?;
        let mut f = std::fs::File::open(entry.path())?;
        std::io::copy(&mut f, zip)?;
    }
    Ok(())
}

pub fn backup_world(id: &str) -> Result<String> {
    let (_, dirs) = Settings::load()?;
    let dir = dirs.local_server_dir(id);
    let world = dir.join("world");
    let nether = dir.join("world_nether");
    let end = dir.join("world_the_end");
    if !world.exists() && !nether.exists() && !end.exists() {
        return Err(Error::msg(
            "Brak świata do kopii — uruchom serwer raz, żeby go wygenerować.",
        ));
    }
    let backups = backups_dir(&dir);
    std::fs::create_dir_all(&backups)?;
    let stamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let dest = backups.join(format!("world-{stamp}.zip"));
    let file = std::fs::File::create(&dest)?;
    let mut zip = zip::ZipWriter::new(file);
    zip_tree(&mut zip, &world, "world/")?;
    zip_tree(&mut zip, &nether, "world_nether/")?;
    zip_tree(&mut zip, &end, "world_the_end/")?;
    zip.finish()?;
    Ok(dest.to_string_lossy().to_string())
}
