use std::path::Path;

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
        sl.SetDescription(&HSTRING::from("Octra Launcher"))
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

pub fn start_menu_link(all_users: bool) -> std::path::PathBuf {
    start_menu_dir(all_users).join("Octra Launcher.lnk")
}

pub fn desktop_link(all_users: bool) -> std::path::PathBuf {
    desktop_dir(all_users).join("Octra Launcher.lnk")
}

pub fn start_menu_dir(all_users: bool) -> std::path::PathBuf {
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

fn desktop_dir(all_users: bool) -> std::path::PathBuf {
    if all_users {
        public_dir().join("Desktop")
    } else {
        std::env::var_os("USERPROFILE")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("Desktop")
    }
}

fn program_data() -> std::path::PathBuf {
    std::env::var_os("ProgramData")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\ProgramData"))
}

fn public_dir() -> std::path::PathBuf {
    std::env::var_os("PUBLIC")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Users\Public"))
}
