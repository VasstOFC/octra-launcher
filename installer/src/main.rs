#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod args;
mod elevate;
mod install;
mod payload;
mod process;
mod registry;
mod shortcuts;
mod theme;
mod webview2;

use std::path::PathBuf;

use eframe::egui;

use app::InstallerApp;
use args::Mode;
use install::InstallRequest;

fn main() {
    match args::parse() {
        Ok(Mode::Pack {
            stub,
            payload_dir,
            out,
        }) => {
            attach_console();
            eprintln!("Pakowanie instalatora Octra…");
            match payload::make_sfx(&stub, &payload_dir, &out) {
                Ok(()) => eprintln!("Zapisano {}", out.display()),
                Err(e) => {
                    eprintln!("Błąd: {e}");
                    std::process::exit(1);
                }
            }
        }
        Ok(Mode::Unattended {
            uninstall,
            restart,
            update,
            no_shortcuts,
            dest,
            restart_args,
            all_users,
            hide_ui,
        }) => {
            let dest = dest.unwrap_or_else(|| install::default_dest(all_users));
            if hide_ui {
                if let Err(e) = run_unattended(
                    uninstall,
                    dest,
                    all_users,
                    !no_shortcuts,
                    !no_shortcuts,
                    update,
                    restart,
                    restart_args,
                ) {
                    attach_console();
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            } else {
                run_progress_window(
                    uninstall,
                    dest,
                    all_users,
                    !no_shortcuts,
                    !no_shortcuts,
                    update,
                    restart,
                    restart_args,
                );
            }
        }
        Ok(Mode::Gui {
            uninstall,
            elevated,
            preset,
        }) => run_gui(uninstall, elevated, preset),
        Err(e) => {
            attach_console();
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

fn run_unattended(
    uninstall: bool,
    dest: PathBuf,
    all_users: bool,
    start_menu: bool,
    desktop: bool,
    update: bool,
    restart: bool,
    restart_args: Vec<String>,
) -> Result<(), String> {
    if uninstall {
        install::run_uninstall(
            &install::UninstallRequest {
                dest,
                all_users,
                remove_data: false,
            },
            |_, _| {},
        )
    } else {
        install::run_install(
            &InstallRequest {
                dest,
                start_menu,
                desktop,
                all_users,
                update,
                restart,
                restart_args,
            },
            |_, _| {},
        )
        .map(|_| ())
    }
}

fn run_gui(uninstall: bool, elevated: bool, preset: args::InstallPreset) {
    let options = native_options();
    let _ = eframe::run_native(
        "Octra Launcher",
        options,
        Box::new(move |cc| {
            let mut app = InstallerApp::new(cc, uninstall, preset);
            if elevated && !uninstall {
                app.start_elevated_install();
            }
            Ok(Box::new(app))
        }),
    );
}

fn run_progress_window(
    uninstall: bool,
    dest: PathBuf,
    all_users: bool,
    start_menu: bool,
    desktop: bool,
    update: bool,
    restart: bool,
    restart_args: Vec<String>,
) {
    let preset = args::InstallPreset {
        dest: Some(dest.clone()),
        start_menu,
        desktop,
        all_users,
    };
    let options = native_options();
    let _ = eframe::run_native(
        "Octra Launcher",
        options,
        Box::new(move |cc| {
            let mut app = InstallerApp::new(cc, uninstall, preset);
            if uninstall {
                app.start_unattended_uninstall();
            } else {
                let req = InstallRequest {
                    dest,
                    start_menu,
                    desktop,
                    all_users,
                    update,
                    restart,
                    restart_args,
                };
                app.start_unattended(req);
            }
            Ok(Box::new(app))
        }),
    );
}

fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([860.0, 560.0])
            .with_min_inner_size([860.0, 560.0])
            .with_max_inner_size([860.0, 560.0])
            .with_resizable(false)
            .with_decorations(false)
            .with_title("Octra Launcher")
            .with_icon(window_icon())
            .with_taskbar(true),
        centered: true,
        ..Default::default()
    }
}

fn window_icon() -> egui::IconData {
    let bytes = include_bytes!("../../src-tauri/icons/32x32.png");
    let img = image::load_from_memory(bytes)
        .expect("icon")
        .into_rgba8();
    let (w, h) = img.dimensions();
    egui::IconData {
        rgba: img.into_raw(),
        width: w,
        height: h,
    }
}

fn attach_console() {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::System::Console::{AllocConsole, AttachConsole, ATTACH_PARENT_PROCESS};
        if AttachConsole(ATTACH_PARENT_PROCESS).is_err() {
            let _ = AllocConsole();
        }
    }
}
