mod auth;
mod cf_instance;
mod channel;
mod commands;
mod config;
mod curseforge;
mod discord_rpc;
mod download;
mod error;
mod github_release;
mod icon;
mod import_launchers;
mod install;
mod instances;
mod java;
mod launch;
mod loaders;
mod local_server;
mod meta;
mod migrate;
mod mojang_skins;
mod mrpack;
mod news;
mod paths;
mod relay;
mod servers;
mod settings;
mod skin_loader;
mod skins;
mod thumbs;
mod winhide;
mod yggdrasil;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::Mutex;
use tauri::Manager;
use tokio_util::sync::CancellationToken;

static QUITTING: AtomicBool = AtomicBool::new(false);

pub struct AppState {
    pub http: reqwest::Client,
    pub login_cancel: Mutex<Option<CancellationToken>>,
    pub install_cancel: Mutex<Option<CancellationToken>>,
    /// instance+account key → living game PIDs (JVM may re-exec; keep the whole tree)
    pub running: Mutex<HashMap<String, Vec<u32>>>,
    pub play_started: Mutex<HashMap<String, std::time::Instant>>,
    pub local_servers: Mutex<HashMap<String, local_server::LocalServerProc>>,
    pub skins: yggdrasil::SkinHub,
    pub relay: relay::RelayHub,
    pub discord: discord_rpc::DiscordRpc,
}

fn show_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    let show = MenuItem::with_id(app, "show", "Pokaż", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Zakończ", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;
    let Some(icon) = app.default_window_icon().cloned() else {
        eprintln!("Octra: brak ikony zasobnika");
        return Ok(());
    };

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip(crate::channel::current().window_title())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main(app),
            "quit" => {
                QUITTING.store(true, Ordering::SeqCst);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let http = reqwest::Client::builder()
        .user_agent(concat!(
            "Octra/",
            env!("CARGO_PKG_VERSION"),
            " (pl.octra.launcher; experimental Minecraft launcher)",
        ))
        .pool_max_idle_per_host(16)
        .build()
        .expect("http client");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            http,
            login_cancel: Mutex::new(None),
            install_cancel: Mutex::new(None),
            running: Mutex::new(HashMap::new()),
            play_started: Mutex::new(HashMap::new()),
            local_servers: Mutex::new(HashMap::new()),
            skins: yggdrasil::SkinHub::new(),
            relay: relay::RelayHub::new(),
            discord: discord_rpc::DiscordRpc::new(),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::get_data_dir,
            commands::get_app_info,
            commands::system_memory_mb,
            commands::scan_java,
            commands::download_java,
            commands::list_instances,
            commands::get_instance,
            commands::create_instance,
            commands::update_instance,
            commands::delete_instance,
            commands::unlink_instance_pack,
            commands::resync_instance_pack,
            commands::set_instance_icon_glyph,
            commands::pick_instance_icon_file,
            commands::pick_profile_wallpaper,
            commands::set_profile_wallpaper,
            commands::read_instance_wallpaper,
            commands::list_instance_content,
            commands::toggle_instance_content,
            commands::delete_instance_content,
            commands::import_local_content,
            commands::fetch_minecraft_versions,
            commands::fetch_loader_versions,
            commands::get_accounts,
            commands::account_has_token,
            commands::fetch_mojang_news,
            commands::set_active_account,
            commands::logout_account,
            commands::add_offline_account,
            commands::get_offline_skin,
            commands::save_offline_skin,
            commands::set_offline_skin_model,
            commands::reset_offline_skin,
            commands::start_login,
            commands::cancel_login,
            commands::install_instance,
            commands::cancel_install,
            commands::launch_instance,
            commands::stop_instance,
            commands::read_instance_log,
            commands::instance_game_dir,
            commands::open_instance_folder,
            commands::open_data_dir,
            commands::list_servers,
            commands::save_servers,
            commands::pick_mrpack_file,
            commands::import_mrpack,
            commands::import_modrinth_pack,
            commands::read_instance_icon,
            commands::search_modrinth_packs,
            commands::search_modrinth_content,
            commands::list_modrinth_content_versions,
            commands::install_modrinth_content,
            commands::get_featured_pack,
            commands::install_featured_pack,
            commands::duplicate_instance,
            commands::list_worlds,
            commands::delete_world,
            commands::copy_world,
            commands::open_world_folder,
            commands::list_crash_reports,
            commands::list_screenshots,
            commands::read_screenshot,
            commands::open_instance_subdir,
            commands::pick_java_exe,
            commands::pick_directory,
            commands::scan_curseforge_instances,
            commands::import_curseforge_instance,
            commands::probe_java_path,
            commands::purge_cache,
            commands::export_mrpack,
            commands::check_content_updates,
            commands::required_java_for_version,
            commands::list_local_servers,
            commands::get_local_server,
            commands::create_local_server,
            commands::probe_local_server,
            commands::list_paper_versions,
            commands::update_local_server,
            commands::delete_local_server,
            commands::install_local_server,
            commands::start_local_server,
            commands::stop_local_server,
            commands::send_local_server_command,
            commands::read_local_server_log,
            commands::open_local_server_folder,
            commands::open_local_server_backups,
            commands::open_local_server_properties,
            commands::backup_local_server_world,
            commands::check_github_release,
            commands::skins_lan_url,
            commands::list_all_screenshots,
            commands::get_account_skin,
            commands::get_mojang_skin_catalog,
            commands::get_minecraft_profile,
            commands::equip_mojang_skin,
            commands::upload_mojang_skin,
            commands::set_minecraft_cape,
            commands::get_mojang_texture_preview,
            commands::relay_start,
            commands::relay_stop,
            commands::relay_list_peers,
            commands::relay_send,
            commands::scan_prism_instances,
            commands::scan_multimc_instances,
            commands::import_launcher_instance,
        ])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title(&crate::channel::current().window_title());
            }
            if let Err(e) = setup_tray(app) {
                eprintln!("Octra zasobnik: {e}");
            }
            let _ = settings::Settings::load();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<AppState>();
                let http = state.http.clone();
                let skins = state.skins.clone();
                if let Err(e) = skins.ensure_started(http.clone()).await {
                    eprintln!("Octra skins: {e}");
                }
                if let Ok((settings, dirs)) = settings::Settings::load() {
                    let _ = skins::ensure_authlib_injector(&http, &dirs).await;
                    state.discord.set_enabled(settings.discord_rpc);
                    if settings.discord_rpc {
                        state.discord.set_idle();
                    }
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() != "main" || QUITTING.load(Ordering::SeqCst) {
                    return;
                }
                let hide = settings::Settings::load()
                    .map(|(s, _)| s.hide_to_tray)
                    .unwrap_or(true);
                if hide {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Octra");
}
