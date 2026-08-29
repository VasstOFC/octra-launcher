use std::path::Path;

use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
use winreg::RegKey;

const UNINSTALL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Octra";
const PRODUCT_KEY: &str = r"Software\octra\Octra";

fn hive(all_users: bool) -> winreg::HKEY {
    if all_users {
        HKEY_LOCAL_MACHINE
    } else {
        HKEY_CURRENT_USER
    }
}

pub fn write_uninstall(
    dest: &Path,
    all_users: bool,
    estimated_kb: u32,
) -> Result<(), String> {
    let root = RegKey::predef(hive(all_users));
    let (key, _) = root
        .create_subkey(UNINSTALL_KEY)
        .map_err(|e| format!("Rejestr: {e}"))?;
    let lumen = dest.join("octra.exe");
    let uninstall = dest.join("uninstall.exe");
    key.set_value("DisplayName", &"Octra Launcher")
        .map_err(|e| e.to_string())?;
    key.set_value("DisplayVersion", &env!("LUMEN_VERSION"))
        .map_err(|e| e.to_string())?;
    key.set_value("Publisher", &"Octra Launcher")
        .map_err(|e| e.to_string())?;
    key.set_value("InstallLocation", &dest.to_string_lossy().as_ref())
        .map_err(|e| e.to_string())?;
    key.set_value("DisplayIcon", &lumen.to_string_lossy().as_ref())
        .map_err(|e| e.to_string())?;
    key.set_value(
        "UninstallString",
        &format!("\"{}\"", uninstall.display()),
    )
    .map_err(|e| e.to_string())?;
    key.set_value("NoModify", &1u32).map_err(|e| e.to_string())?;
    key.set_value("NoRepair", &1u32).map_err(|e| e.to_string())?;
    key.set_value("EstimatedSize", &estimated_kb)
        .map_err(|e| e.to_string())?;
    key.set_value("MainBinaryName", &"octra.exe")
        .map_err(|e| e.to_string())?;

    let (prod, _) = root
        .create_subkey(PRODUCT_KEY)
        .map_err(|e| e.to_string())?;
    prod.set_value("", &dest.to_string_lossy().as_ref())
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn read_install_location(prefer_all_users: bool) -> Option<std::path::PathBuf> {
    for all in [prefer_all_users, !prefer_all_users] {
        if let Some(p) = read_location_from(hive(all)) {
            return Some(p);
        }
    }
    None
}

fn read_location_from(h: winreg::HKEY) -> Option<std::path::PathBuf> {
    let root = RegKey::predef(h);
    if let Ok(key) = root.open_subkey_with_flags(PRODUCT_KEY, KEY_READ) {
        if let Ok(val) = key.get_value::<String, _>("") {
            let p = std::path::PathBuf::from(val.trim_matches('"'));
            if !p.as_os_str().is_empty() {
                return Some(p);
            }
        }
    }
    if let Ok(key) = root.open_subkey_with_flags(UNINSTALL_KEY, KEY_READ) {
        if let Ok(val) = key.get_value::<String, _>("InstallLocation") {
            let p = std::path::PathBuf::from(val.trim_matches('"'));
            if !p.as_os_str().is_empty() {
                return Some(p);
            }
        }
    }
    None
}

pub fn remove(all_users: bool) {
    let root = RegKey::predef(hive(all_users));
    let _ = root.delete_subkey_all(UNINSTALL_KEY);
    let _ = root.delete_subkey_all(PRODUCT_KEY);
    if let Ok(lumen) = root.open_subkey_with_flags(r"Software\octra", KEY_WRITE) {
        let _ = lumen.delete_subkey("Octra Launcher");
    }
}
