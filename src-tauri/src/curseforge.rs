//! Import paczek CurseForge (ZIP z `manifest.json`).
//!
//! Pobieranie plików — bez wbudowanego klucza Overwolf:
//! 1. Oficjalne API, tylko gdy użytkownik poda własny `LUMEN_CURSEFORGE_API_KEY`.
//! 2. Modrinth (`api.modrinth.com`) — dopasowanie po nazwie pliku i zgadniętym slugu.
//! 3. Cursemaven (`cursemaven.com`) — publiczny Maven, bez klucza po naszej stronie.
//! 4. Publiczny endpoint strony CurseForge `/api/v1/mods/{id}/files/{fileId}/download`
//!    (przekierowanie jak w przeglądarce; nie zapisujemy żadnego klucza).
//!
//! Bezpośredni CDN (`edge.forgecdn.net` / `mediafilez.forgecdn.net`) od lipca 2026
//! zwraca 403 bez klucza API — nie wołamy go wprost.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use futures::StreamExt;
use parking_lot::Mutex;
use serde::Deserialize;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use crate::download;
use crate::error::{Error, Result};
use crate::install::{emit_progress, InstallProgress};
use crate::instances::{CreateInstance, Instance, Loader};
use crate::paths::Dirs;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    #[serde(default)]
    name: String,
    #[serde(default)]
    overrides: Option<String>,
    minecraft: ManifestMinecraft,
    #[serde(default)]
    files: Vec<ManifestFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestMinecraft {
    version: String,
    #[serde(default)]
    mod_loaders: Vec<ModLoader>,
}

#[derive(Debug, Deserialize)]
struct ModLoader {
    id: String,
    #[serde(default)]
    primary: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestFile {
    #[serde(rename = "projectID", alias = "projectId")]
    project_id: i64,
    #[serde(rename = "fileID", alias = "fileId")]
    file_id: i64,
    #[serde(default = "default_true")]
    required: bool,
    #[serde(default, rename = "fileName", alias = "filename")]
    file_name: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct WebsiteFileResponse {
    data: WebsiteFile,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebsiteFile {
    #[serde(default)]
    file_name: Option<String>,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    file_length: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OfficialFileResponse {
    data: OfficialFile,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OfficialFile {
    #[serde(default)]
    file_name: Option<String>,
    #[serde(default)]
    file_length: Option<u64>,
    #[serde(default)]
    download_url: Option<String>,
    #[serde(default)]
    hashes: Vec<OfficialHash>,
}

#[derive(Debug, Deserialize)]
struct OfficialHash {
    #[serde(default)]
    value: String,
    /// 1 = SHA-1, 2 = MD5
    #[serde(default)]
    algo: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct ModrinthVersion {
    #[serde(default)]
    files: Vec<ModrinthFile>,
}

#[derive(Debug, Clone, Deserialize)]
struct ModrinthFile {
    url: String,
    #[serde(default)]
    filename: String,
    #[serde(default)]
    primary: bool,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    hashes: ModrinthHashes,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ModrinthHashes {
    #[serde(default)]
    sha1: Option<String>,
}

struct Resolved {
    url: String,
    file_name: String,
    sha1: Option<String>,
    size: Option<u64>,
}

struct MrCache {
    /// slug → wersje już pobrane (także puste, gdy projektu nie ma)
    versions: HashMap<String, Option<Vec<ModrinthVersion>>>,
}

pub fn is_curseforge_pack(path: &Path) -> bool {
    let Ok(raw) = download::read_zip_text(path, "manifest.json") else {
        return false;
    };
    parse_manifest(&raw).is_ok()
}

fn parse_manifest(raw: &str) -> Result<Manifest> {
    let raw = raw.trim_start_matches('\u{feff}');
    let manifest: Manifest = serde_json::from_str(raw)?;
    if manifest.minecraft.version.trim().is_empty() {
        return Err(Error::msg(
            "manifest.json nie wygląda na paczkę CurseForge (brak wersji Minecraft).",
        ));
    }
    Ok(manifest)
}

pub fn validate_curseforge_pack(path: &Path) -> Result<()> {
    download::validate_zip_archive(path)?;
    let raw = download::read_zip_text(path, "manifest.json").map_err(|_| {
        Error::msg(format!(
            "To nie jest paczka CurseForge (brak manifest.json): {}",
            path.display()
        ))
    })?;
    parse_manifest(&raw)?;
    Ok(())
}

pub async fn import_curseforge(
    _client: &reqwest::Client,
    app: &AppHandle,
    pack_path: &Path,
    _cancel: &CancellationToken,
) -> Result<CreateInstance> {
    if !pack_path.exists() {
        return Err(Error::msg("Nie znaleziono pliku paczki CurseForge."));
    }
    validate_curseforge_pack(pack_path)?;
    let raw = download::read_zip_text(pack_path, "manifest.json")?;
    let manifest = parse_manifest(&raw)?;
    let req = manifest_to_create(&manifest, pack_path)?;
    emit_progress(
        app,
        InstallProgress {
            instance_id: String::new(),
            stage: "modpack".into(),
            current: 0,
            total: 1,
            file: Some(req.name.clone()),
            message: format!("Paczka CurseForge: {}", req.name),
        },
    );
    Ok(req)
}

pub async fn populate_instance_from_pack(
    client: &reqwest::Client,
    app: &AppHandle,
    dirs: &Dirs,
    inst: &Instance,
    pack_path: &Path,
    cancel: &CancellationToken,
) -> Result<()> {
    let raw = download::read_zip_text(pack_path, "manifest.json")?;
    let manifest = parse_manifest(&raw)?;
    let game_dir = dirs.game_dir(&inst.id);
    std::fs::create_dir_all(&game_dir)?;

    let files: Vec<ManifestFile> = manifest
        .files
        .iter()
        .filter(|f| f.required && f.project_id > 0 && f.file_id > 0)
        .cloned()
        .collect();
    let total = files.len() as u64;
    let done = Arc::new(AtomicU64::new(0));
    let cache = Arc::new(Mutex::new(MrCache {
        versions: HashMap::new(),
    }));
    let tmp_root = game_dir.join(".lumen-cf-tmp");
    let _ = std::fs::remove_dir_all(&tmp_root);
    std::fs::create_dir_all(&tmp_root)?;

    let api_key = user_cf_api_key();
    let fails = Arc::new(Mutex::new(Vec::<String>::new()));

    let outcomes = futures::stream::iter(files.into_iter().map(|file| {
        let client = client.clone();
        let app = app.clone();
        let inst_id = inst.id.clone();
        let game_dir = game_dir.clone();
        let tmp_root = tmp_root.clone();
        let done = done.clone();
        let cache = cache.clone();
        let fails = fails.clone();
        let cancel = cancel.clone();
        let api_key = api_key.clone();
        async move {
            if cancel.is_cancelled() {
                return Err(Error::msg("Instalacja anulowana."));
            }
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            emit_progress(
                &app,
                InstallProgress {
                    instance_id: inst_id,
                    stage: "modpack".into(),
                    current: n,
                    total: total.max(1),
                    file: Some(format!("{} / {}", file.project_id, file.file_id)),
                    message: format!("Pliki paczki CurseForge {n}/{}", total.max(1)),
                },
            );
            match download_manifest_file(
                &client,
                &game_dir,
                &tmp_root,
                &file,
                api_key.as_deref(),
                &cache,
            )
            .await
            {
                Ok(()) => Ok(()),
                Err(e) => {
                    fails.lock().push(format!(
                        "projekt {} / plik {}{}: {e}",
                        file.project_id,
                        file.file_id,
                        file.file_name
                            .as_deref()
                            .filter(|s| !s.is_empty())
                            .map(|s| format!(" ({s})"))
                            .unwrap_or_default()
                    ));
                    Ok(())
                }
            }
        }
    }))
    .buffer_unordered(6)
    .collect::<Vec<Result<()>>>()
    .await;

    let _ = std::fs::remove_dir_all(&tmp_root);

    if outcomes.iter().any(|r| {
        r.as_ref()
            .err()
            .is_some_and(|e| e.to_string().contains("anulowana"))
    }) {
        return Err(Error::msg("Instalacja anulowana."));
    }
    for r in outcomes {
        r?;
    }

    let failed = fails.lock().clone();
    if !failed.is_empty() {
        let shown = failed.len().min(12);
        let mut msg = format!(
            "Nie udało się pobrać {} {} paczki CurseForge:\n",
            failed.len(),
            if failed.len() == 1 { "pliku" } else { "plików" }
        );
        for line in failed.iter().take(shown) {
            msg.push_str("• ");
            msg.push_str(line);
            msg.push('\n');
        }
        if failed.len() > shown {
            msg.push_str(&format!("• …i {} więcej\n", failed.len() - shown));
        }
        msg.push_str(
            "Bezpośredni CDN CurseForge wymaga klucza API (od lipca 2026). Lumen próbuje Modrinth, Cursemaven i publiczny endpoint strony — bez wbudowanego klucza Overwolf. Własny klucz: zmienna LUMEN_CURSEFORGE_API_KEY (console.curseforge.com).",
        );
        return Err(Error::msg(msg.trim().to_string()));
    }

    if cancel.is_cancelled() {
        return Err(Error::msg("Instalacja anulowana."));
    }

    let prefix = manifest
        .overrides
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("overrides")
        .trim_matches('/')
        .to_string();
    emit_progress(
        app,
        InstallProgress {
            instance_id: inst.id.clone(),
            stage: "modpack".into(),
            current: total,
            total: total.max(1),
            file: Some(prefix.clone()),
            message: "Kopiowanie overrides…".into(),
        },
    );
    let pack = pack_path.to_path_buf();
    let dest = game_dir.clone();
    tokio::task::spawn_blocking(move || download::extract_zip_prefix(&pack, &dest, &prefix))
        .await
        .map_err(|e| Error::msg(e.to_string()))??;
    Ok(())
}

fn user_cf_api_key() -> Option<String> {
    for var in ["LUMEN_CURSEFORGE_API_KEY", "CURSEFORGE_API_KEY"] {
        if let Ok(k) = std::env::var(var) {
            let k = k.trim().to_string();
            if !k.is_empty() {
                return Some(k);
            }
        }
    }
    None
}

async fn download_manifest_file(
    client: &reqwest::Client,
    game_dir: &Path,
    tmp_root: &Path,
    file: &ManifestFile,
    api_key: Option<&str>,
    cache: &Mutex<MrCache>,
) -> Result<()> {
    let candidates = resolve_candidates(client, file, api_key, cache).await?;
    let tmp = tmp_root.join(format!("{}-{}", file.project_id, file.file_id));
    let mut last_err = String::new();
    let mut used: Option<Resolved> = None;
    for cand in candidates {
        if tmp.exists() {
            tokio::fs::remove_file(&tmp).await.ok();
        }
        let headers = cdn_headers(&cand.url, api_key);
        match download::download_file_with_headers(
            client,
            &cand.url,
            &tmp,
            cand.sha1.as_deref(),
            cand.size,
            None,
            &headers,
        )
        .await
        {
            Ok(()) if looks_like_archive(&tmp) => {
                used = Some(cand);
                break;
            }
            Ok(()) => {
                let _ = tokio::fs::remove_file(&tmp).await;
                last_err = "pobrany plik to nie archiwum JAR/ZIP".into();
            }
            Err(e) => last_err = e.to_string(),
        }
    }
    let resolved = used.ok_or_else(|| {
        Error::msg(if last_err.is_empty() {
            "nie znaleziono publicznego źródła (Modrinth / Cursemaven / strona CF)".into()
        } else {
            last_err
        })
    })?;
    let name = safe_filename(&resolved.file_name);
    let folder = classify_file(&tmp, &name);
    let dest = game_dir.join(folder).join(&name);
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    if dest.exists() {
        tokio::fs::remove_file(&dest).await.ok();
    }
    tokio::fs::rename(&tmp, &dest).await?;
    Ok(())
}

async fn resolve_candidates(
    client: &reqwest::Client,
    file: &ManifestFile,
    api_key: Option<&str>,
    cache: &Mutex<MrCache>,
) -> Result<Vec<Resolved>> {
    let mut file_name = file
        .file_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let mut size = None;
    let mut out: Vec<Resolved> = Vec::new();

    if let Some(key) = api_key {
        if let Ok(off) = fetch_official_file(client, file.project_id, file.file_id, key).await {
            if file_name.is_none() {
                file_name = off
                    .file_name
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());
            }
            size = off.file_length.or(size);
            let sha1 = off
                .hashes
                .iter()
                .find(|h| h.algo == 1 && !h.value.is_empty())
                .map(|h| h.value.clone());
            if let Some(url) = off
                .download_url
                .as_deref()
                .map(str::trim)
                .filter(|u| u.starts_with("http"))
            {
                out.push(Resolved {
                    url: url.to_string(),
                    file_name: file_name
                        .clone()
                        .unwrap_or_else(|| fallback_name(file.project_id, file.file_id)),
                    sha1,
                    size,
                });
            }
        }
    }

    if file_name.is_none() || size.is_none() {
        if let Ok(web) = fetch_website_file(client, file.project_id, file.file_id).await {
            if file_name.is_none() {
                file_name = web
                    .file_name
                    .or(web.display_name)
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
            }
            if size.is_none() {
                size = web.file_length;
            }
        }
    }

    let name = file_name
        .clone()
        .unwrap_or_else(|| fallback_name(file.project_id, file.file_id));

    if let Some(mr) = lookup_modrinth(client, &name, size, cache).await {
        out.push(mr);
    }

    out.push(Resolved {
        url: cursemaven_url(file.project_id, file.file_id),
        file_name: name.clone(),
        sha1: None,
        size,
    });
    out.push(Resolved {
        url: format!(
            "https://www.curseforge.com/api/v1/mods/{}/files/{}/download",
            file.project_id, file.file_id
        ),
        file_name: name,
        sha1: None,
        size,
    });
    Ok(out)
}

async fn fetch_website_file(
    client: &reqwest::Client,
    project_id: i64,
    file_id: i64,
) -> Result<WebsiteFile> {
    let url = format!("https://www.curseforge.com/api/v1/mods/{project_id}/files/{file_id}");
    let resp: WebsiteFileResponse = download::download_json(client, &url).await.map_err(|_| {
        Error::msg(format!(
            "strona CurseForge nie zwróciła metadanych pliku {file_id}"
        ))
    })?;
    Ok(resp.data)
}

async fn fetch_official_file(
    client: &reqwest::Client,
    project_id: i64,
    file_id: i64,
    api_key: &str,
) -> Result<OfficialFile> {
    let url = format!("https://api.curseforge.com/v1/mods/{project_id}/files/{file_id}");
    let resp = client
        .get(&url)
        .header("x-api-key", api_key)
        .header("Accept", "application/json")
        .send()
        .await?
        .error_for_status()?;
    let body: OfficialFileResponse = resp.json().await?;
    Ok(body.data)
}

async fn lookup_modrinth(
    client: &reqwest::Client,
    file_name: &str,
    size: Option<u64>,
    cache: &Mutex<MrCache>,
) -> Option<Resolved> {
    let want = file_name.rsplit('/').next().unwrap_or(file_name);
    if want.is_empty() {
        return None;
    }
    for slug in slug_candidates(want) {
        let versions = {
            let hit = cache.lock().versions.get(&slug).cloned();
            if let Some(v) = hit {
                v
            } else {
                let fetched = fetch_modrinth_versions(client, &slug).await;
                cache.lock().versions.insert(slug.clone(), fetched.clone());
                fetched
            }
        };
        let Some(versions) = versions else {
            continue;
        };
        if let Some(file) = find_modrinth_file(&versions, want) {
            return Some(Resolved {
                url: file.url.clone(),
                file_name: if file.filename.is_empty() {
                    want.to_string()
                } else {
                    file.filename.clone()
                },
                sha1: file.hashes.sha1.clone(),
                size: file.size.or(size),
            });
        }
    }
    None
}

async fn fetch_modrinth_versions(
    client: &reqwest::Client,
    slug: &str,
) -> Option<Vec<ModrinthVersion>> {
    let url = format!("https://api.modrinth.com/v2/project/{slug}/version");
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json().await.ok()
}

fn find_modrinth_file<'a>(
    versions: &'a [ModrinthVersion],
    file_name: &str,
) -> Option<&'a ModrinthFile> {
    let want = file_name.to_ascii_lowercase();
    let mut fallback = None;
    for v in versions {
        for f in &v.files {
            if f.filename.eq_ignore_ascii_case(&want)
                || f.url
                    .rsplit('/')
                    .next()
                    .is_some_and(|n| n.eq_ignore_ascii_case(&want))
            {
                if f.primary {
                    return Some(f);
                }
                if fallback.is_none() {
                    fallback = Some(f);
                }
            }
        }
    }
    fallback
}

fn slug_candidates(file_name: &str) -> Vec<String> {
    let stem = file_name
        .rsplit_once('.')
        .map(|(s, _)| s)
        .unwrap_or(file_name)
        .to_ascii_lowercase();
    let parts: Vec<&str> = stem
        .split(|c: char| c == '-' || c == '_' || c == '+')
        .filter(|p| !p.is_empty())
        .collect();
    let mut out = Vec::new();
    let push = |out: &mut Vec<String>, s: String| {
        if s.len() >= 2 && s.chars().any(|c| c.is_ascii_alphabetic()) && !out.contains(&s) {
            out.push(s);
        }
    };
    if let Some(a) = parts.first() {
        push(&mut out, (*a).to_string());
    }
    if parts.len() >= 2 && parts[1].chars().any(|c| c.is_ascii_alphabetic()) {
        push(&mut out, format!("{}-{}", parts[0], parts[1]));
    }
    out.truncate(2);
    out
}

fn cursemaven_url(project_id: i64, file_id: i64) -> String {
    format!(
        "https://cursemaven.com/curse/maven/_-{project_id}/{file_id}/_-{project_id}-{file_id}.jar"
    )
}

fn cdn_headers<'a>(url: &str, api_key: Option<&'a str>) -> Vec<(&'a str, &'a str)> {
    let Some(key) = api_key else {
        return Vec::new();
    };
    let host = url
        .split('/')
        .nth(2)
        .unwrap_or("")
        .trim()
        .trim_start_matches("www.")
        .to_ascii_lowercase();
    if host == "api.curseforge.com"
        || host.ends_with(".forgecdn.net")
        || host == "forgecdn.net"
    {
        vec![("x-api-key", key)]
    } else {
        Vec::new()
    }
}

fn looks_like_archive(path: &Path) -> bool {
    use std::io::Read;
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let Ok(meta) = file.metadata() else {
        return false;
    };
    if meta.len() < 22 {
        return false;
    }
    let mut magic = [0u8; 4];
    if file.read_exact(&mut magic).is_err() {
        return false;
    }
    magic[0] == b'P' && magic[1] == b'K'
}

fn fallback_name(project_id: i64, file_id: i64) -> String {
    format!("{project_id}-{file_id}.jar")
}

fn safe_filename(name: &str) -> String {
    let base = name.replace('\\', "/");
    let base = base.rsplit('/').next().unwrap_or(&base).trim();
    let cleaned: String = base
        .chars()
        .filter(|c| !matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*' | '\0'))
        .collect();
    let cleaned = cleaned.trim().trim_matches('.');
    if cleaned.is_empty() || cleaned == "." || cleaned == ".." {
        "mod.jar".into()
    } else {
        cleaned.to_string()
    }
}

fn classify_file(path: &Path, file_name: &str) -> &'static str {
    let lower = file_name.to_ascii_lowercase();
    if lower.ends_with(".jar") {
        return "mods";
    }
    let names = zip_entry_basenames(path);
    let has = |needle: &str| names.iter().any(|n| n == needle || n.starts_with(needle));
    let pack_meta = names.iter().any(|n| n == "pack.mcmeta");
    if pack_meta && (has("shaders/") || names.iter().any(|n| n.starts_with("shaders"))) {
        return "shaderpacks";
    }
    if pack_meta && names.iter().any(|n| n.starts_with("data/")) && !names.iter().any(|n| n.starts_with("assets/")) {
        return "datapacks";
    }
    if pack_meta {
        return "resourcepacks";
    }
    if names.iter().any(|n| {
        matches!(
            n.as_str(),
            "fabric.mod.json"
                | "quilt.mod.json"
                | "mcmod.info"
                | "mods.toml"
                | "neoforge.mods.toml"
        ) || n.ends_with("/mods.toml")
            || n.ends_with("/neoforge.mods.toml")
    }) {
        return "mods";
    }
    if lower.ends_with(".zip") {
        "resourcepacks"
    } else {
        "mods"
    }
}

fn zip_entry_basenames(path: &Path) -> Vec<String> {
    let Ok(file) = std::fs::File::open(path) else {
        return Vec::new();
    };
    let Ok(mut zip) = zip::ZipArchive::new(file) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for i in 0..zip.len().min(400) {
        if let Ok(entry) = zip.by_index(i) {
            names.push(
                entry
                    .name()
                    .replace('\\', "/")
                    .trim_start_matches('/')
                    .to_ascii_lowercase(),
            );
        }
    }
    names
}

fn manifest_to_create(manifest: &Manifest, pack_path: &Path) -> Result<CreateInstance> {
    let game = manifest.minecraft.version.trim().to_string();
    if game.is_empty() {
        return Err(Error::msg("W paczce CurseForge brak wersji Minecraft."));
    }
    let loader_src = manifest
        .minecraft
        .mod_loaders
        .iter()
        .find(|l| l.primary)
        .or_else(|| manifest.minecraft.mod_loaders.first());
    let (loader, loader_version) = match loader_src {
        Some(l) => parse_loader(&l.id, &game),
        None => (Loader::Vanilla, None),
    };
    let mut name = manifest.name.trim().to_string();
    if name.is_empty() {
        name = pack_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("CurseForge")
            .to_string();
    }
    Ok(CreateInstance {
        name,
        game_version: game,
        loader,
        loader_version,
        memory_max_mb: None,
    })
}

fn parse_loader(id: &str, mc: &str) -> (Loader, Option<String>) {
    let id = id.trim();
    let lower = id.to_ascii_lowercase();
    let pairs: [(&str, Loader); 6] = [
        ("neoforge-", Loader::Neoforge),
        ("forge-", Loader::Forge),
        ("fabric-loader-", Loader::Fabric),
        ("quilt-loader-", Loader::Quilt),
        ("fabric-", Loader::Fabric),
        ("quilt-", Loader::Quilt),
    ];
    for (prefix, loader) in pairs {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let ver = id
                .get(prefix.len()..)
                .unwrap_or(rest)
                .trim()
                .to_string();
            let ver = strip_mc_prefix(mc, &ver);
            if ver.is_empty() {
                return (loader, None);
            }
            return (loader, Some(ver));
        }
    }
    (Loader::Vanilla, None)
}

fn strip_mc_prefix(mc: &str, ver: &str) -> String {
    ver.strip_prefix(&format!("{mc}-"))
        .unwrap_or(ver)
        .to_string()
}
