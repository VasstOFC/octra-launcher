//! Remove leftover Octra Launcher 1.14.0 installs after migrating to Octra App.
//!
//! The old identifier (`pl.octra.launcher`) installs next to this app, so an
//! in-app update can launch Octra App once and still leave 1.14.0 pinned.

use tracing::{info, warn};

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn uninstall_legacy_launcher() {}

#[cfg(windows)]
pub fn uninstall_legacy_launcher() {
    if let Err(error) = uninstall_legacy_launcher_inner() {
        warn!("Legacy Octra Launcher cleanup failed: {error}");
    }
}

#[cfg(windows)]
fn uninstall_legacy_launcher_inner() -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    const LEGACY_UNINSTALL_IDS: &[&str] = &["pl.octra.launcher"];
    const LEGACY_DISPLAY_NAMES: &[&str] = &["Octra Launcher"];
    const LEGACY_PROCESS_NAMES: &[&str] =
        &["Octra Launcher.exe", "octra-launcher.exe"];
    const UNINSTALL_REL: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Uninstall";

    let current_exe = std::env::current_exe().ok();
    let current_dir = current_exe
        .as_ref()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf);

    let mut commands = Vec::<String>::new();
    let mut install_dirs = Vec::<PathBuf>::new();

    for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        let Ok(root) = RegKey::predef(hive)
            .open_subkey_with_flags(UNINSTALL_REL, KEY_READ)
        else {
            continue;
        };
        let keys: Vec<String> =
            root.enum_keys().filter_map(|key| key.ok()).collect();
        for name in keys {
            let Ok(sub) = root.open_subkey_with_flags(&name, KEY_READ) else {
                continue;
            };
            let display: String =
                sub.get_value("DisplayName").unwrap_or_default();
            let is_legacy = LEGACY_UNINSTALL_IDS
                .iter()
                .any(|id| name.eq_ignore_ascii_case(id))
                || LEGACY_DISPLAY_NAMES
                    .iter()
                    .any(|legacy| display.eq_ignore_ascii_case(legacy));
            if !is_legacy {
                continue;
            }

            if let Ok(location) = sub.get_value::<String, _>("InstallLocation")
            {
                let trimmed = location.trim().trim_matches('"');
                if !trimmed.is_empty() {
                    install_dirs.push(PathBuf::from(trimmed));
                }
            }

            let quiet: String =
                sub.get_value("QuietUninstallString").unwrap_or_default();
            let uninstall: String =
                sub.get_value("UninstallString").unwrap_or_default();
            if !quiet.trim().is_empty() {
                commands.push(quiet);
            } else if !uninstall.trim().is_empty() {
                let uninstall = uninstall.trim().to_string();
                if uninstall.to_ascii_lowercase().contains("/s") {
                    commands.push(uninstall);
                } else {
                    commands.push(format!("{uninstall} /S"));
                }
            }
        }
    }

    if let Some(local) = dirs::data_local_dir() {
        install_dirs.push(local.join("Octra Launcher"));
    }

    let occupying_current_install = current_dir.as_ref().is_some_and(|dir| {
        install_dirs
            .iter()
            .any(|install| install.exists() && dir.starts_with(install))
    });
    if occupying_current_install {
        return Ok(());
    }

    if commands.is_empty()
        && install_dirs.iter().all(|dir| !dir.exists())
        && !legacy_shortcut_exists()
    {
        return Ok(());
    }

    info!("Removing leftover Octra Launcher 1.14.0 install");

    for process in LEGACY_PROCESS_NAMES {
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", process])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }

    for command in commands {
        let status = Command::new("cmd")
            .args(["/C", &command])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|error| {
                format!("failed to run legacy uninstaller: {error}")
            })?;
        if !status.success() {
            warn!("Legacy uninstaller exited with {status}: {command}");
        }
    }

    for dir in &install_dirs {
        if dir.exists() {
            if let Err(error) = std::fs::remove_dir_all(dir) {
                warn!(
                    "Could not delete leftover install dir {}: {error}",
                    dir.display()
                );
            }
        }
    }

    rewrite_launcher_shortcuts(current_exe.as_deref());
    Ok(())
}

#[cfg(windows)]
fn legacy_shortcut_exists() -> bool {
    shortcut_locations().into_iter().any(|path| path.exists())
}

#[cfg(windows)]
fn shortcut_locations() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    if let Some(desktop) = dirs::desktop_dir() {
        paths.push(desktop.join("Octra Launcher.lnk"));
    }
    if let Some(data) = dirs::data_dir() {
        paths.push(
            data.join(
                r"Microsoft\Windows\Start Menu\Programs\Octra Launcher.lnk",
            ),
        );
        paths.push(data.join(
            r"Microsoft\Windows\Start Menu\Programs\Octra\Octra Launcher.lnk",
        ));
    }
    paths
}

#[cfg(windows)]
fn rewrite_launcher_shortcuts(current_exe: Option<&std::path::Path>) {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let Some(exe) = current_exe else {
        return;
    };
    if !exe.exists() {
        return;
    }

    let exe_str = exe.to_string_lossy().replace('\'', "''");
    let workdir = exe
        .parent()
        .map(|path| path.to_string_lossy().replace('\'', "''"))
        .unwrap_or_default();
    let locations = shortcut_locations();
    if locations.is_empty() {
        return;
    }

    let mut script =
        String::from("$shell = New-Object -ComObject WScript.Shell\n");
    for path in &locations {
        let path_str = path.to_string_lossy().replace('\'', "''");
        script.push_str(&format!(
            "$s = $shell.CreateShortcut('{path_str}'); $s.TargetPath = '{exe_str}'; $s.WorkingDirectory = '{workdir}'; $s.Save()\n"
        ));
    }

    let _ = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(CREATE_NO_WINDOW)
        .status();
}
