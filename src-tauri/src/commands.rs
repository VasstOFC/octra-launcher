use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio_util::sync::CancellationToken;

use crate::auth::{self, AccountsFile, DeviceCode};
use crate::cf_instance;
use crate::curseforge;
use crate::download::{self, download_file};
use crate::error::{Error, Result};
use crate::github_release::{self, GithubReleaseCheck};
use crate::icon;
use crate::install::{self, emit_install_cleared, emit_progress, InstallProgress};
use crate::instances::{self, ContentKind, CreateInstance, Instance, Loader};
use crate::java::{self, JavaRuntime};
use crate::launch;
use crate::loaders;
use crate::local_server::{self, CreateLocalServer, LocalServerInfo, UpdateLocalServer};
use crate::meta::{self, ManifestVersion};
use crate::mrpack;
use crate::import_launchers;
use crate::paths::Dirs;
use crate::relay;
use crate::servers::{self, ServerEntry};
use crate::server_ping;
use crate::settings::Settings;
use crate::AppState;

fn ctx() -> Result<(Settings, Dirs)> {
    Settings::load()
}

#[tauri::command]
pub fn get_settings() -> Result<Settings> {
    Ok(ctx()?.0)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<Settings> {
    let dirs = Dirs::resolve(&settings);
    dirs.ensure()?;
    settings.save(&dirs)?;
    if let Some(state) = app.try_state::<AppState>() {
        state.discord.set_enabled(settings.discord_rpc);
    }
    Ok(settings)
}

#[tauri::command]
pub fn get_data_dir() -> Result<String> {
    Ok(ctx()?.1.root.to_string_lossy().to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub channel: String,
    pub display_name: String,
    pub data_dir: String,
    pub updates_enabled: bool,
}

#[tauri::command]
pub fn get_app_info() -> Result<AppInfo> {
    let ch = crate::channel::current();
    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION").into(),
        channel: ch.as_str().into(),
        display_name: ch.display_name().into(),
        data_dir: ctx()?.1.root.to_string_lossy().to_string(),
        updates_enabled: ch.is_stable(),
    })
}

#[tauri::command]
pub fn system_memory_mb() -> u64 {
    java::system_memory_mb()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaStatus {
    pub runtimes: Vec<JavaRuntime>,
    pub memory_mb: u64,
}

#[tauri::command]
pub fn scan_java() -> Result<JavaStatus> {
    let (settings, dirs) = ctx()?;
    Ok(JavaStatus {
        runtimes: java::scan(&dirs, &settings),
        memory_mb: java::system_memory_mb(),
    })
}

#[tauri::command]
pub async fn download_java(app: AppHandle, state: State<'_, AppState>, major: u32) -> Result<JavaRuntime> {
    let (settings, dirs) = ctx()?;
    let _ = settings;
    let client = state.http.clone();
    java::download_temurin(&client, &dirs, major, |msg| {
        emit_progress(
            &app,
            InstallProgress {
                instance_id: String::new(),
                stage: "java".into(),
                current: 0,
                total: 1,
                file: None,
                message: msg.to_string(),
            },
        );
    })
    .await
}

#[tauri::command]
pub fn list_instances() -> Result<Vec<Instance>> {
    let (_, dirs) = ctx()?;
    instances::list(&dirs)
}

#[tauri::command]
pub fn get_instance(id: String) -> Result<Instance> {
    let (_, dirs) = ctx()?;
    instances::get(&dirs, &id)
}

#[tauri::command]
pub fn read_instance_icon(id: String) -> Result<Option<String>> {
    let (_, dirs) = ctx()?;
    let inst = instances::get(&dirs, &id)?;
    Ok(icon::read_data_url(&dirs, &inst))
}

#[tauri::command]
pub fn create_instance(req: CreateInstance) -> Result<Instance> {
    let (settings, dirs) = ctx()?;
    instances::create(&dirs, &settings, req)
}

#[tauri::command]
pub fn update_instance(inst: Instance) -> Result<Instance> {
    let (_, dirs) = ctx()?;
    let existing = instances::get(&dirs, &inst.id)?;
    let mut next = inst;
    instances::merge_pack_link(&existing, &mut next);
    instances::ensure_icon_unlocked(&existing, &next)?;
    instances::merge_icon(&dirs, &existing, &mut next);
    if next.wallpaper_path.is_none() {
        next.wallpaper_path = existing.wallpaper_path.clone();
    }
    next.version_id = instances::compute_version_id(&CreateInstance {
        name: next.name.clone(),
        game_version: next.game_version.clone(),
        loader: next.loader,
        loader_version: next.loader_version.clone(),
        memory_max_mb: Some(next.memory_max_mb),
    });
    instances::save(&dirs, &next)?;
    Ok(next)
}

#[tauri::command]
pub fn delete_instance(id: String) -> Result<()> {
    let (_, dirs) = ctx()?;
    instances::delete(&dirs, &id)
}

#[tauri::command]
pub fn unlink_instance_pack(id: String) -> Result<Instance> {
    let (_, dirs) = ctx()?;
    instances::unlink_pack(&dirs, &id)
}

#[tauri::command]
pub async fn resync_instance_pack(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    path: Option<String>,
) -> Result<Instance> {
    let result = resync_instance_pack_inner(&app, &state, &id, path.as_deref()).await;
    if result.is_err() {
        emit_install_cleared(&app);
    }
    result
}

async fn resync_instance_pack_inner(
    app: &AppHandle,
    state: &State<'_, AppState>,
    id: &str,
    path: Option<&str>,
) -> Result<Instance> {
    if launch::instance_has_running(&state.running.lock(), id) {
        return Err(Error::msg(
            "Zatrzymaj grę, zanim zsynchronizujesz paczkę.",
        ));
    }
    let (settings, dirs) = ctx()?;
    let mut inst = instances::get(&dirs, id)?;
    let linked = inst
        .linked_pack
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::msg("Ta instancja nie pochodzi z paczki."))?
        .to_string();

    emit_progress(
        app,
        InstallProgress {
            instance_id: inst.id.clone(),
            stage: "modpack".into(),
            current: 0,
            total: 1,
            file: None,
            message: "Przygotowanie synchronizacji paczki…".into(),
        },
    );

    let pack = resolve_resync_pack(app, &state.http, &dirs, &settings, &linked, path).await?;
    let format = detect_pack_format(&pack)?;
    let cancel = CancellationToken::new();
    {
        let mut g = state.install_cancel.lock();
        if let Some(old) = g.take() {
            old.cancel();
        }
        *g = Some(cancel.clone());
    }
    let req = match format {
        PackFormat::Mrpack => {
            mrpack::import_mrpack(&state.http, app, &dirs, &settings, &pack, &cancel).await?
        }
        PackFormat::CurseForge => {
            curseforge::import_curseforge(&state.http, app, &pack, &cancel).await?
        }
    };
    instances::clear_mods_dir(&dirs, &inst.id)?;
    instances::apply_pack_profile(&mut inst, &req);
    inst.linked_pack = Some(linked.clone());
    instances::save(&dirs, &inst)?;
    mrpack::apply_pack_icon(&state.http, &dirs, &mut inst, &pack, &linked, None).await;
    match format {
        PackFormat::Mrpack => {
            mrpack::populate_instance_from_pack(
                &state.http, app, &dirs, &inst, &pack, &cancel,
            )
            .await?;
        }
        PackFormat::CurseForge => {
            curseforge::populate_instance_from_pack(
                &state.http, app, &dirs, &inst, &pack, &cancel,
            )
            .await?;
        }
    }
    mrpack::adopt_extracted_icon(&dirs, &mut inst);
    run_install(app.clone(), state.inner(), inst.id.clone()).await
}

async fn resolve_resync_pack(
    app: &AppHandle,
    client: &reqwest::Client,
    dirs: &Dirs,
    settings: &Settings,
    linked: &str,
    path: Option<&str>,
) -> Result<PathBuf> {
    if let Some(p) = path.map(str::trim).filter(|s| !s.is_empty()) {
        let pb = PathBuf::from(p);
        if !pb.is_file() {
            return Err(Error::msg("Nie znaleziono wskazanego pliku paczki."));
        }
        return Ok(pb);
    }
    if mrpack::is_catalog_pack_slug(linked) {
        emit_progress(
            app,
            InstallProgress {
                instance_id: String::new(),
                stage: "modpack".into(),
                current: 0,
                total: 1,
                file: None,
                message: format!("Pobieranie paczki {linked}…"),
            },
        );
        return mrpack::download_modrinth_mrpack(client, dirs, linked, None).await;
    }
    let featured = settings.featured_pack_query();
    if !featured.is_empty() {
        if let Ok(found) = resolve_pack_file(app, client, dirs, &featured).await {
            return Ok(found);
        }
    }
    emit_install_cleared(app);
    let kind = if linked.eq_ignore_ascii_case("curseforge") {
        Some("zip")
    } else {
        Some("mrpack")
    };
    let picked = mrpack::pick_mrpack_file(kind).await.ok_or_else(|| {
        Error::msg("Wskaż plik paczki, żeby zsynchronizować instancję.")
    })?;
    Ok(PathBuf::from(picked))
}

#[tauri::command]
pub fn set_instance_icon_glyph(id: String, color: String, symbol: String) -> Result<Instance> {
    let (_, dirs) = ctx()?;
    instances::apply_glyph_icon(&dirs, &id, &color, &symbol)
}

#[tauri::command]
pub fn set_instance_icon_bytes(id: String, bytes: Vec<u8>) -> Result<Instance> {
    let (_, dirs) = ctx()?;
    instances::apply_icon_bytes(&dirs, &id, &bytes)
}

#[tauri::command]
pub async fn pick_instance_icon_file(id: String) -> Result<Option<Instance>> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("Obraz", &["png", "jpg", "jpeg", "webp", "gif"])
        .set_title("Wybierz ikonę instancji")
        .pick_file()
        .await;
    let Some(file) = file else {
        return Ok(None);
    };
    let (_, dirs) = ctx()?;
    Ok(Some(instances::apply_file_icon(
        &dirs,
        &id,
        file.path(),
    )?))
}

#[tauri::command]
pub async fn pick_profile_wallpaper(id: String) -> Result<Option<Instance>> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("Obraz", &["png", "jpg", "jpeg", "webp"])
        .set_title("Wybierz tapetę profilu")
        .pick_file()
        .await;
    let Some(file) = file else {
        return Ok(None);
    };
    let (_, dirs) = ctx()?;
    Ok(Some(instances::apply_profile_wallpaper(
        &dirs,
        &id,
        file.path(),
    )?))
}

#[tauri::command]
pub fn set_profile_wallpaper(id: String, path: String) -> Result<Instance> {
    let (_, dirs) = ctx()?;
    instances::apply_profile_wallpaper(&dirs, &id, Path::new(&path))
}

#[tauri::command]
pub fn set_profile_wallpaper_bytes(id: String, bytes: Vec<u8>) -> Result<Instance> {
    let (_, dirs) = ctx()?;
    instances::apply_profile_wallpaper_bytes(&dirs, &id, &bytes)
}

#[tauri::command]
pub fn clear_profile_wallpaper(id: String) -> Result<Instance> {
    let (_, dirs) = ctx()?;
    instances::clear_profile_wallpaper(&dirs, &id)
}

#[tauri::command]
pub fn read_instance_wallpaper(id: String) -> Result<Option<String>> {
    let (_, dirs) = ctx()?;
    let inst = instances::get(&dirs, &id)?;
    Ok(instances::read_wallpaper_thumb_path(&dirs, &inst))
}

#[tauri::command]
pub fn list_instance_content(id: String) -> Result<Vec<instances::ContentFile>> {
    let (_, dirs) = ctx()?;
    instances::list_content(&dirs, &id)
}

#[tauri::command]
pub fn toggle_instance_content(
    id: String,
    kind: ContentKind,
    name: String,
) -> Result<Vec<instances::ContentFile>> {
    let (_, dirs) = ctx()?;
    instances::toggle_content(&dirs, &id, kind, &name)
}

#[tauri::command]
pub fn delete_instance_content(
    id: String,
    kind: ContentKind,
    name: String,
) -> Result<Vec<instances::ContentFile>> {
    let (_, dirs) = ctx()?;
    instances::delete_content(&dirs, &id, kind, &name)
}

#[tauri::command]
pub fn import_local_content(
    id: String,
    kind: ContentKind,
    path: String,
) -> Result<Vec<instances::ContentFile>> {
    let (_, dirs) = ctx()?;
    instances::import_local_content(&dirs, &id, kind, std::path::Path::new(&path))
}

#[tauri::command]
pub async fn fetch_minecraft_versions(
    state: State<'_, AppState>,
    include_all: Option<bool>,
) -> Result<Vec<ManifestVersion>> {
    let (settings, dirs) = ctx()?;
    let manifest = install::fetch_manifest(&state.http, &dirs).await?;
    let versions = if include_all.unwrap_or(false) {
        manifest.versions
    } else if settings.show_snapshots {
        manifest.versions
    } else {
        manifest
            .versions
            .into_iter()
            .filter(|v| v.version_type == "release")
            .collect()
    };
    Ok(versions)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderVersions {
    pub versions: Vec<String>,
    pub recommended: Option<String>,
}

#[tauri::command]
pub async fn fetch_loader_versions(
    state: State<'_, AppState>,
    loader: Loader,
    game_version: String,
) -> Result<LoaderVersions> {
    match loader {
        Loader::Vanilla => Ok(LoaderVersions {
            versions: vec![],
            recommended: None,
        }),
        Loader::Fabric => {
            let v = loaders::fabric::list_loaders(&state.http, &game_version).await?;
            let recommended = v.first().cloned();
            Ok(LoaderVersions {
                versions: v,
                recommended,
            })
        }
        Loader::Quilt => {
            let v = loaders::quilt::list_loaders(&state.http, &game_version).await?;
            let recommended = v.first().cloned();
            Ok(LoaderVersions {
                versions: v,
                recommended,
            })
        }
        Loader::Forge => {
            let v = loaders::forge::list_forge(&state.http, &game_version).await?;
            let recommended = v.iter().find(|x| x.recommended).map(|x| x.version.clone());
            Ok(LoaderVersions {
                versions: v.into_iter().map(|x| x.version).collect(),
                recommended,
            })
        }
        Loader::Neoforge => {
            let v = loaders::forge::list_neoforge(&state.http, &game_version).await?;
            let recommended = v.iter().find(|x| x.recommended).map(|x| x.version.clone());
            Ok(LoaderVersions {
                versions: v.into_iter().map(|x| x.version).collect(),
                recommended,
            })
        }
    }
}

#[tauri::command]
pub async fn fetch_mojang_news(state: State<'_, AppState>) -> Result<Vec<crate::news::MojangNewsItem>> {
    crate::news::fetch_news(&state.http).await
}

#[tauri::command]
pub fn get_accounts() -> Result<AccountsFile> {
    let (_, dirs) = ctx()?;
    auth::load_accounts(&dirs)
}

#[tauri::command]
pub fn account_has_token(uuid: String) -> Result<bool> {
    let (_, dirs) = ctx()?;
    let file = auth::load_accounts(&dirs)?;
    let account = file
        .accounts
        .iter()
        .find(|a| a.uuid == uuid)
        .ok_or_else(|| Error::msg("Nie znaleziono konta."))?;
    Ok(auth::has_refresh_token(&dirs, account))
}

#[tauri::command]
pub fn set_active_account(uuid: String) -> Result<AccountsFile> {
    let (_, dirs) = ctx()?;
    auth::set_active(&dirs, &uuid)
}

#[tauri::command]
pub fn logout_account(uuid: String) -> Result<AccountsFile> {
    let (_, dirs) = ctx()?;
    auth::logout(&dirs, &uuid)
}

#[tauri::command]
pub fn add_offline_account(name: String) -> Result<auth::Account> {
    let (_, dirs) = ctx()?;
    auth::add_offline_account(&dirs, &name)
}

#[tauri::command]
pub fn get_offline_skin(uuid: String) -> Result<crate::skins::OfflineSkin> {
    let (_, dirs) = ctx()?;
    crate::skins::get_offline_skin(&dirs, &uuid)
}

#[tauri::command]
pub async fn save_offline_skin(
    state: State<'_, AppState>,
    uuid: String,
    png: Vec<u8>,
    model: String,
) -> Result<crate::skins::OfflineSkin> {
    let (_, dirs) = ctx()?;
    let (info, bytes, hash) = crate::skins::save_offline_skin(&dirs, &uuid, &png, &model)?;
    state.skins.put_texture(hash, bytes.clone());
    state.skins.reindex(&dirs);
    state.skins.notify_lan();
    let name = auth::load_accounts(&dirs)
        .ok()
        .and_then(|f| f.accounts.into_iter().find(|a| a.uuid == uuid))
        .map(|a| a.name)
        .unwrap_or_default();
    crate::yggdrasil::push_registry(&state.http, &uuid, &bytes, &model, &name).await;
    if let Ok(instances) = crate::instances::list(&dirs) {
        for inst in instances {
            let game_dir = dirs.game_dir(&inst.id);
            if game_dir.exists() {
                let _ = crate::skin_loader::sync_username_skin_files(&game_dir, &dirs);
            }
        }
    }
    Ok(info)
}

#[tauri::command]
pub fn set_offline_skin_model(
    state: State<'_, AppState>,
    uuid: String,
    model: String,
) -> Result<crate::skins::OfflineSkin> {
    let (_, dirs) = ctx()?;
    let info = crate::skins::set_offline_skin_model(&dirs, &uuid, &model)?;
    state.skins.notify_lan();
    Ok(info)
}

#[tauri::command]
pub fn reset_offline_skin(state: State<'_, AppState>, uuid: String) -> Result<()> {
    let (_, dirs) = ctx()?;
    crate::skins::reset_offline_skin(&dirs, &uuid)?;
    state.skins.notify_lan();
    Ok(())
}

#[tauri::command]
pub fn list_offline_skin_library(uuid: String) -> Result<Vec<crate::skin_library::SkinLibraryEntryView>> {
    let (_, dirs) = ctx()?;
    crate::skin_library::list_offline_library(&dirs, &uuid)
}

#[tauri::command]
pub fn add_offline_skin_library_entry(
    uuid: String,
    png: Vec<u8>,
    model: String,
    name: String,
) -> Result<crate::skin_library::SkinLibraryEntryView> {
    let (_, dirs) = ctx()?;
    crate::skin_library::add_offline_library_entry(&dirs, &uuid, &png, &model, &name)
}

use base64::Engine as _;

#[tauri::command]
pub async fn equip_offline_skin_library_entry(
    state: State<'_, AppState>,
    uuid: String,
    entry_id: String,
) -> Result<crate::skins::OfflineSkin> {
    let (_, dirs) = ctx()?;
    let info = crate::skin_library::equip_offline_library_entry(&dirs, &uuid, &entry_id)?;
    if let Some(bytes) = info
        .png_base64
        .as_ref()
        .and_then(|b| base64::engine::general_purpose::STANDARD.decode(b).ok())
    {
        let hash = crate::skins::sha256_hex(&bytes);
        state.skins.put_texture(hash, bytes.clone());
        state.skins.reindex(&dirs);
        state.skins.notify_lan();
        let name = auth::load_accounts(&dirs)
            .ok()
            .and_then(|f| f.accounts.into_iter().find(|a| a.uuid == uuid))
            .map(|a| a.name)
            .unwrap_or_default();
        crate::yggdrasil::push_registry(&state.http, &uuid, &bytes, &info.model, &name).await;
        if let Ok(instances) = crate::instances::list(&dirs) {
            for inst in instances {
                let game_dir = dirs.game_dir(&inst.id);
                if game_dir.exists() {
                    let _ = crate::skin_loader::sync_username_skin_files(&game_dir, &dirs);
                }
            }
        }
    }
    Ok(info)
}

#[tauri::command]
pub fn delete_offline_skin_library_entry(uuid: String, entry_id: String) -> Result<()> {
    let (_, dirs) = ctx()?;
    crate::skin_library::delete_offline_library_entry(&dirs, &uuid, &entry_id)
}

#[tauri::command]
pub fn set_offline_skin_library_model(
    uuid: String,
    entry_id: String,
    model: String,
) -> Result<crate::skin_library::SkinLibraryEntryView> {
    let (_, dirs) = ctx()?;
    crate::skin_library::set_offline_library_entry_model(&dirs, &uuid, &entry_id, &model)
}

#[tauri::command]
pub fn list_premium_skin_library(uuid: String) -> Result<Vec<crate::skin_library::SkinLibraryEntryView>> {
    let (_, dirs) = ctx()?;
    crate::skin_library::list_premium_library(&dirs, &uuid)
}

#[tauri::command]
pub fn save_premium_skin_library_entry(
    uuid: String,
    req: crate::skin_library::SavePremiumLibraryReq,
) -> Result<crate::skin_library::SkinLibraryEntryView> {
    let (_, dirs) = ctx()?;
    crate::skin_library::save_premium_library_entry(&dirs, &uuid, req)
}

#[tauri::command]
pub fn delete_premium_skin_library_entry(uuid: String, entry_id: String) -> Result<()> {
    let (_, dirs) = ctx()?;
    crate::skin_library::delete_premium_library_entry(&dirs, &uuid, &entry_id)
}

#[tauri::command]
pub fn sync_premium_skin_library_active(
    uuid: String,
    texture_key: Option<String>,
    variant: String,
    png: Option<Vec<u8>>,
) -> Result<Vec<crate::skin_library::SkinLibraryEntryView>> {
    let (_, dirs) = ctx()?;
    crate::skin_library::sync_premium_library_active(
        &dirs,
        &uuid,
        texture_key.as_deref(),
        &variant,
        png.as_deref(),
    )
}

#[tauri::command]
pub fn set_premium_skin_library_active(
    uuid: String,
    entry_id: String,
) -> Result<Vec<crate::skin_library::SkinLibraryEntryView>> {
    let (_, dirs) = ctx()?;
    crate::skin_library::set_premium_library_active(&dirs, &uuid, &entry_id)
}

#[tauri::command]
pub fn find_premium_skin_library_duplicate(
    uuid: String,
    texture_key: Option<String>,
    variant: String,
    cape_id: Option<String>,
    png: Option<Vec<u8>>,
) -> Result<Option<crate::skin_library::SkinLibraryEntryView>> {
    let (_, dirs) = ctx()?;
    Ok(crate::skin_library::find_premium_library_duplicate(
        &dirs,
        &uuid,
        texture_key.as_deref(),
        &variant,
        cape_id.as_deref(),
        png.as_deref(),
    ))
}

#[tauri::command]
pub async fn start_login(app: AppHandle, state: State<'_, AppState>) -> Result<DeviceCode> {
    let (settings, _) = ctx()?;
    let client_id = settings.azure_client_id();
    let code = auth::request_device_code(&state.http, &client_id).await?;
    let cancel = CancellationToken::new();
    {
        let mut g = state.login_cancel.lock();
        if let Some(old) = g.take() {
            old.cancel();
        }
        *g = Some(cancel.clone());
    }
    let http = state.http.clone();
    let device_code = code.device_code.clone();
    let interval = code.interval;
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        let result = auth::poll_device_code(&http, &client_id, &device_code, interval, &cancel).await;
        match result {
            Ok(tokens) => {
                let dirs = match Settings::load() {
                    Ok((_, d)) => d,
                    Err(e) => {
                        let _ = app2.emit("auth-error", e.to_string());
                        return;
                    }
                };
                match auth::complete_login(&http, &dirs, &client_id, tokens).await {
                    Ok(acc) => {
                        let _ = app2.emit("auth-success", acc);
                    }
                    Err(e) => {
                        let _ = app2.emit("auth-error", e.to_string());
                    }
                }
            }
            Err(e) => {
                let _ = app2.emit("auth-error", e.to_string());
            }
        }
    });
    let _ = app.emit("auth-device-code", &code);
    Ok(code)
}

#[tauri::command]
pub fn cancel_login(state: State<'_, AppState>) {
    if let Some(c) = state.login_cancel.lock().take() {
        c.cancel();
    }
}

#[tauri::command]
pub async fn install_instance(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<Instance> {
    run_install(app, state.inner(), id).await
}

async fn run_install(app: AppHandle, state: &AppState, id: String) -> Result<Instance> {
    let (settings, dirs) = ctx()?;
    let inst = instances::get(&dirs, &id)?;
    let cancel = CancellationToken::new();
    {
        let mut g = state.install_cancel.lock();
        if let Some(old) = g.take() {
            old.cancel();
        }
        *g = Some(cancel.clone());
    }
    emit_progress(
        &app,
        InstallProgress {
            instance_id: id.clone(),
            stage: "java".into(),
            current: 0,
            total: 1,
            file: None,
            message: "Przygotowanie Javy".into(),
        },
    );
    // Need vanilla meta for java requirement; fetch lightweight.
    let vanilla = install::load_or_fetch_version_json(&state.http, &dirs, &inst.game_version, None, None)
        .await?;
    let required = meta::required_java(&vanilla);
    let runtimes = java::scan(&dirs, &settings);
    let java_rt = match java::pick(&runtimes, required, &settings) {
        Ok(j) => j,
        Err(_) => {
            java::download_temurin(&state.http, &dirs, required, |msg| {
                emit_progress(
                    &app,
                    InstallProgress {
                        instance_id: id.clone(),
                        stage: "java".into(),
                        current: 0,
                        total: 1,
                        file: None,
                        message: msg.to_string(),
                    },
                );
            })
            .await?
        }
    };
    let version_id = match inst.loader {
        Loader::Vanilla => {
            install::install_vanilla(
                &state.http,
                &app,
                &dirs,
                &id,
                &inst.game_version,
                &cancel,
            )
            .await?;
            inst.game_version.clone()
        }
        Loader::Fabric => {
            let loader = inst
                .loader_version
                .clone()
                .ok_or_else(|| Error::msg("Brak wersji Fabric."))?;
            loaders::fabric::install_fabric(
                &state.http,
                &app,
                &dirs,
                &id,
                &inst.game_version,
                &loader,
                &cancel,
            )
            .await?
        }
        Loader::Quilt => {
            let loader = inst
                .loader_version
                .clone()
                .ok_or_else(|| Error::msg("Brak wersji Quilt."))?;
            loaders::quilt::install_quilt(
                &state.http,
                &app,
                &dirs,
                &id,
                &inst.game_version,
                &loader,
                &cancel,
            )
            .await?
        }
        Loader::Forge | Loader::Neoforge => {
            let loader = inst
                .loader_version
                .clone()
                .ok_or_else(|| Error::msg("Brak wersji loadera."))?;
            loaders::forge::install_modded(
                &state.http,
                &app,
                &dirs,
                &id,
                inst.loader,
                &inst.game_version,
                &loader,
                PathBuf::from(&java_rt.path).as_path(),
                &cancel,
            )
            .await?
        }
    };
    let mut inst = inst;
    inst.version_id = version_id;
    instances::save(&dirs, &inst)?;
    emit_progress(
        &app,
        InstallProgress {
            instance_id: id,
            stage: "done".into(),
            current: 1,
            total: 1,
            file: None,
            message: "Gotowe".into(),
        },
    );
    let _ = app.emit("install-finished", &inst);
    Ok(inst)
}

#[tauri::command]
pub fn cancel_install(state: State<'_, AppState>) {
    if let Some(c) = state.install_cancel.lock().take() {
        c.cancel();
    }
}

#[tauri::command]
pub async fn launch_instance(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<u32> {
    let (settings, dirs) = ctx()?;
    let account = auth::active_account(&dirs)?
        .ok_or_else(|| Error::msg("Dodaj konto (Microsoft albo offline), żeby zagrać."))?;
    let allow_multi = settings.allow_multiple_instances;
    let session_id = if allow_multi {
        uuid::Uuid::new_v4().to_string()
    } else {
        String::new()
    };
    let run_key = if allow_multi {
        launch::running_key_session(&id, &account.uuid, &session_id)
    } else {
        let run_key = launch::running_key(&id, &account.uuid);
        if state.running.lock().contains_key(&run_key) {
            return Err(Error::msg(
                "To konto już gra na tej instancji. Zatrzymaj grę, żeby uruchomić ponownie.",
            ));
        }
        run_key
    };
    let mut inst = instances::get(&dirs, &id)?;
    if !instances::is_installed(&dirs, &inst) {
        inst = run_install(app.clone(), state.inner(), id.clone()).await?;
    }
    if !account.is_offline() && settings.azure_client_id().trim().is_empty() {
        return Err(Error::msg(
            "Brak Azure Client ID w launcherze. Wklej go do src-tauri/src/config.rs i przebuduj aplikację.",
        ));
    }
    let session = auth::session_for_account(&state.http, &settings.azure_client_id(), &dirs, &account).await?;
    state
        .skins
        .ensure_started(state.http.clone())
        .await
        .map_err(|e| Error::msg(format!("Serwer skinów Octra nie wystartował: {e}")))?;
    state
        .skins
        .prefetch_account_skins(&state.http, &dirs)
        .await;
    let mut launch_inst = inst.clone();
    if let Some(ygg_url) = state.skins.ygg_root() {
        if let Err(e) =
            crate::skin_loader::ensure_skin_support(&state.http, &dirs, &mut launch_inst, &ygg_url)
                .await
        {
            eprintln!("Lumen skins: CustomSkinLoader: {e}");
        }
    }
    let resolved = install::resolve_version(&state.http, &dirs, &launch_inst.version_id).await?;
    let game_dir = dirs.game_dir(&inst.id);
    if !allow_multi
        && !launch::instance_has_running(&state.running.lock(), &inst.id)
        && launch::game_dir_has_jvm(&game_dir)
    {
        return Err(Error::msg(
            "Minecraft dla tego profilu wciąż działa w tle (javaw.exe). \
             Kliknij ZATRZYMAJ na ekranie Start albo zamknij javaw.exe w Menedżerze zadań, potem spróbuj ponownie.",
        ));
    }
    let required = meta::required_java(&resolved);
    let runtimes = java::scan(&dirs, &settings);
    let java_rt = if inst.custom_java {
        let path = inst
            .java_path
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| Error::msg("Włączono własną Javę, ale nie podano ścieżki."))?;
        java::probe_java(std::path::Path::new(path))
            .ok_or_else(|| Error::msg("Wybrana Java instancji nie działa."))?
    } else {
        match java::pick(&runtimes, required, &settings) {
            Ok(j) => j,
            Err(_) => java::download_temurin(&state.http, &dirs, required, |_| {}).await?,
        }
    };
    let natives = if launch::instance_has_running(&state.running.lock(), &inst.id) {
        dirs.natives_dir(&inst.id)
    } else {
        launch::extract_natives(&dirs, &launch_inst, &resolved).map_err(|e| {
            Error::msg(format!(
                "Nie udało się przygotować bibliotek natywnych (natives) dla „{}”: {e}",
                inst.name
            ))
        })?
    };
    let (java_path, mut args) = launch::build_command_line(
        PathBuf::from(&java_rt.path).as_path(),
        &dirs,
        &settings,
        &launch_inst,
        &resolved,
        &session,
        &natives,
    )?;
    let needs_lumen_skin = account.is_offline()
        && crate::skins::load_local_skin(&dirs, &account.uuid).is_some();
    let mut lumen_agent = false;
    match crate::skins::ensure_authlib_injector(&state.http, &dirs).await {
        Ok(jar) => {
            if let Some(url) = state.skins.ygg_root() {
                launch::prepend_javaagent(&mut args, &jar, &url);
                lumen_agent = true;
            }
        }
        Err(e) => {
            if needs_lumen_skin {
                return Err(Error::msg(format!(
                    "Nie udało się przygotować authlib-injector (skin offline nie zadziała w grze): {e}"
                )));
            }
            eprintln!("Lumen skins: authlib-injector: {e}");
        }
    }
    if needs_lumen_skin && !lumen_agent {
        return Err(Error::msg(
            "Skin offline jest zapisany, ale gra nie dostała authlib-injector. \
             Uruchom ponownie albo zgłoś błąd — bez tego w grze widać domyślny Steve/Alex.",
        ));
    }
    instances::touch_played(&dirs, &inst.id)?;
    let pid = launch::spawn_game(
        app.clone(),
        dirs,
        settings.clone(),
        launch_inst,
        account.uuid.clone(),
        run_key,
        session_id,
        java_path,
        args,
    )
    .await?;
    if settings.close_on_launch {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.minimize();
        }
    }
    Ok(pid)
}

#[tauri::command]
pub fn stop_instance(app: AppHandle, id: String, account_uuid: Option<String>) -> Result<()> {
    let (_, dirs) = ctx()?;
    let uuid = match account_uuid {
        Some(uuid) if !uuid.trim().is_empty() => uuid,
        _ => auth::active_account(&dirs)?
            .map(|a| a.uuid)
            .ok_or_else(|| Error::msg("Wybierz konto, dla którego chcesz zatrzymać grę."))?,
    };
    launch::stop_games_for_instance_account(&app, &id, &uuid)
}

#[tauri::command]
pub fn read_instance_log(id: String) -> Result<String> {
    let (_, dirs) = ctx()?;
    launch::read_log(&dirs, &id)
}

#[tauri::command]
pub fn instance_game_dir(id: String) -> Result<String> {
    let (_, dirs) = ctx()?;
    Ok(dirs.game_dir(&id).to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_instance_folder(id: String) -> Result<()> {
    let (_, dirs) = ctx()?;
    let path = dirs.game_dir(&id);
    std::fs::create_dir_all(&path)?;
    opener_open(&path)
}

#[tauri::command]
pub fn open_data_dir() -> Result<()> {
    let (_, dirs) = ctx()?;
    opener_open(&dirs.root)
}

#[tauri::command]
pub fn list_servers() -> Result<Vec<ServerEntry>> {
    let (_, dirs) = ctx()?;
    Ok(servers::collect_all(&dirs)?.servers)
}

#[tauri::command]
pub fn save_servers(servers: Vec<ServerEntry>) -> Result<Vec<ServerEntry>> {
    let (_, dirs) = ctx()?;
    Ok(servers::save(&dirs, &servers::ServerList { servers })?.servers)
}

#[tauri::command]
pub fn sync_servers_to_instance(id: String) -> Result<usize> {
    let (_, dirs) = ctx()?;
    servers::collect_all(&dirs)?;
    let game_dir = dirs.game_dir(&id);
    servers::sync_instance(&dirs, &game_dir)
}

#[tauri::command]
pub async fn ping_server(address: String) -> Result<server_ping::ServerPingResult> {
    server_ping::ping_server(&address).await
}

#[tauri::command]
pub async fn pick_mrpack_file(kind: Option<String>) -> Option<String> {
    mrpack::pick_mrpack_file(kind.as_deref()).await
}

#[tauri::command]
pub async fn import_mrpack(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<Instance> {
    import_pack_file(app, state.inner(), PathBuf::from(path), "mrpack", None).await
}

#[tauri::command]
pub async fn import_modrinth_pack(
    app: AppHandle,
    state: State<'_, AppState>,
    query: String,
    icon_url: Option<String>,
) -> Result<Instance> {
    let (settings, dirs) = ctx()?;
    let (slug, version) = mrpack::parse_modrinth_query(&query)?;
    emit_progress(
        &app,
        InstallProgress {
            instance_id: String::new(),
            stage: "modpack".into(),
            current: 0,
            total: 1,
            file: None,
            message: format!("Pobieranie modpacka {slug}…"),
        },
    );
    let pack = mrpack::download_modrinth_mrpack(
        &state.http,
        &dirs,
        &slug,
        version.as_deref(),
    )
    .await?;
    let _ = settings;
    import_pack_file(
        app,
        state.inner(),
        pack,
        &slug,
        icon_url.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn search_modrinth_packs(
    state: State<'_, AppState>,
    query: Option<String>,
    offset: Option<u32>,
    limit: Option<u32>,
    sort: Option<String>,
) -> Result<mrpack::ModrinthSearchResult> {
    mrpack::search_modrinth_packs(
        &state.http,
        query.as_deref().unwrap_or(""),
        offset.unwrap_or(0),
        limit.unwrap_or_else(mrpack::search_page_size),
        sort.as_deref().unwrap_or("downloads"),
    )
    .await
}

#[tauri::command]
pub async fn get_modrinth_project(
    state: State<'_, AppState>,
    slug: String,
) -> Result<mrpack::ModrinthProjectDetail> {
    mrpack::get_modrinth_project(&state.http, &slug).await
}

#[tauri::command]
pub async fn get_modrinth_pack_versions(
    state: State<'_, AppState>,
    slug: String,
) -> Result<Vec<mrpack::ModrinthPackVersionHit>> {
    mrpack::get_modrinth_pack_versions(&state.http, &slug).await
}

#[tauri::command]
pub async fn search_modrinth_content(
    state: State<'_, AppState>,
    query: Option<String>,
    offset: Option<u32>,
    limit: Option<u32>,
    sort: Option<String>,
    project_type: String,
    game_version: Option<String>,
    loader: Option<String>,
) -> Result<mrpack::ModrinthSearchResult> {
    mrpack::search_modrinth_content(
        &state.http,
        query.as_deref().unwrap_or(""),
        offset.unwrap_or(0),
        limit.unwrap_or_else(mrpack::search_page_size),
        sort.as_deref().unwrap_or("downloads"),
        &project_type,
        game_version.as_deref(),
        loader.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn list_modrinth_content_versions(
    state: State<'_, AppState>,
    id: String,
    slug: String,
    project_type: Option<String>,
) -> Result<mrpack::ModrinthContentVersions> {
    let (_, dirs) = ctx()?;
    let inst = instances::get(&dirs, &id)?;
    mrpack::list_modrinth_content_versions(
        &state.http,
        &inst,
        &slug,
        project_type.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn install_modrinth_content(
    state: State<'_, AppState>,
    id: String,
    slug: String,
    project_type: Option<String>,
    version_id: Option<String>,
    optional_project_ids: Option<Vec<String>>,
) -> Result<mrpack::InstallContentResult> {
    let (_, dirs) = ctx()?;
    let inst = instances::get(&dirs, &id)?;
    mrpack::install_modrinth_content(
        &state.http,
        &dirs,
        &inst,
        &slug,
        project_type.as_deref(),
        version_id.as_deref(),
        optional_project_ids.as_deref().unwrap_or(&[]),
    )
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeaturedPackInfo {
    pub enabled: bool,
    pub slug: String,
    pub title: String,
    pub blurb: String,
    pub server_name: String,
    pub server_address: String,
}

#[tauri::command]
pub fn get_featured_pack() -> Result<FeaturedPackInfo> {
    let (settings, _) = ctx()?;
    let query = settings.featured_pack_query();
    let slug = mrpack::parse_modrinth_query(&query)
        .map(|(s, _)| s)
        .unwrap_or_default();
    Ok(FeaturedPackInfo {
        enabled: !query.is_empty(),
        slug,
        title: settings.featured_pack_title(),
        blurb: settings.featured_pack_blurb(),
        server_name: settings.featured_server_name(),
        server_address: settings.featured_server_address(),
    })
}

#[tauri::command]
pub async fn install_featured_pack(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Instance> {
    let result = install_featured_pack_inner(&app, &state).await;
    if result.is_err() {
        emit_install_cleared(&app);
    }
    result
}

async fn install_featured_pack_inner(
    app: &AppHandle,
    state: &State<'_, AppState>,
) -> Result<Instance> {
    let (settings, dirs) = ctx()?;
    let query = settings.featured_pack_query();
    if query.is_empty() {
        return Err(Error::msg(
            "Brak wbudowanej paczki. Oczekiwany plik: packs/Cobblemon vasst 1.0.0.mrpack",
        ));
    }
    emit_progress(
        app,
        InstallProgress {
            instance_id: String::new(),
            stage: "modpack".into(),
            current: 0,
            total: 1,
            file: None,
            message: format!("Pobieranie paczki {}…", settings.featured_pack_title()),
        },
    );
    let pack = resolve_pack_file(app, &state.http, &dirs, &query).await?;
    mrpack::validate_mrpack(&pack)?;
    let linked = mrpack::parse_modrinth_query(&query)
        .map(|(slug, _)| slug)
        .unwrap_or_else(|_| "mrpack".to_string());
    let mut inst = import_pack_file(app.clone(), state.inner(), pack, &linked, None).await?;
    let addr = settings.featured_server_address();
    if !addr.is_empty() {
        inst.join_server = addr.clone();
        instances::save(&dirs, &inst)?;
        let mut list = servers::load(&dirs)?;
        list.servers.push(ServerEntry {
            name: settings.featured_server_name(),
            address: addr,
        });
        servers::save(&dirs, &list)?;
    }
    Ok(inst)
}

async fn resolve_pack_file(
    app: &AppHandle,
    client: &reqwest::Client,
    dirs: &Dirs,
    query: &str,
) -> Result<PathBuf> {
    let q = query.trim();
    let lower = q.to_ascii_lowercase();
    if (lower.starts_with("http://") || lower.starts_with("https://")) && lower.contains(".mrpack")
    {
        let dest = dirs.cache.join("featured-pack.mrpack");
        download_file(client, q, &dest, None, None, None).await?;
        mrpack::validate_mrpack(&dest)?;
        return Ok(dest);
    }
    let as_path = PathBuf::from(q);
    if as_path.is_file() {
        mrpack::validate_mrpack(&as_path)?;
        return stage_pack_in_data_dir(dirs, &as_path);
    }
    if let Some(found) = find_local_pack(app, dirs, q)? {
        return stage_pack_in_data_dir(dirs, &found);
    }
    if looks_like_local_pack(q) {
        return Err(Error::msg(format!(
            "Nie znaleziono pliku paczki „{q}”. Skopiuj .mrpack do folderu packs obok launchera albo do %APPDATA%\\.octralauncher-dev\\packs\\."
        )));
    }
    let (slug, version) = mrpack::parse_modrinth_query(q)?;
    mrpack::download_modrinth_mrpack(client, dirs, &slug, version.as_deref()).await
}

fn looks_like_local_pack(q: &str) -> bool {
    let lower = q.to_ascii_lowercase();
    lower.ends_with(".mrpack") || q.contains('/') || q.contains('\\') || Path::new(q).is_absolute()
}

fn find_local_pack(app: &AppHandle, dirs: &Dirs, query: &str) -> Result<Option<PathBuf>> {
    let mut incomplete: Option<PathBuf> = None;
    for candidate in local_pack_candidates(app, dirs, query) {
        if !candidate.is_file() {
            continue;
        }
        if download::zip_is_complete(&candidate) {
            return Ok(Some(candidate));
        }
        if incomplete.is_none() {
            incomplete = Some(candidate);
        }
    }
    if let Some(path) = incomplete {
        return Err(mrpack::incomplete_pack_error(&path));
    }
    Ok(None)
}

fn local_pack_candidates(app: &AppHandle, dirs: &Dirs, query: &str) -> Vec<PathBuf> {
    let query_path = Path::new(query);
    let file_name = query_path
        .file_name()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(query));
    let mut out: Vec<PathBuf> = Vec::new();
    let mut push = |p: PathBuf| {
        if !out.iter().any(|e| e == &p) {
            out.push(p);
        }
    };

    push(query_path.to_path_buf());
    push(dirs.root.join(query));
    push(dirs.root.join("packs").join(&file_name));
    push(dirs.cache.join(&file_name));

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            push(dir.join(query));
            push(dir.join("packs").join(&file_name));
            push(dir.join(&file_name));
            push(dir.join("resources").join(query));
            push(dir.join("resources").join("packs").join(&file_name));
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        push(cwd.join(query));
        push(cwd.join("packs").join(&file_name));
        push(cwd.join(&file_name));
        if let Some(parent) = cwd.parent() {
            push(parent.join(query));
            push(parent.join("packs").join(&file_name));
        }
    }

    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    push(manifest.join(query));
    push(manifest.join("packs").join(&file_name));
    if let Some(parent) = manifest.parent() {
        push(parent.join(query));
        push(parent.join("packs").join(&file_name));
    }

    if let Ok(p) = app.path().resolve(query, BaseDirectory::Resource) {
        push(p);
    }
    if let Ok(p) = app.path().resolve(&file_name, BaseDirectory::Resource) {
        push(p);
    }
    if let Ok(dir) = app.path().resource_dir() {
        push(dir.join(query));
        push(dir.join("packs").join(&file_name));
        push(dir.join(&file_name));
    }

    out
}

fn stage_pack_in_data_dir(dirs: &Dirs, src: &Path) -> Result<PathBuf> {
    let Some(name) = src.file_name() else {
        return Ok(src.to_path_buf());
    };
    if src.starts_with(&dirs.root) {
        return Ok(src.to_path_buf());
    }
    let dest_dir = dirs.root.join("packs");
    let dest = dest_dir.join(name);
    let copy_needed = match (std::fs::metadata(src), std::fs::metadata(&dest)) {
        (Ok(a), Ok(b)) => a.len() != b.len() || !download::zip_is_complete(&dest),
        (Ok(_), Err(_)) => true,
        _ => false,
    };
    if copy_needed {
        std::fs::create_dir_all(&dest_dir)?;
        if std::fs::copy(src, &dest).is_err() && src.is_file() {
            return Ok(src.to_path_buf());
        }
    }
    if dest.is_file() {
        Ok(dest)
    } else {
        Ok(src.to_path_buf())
    }
}

#[derive(Clone, Copy)]
enum PackFormat {
    Mrpack,
    CurseForge,
}

fn detect_pack_format(path: &Path) -> Result<PackFormat> {
    download::validate_zip_archive(path).map_err(|_| {
        Error::msg(format!(
            "Archiwum ZIP jest niekompletne lub uszkodzone: {}",
            path.display()
        ))
    })?;
    if download::zip_has_file(path, "modrinth.index.json") {
        return Ok(PackFormat::Mrpack);
    }
    if download::zip_has_file(path, "manifest.json") {
        if curseforge::is_curseforge_pack(path) {
            return Ok(PackFormat::CurseForge);
        }
        return Err(Error::msg(
            "manifest.json w tym ZIP nie wygląda na paczkę CurseForge (brak wersji Minecraft).",
        ));
    }
    Err(Error::msg(
        "To nie jest paczka Modrinth (.mrpack) ani CurseForge (ZIP z manifest.json).",
    ))
}

async fn import_pack_file(
    app: AppHandle,
    state: &AppState,
    pack: PathBuf,
    linked_pack: &str,
    icon_url: Option<&str>,
) -> Result<Instance> {
    let (settings, dirs) = ctx()?;
    let cancel = CancellationToken::new();
    {
        let mut g = state.install_cancel.lock();
        if let Some(old) = g.take() {
            old.cancel();
        }
        *g = Some(cancel.clone());
    }
    let format = detect_pack_format(&pack)?;
    let linked = if linked_pack.eq_ignore_ascii_case("mrpack")
        && matches!(format, PackFormat::CurseForge)
    {
        "curseforge".to_string()
    } else {
        linked_pack.to_string()
    };
    let req = match format {
        PackFormat::Mrpack => {
            mrpack::import_mrpack(&state.http, &app, &dirs, &settings, &pack, &cancel).await?
        }
        PackFormat::CurseForge => {
            curseforge::import_curseforge(&state.http, &app, &pack, &cancel).await?
        }
    };
    let mut inst = instances::create(&dirs, &settings, req)?;
    inst.linked_pack = Some(linked.clone());
    inst.pack_locked = true;
    instances::save(&dirs, &inst)?;
    mrpack::apply_pack_icon(
        &state.http,
        &dirs,
        &mut inst,
        &pack,
        &linked,
        icon_url,
    )
    .await;
    let result = async {
        match format {
            PackFormat::Mrpack => {
                mrpack::populate_instance_from_pack(
                    &state.http, &app, &dirs, &inst, &pack, &cancel,
                )
                .await?;
            }
            PackFormat::CurseForge => {
                curseforge::populate_instance_from_pack(
                    &state.http, &app, &dirs, &inst, &pack, &cancel,
                )
                .await?;
            }
        }
        mrpack::adopt_extracted_icon(&dirs, &mut inst);
        run_install(app.clone(), state, inst.id.clone()).await
    }
    .await;
    match result {
        Ok(inst) => Ok(inst),
        Err(e) => {
            let _ = instances::delete(&dirs, &inst.id);
            emit_install_cleared(&app);
            Err(e)
        }
    }
}

#[tauri::command]
pub fn duplicate_instance(id: String) -> Result<Instance> {
    let (_, dirs) = ctx()?;
    instances::duplicate(&dirs, &id)
}

#[tauri::command]
pub fn list_worlds(id: String) -> Result<Vec<instances::WorldEntry>> {
    let (_, dirs) = ctx()?;
    instances::list_worlds(&dirs, &id)
}

#[tauri::command]
pub fn delete_world(id: String, folder: String) -> Result<Vec<instances::WorldEntry>> {
    let (_, dirs) = ctx()?;
    instances::delete_world(&dirs, &id, &folder)
}

#[tauri::command]
pub fn copy_world(from_id: String, folder: String, to_id: String) -> Result<Vec<instances::WorldEntry>> {
    let (_, dirs) = ctx()?;
    instances::copy_world(&dirs, &from_id, &folder, &to_id)
}

#[tauri::command]
pub fn open_world_folder(id: String, folder: String) -> Result<()> {
    let (_, dirs) = ctx()?;
    let path = instances::world_dir(&dirs, &id, &folder)?;
    opener_open(&path)
}

#[tauri::command]
pub fn list_crash_reports(id: String) -> Result<Vec<instances::CrashReport>> {
    let (_, dirs) = ctx()?;
    instances::list_crash_reports(&dirs, &id)
}

#[tauri::command]
pub fn list_screenshots(id: String) -> Result<Vec<instances::ScreenshotEntry>> {
    let (_, dirs) = ctx()?;
    instances::list_screenshots(&dirs, &id)
}

#[tauri::command]
pub fn read_screenshot(id: String, name: String, full: Option<bool>) -> Result<String> {
    let (_, dirs) = ctx()?;
    instances::read_screenshot_path(&dirs, &id, &name, full.unwrap_or(false))
}

#[tauri::command]
pub async fn save_screenshot_as(id: String, name: String) -> Result<Option<String>> {
    let (_, dirs) = ctx()?;
    let src = instances::read_screenshot_path(&dirs, &id, &name, true)?;
    let src_path = Path::new(&src);
    let mut dialog = rfd::AsyncFileDialog::new()
        .set_file_name(&name)
        .set_title("Zapisz zrzut ekranu");
    if name.to_ascii_lowercase().ends_with(".png") {
        dialog = dialog.add_filter("PNG", &["png"]);
    } else {
        dialog = dialog.add_filter("Obraz", &["png", "jpg", "jpeg", "webp"]);
    }
    let Some(file) = dialog.save_file().await else {
        return Ok(None);
    };
    std::fs::copy(src_path, file.path())?;
    Ok(Some(file.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub fn open_instance_subdir(id: String, folder: String) -> Result<()> {
    let (_, dirs) = ctx()?;
    let path = instances::game_subdir(&dirs, &id, &folder)?;
    opener_open(&path)
}

#[tauri::command]
pub async fn pick_java_exe() -> Option<String> {
    rfd::AsyncFileDialog::new()
        .add_filter("Java", &["exe"])
        .set_title("Wskaż java.exe")
        .pick_file()
        .await
        .map(|p| p.path().to_string_lossy().to_string())
}

#[tauri::command]
pub async fn pick_directory() -> Option<String> {
    rfd::AsyncFileDialog::new()
        .set_title("Wybierz folder")
        .pick_folder()
        .await
        .map(|p| p.path().to_string_lossy().to_string())
}

#[tauri::command]
pub fn scan_curseforge_instances(root: Option<String>) -> Result<Vec<cf_instance::CurseForgeInstanceHit>> {
    let path = root
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(Path::new);
    cf_instance::scan(path)
}

#[tauri::command]
pub async fn import_curseforge_instance(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<Instance> {
    let (settings, dirs) = ctx()?;
    emit_progress(
        &app,
        InstallProgress {
            instance_id: String::new(),
            stage: "modpack".into(),
            current: 0,
            total: 1,
            file: None,
            message: "Import folderu instancji CurseForge".into(),
        },
    );
    let inst = match cf_instance::import_folder(&dirs, &settings, Path::new(&path)) {
        Ok(i) => i,
        Err(e) => {
            emit_install_cleared(&app);
            return Err(e);
        }
    };
    match run_install(app.clone(), state.inner(), inst.id.clone()).await {
        Ok(inst) => Ok(inst),
        Err(e) => {
            let _ = instances::delete(&dirs, &inst.id);
            emit_install_cleared(&app);
            Err(e)
        }
    }
}

#[tauri::command]
pub fn probe_java_path(path: String) -> Option<java::JavaRuntime> {
    java::probe_java(Path::new(&path))
}

#[tauri::command]
pub fn purge_cache() -> Result<()> {
    let (_, dirs) = ctx()?;
    if dirs.cache.exists() {
        std::fs::remove_dir_all(&dirs.cache)?;
    }
    std::fs::create_dir_all(&dirs.cache)?;
    Ok(())
}

#[tauri::command]
pub async fn export_mrpack(id: String) -> Result<Option<String>> {
    let dest = rfd::AsyncFileDialog::new()
        .add_filter("Modrinth pack", &["mrpack"])
        .set_file_name("instancja.mrpack")
        .set_title("Eksportuj instancję")
        .save_file()
        .await;
    let Some(file) = dest else {
        return Ok(None);
    };
    let mut path = file.path().to_path_buf();
    if path
        .extension()
        .and_then(|e| e.to_str())
        .is_none_or(|e| !e.eq_ignore_ascii_case("mrpack"))
    {
        path.set_extension("mrpack");
    }
    let (_, dirs) = ctx()?;
    let inst = instances::get(&dirs, &id)?;
    let saved = mrpack::export_mrpack(&dirs, &inst, &path).await?;
    Ok(Some(saved.to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn check_content_updates(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<mrpack::ContentUpdate>> {
    let (_, dirs) = ctx()?;
    let inst = instances::get(&dirs, &id)?;
    mrpack::check_content_updates(&state.http, &dirs, &inst).await
}

#[tauri::command]
pub async fn check_pack_update(
    state: State<'_, AppState>,
    id: String,
) -> Result<mrpack::PackUpdateInfo> {
    let (_, dirs) = ctx()?;
    let inst = instances::get(&dirs, &id)?;
    mrpack::check_pack_update(&state.http, &dirs, &inst).await
}

#[tauri::command]
pub fn required_java_for_version(id: String) -> u32 {
    meta::required_java_for_id(&id)
}

#[tauri::command]
pub fn list_local_servers(state: State<'_, AppState>) -> Result<Vec<LocalServerInfo>> {
    local_server::list(state.inner())
}

#[tauri::command]
pub fn get_local_server(state: State<'_, AppState>, id: String) -> Result<LocalServerInfo> {
    local_server::get(state.inner(), &id)
}

#[tauri::command]
pub async fn create_local_server(
    app: AppHandle,
    req: CreateLocalServer,
) -> Result<LocalServerInfo> {
    local_server::create(app, req).await
}

#[tauri::command]
pub async fn probe_local_server(
    app: AppHandle,
    software: local_server::LocalSoftware,
    game_version: String,
    loader_version: Option<String>,
) -> Result<()> {
    let (_, dirs) = Settings::load()?;
    dirs.ensure()?;
    let client = app.state::<AppState>().http.clone();
    local_server::probe_software(
        &client,
        &dirs,
        software,
        &game_version,
        loader_version.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn list_paper_versions(app: AppHandle) -> Result<Vec<String>> {
    let client = app.state::<AppState>().http.clone();
    local_server::list_paper_versions(&client).await
}

#[tauri::command]
pub fn update_local_server(
    state: State<'_, AppState>,
    id: String,
    patch: UpdateLocalServer,
) -> Result<LocalServerInfo> {
    local_server::update(state.inner(), &id, patch)
}

#[tauri::command]
pub async fn delete_local_server(app: AppHandle, id: String) -> Result<()> {
    local_server::delete(app, id).await
}

#[tauri::command]
pub async fn install_local_server(app: AppHandle, id: String) -> Result<LocalServerInfo> {
    local_server::install(app, id).await
}

#[tauri::command]
pub async fn start_local_server(app: AppHandle, id: String) -> Result<LocalServerInfo> {
    local_server::start(app, id).await
}

#[tauri::command]
pub async fn stop_local_server(app: AppHandle, id: String) -> Result<()> {
    local_server::stop(app, id).await
}

#[tauri::command]
pub fn send_local_server_command(
    state: State<'_, AppState>,
    id: String,
    command: String,
) -> Result<()> {
    local_server::send_command(state.inner(), &id, &command)
}

#[tauri::command]
pub fn read_local_server_log(id: String) -> Result<String> {
    local_server::read_log(&id)
}

#[tauri::command]
pub fn open_local_server_folder(id: String) -> Result<()> {
    local_server::open_dir(&id)
}

#[tauri::command]
pub fn open_local_server_backups(id: String) -> Result<()> {
    local_server::open_backups(&id)
}

#[tauri::command]
pub fn open_local_server_properties(id: String) -> Result<()> {
    local_server::open_properties(&id)
}

#[tauri::command]
pub fn backup_local_server_world(id: String) -> Result<String> {
    local_server::backup_world(&id)
}

#[tauri::command]
pub async fn check_github_release(state: State<'_, AppState>) -> Result<GithubReleaseCheck> {
    github_release::check_latest(&state.http).await
}

#[tauri::command]
pub async fn skins_lan_url(state: State<'_, AppState>) -> Result<Option<String>> {
    state.skins.ensure_started(state.http.clone()).await?;
    Ok(state.skins.lan_advertise_url())
}

#[tauri::command]
pub fn list_all_screenshots() -> Result<Vec<instances::GlobalScreenshotEntry>> {
    let (_, dirs) = ctx()?;
    instances::list_all_screenshots(&dirs)
}

#[tauri::command]
pub async fn get_account_skin(
    state: State<'_, AppState>,
    uuid: String,
    refresh: Option<bool>,
) -> Result<auth::AccountSkin> {
    let (settings, dirs) = ctx()?;
    let file = auth::load_accounts(&dirs)?;
    let wanted = auth::hyphenate_uuid(&uuid);
    let account = file
        .accounts
        .iter()
        .find(|a| auth::hyphenate_uuid(&a.uuid) == wanted)
        .ok_or_else(|| Error::msg("Nie znaleziono konta."))?
        .clone();
    auth::fetch_account_skin(
        &state.http,
        &settings.azure_client_id(),
        &dirs,
        &account,
        refresh.unwrap_or(false),
    )
    .await
}

#[tauri::command]
pub fn get_mojang_skin_catalog() -> Vec<crate::mojang_skins::CatalogGroup> {
    crate::mojang_skins::catalog()
}

#[tauri::command]
pub async fn get_minecraft_profile(
    state: State<'_, AppState>,
    uuid: String,
    refresh: Option<bool>,
) -> Result<crate::mojang_skins::McPlayerProfile> {
    let (settings, dirs) = ctx()?;
    crate::mojang_skins::profile_for_account_with_refresh(
        &state.http,
        &settings.azure_client_id(),
        &dirs,
        &uuid,
        refresh.unwrap_or(false),
    )
    .await
}

#[tauri::command]
pub async fn equip_mojang_skin(
    state: State<'_, AppState>,
    uuid: String,
    texture_key: String,
    variant: String,
) -> Result<crate::mojang_skins::McPlayerProfile> {
    let (settings, dirs) = ctx()?;
    crate::mojang_skins::equip_for_account(
        &state.http,
        &settings.azure_client_id(),
        &dirs,
        &uuid,
        &texture_key,
        &variant,
    )
    .await
}

#[tauri::command]
pub async fn set_minecraft_cape(
    state: State<'_, AppState>,
    uuid: String,
    cape_id: Option<String>,
) -> Result<crate::mojang_skins::McPlayerProfile> {
    let (settings, dirs) = ctx()?;
    crate::mojang_skins::set_cape_for_account(
        &state.http,
        &settings.azure_client_id(),
        &dirs,
        &uuid,
        cape_id.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn upload_mojang_skin(
    state: State<'_, AppState>,
    uuid: String,
    png: Vec<u8>,
    variant: String,
) -> Result<crate::mojang_skins::McPlayerProfile> {
    let (settings, dirs) = ctx()?;
    crate::mojang_skins::upload_for_account(
        &state.http,
        &settings.azure_client_id(),
        &dirs,
        &uuid,
        &png,
        &variant,
    )
    .await
}

#[tauri::command]
pub async fn get_mojang_texture_preview(
    state: State<'_, AppState>,
    texture_key: String,
) -> Result<String> {
    crate::mojang_skins::texture_png_base64(&state.http, &texture_key).await
}

// ── Minecraft skins (Modrinth-style API) ─────────────────────────────────────

#[tauri::command]
pub async fn get_available_skins(
    state: State<'_, AppState>,
    uuid: String,
) -> Result<Vec<crate::minecraft_skins::Skin>> {
    let (settings, dirs) = ctx()?;
    crate::minecraft_skins::get_available_skins(
        &state.http,
        &settings.azure_client_id(),
        &dirs,
        &uuid,
    )
    .await
}

#[tauri::command]
pub async fn get_available_capes(
    state: State<'_, AppState>,
    uuid: String,
) -> Result<Vec<crate::minecraft_skins::Cape>> {
    let (settings, dirs) = ctx()?;
    crate::minecraft_skins::get_available_capes(
        &state.http,
        &settings.azure_client_id(),
        &dirs,
        &uuid,
    )
    .await
}

#[tauri::command]
pub async fn equip_skin(
    state: State<'_, AppState>,
    req: crate::minecraft_skins::EquipSkinReq,
) -> Result<Option<crate::mojang_skins::McPlayerProfile>> {
    let (settings, dirs) = ctx()?;
    let profile = crate::minecraft_skins::equip_skin(
        &state.http,
        &settings.azure_client_id(),
        &dirs,
        &req.uuid,
        &req.skin,
        req.png.as_deref(),
    )
    .await?;
    state.skins.reindex(&dirs);
    state.skins.notify_lan();
    crate::yggdrasil::push_account_skin(&state.http, &dirs, &req.uuid).await;
    if let Some(ygg) = state.skins.ygg_root() {
        let registry = settings.skins_url();
        let _ = crate::skin_loader::refresh_csl_configs(&ygg, &registry, &dirs);
    }
    Ok(profile)
}

#[tauri::command]
pub async fn save_custom_skin(
    state: State<'_, AppState>,
    req: crate::minecraft_skins::SaveCustomSkinReq,
) -> Result<crate::skin_library::SkinLibraryEntryView> {
    let (settings, dirs) = ctx()?;
    let entry = crate::minecraft_skins::save_custom_skin(
        &dirs,
        &req.uuid,
        &req.skin,
        &req.variant,
        req.cape_id.as_deref(),
        req.png.as_deref(),
        req.replace_texture,
    )?;
    if entry.is_active {
        state.skins.reindex(&dirs);
        state.skins.notify_lan();
        crate::yggdrasil::push_account_skin(&state.http, &dirs, &req.uuid).await;
        if let Some(ygg) = state.skins.ygg_root() {
            let registry = settings.skins_url();
            let _ = crate::skin_loader::refresh_csl_configs(&ygg, &registry, &dirs);
        }
    }
    Ok(entry)
}

#[tauri::command]
pub fn remove_custom_skin(uuid: String, skin: crate::minecraft_skins::Skin) -> Result<()> {
    let (_, dirs) = ctx()?;
    crate::minecraft_skins::remove_custom_skin(&dirs, &uuid, &skin)
}

#[tauri::command]
pub async fn normalize_skin_texture(
    state: State<'_, AppState>,
    texture: serde_json::Value,
) -> Result<Vec<u8>> {
    use crate::minecraft_skins::TextureInput;
    let input = if let Some(s) = texture.as_str() {
        TextureInput::Url(s.to_string())
    } else if let Some(arr) = texture.as_array() {
        let bytes: Vec<u8> = arr
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as u8))
            .collect();
        TextureInput::Bytes(bytes)
    } else {
        return Err(Error::msg("texture musi być URL (string) albo tablicą bajtów."));
    };
    crate::minecraft_skins::normalize_skin_texture_input(&state.http, input).await
}

#[tauri::command]
pub async fn flush_pending_skin_change() -> Result<()> {
    Ok(())
}

/// Pobiera obraz (skin z mc-heads itd.) po HTTP — omija CORS w WebGL.
#[tauri::command]
pub async fn fetch_image_base64(state: State<'_, AppState>, url: String) -> Result<String> {
    use base64::Engine as _;
    let url = url.trim();
    if url.is_empty() {
        return Err(Error::msg("Brak adresu URL obrazu."));
    }
    if url.starts_with("data:") {
        return Err(Error::msg("URL data: nie jest obsługiwany w tym API."));
    }
    let resp = state.http.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(Error::msg(format!(
            "Nie udało się pobrać obrazu (HTTP {}).",
            resp.status().as_u16()
        )));
    }
    let bytes = resp.bytes().await?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

#[tauri::command]
pub fn relay_start(app: AppHandle, state: State<'_, AppState>, name: String) -> Result<()> {
    state.relay.start(app, name)
}

#[tauri::command]
pub fn relay_stop(state: State<'_, AppState>) -> Result<()> {
    state.relay.stop();
    Ok(())
}

#[tauri::command]
pub fn relay_list_peers(state: State<'_, AppState>) -> Result<Vec<relay::RelayPeerInfo>> {
    Ok(state.relay.list_peers())
}

#[tauri::command]
pub fn relay_send(state: State<'_, AppState>, peer_id: String, text: String) -> Result<()> {
    state.relay.send(&peer_id, &text)
}

#[tauri::command]
pub fn scan_prism_instances() -> Result<Vec<import_launchers::LauncherInstanceHit>> {
    import_launchers::scan_prism_instances()
}

#[tauri::command]
pub fn scan_multimc_instances(root: Option<String>) -> Result<Vec<import_launchers::LauncherInstanceHit>> {
    let path = root
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(std::path::Path::new);
    import_launchers::scan_multimc_instances(path)
}

#[tauri::command]
pub fn import_launcher_instance(path: String, source: String) -> Result<Instance> {
    let (settings, dirs) = ctx()?;
    import_launchers::import_launcher_instance(&dirs, &settings, &path, &source)
}

fn opener_open(path: &std::path::Path) -> Result<()> {
    #[cfg(windows)]
    {
        let mut cmd = std::process::Command::new("explorer");
        cmd.arg(path);
        crate::winhide::hide_std(&mut cmd);
        cmd.spawn()
            .map_err(|e| Error::msg(format!("Nie otwarto folderu: {e}")))?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        Err(Error::msg("Otwieranie folderu jest dostępne na Windows."))
    }
}
