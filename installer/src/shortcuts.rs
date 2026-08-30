use std::path::{Path, PathBuf};

pub const SHORTCUT_NAME: &str = "Octra App";
const LEGACY_SHORTCUT_NAME: &str = "Octra Launcher";

#[cfg(windows)]
pub fn create_shortcut(link: &Path, target: &Path, workdir: &Path) -> Result<(), String> {
    use windows::core::{Interface, HSTRING};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED, IPersistFile,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    if !target.is_file() {
        return Err(format!(
            "Nie znaleziono pliku docelowego skrótu: {}",
            target.display()
        ));
    }

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let sl: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| format!("Skrót: {e}"))?;
        sl.SetPath(&HSTRING::from(target.to_string_lossy().as_ref()))
            .map_err(|e| e.to_string())?;
        sl.SetWorkingDirectory(&HSTRING::from(workdir.to_string_lossy().as_ref()))
            .map_err(|e| e.to_string())?;
        sl.SetIconLocation(&HSTRING::from(target.to_string_lossy().as_ref()), 0)
            .map_err(|e| e.to_string())?;
        sl.SetDescription(&HSTRING::from(SHORTCUT_NAME))
            .map_err(|e| e.to_string())?;
        let persist: IPersistFile = sl.cast().map_err(|e| e.to_string())?;
        persist
            .Save(&HSTRING::from(link.to_string_lossy().as_ref()), true)
            .map_err(|e| e.to_string())?;
        CoUninitialize();
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn create_shortcut(_link: &Path, _target: &Path, _workdir: &Path) -> Result<(), String> {
    Ok(())
}

pub fn remove_if_exists(path: &Path) {
    let _ = std::fs::remove_file(path);
}

pub fn remove_shortcuts(all_users: bool) {
    remove_if_exists(&start_menu_link(all_users));
    remove_if_exists(&desktop_link(all_users));
    remove_if_exists(&legacy_start_menu_link(all_users));
    remove_if_exists(&legacy_desktop_link(all_users));
}

pub fn start_menu_link(all_users: bool) -> PathBuf {
    start_menu_dir(all_users).join(format!("{SHORTCUT_NAME}.lnk"))
}

pub fn desktop_link(all_users: bool) -> PathBuf {
    desktop_dir(all_users).join(format!("{SHORTCUT_NAME}.lnk"))
}

fn legacy_start_menu_link(all_users: bool) -> PathBuf {
    start_menu_dir(all_users).join(format!("{LEGACY_SHORTCUT_NAME}.lnk"))
}

fn legacy_desktop_link(all_users: bool) -> PathBuf {
    desktop_dir(all_users).join(format!("{LEGACY_SHORTCUT_NAME}.lnk"))
}

pub fn start_menu_dir(all_users: bool) -> PathBuf {
    if all_users {
        program_data()
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
    } else {
        crate::args::roaming_app_data()
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
    }
}

fn desktop_dir(all_users: bool) -> PathBuf {
    shell_folder_desktop(all_users).unwrap_or_else(|| fallback_desktop_dir(all_users))
}

/// Resolves the real Desktop folder (respects OneDrive redirection).
fn shell_folder_desktop(all_users: bool) -> Option<PathBuf> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    let hive = if all_users {
        HKEY_LOCAL_MACHINE
    } else {
        HKEY_CURRENT_USER
    };
    let value = if all_users {
        "Common Desktop"
    } else {
        "Desktop"
    };
    let root = RegKey::predef(hive);
    let key = root
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders")
        .ok()?;
    let raw: String = key.get_value(value).ok()?;
    Some(expand_env_path(&raw))
}

fn expand_env_path(raw: &str) -> PathBuf {
    let mut out = raw.to_string();
    for (name, value) in [
        ("USERPROFILE", std::env::var("USERPROFILE").ok()),
        ("PUBLIC", std::env::var("PUBLIC").ok()),
        (
            "ProgramData",
            std::env::var("ProgramData").ok(),
        ),
        (
            "APPDATA",
            std::env::var("APPDATA").ok(),
        ),
    ] {
        if let Some(v) = value {
            out = out.replace(&format!("%{name}%"), &v);
        }
    }
    PathBuf::from(out)
}

fn fallback_desktop_dir(all_users: bool) -> PathBuf {
    if all_users {
        public_dir().join("Desktop")
    } else {
        std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Desktop")
    }
}

fn program_data() -> PathBuf {
    std::env::var_os("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
}

fn public_dir() -> PathBuf {
    std::env::var_os("PUBLIC")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Users\Public"))
}
