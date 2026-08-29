use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::Serialize;
use sysinfo::{Pid, Process, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tauri::{AppHandle, Emitter, Manager};

use crate::auth::McSession;
use crate::error::{Error, Result};
use crate::instances::Instance;
use crate::meta::{self, flatten_args, Features, VersionMeta};
use crate::paths::Dirs;
use crate::settings::Settings;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEvent {
    pub instance_id: String,
    pub account_uuid: String,
    pub pid: u32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameExitEvent {
    pub instance_id: String,
    pub account_uuid: String,
    pub code: i32,
}

pub fn running_key(instance_id: &str, account_uuid: &str) -> String {
    format!("{instance_id}:{account_uuid}")
}

pub fn instance_has_running(running: &HashMap<String, Vec<u32>>, instance_id: &str) -> bool {
    let prefix = format!("{instance_id}:");
    running.keys().any(|k| k.starts_with(&prefix))
}

/// `-javaagent` musi być wśród pierwszych argumentów JVM (przed main class).
pub fn prepend_javaagent(args: &mut Vec<String>, jar: &Path, api_root: &str) {
    let agent = format!("-javaagent:{}={}", jar.display(), api_root.trim_end_matches('/'));
    args.insert(0, agent);
    args.insert(1, "-Dauthlibinjector.legacySkinPolyfill=enabled".into());
}

pub fn classpath_sep() -> &'static str {
    if cfg!(windows) { ";" } else { ":" }
}

pub fn extract_natives(dirs: &Dirs, inst: &Instance, meta: &VersionMeta) -> Result<PathBuf> {
    let dest = dirs.natives_dir(&inst.id);
    if dest.exists() {
        std::fs::remove_dir_all(&dest)?;
    }
    std::fs::create_dir_all(&dest)?;
    let features = Features::default();
    for lib in &meta.libraries {
        if !meta::rules_allow(&lib.rules, &features) {
            continue;
        }
        let files = crate::install::library_files(dirs, lib);
        for f in files {
            if !f.native {
                continue;
            }
            if !f.path.exists() {
                continue;
            }
            let excludes: Vec<&str> = f.exclude.iter().map(|s| s.as_str()).collect();
            crate::download::extract_zip(&f.path, &dest, &excludes)?;
        }
    }
    Ok(dest)
}

pub fn build_classpath(dirs: &Dirs, inst: &Instance, meta: &VersionMeta) -> Result<String> {
    let features = Features::default();
    let mut entries = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for lib in &meta.libraries {
        if !meta::rules_allow(&lib.rules, &features) {
            continue;
        }
        if lib.natives.is_some() && lib.downloads.as_ref().and_then(|d| d.artifact.as_ref()).is_none() {
            continue;
        }
        for f in crate::install::library_files(dirs, lib) {
            if f.native && lib.natives.is_some() {
                continue;
            }
            // natives-windows jars still go on classpath for LWJGL 3
            if f.path.exists() {
                let s = f.path.to_string_lossy().to_string();
                if seen.insert(s.clone()) {
                    entries.push(s);
                }
            }
        }
    }
    let vanilla_jar = dirs.version_jar(&inst.game_version);
    let version_jar = dirs.version_jar(&inst.version_id);
    let client = if version_jar.exists() && version_jar != vanilla_jar {
        version_jar
    } else {
        vanilla_jar
    };
    if client.exists() {
        entries.push(client.to_string_lossy().to_string());
    }
    Ok(entries.join(classpath_sep()))
}

fn replace_vars(arg: &str, map: &HashMap<&str, String>) -> String {
    let mut out = arg.to_string();
    for (k, v) in map {
        out = out.replace(&format!("${{{k}}}"), v);
        out = out.replace(&format!("${k}"), v);
    }
    out
}

pub fn build_command_line(
    java: &Path,
    dirs: &Dirs,
    settings: &Settings,
    inst: &Instance,
    meta: &VersionMeta,
    session: &McSession,
    natives: &Path,
) -> Result<(PathBuf, Vec<String>)> {
    let join = inst.join_server.trim();
    let quick_play = !join.is_empty();
    let (width, height, _fullscreen) = inst.window_size(settings);
    let features = Features {
        has_custom_resolution: true,
        has_quick_plays_support: quick_play,
        is_quick_play_multiplayer: quick_play,
    };
    let cp = build_classpath(dirs, inst, meta)?;
    let game_dir = dirs.game_dir(&inst.id);
    std::fs::create_dir_all(&game_dir)?;
    let assets_root = dirs.assets.clone();
    let assets_index = meta
        .asset_index
        .as_ref()
        .map(|a| a.id.clone())
        .or_else(|| meta.assets.clone())
        .unwrap_or_else(|| inst.game_version.clone());
    let main_class = meta
        .main_class
        .clone()
        .ok_or_else(|| Error::msg("Brak mainClass w metadanych wersji."))?;
    let log_cfg = meta
        .logging
        .as_ref()
        .and_then(|l| l.client.as_ref())
        .and_then(|c| c.file.as_ref())
        .and_then(|f| f.id.clone());

    let mut vars: HashMap<&str, String> = HashMap::new();
    vars.insert("auth_player_name", session.name.clone());
    vars.insert("version_name", inst.version_id.clone());
    vars.insert("game_directory", game_dir.to_string_lossy().to_string());
    vars.insert("assets_root", assets_root.to_string_lossy().to_string());
    vars.insert("game_assets", assets_root.join("virtual").join("legacy").to_string_lossy().to_string());
    vars.insert("assets_index_name", assets_index);
    vars.insert("auth_uuid", session.uuid.replace('-', ""));
    let client_id = if session.access_token == "0" || settings.azure_client_id().trim().is_empty() {
        session.uuid.replace('-', "")
    } else {
        settings.azure_client_id()
    };
    vars.insert("auth_access_token", session.access_token.clone());
    vars.insert(
        "auth_session",
        format!("token:{}:{}", session.access_token, session.uuid.replace('-', "")),
    );
    vars.insert("clientid", client_id);
    vars.insert("auth_xuid", if session.xuid.is_empty() { "0".into() } else { session.xuid.clone() });
    vars.insert("user_type", session.user_type.clone());
    vars.insert("version_type", meta.version_type.clone().unwrap_or_else(|| "release".into()));
    vars.insert("natives_directory", natives.to_string_lossy().to_string());
    vars.insert("launcher_name", "Octra".into());
    vars.insert("launcher_version", env!("CARGO_PKG_VERSION").into());
    vars.insert("classpath", cp);
    vars.insert("library_directory", dirs.libraries.to_string_lossy().to_string());
    vars.insert("classpath_separator", classpath_sep().into());
    vars.insert("user_properties", "{}".into());
    vars.insert("resolution_width", width.to_string());
    vars.insert("resolution_height", height.to_string());
    vars.insert("quickPlayMultiplayer", join.to_string());

    let mem_min = inst.memory_min(settings).min(inst.memory_max(settings));
    let mem_max = inst.memory_max(settings);
    let mut jvm = vec![
        format!("-Xms{}M", mem_min),
        format!("-Xmx{}M", mem_max),
        "-Dfile.encoding=UTF-8".into(),
        format!("-Dminecraft.launcher.brand={}", "Octra"),
        format!("-Dminecraft.launcher.version={}", env!("CARGO_PKG_VERSION")),
    ];
    let extra_args = inst.extra_java_args(settings);
    if !extra_args.trim().is_empty() {
        jvm.extend(split_args(&extra_args));
    }
    if let Some(id) = &log_cfg {
        let p = dirs.assets.join("log_configs").join(id);
        if p.exists() {
            jvm.push(format!("-Dlog4j.configurationFile={}", p.to_string_lossy()));
        }
    }

    if let Some(args) = &meta.arguments {
        for a in flatten_args(&args.jvm, &features) {
            jvm.push(replace_vars(&a, &vars));
        }
    } else {
        jvm.push(format!("-Djava.library.path={}", natives.to_string_lossy()));
        jvm.push("-cp".into());
        jvm.push(vars["classpath"].clone());
    }

    // Guarantee natives path even if version JSON omitted it.
    if !jvm.iter().any(|a| a.contains("-Djava.library.path")) {
        jvm.insert(2, format!("-Djava.library.path={}", natives.to_string_lossy()));
    }

    let mut game = Vec::new();
    if let Some(args) = &meta.arguments {
        game.extend(
            flatten_args(&args.game, &features)
                .into_iter()
                .map(|a| replace_vars(&a, &vars)),
        );
    } else if let Some(legacy) = &meta.minecraft_arguments {
        game.extend(
            legacy
                .split_whitespace()
                .map(|a| replace_vars(a, &vars)),
        );
    } else {
        game.extend([
            "--username".into(),
            session.name.clone(),
            "--version".into(),
            inst.version_id.clone(),
            "--gameDir".into(),
            game_dir.to_string_lossy().to_string(),
            "--assetsDir".into(),
            assets_root.to_string_lossy().to_string(),
            "--assetIndex".into(),
            vars["assets_index_name"].clone(),
            "--uuid".into(),
            vars["auth_uuid"].clone(),
            "--accessToken".into(),
            session.access_token.clone(),
            "--userType".into(),
            session.user_type.clone(),
            "--versionType".into(),
            vars["version_type"].clone(),
        ]);
    }

    if !game.iter().any(|a| a == "--username") {
        game.insert(0, session.name.clone());
        game.insert(0, "--username".into());
    }

    if quick_play
        && !game.iter().any(|a| a == "--server" || a.contains("quickPlayMultiplayer"))
    {
        let (host, port) = split_host_port(join);
        game.push("--server".into());
        game.push(host);
        game.push("--port".into());
        game.push(port);
    }

    if !game.iter().any(|a| a == "--width") {
        game.push("--width".into());
        game.push(width.to_string());
        game.push("--height".into());
        game.push(height.to_string());
    }

    let mut args = jvm;
    args.push(main_class);
    args.extend(game);
    Ok((java.to_path_buf(), args))
}

fn split_host_port(addr: &str) -> (String, String) {
    let addr = addr.trim();
    if let Some((host, port)) = addr.rsplit_once(':') {
        if !host.is_empty() && port.chars().all(|c| c.is_ascii_digit()) {
            return (host.to_string(), port.to_string());
        }
    }
    (addr.to_string(), "25565".into())
}

fn split_args(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote = None;
    for c in s.chars() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            } else {
                cur.push(c);
            }
        } else if c == '"' || c == '\'' {
            quote = Some(c);
        } else if c.is_whitespace() {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

pub fn write_options_txt(game_dir: &Path, fullscreen: bool) -> Result<()> {
    let path = game_dir.join("options.txt");
    let mut lines: Vec<String> = if path.exists() {
        std::fs::read_to_string(&path)?
            .lines()
            .map(|l| l.to_string())
            .collect()
    } else {
        Vec::new()
    };
    let want = format!("fullscreen:{}", if fullscreen { "true" } else { "false" });
    if let Some(line) = lines
        .iter_mut()
        .find(|l| l.trim_start().to_ascii_lowercase().starts_with("fullscreen:"))
    {
        *line = want;
    } else {
        lines.push(want);
    }
    let mut text = lines.join("\n");
    if !text.ends_with('\n') {
        text.push('\n');
    }
    std::fs::write(path, text)?;
    Ok(())
}

fn parse_env_vars(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for raw in text.split(|c| c == '\n' || c == '\r' || c == ';') {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let k = k.trim();
            if !k.is_empty() {
                out.push((k.to_string(), v.trim().to_string()));
            }
        }
    }
    out
}

fn expand_hook(cmd: &str, vars: &HashMap<&'static str, String>) -> String {
    let mut out = cmd.to_string();
    let mut pairs: Vec<(&str, &String)> = vars.iter().map(|(k, v)| (*k, v)).collect();
    pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    for (k, v) in pairs {
        out = out.replace(&format!("${{{k}}}"), v);
        out = out.replace(&format!("${k}"), v);
    }
    out
}

fn hook_vars(inst: &Instance, dirs: &Dirs, java: &Path, java_args: &str) -> HashMap<&'static str, String> {
    let inst_dir = dirs.instance_dir(&inst.id);
    let game_dir = dirs.game_dir(&inst.id);
    let mut vars = HashMap::new();
    vars.insert("INST_NAME", inst.name.clone());
    vars.insert("INST_ID", inst.id.clone());
    vars.insert("INST_DIR", inst_dir.to_string_lossy().to_string());
    vars.insert("INST_MC_DIR", game_dir.to_string_lossy().to_string());
    vars.insert("INST_JAVA", java.to_string_lossy().to_string());
    vars.insert("INST_JAVA_ARGS", java_args.to_string());
    vars
}

pub fn run_hook(command: &str, cwd: &Path, vars: &HashMap<&'static str, String>) -> Result<()> {
    let expanded = expand_hook(command, vars);
    let expanded = expanded.trim();
    if expanded.is_empty() {
        return Ok(());
    }
    #[cfg(windows)]
    {
        let mut hook = std::process::Command::new("cmd");
        hook.args(["/C", expanded]).current_dir(cwd);
        crate::winhide::hide_std(&mut hook);
        let status = hook
            .status()
            .map_err(|e| Error::msg(format!("Nie uruchomiono hooka: {e}")))?;
        if !status.success() {
            return Err(Error::msg(format!(
                "Hook zakończył się kodem {}.",
                status.code().unwrap_or(-1)
            )));
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = (cwd, expanded);
        Err(Error::msg("Hooki są dostępne na Windows."))
    }
}

fn is_jvm_name(name: &OsStr) -> bool {
    let n = name.to_string_lossy().to_ascii_lowercase();
    matches!(n.as_str(), "java" | "java.exe" | "javaw" | "javaw.exe")
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_ascii_lowercase()
}

fn path_in_game_dir(path: &Path, game_dir: &Path) -> bool {
    let a = normalize_path(path);
    let b = normalize_path(game_dir);
    a == b || a.starts_with(&(b + "\\"))
}

fn matches_instance_jvm(proc: &Process, game_dir: &Path) -> bool {
    if !is_jvm_name(proc.name()) {
        return false;
    }
    if proc.cwd().is_some_and(|cwd| path_in_game_dir(cwd, game_dir)) {
        return true;
    }
    let needle = normalize_path(game_dir);
    proc.cmd().iter().any(|arg| {
        normalize_path(Path::new(arg)).contains(&needle)
    })
}

fn refresh_processes(sys: &mut System) {
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_cmd(UpdateKind::Always)
            .with_cwd(UpdateKind::Always)
            .with_exe(UpdateKind::OnlyIfNotSet),
    );
}

/// Living PIDs that belong to this launch: the spawned process, its descendants,
/// and any JVM whose cwd/cmdline points at this instance's game dir (re-exec).
fn snapshot_game_pids(sys: &mut System, root: u32, game_dir: &Path) -> Vec<u32> {
    refresh_processes(sys);
    let mut tree: HashSet<u32> = HashSet::new();
    if sys.process(Pid::from_u32(root)).is_some() {
        tree.insert(root);
    }
    loop {
        let mut extra = Vec::new();
        for (pid, proc) in sys.processes() {
            let p = pid.as_u32();
            if tree.contains(&p) {
                continue;
            }
            if proc
                .parent()
                .is_some_and(|parent| tree.contains(&parent.as_u32()))
            {
                extra.push(p);
            }
        }
        if extra.is_empty() {
            break;
        }
        tree.extend(extra);
    }

    let mut tracked: HashSet<u32> = HashSet::new();
    if tree.contains(&root) {
        tracked.insert(root);
    }
    for p in &tree {
        if sys
            .process(Pid::from_u32(*p))
            .is_some_and(|proc| is_jvm_name(proc.name()))
        {
            tracked.insert(*p);
        }
    }
    for (pid, proc) in sys.processes() {
        if matches_instance_jvm(proc, game_dir) {
            tracked.insert(pid.as_u32());
        }
    }
    tracked.into_iter().collect()
}

fn store_running(app: &AppHandle, id: &str, pids: Vec<u32>) {
    if let Some(state) = app.try_state::<crate::AppState>() {
        if pids.is_empty() {
            return;
        }
        state.running.lock().insert(id.to_string(), pids);
    }
}

async fn watch_game_until_exit(
    app: AppHandle,
    instance_id: String,
    account_uuid: String,
    run_key: String,
    mut child: tokio::process::Child,
    root: u32,
    game_dir: PathBuf,
    post_exit: Option<(PathBuf, String, HashMap<&'static str, String>)>,
) {
    let mut sys = System::new();
    let mut child_done = false;
    let mut exit_code = -1;
    let mut empty_since: Option<Instant> = None;
    const GRACE: Duration = Duration::from_millis(2000);

    loop {
        if !child_done {
            match tokio::time::timeout(Duration::from_millis(300), child.wait()).await {
                Ok(status) => {
                    child_done = true;
                    exit_code = status.ok().and_then(|s| s.code()).unwrap_or(-1);
                }
                Err(_) => {}
            }
        } else {
            tokio::time::sleep(Duration::from_millis(400)).await;
        }

        let pids = snapshot_game_pids(&mut sys, root, &game_dir);
        if !pids.is_empty() {
            store_running(&app, &run_key, pids);
            empty_since = None;
            continue;
        }

        if !child_done {
            empty_since = None;
            continue;
        }

        match empty_since {
            None => empty_since = Some(Instant::now()),
            Some(t) if t.elapsed() >= GRACE => break,
            Some(_) => {}
        }
    }

    if let Some(state) = app.try_state::<crate::AppState>() {
        state.running.lock().remove(&run_key);
        if let Some(started) = state.play_started.lock().remove(&run_key) {
            let secs = started.elapsed().as_secs();
            if let Ok((_, dirs)) = crate::settings::Settings::load() {
                let _ = crate::instances::add_play_time(&dirs, &instance_id, secs);
            }
        }
        if state.running.lock().is_empty() {
            state.discord.set_idle();
        }
    }
    if let Some((cwd, cmd, vars)) = post_exit {
        if !cmd.trim().is_empty() {
            let _ = run_hook(&cmd, &cwd, &vars);
        }
    }
    if let Ok((_, dirs)) = crate::settings::Settings::load() {
        let _ = crate::servers::collect_all(&dirs);
    }
    let _ = app.emit(
        "game-exited",
        GameExitEvent {
            instance_id,
            account_uuid,
            code: exit_code,
        },
    );
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

fn java_launch_exe(java: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        if java
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|n| n.eq_ignore_ascii_case("java.exe"))
        {
            let javaw = java.with_file_name("javaw.exe");
            if javaw.is_file() {
                return javaw;
            }
        }
    }
    java.to_path_buf()
}

fn spawn_java_process(
    java: &Path,
    args: &[String],
    cwd: &Path,
    stdout: std::fs::File,
    stderr: std::fs::File,
    env: &[(String, String)],
) -> Result<tokio::process::Child> {
    let spawn_with = |flags: Option<u32>| -> std::io::Result<tokio::process::Child> {
        let mut cmd = tokio::process::Command::new(java);
        cmd.args(args)
            .current_dir(cwd)
            .stdout(stdout.try_clone()?)
            .stderr(stderr.try_clone()?);
        for (k, v) in env {
            cmd.env(k, v);
        }
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
        const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x01000000;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        // BREAKAWAY often returns ERROR_ACCESS_DENIED (5) when WebView2 puts
        // the launcher in a job that forbids breakaway. Retry without it.
        let attempts = [
            Some(CREATE_NEW_PROCESS_GROUP | CREATE_BREAKAWAY_FROM_JOB | CREATE_NO_WINDOW),
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
                    return Err(Error::msg(format!("Nie udało się uruchomić Javy: {e}")));
                }
            }
        }
        Err(Error::msg(format!(
            "Nie udało się uruchomić Javy: {}",
            last.map(|e| e.to_string()).unwrap_or_else(|| "odmowa dostępu".into())
        )))
    }
    #[cfg(not(windows))]
    {
        spawn_with(None).map_err(|e| Error::msg(format!("Nie udało się uruchomić Javy: {e}")))
    }
}

pub async fn spawn_game(
    app: AppHandle,
    dirs: Dirs,
    settings: Settings,
    inst: Instance,
    account_uuid: String,
    java: PathBuf,
    args: Vec<String>,
) -> Result<u32> {
    let run_key = running_key(&inst.id, &account_uuid);
    let logs = dirs.instance_logs(&inst.id);
    std::fs::create_dir_all(&logs)?;
    let log_path = logs.join("latest.log");
    let mut log = std::fs::File::create(&log_path)?;
    writeln!(log, "Octra launch {}", inst.version_id)?;
    writeln!(log, "{} {}", java.display(), args.join(" "))?;
    let game_dir = dirs.game_dir(&inst.id);
    std::fs::create_dir_all(&game_dir)?;
    let (_, _, fullscreen) = inst.window_size(&settings);
    match write_options_txt(&game_dir, fullscreen) {
        Ok(()) => writeln!(log, "options.txt: fullscreen={fullscreen}")?,
        Err(e) => writeln!(log, "options.txt: {e}")?,
    }
    match crate::servers::sync_instance(&dirs, &game_dir) {
        Ok(n) => writeln!(log, "servers.dat: {n} serwer(ów) ze wspólnej listy")?,
        Err(e) => writeln!(log, "servers.dat: {e}")?,
    }

    let extra_args = inst.extra_java_args(&settings);
    let vars = hook_vars(&inst, &dirs, &java, &extra_args);
    let inst_dir = dirs.instance_dir(&inst.id);
    if inst.custom_hooks && !inst.pre_launch.trim().is_empty() {
        writeln!(log, "pre-launch: {}", inst.pre_launch)?;
        drop(log);
        run_hook(&inst.pre_launch, &inst_dir, &vars)?;
    } else {
        drop(log);
    }

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    let err_file = log_file.try_clone()?;

    let env = parse_env_vars(&inst.env_vars_text(&settings));
    let wrapper = if inst.custom_hooks {
        split_args(&inst.wrapper)
    } else {
        Vec::new()
    };

    let child = if wrapper.is_empty() {
        let java_exe = java_launch_exe(&java);
        spawn_java_process(&java_exe, &args, &game_dir, log_file, err_file, &env)?
    } else {
        let mut wrapped = wrapper;
        let exe = PathBuf::from(wrapped.remove(0));
        wrapped.push(java.to_string_lossy().to_string());
        wrapped.extend(args.iter().cloned());
        spawn_java_process(&exe, &wrapped, &game_dir, log_file, err_file, &env)?
    };
    let pid = child
        .id()
        .ok_or_else(|| Error::msg("Java wystartowała, ale nie ma PID."))?;
    store_running(&app, &run_key, vec![pid]);
    if let Some(state) = app.try_state::<crate::AppState>() {
        state
            .play_started
            .lock()
            .insert(run_key.clone(), Instant::now());
        state.discord.set_playing(&inst.name);
    }
    let _ = app.emit(
        "game-started",
        GameEvent {
            instance_id: inst.id.clone(),
            account_uuid: account_uuid.clone(),
            pid,
        },
    );
    let post_exit = if inst.custom_hooks && !inst.post_exit.trim().is_empty() {
        Some((inst_dir, inst.post_exit.clone(), vars))
    } else {
        None
    };
    tokio::spawn(watch_game_until_exit(
        app.clone(),
        inst.id.clone(),
        account_uuid,
        run_key,
        child,
        pid,
        game_dir,
        post_exit,
    ));
    Ok(pid)
}

pub fn stop_game(app: &AppHandle, run_key: &str) -> Result<()> {
    let state = app
        .try_state::<crate::AppState>()
        .ok_or_else(|| Error::msg("Brak stanu launchera."))?;
    let pids = state
        .running
        .lock()
        .get(run_key)
        .cloned()
        .filter(|p| !p.is_empty())
        .ok_or_else(|| Error::msg("Ta sesja gry nie jest uruchomiona."))?;
    let mut killed = false;
    let mut last_err: Option<std::io::Error> = None;
    for pid in pids {
        match kill_pid_tree(pid) {
            Ok(true) => killed = true,
            Ok(false) => {}
            Err(e) => last_err = Some(e),
        }
    }
    if killed {
        return Ok(());
    }
    state.running.lock().remove(run_key);
    if let Some(e) = last_err {
        return Err(Error::msg(format!("Nie zatrzymano gry: {e}")));
    }
    #[cfg(windows)]
    {
        Err(Error::msg(
            "Nie udało się zatrzymać procesu gry (może już się zamknął).",
        ))
    }
    #[cfg(not(windows))]
    {
        Err(Error::msg("Zatrzymywanie gry jest dostępne na Windows."))
    }
}

pub fn read_log(dirs: &Dirs, id: &str) -> Result<String> {
    let path = dirs.instance_logs(id).join("latest.log");
    if !path.exists() {
        return Ok(String::new());
    }
    Ok(std::fs::read_to_string(path)?)
}
