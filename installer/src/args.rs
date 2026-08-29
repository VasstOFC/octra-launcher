use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum Mode {
    Pack {
        stub: PathBuf,
        payload_dir: PathBuf,
        out: PathBuf,
    },
    Gui {
        uninstall: bool,
        elevated: bool,
        preset: InstallPreset,
    },
    Unattended {
        uninstall: bool,
        restart: bool,
        update: bool,
        no_shortcuts: bool,
        dest: Option<PathBuf>,
        restart_args: Vec<String>,
        all_users: bool,
        hide_ui: bool,
    },
}

#[derive(Debug, Clone, Default)]
pub struct InstallPreset {
    pub dest: Option<PathBuf>,
    pub start_menu: bool,
    pub desktop: bool,
    pub all_users: bool,
}

pub fn parse() -> Result<Mode, String> {
    let raw: Vec<String> = env::args().skip(1).collect();
    parse_from(raw, &current_exe_name())
}

fn current_exe_name() -> String {
    env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn parse_from(raw: Vec<String>, exe_name: &str) -> Result<Mode, String> {
    if raw.first().map(|s| eq_ci(s, "--make-sfx")).unwrap_or(false) {
        let mut stub = None;
        let mut payload_dir = None;
        let mut out = None;
        let mut i = 1;
        while i < raw.len() {
            match raw[i].as_str() {
                "--stub" => {
                    stub = Some(PathBuf::from(need(&raw, i + 1, "--stub")?));
                    i += 2;
                }
                "--payload-dir" => {
                    payload_dir = Some(PathBuf::from(need(&raw, i + 1, "--payload-dir")?));
                    i += 2;
                }
                "--out" => {
                    out = Some(PathBuf::from(need(&raw, i + 1, "--out")?));
                    i += 2;
                }
                other => return Err(format!("Nieznany argument pakowania: {other}")),
            }
        }
        return Ok(Mode::Pack {
            stub: stub.ok_or("Brak --stub")?,
            payload_dir: payload_dir.ok_or("Brak --payload-dir")?,
            out: out.ok_or("Brak --out")?,
        });
    }

    let mut uninstall = exe_name == "uninstall.exe" || exe_name == "uninstall";
    let mut restart = false;
    let mut update = false;
    let mut no_shortcuts = false;
    let mut silent = false;
    let mut passive = false;
    let mut elevated = false;
    let mut all_users = false;
    let mut dest = None;
    let mut restart_args = Vec::new();
    let mut start_menu = true;
    let mut desktop = true;
    let mut i = 0;

    while i < raw.len() {
        let a = raw[i].as_str();
        if eq_ci(a, "--uninstall") || eq_ci(a, "/uninstall") {
            uninstall = true;
            i += 1;
            continue;
        }
        if eq_ci(a, "--elevated") {
            elevated = true;
            i += 1;
            continue;
        }
        if eq_ci(a, "--all-users") {
            all_users = true;
            i += 1;
            continue;
        }
        if eq_ci(a, "--start-menu") {
            start_menu = true;
            i += 1;
            continue;
        }
        if eq_ci(a, "--no-start-menu") {
            start_menu = false;
            i += 1;
            continue;
        }
        if eq_ci(a, "--desktop") {
            desktop = true;
            i += 1;
            continue;
        }
        if eq_ci(a, "--no-desktop") {
            desktop = false;
            i += 1;
            continue;
        }
        if eq_ci(a, "--dir") {
            dest = Some(PathBuf::from(need(&raw, i + 1, "--dir")?));
            i += 2;
            continue;
        }
        if looks_flag(a, "S") {
            silent = true;
            i += 1;
            continue;
        }
        if looks_flag(a, "P") {
            passive = true;
            i += 1;
            continue;
        }
        if looks_flag(a, "R") {
            restart = true;
            i += 1;
            continue;
        }
        if looks_flag(a, "NS") {
            no_shortcuts = true;
            i += 1;
            continue;
        }
        if looks_flag(a, "UPDATE") {
            update = true;
            i += 1;
            continue;
        }
        if looks_flag(a, "ARGS") {
            restart_args.extend(raw[i + 1..].iter().cloned());
            break;
        }
        if let Some(path) = dest_flag(a) {
            dest = Some(PathBuf::from(path));
            i += 1;
            continue;
        }
        i += 1;
    }

    if silent || passive || update {
        Ok(Mode::Unattended {
            uninstall,
            restart,
            update,
            no_shortcuts,
            dest,
            restart_args,
            all_users,
            hide_ui: silent && !passive,
        })
    } else {
        Ok(Mode::Gui {
            uninstall,
            elevated,
            preset: InstallPreset {
                dest,
                start_menu,
                desktop,
                all_users,
            },
        })
    }
}

fn need(raw: &[String], i: usize, flag: &str) -> Result<String, String> {
    raw.get(i)
        .cloned()
        .ok_or_else(|| format!("Brak wartości dla {flag}"))
}

fn eq_ci(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

fn looks_flag(a: &str, name: &str) -> bool {
    let t = a.trim();
    let t = t.strip_prefix('/').or_else(|| t.strip_prefix('-')).unwrap_or(t);
    let t = t.strip_prefix('-').unwrap_or(t);
    t.eq_ignore_ascii_case(name)
}

fn dest_flag(a: &str) -> Option<&str> {
    let t = a.trim();
    t.strip_prefix("/D=")
        .or_else(|| t.strip_prefix("/d="))
        .or_else(|| t.strip_prefix("--dir="))
}

pub fn default_user_dir() -> PathBuf {
    local_app_data().join("Octra Launcher")
}

pub fn default_machine_dir() -> PathBuf {
    env::var_os("ProgramW6432")
        .or_else(|| env::var_os("ProgramFiles"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files"))
        .join("Octra Launcher")
}

pub fn local_app_data() -> PathBuf {
    env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs_fallback().join("AppData").join("Local")
        })
}

pub fn roaming_app_data() -> PathBuf {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs_fallback().join("AppData").join("Roaming"))
}

fn dirs_fallback() -> PathBuf {
    env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn is_uninstall_exe(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.eq_ignore_ascii_case("uninstall.exe"))
        .unwrap_or(false)
}
