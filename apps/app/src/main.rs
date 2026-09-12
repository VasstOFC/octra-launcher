#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![recursion_limit = "256"]

use native_dialog::{DialogBuilder, MessageLevel};
use std::env;
use std::sync::atomic::Ordering;
use tauri::{Emitter, Listener, Manager};
use tauri_plugin_fs::FsExt;
use theseus::prelude::*;

mod api;
mod error;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(feature = "updater")]
mod updater_impl;
#[cfg(not(feature = "updater"))]
mod updater_impl_noop;

// Should be called in launcher initialization
#[tracing::instrument(skip_all)]
#[tauri::command]
async fn initialize_state(
    app: tauri::AppHandle,
    events: tauri::ipc::Channel<tauri::ipc::InvokeResponseBody>,
) -> api::Result<()> {
    #[cfg(all(windows, not(debug_assertions)))]
    steer_nsis_updates_to_current_dir();

    tracing::info!("Initializing app event state...");
    theseus::EventState::init(app.clone(), events).await?;

    tracing::info!("Initializing app state...");
    State::init(app.config().identifier.clone()).await?;

    let state = State::get().await?;
    app.asset_protocol_scope()
        .allow_directory(state.directories.caches_dir(), true)?;
    app.asset_protocol_scope()
        .allow_directory(state.directories.caches_dir().join("icons"), true)?;
    app.fs_scope()
        .allow_directory(state.directories.instances_dir(), true)?;

    if let Some(lumen_dir) = theseus::lumen_sync::octra_launcher_dir() {
        let lumen_instances = lumen_dir.join("instances");
        let _ = app.fs_scope().allow_directory(&lumen_instances, true);
        let _ = app
            .asset_protocol_scope()
            .allow_directory(&lumen_instances, true);
    }

    Ok(())
}

// Should be call once Vue has mounted the app
#[tracing::instrument(skip_all)]
#[tauri::command]
fn show_window(app: tauri::AppHandle) {
    let win = app.get_window("main").unwrap();

    if let Ok(size) = win.outer_size() {
        if size.width < 200 || size.height < 200 {
            tracing::warn!(
                "main window size {size:?} is too small; resetting to 1280x800"
            );
            let _ = win.set_size(tauri::PhysicalSize::new(1280, 800));
            let _ = win.center();
        }
    }

    if let Err(e) = win.show() {
        DialogBuilder::message()
            .set_level(MessageLevel::Error)
            .set_title("Initialization error")
            .set_text(format!(
                "Cannot display application window due to an error:\n{e}"
            ))
            .alert()
            .show()
            .unwrap();
        panic!("cannot display application window")
    } else {
        let _ = win.set_focus();
    }
}

#[tauri::command]
fn is_dev() -> bool {
    cfg!(debug_assertions)
}

#[tauri::command]
fn are_updates_enabled() -> bool {
    cfg!(feature = "updater")
        && env::var("LUMEN_EXTERNAL_UPDATE_PROVIDER").is_err()
}

#[cfg(feature = "updater")]
pub use updater_impl::*;

#[cfg(not(feature = "updater"))]
pub use updater_impl_noop::*;

// Toggles decorations
#[tauri::command]
async fn toggle_decorations(b: bool, window: tauri::Window) -> api::Result<()> {
    window.set_decorations(b).map_err(|e| {
        theseus::Error::from(theseus::ErrorKind::OtherError(format!(
            "Failed to toggle decorations: {e}"
        )))
    })?;
    Ok(())
}

#[tauri::command]
fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

fn current_exe_dir() -> Result<std::path::PathBuf, theseus::Error> {
    let exe = std::env::current_exe().map_err(|e| {
        theseus::Error::from(theseus::ErrorKind::OtherError(format!(
            "Failed to locate application executable: {e}"
        )))
    })?;
    exe.parent()
        .map(|dir| dir.to_path_buf())
        .ok_or_else(|| {
            theseus::Error::from(theseus::ErrorKind::OtherError(
                "Application executable has no parent directory".to_string(),
            ))
        })
}

#[tauri::command]
async fn app_install_dir() -> api::Result<String> {
    Ok(current_exe_dir()?.to_string_lossy().to_string())
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SetupUpdateInfo {
    url: String,
    size: Option<u64>,
}

/// Sprawdza, czy dla danej wersji istnieje instalator custom (Lumen Setup)
/// z wbudowanym payloadem. Zwraca URL i rozmiar albo None (fallback na NSIS).
fn http_error(context: &str, error: impl std::fmt::Display) -> theseus::Error {
    theseus::Error::from(theseus::ErrorKind::OtherError(format!("{context}: {error}")))
}

#[tauri::command]
async fn check_setup_update(version: String) -> api::Result<Option<SetupUpdateInfo>> {
    use tauri_plugin_http::reqwest::header::{HeaderValue, ACCEPT};
    use tauri_plugin_http::reqwest::{ClientBuilder, Url};

    let url_string = format!(
        "https://github.com/VasstOFC/octra-launcher/releases/download/v{version}/Lumen-setup.exe"
    );
    let url: Url = url_string
        .parse()
        .map_err(|e| http_error("Invalid setup update URL", e))?;
    let client = ClientBuilder::new()
        .user_agent(theseus::launcher_user_agent())
        .build()
        .map_err(|e| http_error("Failed to build HTTP client", e))?;
    let response = client
        .head(url.clone())
        .header(ACCEPT, HeaderValue::from_static("application/octet-stream"))
        .send()
        .await
        .map_err(|e| http_error("Setup update check request failed", e))?;
    if !response.status().is_success() {
        return Ok(None);
    }
    let size = response
        .headers()
        .get("Content-Length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok());
    Ok(Some(SetupUpdateInfo {
        url: url_string,
        size,
    }))
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SetupDownloadProgress {
    downloaded: u64,
    total: Option<u64>,
    version: String,
}

#[tauri::command]
async fn download_setup_update(
    app: tauri::AppHandle,
    url: String,
    version: String,
) -> api::Result<String> {
    use tauri_plugin_http::reqwest::{ClientBuilder, Url};
    use tokio::io::AsyncWriteExt;

    let dest = std::env::temp_dir().join(format!("Lumen-Setup-{version}.exe"));
    let client = ClientBuilder::new()
        .user_agent(theseus::launcher_user_agent())
        .build()
        .map_err(|e| http_error("Failed to build HTTP client", e))?;
    let download_url: Url = url
        .parse()
        .map_err(|e| http_error("Invalid setup download URL", e))?;
    let mut response = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| http_error("Setup download request failed", e))?;
    if !response.status().is_success() {
        return Err(theseus::Error::from(theseus::ErrorKind::OtherError(format!(
            "Setup download failed with status: {}",
            response.status()
        )))
        .into());
    }
    let total = response.content_length();
    let mut file = tokio::fs::File::create(&dest).await.map_err(|e| {
        theseus::Error::from(theseus::ErrorKind::OtherError(format!(
            "Failed to create temporary setup file: {e}"
        )))
    })?;
    let mut downloaded: u64 = 0;
    loop {
        match response
            .chunk()
            .await
            .map_err(|e| http_error("Setup download failed", e))? {
            Some(chunk) => {
                file.write_all(&chunk).await.map_err(|e| {
                    theseus::Error::from(theseus::ErrorKind::OtherError(format!(
                        "Failed to write setup file: {e}"
                    )))
                })?;
                downloaded += chunk.len() as u64;
                let _ = app.emit(
                    "setup-download-progress",
                    SetupDownloadProgress {
                        downloaded,
                        total,
                        version: version.clone(),
                    },
                );
            }
            None => break,
        }
    }
    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
fn launch_installer_update(setup_path: String) -> api::Result<()> {
    use std::process::Command;

    let setup = std::path::PathBuf::from(&setup_path);
    if !setup.is_file() {
        return Err(theseus::Error::from(theseus::ErrorKind::OtherError(format!(
            "Update installer not found at {setup_path}"
        )))
        .into());
    }
    let exe_dir = current_exe_dir()?;
    Command::new(&setup)
        .arg("--update")
        .arg(exe_dir)
        .spawn()
        .map_err(|e| {
            theseus::Error::from(theseus::ErrorKind::OtherError(format!(
                "Failed to launch update installer: {e}"
            )))
        })?;
    Ok(())
}

/// Zapewnia, że przyszłe aktualizacje NSIS instalują się w katalogu,
/// z którego działa aplikacja. Szablon NSIS Tauri odczytuje tę wartość
/// (`RestorePreviousInstallLocation`), więc update podmienia działający exe
/// zamiast tworzyć drugą kopię. Idempotentne (zapis tylko przy zmianie),
/// pomijane w buildach dev, żeby nie kierować updatera do katalogów `target/`.
/// Klucz musi odpowiadać stałej NSIS_RESTORE_KEY w instalatorze.
#[cfg(all(windows, not(debug_assertions)))]
fn steer_nsis_updates_to_current_dir() {
    use winreg::{enums::*, RegKey};

    const NSIS_RESTORE_KEY: &str = r"Software\OctraApp\Lumen App";

    let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.to_path_buf()))
    else {
        return;
    };
    let Ok((key, _)) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(NSIS_RESTORE_KEY) else {
        return;
    };
    let current: String = key.get_value("").unwrap_or_default();
    if current.to_lowercase() != exe_dir.to_string_lossy().to_lowercase() {
        let _ = key.set_value("", &exe_dir.to_string_lossy().to_string());
    }
}

#[tauri::command]
async fn set_restart_after_pending_update(
    should_restart: bool,
) -> api::Result<()> {
    let state = State::get().await?;
    state
        .restart_after_pending_update
        .store(should_restart, Ordering::Relaxed);
    Ok(())
}

// if Tauri app is called with arguments, then those arguments will be treated as commands
// ie: deep links or filepaths for .mrpacks
fn main() {
    #[cfg(feature = "export-app-events")]
    theseus::export_app_event_bindings(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../app-frontend/src/generated/app-events"),
    )
    .expect("failed to export app event TypeScript bindings");

    /*
        tracing is set basd on the environment variable RUST_LOG=xxx, depending on the amount of logs to show
            ERROR > WARN > INFO > DEBUG > TRACE
        eg. RUST_LOG=info will show info, warn, and error logs
            RUST_LOG="theseus=trace" will show *all* messages but from theseus only (and not dependencies using similar crates)
            RUST_LOG="theseus=trace" will show *all* messages but from theseus only (and not dependencies using similar crates)

        Error messages returned to Tauri will display as traced error logs if they return an error.
        This will also include an attached span trace if the error is from a tracing error, and the level is set to info, debug, or trace

        on unix:
            RUST_LOG="theseus=trace" {run command}

    */

    let tauri_context = tauri::generate_context!();

    let _log_guard = theseus::start_logger(&tauri_context.config().identifier);

    tracing::info!("Initialized tracing subscriber. Loading Lumen App!");

    let mut builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    {
        builder = builder
            .menu(|app| macos::menu::create(app))
            .on_menu_event(macos::menu::handle_event);
    }

    #[cfg(feature = "updater")]
    {
        use tauri_plugin_http::reqwest::header::{HeaderValue, USER_AGENT};
        use theseus::launcher_user_agent;
        builder = builder.plugin(
            tauri_plugin_updater::Builder::new()
                .header(
                    USER_AGENT,
                    HeaderValue::from_str(&launcher_user_agent()).unwrap(),
                )
                .unwrap()
                .build(),
        );
    }

    builder = builder
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(payload) = args.get(1) {
                tracing::info!("Handling command-line deep link");
                let payload = payload.clone();
                tauri::async_runtime::spawn(api::utils::handle_command(
                    payload,
                ));
            }

            if let Some(win) = app.get_window("main") {
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_filename("app-window-state.json")
                .with_denylist(&["signin"])
                // Use *only* POSITION and SIZE state flags, because saving VISIBLE causes the `visible: false` to not take effect
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::MAXIMIZED,
                )
                .build(),
        )
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                let payload = macos::deep_link::get_or_init_payload(app);

                let mtx_copy = payload.payload;
                app.listen("deep-link://new-url", move |url| {
                    let mtx_copy_copy = mtx_copy.clone();
                    let request = url.payload().to_owned();

                    let actual_request =
                        serde_json::from_str::<Vec<String>>(&request)
                            .ok()
                            .map(|mut x| x.remove(0))
                            .unwrap_or(request);

                    tauri::async_runtime::spawn(async move {
                        tracing::info!("Handling macOS deep link");

                        let mut payload = mtx_copy_copy.lock().await;
                        if payload.is_none() {
                            *payload = Some(actual_request.clone());
                        }

                        let _ =
                            api::utils::handle_command(actual_request).await;
                    });
                });
            };

            #[cfg(not(target_os = "macos"))]
            app.listen("deep-link://new-url", |url| {
                let payload = url.payload().to_owned();
                tracing::info!("Handling deep link");
                tauri::async_runtime::spawn(api::utils::handle_command(
                    payload,
                ));
            });

            #[cfg(not(target_os = "linux"))]
            if let Some(window) = app.get_window("main")
                && let Err(e) = window.set_shadow(true)
            {
                tracing::warn!("Failed to set window shadow: {e}");
            }

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(8)).await;
                let Some(win) = handle.get_window("main") else {
                    return;
                };
                if win.is_visible().unwrap_or(true) {
                    return;
                }
                tracing::warn!(
                    "frontend did not show the main window in time; showing from rust fallback"
                );
                show_window(handle);
            });

            Ok(())
        });

    builder = builder
        .plugin(api::auth::init())
        .plugin(api::mr_auth::init())
        .plugin(api::onboarding_checklist::init())
        .plugin(api::import::init())
        .plugin(api::install::init())
        .plugin(api::instance::init())
        .plugin(api::logs::init())
        .plugin(api::jre::init())
        .plugin(api::metadata::init())
        .plugin(api::minecraft_skins::init())
        .plugin(api::process::init())
        .plugin(api::reports::init())
        .plugin(api::settings::init())
        .plugin(api::shortcuts::init())
        .plugin(api::tags::init())
        .plugin(api::users::init())
        .plugin(api::utils::init())
        .plugin(api::cache::init())
        .plugin(api::files::init())
        .plugin(api::ads::init())
        .plugin(api::friends::init())
        .plugin(api::worlds::init())
        .plugin(api::lumen::init())
        .manage(PendingUpdateData::default())
        .invoke_handler(tauri::generate_handler![
            initialize_state,
            is_dev,
            are_updates_enabled,
            get_update_size,
            enqueue_update_for_installation,
            remove_enqueued_update,
            set_restart_after_pending_update,
            app_install_dir,
            check_setup_update,
            download_setup_update,
            launch_installer_update,
            toggle_decorations,
            show_window,
            restart_app,
        ]);

    tracing::info!("Initializing app...");
    let app = builder.build(tauri_context);

    match app {
        Ok(app) => {
            app.run(|app, event| {
                #[cfg(not(any(feature = "updater", target_os = "macos")))]
                let _ = app;

                if matches!(&event, tauri::RunEvent::ExitRequested { .. })
                    && let Err(error) = tauri::async_runtime::block_on(
                        theseus::minecraft_skins::flush_pending_skin_change(),
                    )
                {
                    tracing::warn!(
                        "Failed to flush pending Minecraft skin change before exit: {error}"
                    );
                }

                #[cfg(feature = "updater")]
                if matches!(&event, tauri::RunEvent::Exit) {
                    let update_data = app.state::<PendingUpdateData>().inner();
                    let should_restart = State::get_if_initialized()
                        .map(|s| {
                            s.restart_after_pending_update.load(Ordering::Relaxed)
                        })
                        .unwrap_or(false);
                    if let Some((update, data)) = &*update_data.0.lock().unwrap()
                    {
                        fn set_changelog_toast(version: Option<String>) {
                            let toast_result: theseus::Result<()> = tauri::async_runtime::block_on(async move {
                                let mut settings = settings::get().await?;
                                settings.pending_update_toast_for_version = version;
                                settings::set(settings).await?;
                                Ok(())
                            });
                            if let Err(e) = toast_result {
                                tracing::warn!(
                                    "Failed to set pending_update_toast: {e}"
                                )
                            }
                        }

                        set_changelog_toast(Some(update.version.clone()));
                        let update = if should_restart {
                            (**update).clone()
                        } else {
                            (**update).clone().restart_after_install(false)
                        };
                        match update.install(data) {
                            Ok(()) => {
                                if should_restart {
                                    tracing::info!(
                                        "Pending update installed successfully (version {}); restarting because user requested reload",
                                        update.version
                                    );
                                    app.restart();
                                } else {
                                    tracing::info!(
                                        "Pending update installed successfully (version {}); exiting without relaunch (user did not request reload)",
                                        update.version
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Pending update install failed (version {}): {e}",
                                    update.version
                                );
                                set_changelog_toast(None);

                                DialogBuilder::message()
                                    .set_level(MessageLevel::Error)
                                    .set_title("Update error")
                                    .set_text(format!("Failed to install update due to an error:\n{e}"))
                                    .alert()
                                    .show()
                                    .unwrap();
                            }
                        }
                    }
                }
                #[cfg(target_os = "macos")]
                if let tauri::RunEvent::Opened { urls } = event {
                    tracing::info!("Handling webview open {urls:?}");

                    let file = urls
                        .into_iter()
                        .find_map(|url| url.to_file_path().ok());

                    if let Some(file) = file {
                        let payload =
                            macos::deep_link::get_or_init_payload(app);

                        let mtx_copy = payload.payload;
                        let request = file.to_string_lossy().to_string();
                        tauri::async_runtime::spawn(async move {
                            let mut payload = mtx_copy.lock().await;
                            if payload.is_none() {
                                *payload = Some(request.clone());
                            }

                            let _ = api::utils::handle_command(request).await;
                        });
                    }
                }
            });
        }
        Err(e) => {
            tracing::error!("Error while running tauri application: {:?}", e);

            #[cfg(target_os = "windows")]
            {
                // tauri doesn't expose runtime errors, so matching a string representation seems like the only solution
                if format!("{e:?}").contains(
                    "Runtime(CreateWebview(WebView2Error(WindowsError",
                ) {
                    DialogBuilder::message()
                        .set_level(MessageLevel::Error)
                        .set_title("Initialization error")
                        .set_text("Your Microsoft Edge WebView2 installation is corrupt.\n\nMicrosoft Edge WebView2 is required to run Lumen App.\n\nInstall or repair WebView2 from Microsoft, then try again.")
                        .alert()
                        .show()
                        .unwrap();

                    panic!("webview2 initialization failed")
                }
            }

            DialogBuilder::message()
                .set_level(MessageLevel::Error)
                .set_title("Initialization error")
                .set_text(format!(
                    "Cannot initialize application due to an error:\n{e:?}"
                ))
                .alert()
                .show()
                .unwrap();

            panic!("{1}: {:?}", e, "error while running tauri application")
        }
    }
}
