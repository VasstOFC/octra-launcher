use std::fs;
use std::path::{Path, PathBuf};

use crate::args::{self, default_machine_dir, default_user_dir};
use crate::{payload, process, registry, shortcuts, webview2};

#[derive(Clone)]
pub struct InstallRequest {
    pub dest: PathBuf,
    pub start_menu: bool,
    pub desktop: bool,
    pub all_users: bool,
    pub update: bool,
    pub restart: bool,
    pub restart_args: Vec<String>,
}

pub struct UninstallRequest {
    pub dest: PathBuf,
    pub all_users: bool,
    pub remove_data: bool,
}

pub fn default_dest(all_users: bool) -> PathBuf {
    registry::read_install_location(all_users).unwrap_or_else(|| {
        if all_users {
            default_machine_dir()
        } else {
            default_user_dir()
        }
    })
}

pub fn run_install(
    req: &InstallRequest,
    mut on_progress: impl FnMut(f32, &str),
) -> Result<InstallResult, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    if payload::read_info(&exe)
        .map_err(|e| e.to_string())?
        .is_none()
        && !args::is_uninstall_exe(&exe)
    {
        return Err(
            "Ten plik nie zawiera pakietu Octra. Pobierz instalator ze wydania Octra.".into(),
        );
    }

    on_progress(0.02, "Sprawdzanie folderu…");
    fs::create_dir_all(&req.dest).map_err(|e| {
        format!(
            "Nie można utworzyć folderu {}.\n{e}",
            req.dest.display()
        )
    })?;

    process::wait_or_stop(true)?;

    on_progress(0.08, "Kopiowanie plików…");
    payload::extract_to(&exe, &req.dest, |p, msg| {
        on_progress(0.08 + p * 0.72, msg);
    })?;

    on_progress(0.82, "Zapisywanie deinstalatora…");
    let stub = payload::extract_stub(&exe).map_err(|e| e.to_string())?;
    fs::write(req.dest.join("uninstall.exe"), stub).map_err(|e| e.to_string())?;

    let mut webview_ok = true;
    if !req.update {
        on_progress(0.86, "Przygotowywanie środowiska…");
        match webview2::ensure(|s| on_progress(0.86, s)) {
            Ok(ok) => webview_ok = ok,
            Err(_) => webview_ok = false,
        }
    }

    if req.start_menu {
        on_progress(0.92, "Tworzenie skrótów…");
        let link = shortcuts::start_menu_link(req.all_users);
        shortcuts::create_shortcut(&link, &req.dest.join("octra.exe"), &req.dest)?;
    }
    if req.desktop {
        on_progress(0.94, "Tworzenie skrótów…");
        let link = shortcuts::desktop_link(req.all_users);
        shortcuts::create_shortcut(&link, &req.dest.join("octra.exe"), &req.dest)?;
    }

    on_progress(0.97, "Kończenie…");
    let size_kb = dir_size_kb(&req.dest);
    registry::write_uninstall(&req.dest, req.all_users, size_kb)?;

    if req.restart {
        let _ = std::process::Command::new(req.dest.join("octra.exe"))
            .args(&req.restart_args)
            .current_dir(&req.dest)
            .spawn();
    }

    on_progress(1.0, "Gotowe.");
    Ok(InstallResult { webview_ok })
}

pub struct InstallResult {
    pub webview_ok: bool,
}

pub fn run_uninstall(
    req: &UninstallRequest,
    mut on_progress: impl FnMut(f32, &str),
) -> Result<(), String> {
    process::wait_or_stop(true)?;
    on_progress(0.1, "Usuwanie skrótów…");
    shortcuts::remove_shortcuts(req.all_users);
    shortcuts::remove_shortcuts(!req.all_users);

    on_progress(0.4, "Usuwanie plików…");
    remove_install_tree(&req.dest)?;

    on_progress(0.8, "Usuwanie wpisu w systemie…");
    registry::remove(req.all_users);
    registry::remove(!req.all_users);

    if req.remove_data {
        on_progress(0.9, "Usuwanie danych…");
        let _ = remove_dir_quiet(&crate::args::roaming_app_data().join(".octralauncher-dev"));
        let _ = remove_dir_quiet(&crate::args::roaming_app_data().join("pl.octra.launcher"));
        let _ = remove_dir_quiet(&crate::args::local_app_data().join("pl.octra.launcher"));
    }

    on_progress(1.0, "Gotowe.");
    Ok(())
}

fn remove_install_tree(dest: &Path) -> Result<(), String> {
    if !dest.exists() {
        return Ok(());
    }
    match fs::remove_dir_all(dest) {
        Ok(()) => Ok(()),
        Err(_) => {
            let _ = fs::remove_file(dest.join("octra.exe"));
            let _ = fs::remove_file(dest.join("uninstall.exe"));
            if dest.join("packs").exists() {
                let _ = fs::remove_dir_all(dest.join("packs"));
            }
            let _ = fs::remove_dir(dest);
            Ok(())
        }
    }
}

fn remove_dir_quiet(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)
    } else {
        Ok(())
    }
}

fn dir_size_kb(path: &Path) -> u32 {
    let mut n = 0u64;
    for e in walkdir::WalkDir::new(path).into_iter().flatten() {
        if let Ok(md) = e.metadata() {
            if md.is_file() {
                n += md.len();
            }
        }
    }
    (n / 1024).min(u32::MAX as u64) as u32
}

pub fn launch_octra(dest: &Path) {
    let _ = std::process::Command::new(dest.join("octra.exe"))
        .current_dir(dest)
        .spawn();
}

pub fn disk_free_bytes(path: &Path) -> Option<u64> {
    #[cfg(windows)]
    {
        use windows::core::HSTRING;
        use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
        let probe = if path.exists() {
            path.to_path_buf()
        } else {
            path.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| path.to_path_buf())
        };
        let mut free = 0u64;
        let h = HSTRING::from(probe.to_string_lossy().as_ref());
        unsafe {
            GetDiskFreeSpaceExW(&h, Some(&mut free), None, None).ok()?;
        }
        Some(free)
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        None
    }
}

pub fn format_bytes(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1} GB", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.0} MB", n as f64 / 1_000_000.0)
    } else {
        format!("{n} B")
    }
}
