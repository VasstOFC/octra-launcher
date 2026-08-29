use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use crate::config;
use crate::download::{self, download_file};
use crate::error::{Error, Result};
use crate::icon;
use crate::install::{emit_progress, InstallProgress};
use crate::instances::{self, ContentFile, ContentKind, CreateInstance, Instance, Loader};
use crate::paths::Dirs;
use crate::settings::Settings;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackIndex {
    name: String,
    #[serde(default)]
    files: Vec<PackFile>,
    #[serde(default)]
    dependencies: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct PackFile {
    path: String,
    hashes: PackHashes,
    #[serde(default)]
    env: Option<PackEnv>,
    downloads: Vec<String>,
    #[serde(default, rename = "fileSize")]
    file_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct PackHashes {
    #[serde(default)]
    sha1: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct PackEnv {
    #[serde(default)]
    client: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ModrinthProject {
    #[serde(default)]
    id: String,
    #[serde(default)]
    slug: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    project_type: String,
    #[serde(default)]
    icon_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ModrinthVersion {
    id: String,
    #[serde(default)]
    project_id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    version_number: String,
    #[serde(default)]
    version_type: String,
    #[serde(default)]
    date_published: String,
    #[serde(default)]
    downloads: u64,
    #[serde(default)]
    files: Vec<ModrinthFile>,
    #[serde(default)]
    game_versions: Vec<String>,
    #[serde(default)]
    loaders: Vec<String>,
    #[serde(default)]
    dependencies: Vec<ModrinthDependency>,
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
    hashes: PackHashes,
}

#[derive(Debug, Clone, Deserialize)]
struct ModrinthDependency {
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    version_id: Option<String>,
    #[serde(default)]
    dependency_type: String,
}

pub async fn pick_mrpack_file(kind: Option<&str>) -> Option<String> {
    let mut dlg = rfd::AsyncFileDialog::new().set_title("Importuj paczkę");
    match kind.map(|s| s.to_ascii_lowercase()).as_deref() {
        Some("zip") | Some("curseforge") => {
            dlg = dlg.add_filter("CurseForge (.zip)", &["zip"]);
        }
        Some("mrpack") | Some("modrinth") => {
            dlg = dlg.add_filter("Modrinth (.mrpack)", &["mrpack"]);
        }
        _ => {
            dlg = dlg
                .add_filter("Paczki Minecraft", &["mrpack", "zip"])
                .add_filter("Modrinth (.mrpack)", &["mrpack"])
                .add_filter("CurseForge (.zip)", &["zip"]);
        }
    }
    dlg.pick_file()
        .await
        .map(|p| p.path().to_string_lossy().to_string())
}

pub fn parse_modrinth_query(query: &str) -> Result<(String, Option<String>)> {
    let q = query.trim();
    if q.is_empty() {
        return Err(Error::msg("Wklej slug albo link do modpacka Modrinth."));
    }
    let q = q.trim_end_matches('/');
    let rest = q
        .strip_prefix("https://")
        .or_else(|| q.strip_prefix("http://"))
        .unwrap_or(q);
    let rest = rest.strip_prefix("www.").unwrap_or(rest);
    if let Some(rest) = rest.strip_prefix("modrinth.com/modpack/") {
        let mut parts = rest.split('/');
        let slug = parts.next().unwrap_or("").trim().to_string();
        if slug.is_empty() {
            return Err(Error::msg("Niepoprawny link Modrinth."));
        }
        let version = match parts.next() {
            Some("version") => parts.next().map(|s| s.to_string()),
            Some("versions") => None,
            Some(other) if !other.is_empty() => Some(other.to_string()),
            _ => None,
        };
        return Ok((slug, version));
    }
    if q.contains('/') || q.contains(' ') {
        return Err(Error::msg(
            "Podaj slug (np. fabulously-optimized) albo pełny link do modpacka.",
        ));
    }
    Ok((q.to_string(), None))
}

pub fn is_catalog_pack_slug(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty()
        || s.eq_ignore_ascii_case("mrpack")
        || s.eq_ignore_ascii_case("curseforge")
    {
        return false;
    }
    parse_modrinth_query(s).is_ok()
}

pub async fn download_modrinth_mrpack(
    client: &reqwest::Client,
    dirs: &Dirs,
    slug: &str,
    version_id: Option<&str>,
) -> Result<PathBuf> {
    let project: ModrinthProject = download::download_json(
        client,
        &format!("https://api.modrinth.com/v2/project/{slug}"),
    )
    .await
    .map_err(|_| Error::msg(format!("Nie znaleziono projektu Modrinth „{slug}")))?;
    if project.project_type != "modpack" && !project.project_type.is_empty() {
        return Err(Error::msg(format!(
            "„{}” to nie modpack (typ: {}).",
            project.title, project.project_type
        )));
    }
    let versions: Vec<ModrinthVersion> = download::download_json(
        client,
        &format!("https://api.modrinth.com/v2/project/{slug}/version"),
    )
    .await?;
    let versions: Vec<ModrinthVersion> = versions
        .into_iter()
        .filter(|v| {
            v.files
                .iter()
                .any(|f| f.filename.ends_with(".mrpack") || f.url.ends_with(".mrpack"))
        })
        .collect();
    let version = if let Some(id) = version_id {
        versions
            .into_iter()
            .find(|v| v.id == id || v.version_number == id)
            .ok_or_else(|| Error::msg("Nie znaleziono tej wersji modpacka."))?
    } else {
        versions
            .into_iter()
            .next()
            .ok_or_else(|| Error::msg("Ten projekt nie ma żadnej wersji .mrpack."))?
    };
    let file = version
        .files
        .iter()
        .find(|f| f.primary && f.filename.ends_with(".mrpack"))
        .or_else(|| {
            version
                .files
                .iter()
                .find(|f| f.filename.ends_with(".mrpack") || f.url.ends_with(".mrpack"))
        })
        .ok_or_else(|| Error::msg("W tej wersji nie ma pliku .mrpack."))?;
    let dest = dirs.cache.join(format!("modrinth-{}-{}.mrpack", slug, version.id));
    download_file(client, &file.url, &dest, None, None, None).await?;
    Ok(dest)
}

const SEARCH_PAGE: u32 = 20;
const MAX_SEARCH_LIMIT: u32 = 50;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthPackHit {
    pub slug: String,
    pub project_id: Option<String>,
    pub title: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub follows: u64,
    pub categories: Vec<String>,
    pub loaders: Vec<String>,
    pub game_versions: Vec<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthSearchResult {
    pub hits: Vec<ModrinthPackHit>,
    pub offset: u32,
    pub limit: u32,
    pub total_hits: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthContentDepInfo {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub dependency_type: String,
    pub title: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthContentVersion {
    pub id: String,
    pub version_number: String,
    pub version_name: String,
    pub version_type: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub date_published: Option<String>,
    pub downloads: u64,
    pub dependencies: Vec<ModrinthContentDepInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthContentVersions {
    pub project_title: String,
    pub project_slug: String,
    pub project_type: String,
    pub versions: Vec<ModrinthContentVersion>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallContentResult {
    pub files: Vec<ContentFile>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ModrinthSearchApi {
    #[serde(default)]
    hits: Vec<ModrinthSearchHitApi>,
    #[serde(default)]
    offset: u32,
    #[serde(default)]
    limit: u32,
    #[serde(default)]
    total_hits: u32,
}

#[derive(Debug, Deserialize)]
struct ModrinthSearchHitApi {
    #[serde(default)]
    slug: String,
    #[serde(default)]
    project_id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    icon_url: Option<String>,
    #[serde(default)]
    downloads: u64,
    #[serde(default)]
    follows: u64,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    versions: Vec<String>,
    #[serde(default)]
    author: Option<String>,
}

pub async fn search_modrinth_packs(
    client: &reqwest::Client,
    query: &str,
    offset: u32,
    limit: u32,
    sort: &str,
) -> Result<ModrinthSearchResult> {
    search_modrinth(
        client,
        query,
        offset,
        limit,
        sort,
        "modpack",
        None,
        None,
    )
    .await
}

pub async fn search_modrinth_content(
    client: &reqwest::Client,
    query: &str,
    offset: u32,
    limit: u32,
    sort: &str,
    project_type: &str,
    game_version: Option<&str>,
    loader: Option<&str>,
) -> Result<ModrinthSearchResult> {
    let ptype = normalize_project_type(project_type)?;
    search_modrinth(
        client,
        query,
        offset,
        limit,
        sort,
        ptype,
        game_version,
        loader,
    )
    .await
}

async fn search_modrinth(
    client: &reqwest::Client,
    query: &str,
    offset: u32,
    limit: u32,
    sort: &str,
    project_type: &str,
    game_version: Option<&str>,
    loader: Option<&str>,
) -> Result<ModrinthSearchResult> {
    let limit = limit.clamp(1, MAX_SEARCH_LIMIT);
    let index = match sort {
        "relevance" | "downloads" | "follows" | "newest" | "updated" => sort,
        _ => "downloads",
    };
    let facets = search_facets(project_type, game_version, loader);
    let mut req = client
        .get("https://api.modrinth.com/v2/search")
        .query(&[
            ("offset", offset.to_string()),
            ("limit", limit.to_string()),
            ("index", index.to_string()),
            ("facets", facets),
        ]);
    let q = query.trim();
    if !q.is_empty() {
        req = req.query(&[("query", q)]);
    }
    let kind_label = match project_type {
        "modpack" => "modpacków",
        "mod" => "modów",
        "shader" => "shaderów",
        "resourcepack" => "paczek zasobów",
        _ => "projektów",
    };
    let api: ModrinthSearchApi = req
        .send()
        .await?
        .error_for_status()
        .map_err(|_| {
            Error::msg(format!(
                "Nie udało się pobrać listy {kind_label} z Modrinth."
            ))
        })?
        .json()
        .await
        .map_err(|_| {
            Error::msg(format!(
                "Modrinth zwrócił niepoprawną listę {kind_label}."
            ))
        })?;
    Ok(ModrinthSearchResult {
        hits: api
            .hits
            .into_iter()
            .filter(|h| !h.slug.is_empty())
            .map(|h| {
                let loaders = loaders_from_categories(&h.categories);
                let game_versions = pick_game_versions(&h.versions);
                ModrinthPackHit {
                    slug: h.slug,
                    project_id: if h.project_id.is_empty() {
                        None
                    } else {
                        Some(h.project_id)
                    },
                    title: if h.title.is_empty() {
                        "Bez nazwy".into()
                    } else {
                        h.title
                    },
                    description: h.description,
                    icon_url: h.icon_url.filter(|s| !s.is_empty()),
                    downloads: h.downloads,
                    follows: h.follows,
                    categories: h.categories,
                    loaders,
                    game_versions,
                    author: h.author.filter(|s| !s.is_empty()),
                }
            })
            .collect(),
        offset: api.offset,
        limit: api.limit,
        total_hits: api.total_hits,
    })
}

pub fn search_page_size() -> u32 {
    SEARCH_PAGE
}

fn normalize_project_type(project_type: &str) -> Result<&'static str> {
    match project_type.trim().to_ascii_lowercase().as_str() {
        "mod" | "mods" => Ok("mod"),
        "shader" | "shaders" | "shaderpack" | "shaderpacks" => Ok("shader"),
        "resourcepack" | "resourcepacks" | "resource-pack" => Ok("resourcepack"),
        "datapack" | "datapacks" | "data-pack" => Ok("datapack"),
        "modpack" => Ok("modpack"),
        other => Err(Error::msg(format!(
            "Nieobsługiwany typ projektu Modrinth: {other}."
        ))),
    }
}

fn search_facets(
    project_type: &str,
    game_version: Option<&str>,
    loader: Option<&str>,
) -> String {
    let mut facets: Vec<Vec<String>> = vec![vec![format!("project_type:{project_type}")]];
    if let Some(ver) = game_version.map(str::trim).filter(|s| !s.is_empty()) {
        facets.push(vec![format!("versions:{ver}")]);
    }
    let cats = loader_category_facets(project_type, loader);
    if !cats.is_empty() {
        facets.push(cats);
    }
    serde_json::to_string(&facets)
        .unwrap_or_else(|_| format!(r#"[["project_type:{project_type}"]]"#))
}

fn loader_category_facets(project_type: &str, loader: Option<&str>) -> Vec<String> {
    if project_type != "mod" {
        return Vec::new();
    }
    match loader.map(str::trim).unwrap_or("") {
        "fabric" => vec!["categories:fabric".into()],
        "quilt" => vec!["categories:quilt".into(), "categories:fabric".into()],
        "forge" => vec!["categories:forge".into()],
        "neoforge" => vec!["categories:neoforge".into()],
        _ => Vec::new(),
    }
}

fn loaders_from_categories(categories: &[String]) -> Vec<String> {
    const ORDER: &[&str] = &[
        "fabric", "quilt", "neoforge", "forge", "iris", "optifine", "canvas",
    ];
    ORDER
        .iter()
        .filter(|loader| {
            categories
                .iter()
                .any(|c| c.eq_ignore_ascii_case(loader))
        })
        .map(|s| (*s).to_string())
        .collect()
}

fn pick_game_versions(versions: &[String]) -> Vec<String> {
    let mut out: Vec<String> = versions
        .iter()
        .filter(|v| v.contains('.') && v.chars().any(|c| c.is_ascii_digit()))
        .cloned()
        .collect();
    out.sort_by(|a, b| cmp_mc_version(b, a));
    out.dedup();
    out.truncate(6);
    out
}

fn cmp_mc_version(a: &str, b: &str) -> std::cmp::Ordering {
    version_parts(a).cmp(&version_parts(b))
}

fn version_parts(s: &str) -> Vec<u32> {
    s.split(|c: char| !c.is_ascii_digit())
        .filter_map(|p| if p.is_empty() { None } else { p.parse().ok() })
        .collect()
}

pub fn incomplete_pack_error(path: &Path) -> Error {
    let featured = Path::new(config::FEATURED_PACK)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Cobblemon vasst 1.0.0.mrpack");
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    if name.eq_ignore_ascii_case(featured) {
        Error::msg(format!(
            "Paczka wbudowana jest niekompletna (ucięty plik ZIP). Podmień `packs/{featured}` na pełne archiwum, potem usuń starą kopię z %APPDATA%\\.lumenlauncher\\packs\\."
        ))
    } else {
        Error::msg(format!(
            "Plik .mrpack jest niekompletny lub uszkodzony (ucięty ZIP — brak końca archiwum): {}",
            path.display()
        ))
    }
}

pub fn validate_mrpack(path: &Path) -> Result<()> {
    download::validate_zip_archive(path).map_err(|_| incomplete_pack_error(path))?;
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(file).map_err(|_| incomplete_pack_error(path))?;
    if zip.by_name("modrinth.index.json").is_err() {
        return Err(Error::msg(format!(
            "To nie jest poprawny plik .mrpack (brak modrinth.index.json): {}",
            path.display()
        )));
    }
    Ok(())
}

pub async fn import_mrpack(
    _client: &reqwest::Client,
    app: &AppHandle,
    _dirs: &Dirs,
    _settings: &Settings,
    pack_path: &Path,
    _cancel: &CancellationToken,
) -> Result<CreateInstance> {
    if !pack_path.exists() {
        return Err(Error::msg("Nie znaleziono pliku .mrpack."));
    }
    validate_mrpack(pack_path)?;
    let index_raw = download::read_zip_text(pack_path, "modrinth.index.json")?;
    let index: PackIndex = serde_json::from_str(&index_raw)?;
    let req = pack_to_create(&index)?;
    emit_progress(
        app,
        InstallProgress {
            instance_id: String::new(),
            stage: "modpack".into(),
            current: 0,
            total: 1,
            file: Some(index.name.clone()),
            message: format!("Modpack: {}", index.name),
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
    let index_raw = download::read_zip_text(pack_path, "modrinth.index.json")?;
    let index: PackIndex = serde_json::from_str(&index_raw)?;
    let game_dir = dirs.game_dir(&inst.id);
    std::fs::create_dir_all(&game_dir)?;
    let files: Vec<&PackFile> = index
        .files
        .iter()
        .filter(|f| f.env.as_ref().and_then(|e| e.client.as_deref()) != Some("unsupported"))
        .collect();
    let total = files.len() as u64;
    for (i, file) in files.iter().enumerate() {
        if cancel.is_cancelled() {
            return Err(Error::msg("Instalacja anulowana."));
        }
        if file.path.contains("..") {
            continue;
        }
        let dest = game_dir.join(&file.path);
        let url = file
            .downloads
            .first()
            .ok_or_else(|| Error::msg(format!("Brak URL dla {}", file.path)))?;
        emit_progress(
            app,
            InstallProgress {
                instance_id: inst.id.clone(),
                stage: "modpack".into(),
                current: i as u64 + 1,
                total: total.max(1),
                file: Some(file.path.clone()),
                message: format!("Pliki modpacka {}/{}", i + 1, total),
            },
        );
        download_file(
            client,
            url,
            &dest,
            file.hashes.sha1.as_deref(),
            file.file_size,
            None,
        )
        .await?;
    }
    let pack = pack_path.to_path_buf();
    let dest = game_dir.clone();
    tokio::task::spawn_blocking(move || {
        download::extract_zip_prefix(&pack, &dest, "overrides")?;
        download::extract_zip_prefix(&pack, &dest, "client-overrides")
    })
    .await
    .map_err(|e| Error::msg(e.to_string()))??;
    Ok(())
}

/// Ustaw logo instancji przy pierwszym imporcie paczki. Błąd ikony nie psuje instalacji.
pub async fn apply_pack_icon(
    client: &reqwest::Client,
    dirs: &Dirs,
    inst: &mut Instance,
    pack_path: &Path,
    linked_pack: &str,
    icon_url: Option<&str>,
) {
    if let Some(bytes) = icon::extract_from_zip(pack_path) {
        if icon::install_icon_bytes(dirs, inst, &bytes).is_ok() && inst.icon_path.is_some() {
            let _ = instances::save(dirs, inst);
            return;
        }
    }
    let mut url = icon_url
        .map(str::trim)
        .filter(|s| !s.is_empty() && (*s).starts_with("http"))
        .map(|s| s.to_string());
    if url.is_none() {
        url = fetch_modrinth_icon_url(client, linked_pack).await;
    }
    if let Some(url) = url {
        if let Some(bytes) = download_icon_bytes(client, &url).await {
            if icon::install_icon_bytes(dirs, inst, &bytes).is_ok() && inst.icon_path.is_some() {
                let _ = instances::save(dirs, inst);
                return;
            }
        }
    }
}

pub fn adopt_extracted_icon(dirs: &Dirs, inst: &mut Instance) {
    if inst.icon_path.is_some() {
        return;
    }
    if icon::adopt_from_game_dir(dirs, inst).unwrap_or(false) {
        let _ = instances::save(dirs, inst);
    }
}

async fn fetch_modrinth_icon_url(client: &reqwest::Client, slug: &str) -> Option<String> {
    let slug = slug.trim();
    if slug.is_empty()
        || slug.eq_ignore_ascii_case("mrpack")
        || slug.eq_ignore_ascii_case("curseforge")
    {
        return None;
    }
    if slug.contains('/') || slug.contains('\\') || slug.contains(' ') {
        return None;
    }
    let project: ModrinthProject = download::download_json(
        client,
        &format!("https://api.modrinth.com/v2/project/{slug}"),
    )
    .await
    .ok()?;
    project.icon_url.filter(|s| !s.is_empty() && s.starts_with("http"))
}

async fn download_icon_bytes(client: &reqwest::Client, url: &str) -> Option<Vec<u8>> {
    let resp = client.get(url).send().await.ok()?.error_for_status().ok()?;
    let bytes = resp.bytes().await.ok()?;
    if bytes.is_empty() || bytes.len() > 2 * 1024 * 1024 {
        return None;
    }
    Some(bytes.to_vec())
}

fn pack_to_create(index: &PackIndex) -> Result<CreateInstance> {
    let game = index
        .dependencies
        .get("minecraft")
        .cloned()
        .ok_or_else(|| Error::msg("W paczce brak wersji Minecraft."))?;
    let (loader, loader_version) = if let Some(v) = index.dependencies.get("fabric-loader") {
        (Loader::Fabric, Some(v.clone()))
    } else if let Some(v) = index.dependencies.get("quilt-loader") {
        (Loader::Quilt, Some(v.clone()))
    } else if let Some(v) = index.dependencies.get("neoforge") {
        (Loader::Neoforge, Some(strip_mc_prefix(&game, v)))
    } else if let Some(v) = index.dependencies.get("forge") {
        (Loader::Forge, Some(strip_mc_prefix(&game, v)))
    } else {
        (Loader::Vanilla, None)
    };
    Ok(CreateInstance {
        name: index.name.clone(),
        game_version: game,
        loader,
        loader_version,
        memory_max_mb: None,
    })
}

fn strip_mc_prefix(mc: &str, ver: &str) -> String {
    ver.strip_prefix(&format!("{mc}-"))
        .unwrap_or(ver)
        .to_string()
}

const MAX_CONTENT_DEP_DEPTH: u32 = 8;
const MAX_CONTENT_FILES: usize = 32;
const MAX_LISTED_VERSIONS: usize = 120;

struct ContentJob {
    id: String,
    version_id: Option<String>,
    expected_type: Option<&'static str>,
    depth: u32,
    strict_version: bool,
    published_before: Option<String>,
}

struct InstallOutcome {
    next: Vec<ContentJob>,
    warnings: Vec<String>,
}

pub async fn list_modrinth_content_versions(
    client: &reqwest::Client,
    inst: &Instance,
    slug: &str,
    project_type: Option<&str>,
) -> Result<ModrinthContentVersions> {
    let slug = slug.trim();
    if slug.is_empty() {
        return Err(Error::msg("Brak sluga projektu Modrinth."));
    }
    let expected = project_type
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(normalize_project_type)
        .transpose()?;
    let project: ModrinthProject = download::download_json(
        client,
        &format!("https://api.modrinth.com/v2/project/{slug}"),
    )
    .await
    .map_err(|_| Error::msg(format!("Nie znaleziono projektu Modrinth „{slug}”.")))?;
    let ptype = if project.project_type.trim().is_empty() {
        expected.unwrap_or("mod")
    } else {
        normalize_project_type(&project.project_type)?
    };
    if let Some(expected) = expected {
        if ptype != expected {
            return Err(Error::msg(format!(
                "„{}” to {}, a nie {}.",
                display_title(&project),
                type_label(ptype),
                type_label(expected)
            )));
        }
    }
    if ptype == "modpack" {
        return Err(Error::msg(
            "To modpack — importuj go z Galerii, nie z zawartości instancji.",
        ));
    }
    let mut versions = compatible_versions(client, inst, slug, ptype).await?;
    versions.truncate(MAX_LISTED_VERSIONS);
    let mut dep_ids = Vec::new();
    for v in &versions {
        for d in &v.dependencies {
            if let Some(id) = dep_project_id(d) {
                if !dep_ids.iter().any(|x: &String| x == id) {
                    dep_ids.push(id.to_string());
                }
            }
        }
    }
    let meta = fetch_projects_meta(client, &dep_ids).await;
    Ok(ModrinthContentVersions {
        project_title: display_title(&project),
        project_slug: if project.slug.is_empty() {
            slug.to_string()
        } else {
            project.slug
        },
        project_type: ptype.to_string(),
        versions: versions
            .into_iter()
            .map(|v| map_content_version(v, &meta))
            .collect(),
    })
}

pub async fn install_modrinth_content(
    client: &reqwest::Client,
    dirs: &Dirs,
    inst: &Instance,
    slug: &str,
    project_type: Option<&str>,
    version_id: Option<&str>,
    optional_project_ids: &[String],
) -> Result<InstallContentResult> {
    instances::ensure_unlocked(inst)?;
    let expected = project_type
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(normalize_project_type)
        .transpose()?;
    let optional: HashSet<String> = optional_project_ids
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let mut installed_names = existing_mod_names(dirs, inst);
    let root_version = version_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let mut seen = HashSet::new();
    let mut warnings = Vec::new();
    let mut queue = vec![ContentJob {
        id: slug.trim().to_string(),
        version_id: root_version.clone(),
        expected_type: expected,
        depth: 0,
        strict_version: root_version.is_some(),
        published_before: None,
    }];
    while let Some(job) = queue.pop() {
        match install_one(
            client,
            dirs,
            inst,
            &job,
            &mut seen,
            &optional,
            &mut installed_names,
        )
        .await
        {
            Ok(out) => {
                warnings.extend(out.warnings);
                if job.depth >= MAX_CONTENT_DEP_DEPTH {
                    continue;
                }
                queue.extend(out.next);
            }
            Err(e) if job.depth == 0 => return Err(e),
            Err(_) => continue,
        }
    }
    Ok(InstallContentResult {
        files: instances::list_content(dirs, &inst.id)?,
        warnings,
    })
}

async fn install_one(
    client: &reqwest::Client,
    dirs: &Dirs,
    inst: &Instance,
    job: &ContentJob,
    seen: &mut HashSet<String>,
    optional: &HashSet<String>,
    installed_names: &mut Vec<String>,
) -> Result<InstallOutcome> {
    if job.id.is_empty() {
        return Err(Error::msg("Brak sluga projektu Modrinth."));
    }
    if seen.len() >= MAX_CONTENT_FILES {
        return Ok(InstallOutcome {
            next: Vec::new(),
            warnings: Vec::new(),
        });
    }
    let project: ModrinthProject = download::download_json(
        client,
        &format!("https://api.modrinth.com/v2/project/{}", job.id),
    )
    .await
    .map_err(|_| Error::msg(format!("Nie znaleziono projektu Modrinth „{}”.", job.id)))?;
    let key = project_key(&project, &job.id);
    if !seen.insert(key.clone()) {
        return Ok(InstallOutcome {
            next: Vec::new(),
            warnings: Vec::new(),
        });
    }
    if !project.id.is_empty() && project.id != key {
        seen.insert(project.id.clone());
    }
    if !project.slug.is_empty() && project.slug != key {
        seen.insert(project.slug.clone());
    }
    let ptype = if project.project_type.trim().is_empty() {
        job.expected_type.unwrap_or("mod")
    } else {
        normalize_project_type(&project.project_type)?
    };
    if let Some(expected) = job.expected_type {
        if ptype != expected && job.depth == 0 {
            return Err(Error::msg(format!(
                "„{}” to {}, a nie {}.",
                display_title(&project),
                type_label(ptype),
                type_label(expected)
            )));
        }
    }
    if ptype == "modpack" {
        return Err(Error::msg(
            "To modpack — importuj go z Galerii, nie z zawartości instancji.",
        ));
    }
    let kind = ContentKind::from_modrinth_type(ptype)?;
    let version = resolve_version(
        client,
        inst,
        &job.id,
        ptype,
        &project,
        job.version_id.as_deref(),
        job.strict_version,
        job.published_before.as_deref(),
    )
    .await?;
    let file = pick_content_file(&version.files, ptype).ok_or_else(|| {
        Error::msg(format!(
            "W wersji „{}” nie ma pliku .jar/.zip.",
            display_version(&version)
        ))
    })?;
    let filename = content_filename(file)?;
    if !instances::content_file_present(dirs, &inst.id, kind, &filename) {
        let dest = instances::prepare_content_dest(dirs, &inst.id, kind, &filename)?;
        download_file(
            client,
            &file.url,
            &dest,
            file.hashes.sha1.as_deref(),
            file.size,
            None,
        )
        .await?;
    }
    let lower = filename.to_ascii_lowercase();
    if !installed_names.iter().any(|n| n == &lower) {
        installed_names.push(lower);
    }
    instances::record_content_meta(
        dirs,
        &inst.id,
        kind,
        &filename,
        if project.slug.is_empty() {
            None
        } else {
            Some(project.slug.clone())
        },
        if project.id.is_empty() {
            None
        } else {
            Some(project.id.clone())
        },
    );
    collect_followups(client, job, &version, optional, installed_names).await
}

async fn collect_followups(
    client: &reqwest::Client,
    job: &ContentJob,
    version: &ModrinthVersion,
    optional: &HashSet<String>,
    installed_names: &[String],
) -> Result<InstallOutcome> {
    let parent_published = if version.date_published.is_empty() {
        None
    } else {
        Some(version.date_published.as_str())
    };
    let mut next = Vec::new();
    let mut warnings = Vec::new();
    for d in &version.dependencies {
        let dtype = d.dependency_type.trim().to_ascii_lowercase();
        match dtype.as_str() {
            "embedded" => continue,
            "incompatible" => {
                if let Some(warning) =
                    incompatible_warning(client, d, installed_names).await
                {
                    warnings.push(warning);
                }
            }
            "optional" => {
                if job.depth != 0 {
                    continue;
                }
                if let Some(pid) = dep_project_id(d) {
                    if optional.contains(pid) {
                        if let Some(dep_job) = dep_to_job(d, job.depth + 1, parent_published) {
                            next.push(dep_job);
                        }
                    }
                }
            }
            "required" => {
                if let Some(dep_job) = dep_to_job(d, job.depth + 1, parent_published) {
                    next.push(dep_job);
                }
            }
            _ => {}
        }
    }
    Ok(InstallOutcome { next, warnings })
}

fn dep_to_job(
    d: &ModrinthDependency,
    depth: u32,
    parent_published: Option<&str>,
) -> Option<ContentJob> {
    let id = dep_project_id(d)?.to_string();
    let version_id = d
        .version_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    Some(ContentJob {
        id,
        version_id,
        expected_type: None,
        depth,
        strict_version: false,
        published_before: parent_published.map(str::to_string),
    })
}

fn dep_project_id(d: &ModrinthDependency) -> Option<&str> {
    d.project_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

async fn incompatible_warning(
    client: &reqwest::Client,
    d: &ModrinthDependency,
    installed_names: &[String],
) -> Option<String> {
    let pid = dep_project_id(d)?;
    let project: ModrinthProject = download::download_json(
        client,
        &format!("https://api.modrinth.com/v2/project/{pid}"),
    )
    .await
    .ok()?;
    let slug = if project.slug.is_empty() {
        pid.to_string()
    } else {
        project.slug.clone()
    };
    let title = display_title(&project);
    if looks_installed(installed_names, &slug, &title) {
        Some(format!(
            "„{title}” jest oznaczony jako niezgodny z wybraną wersją, a wygląda na już zainstalowany."
        ))
    } else {
        None
    }
}

fn existing_mod_names(dirs: &Dirs, inst: &Instance) -> Vec<String> {
    instances::list_content(dirs, &inst.id)
        .unwrap_or_default()
        .into_iter()
        .map(|f| f.display_name.to_ascii_lowercase())
        .collect()
}

fn looks_installed(names: &[String], slug: &str, title: &str) -> bool {
    let slug = slug.trim().to_ascii_lowercase();
    let title_slug = title
        .trim()
        .to_ascii_lowercase()
        .replace(' ', "-")
        .replace('_', "-");
    names.iter().any(|n| {
        let stem = n
            .rsplit_once('.')
            .map(|(a, _)| a)
            .unwrap_or(n)
            .replace('_', "-");
        (!slug.is_empty()
            && (stem == slug
                || stem.starts_with(&format!("{slug}-"))
                || stem.starts_with(&format!("{slug}+"))))
            || (!title_slug.is_empty()
                && title_slug.len() >= 4
                && (stem == title_slug || stem.starts_with(&format!("{title_slug}-"))))
    })
}

fn project_key(project: &ModrinthProject, fallback: &str) -> String {
    if !project.id.is_empty() {
        project.id.clone()
    } else if !project.slug.is_empty() {
        project.slug.clone()
    } else {
        fallback.to_string()
    }
}

fn display_version(version: &ModrinthVersion) -> &str {
    if version.version_number.is_empty() {
        version.id.as_str()
    } else {
        version.version_number.as_str()
    }
}

async fn resolve_version(
    client: &reqwest::Client,
    inst: &Instance,
    id_or_slug: &str,
    project_type: &str,
    project: &ModrinthProject,
    version_id: Option<&str>,
    strict: bool,
    published_before: Option<&str>,
) -> Result<ModrinthVersion> {
    if let Some(vid) = version_id.map(str::trim).filter(|s| !s.is_empty()) {
        match fetch_version_by_id(client, vid).await {
            Ok(v) => return Ok(v),
            Err(e) if strict => return Err(e),
            Err(_) => {}
        }
    }
    pick_compatible_version(
        client,
        inst,
        id_or_slug,
        project_type,
        project,
        published_before,
    )
    .await
}

async fn fetch_version_by_id(
    client: &reqwest::Client,
    version_id: &str,
) -> Result<ModrinthVersion> {
    download::download_json(
        client,
        &format!("https://api.modrinth.com/v2/version/{version_id}"),
    )
    .await
    .map_err(|_| Error::msg("Nie znaleziono wybranej wersji na Modrinth."))
}

async fn compatible_versions(
    client: &reqwest::Client,
    inst: &Instance,
    id_or_slug: &str,
    project_type: &str,
) -> Result<Vec<ModrinthVersion>> {
    let loaders = version_loaders(inst.loader, project_type);
    let mut versions = fetch_project_versions(
        client,
        id_or_slug,
        Some(inst.game_version.as_str()),
        &loaders,
    )
    .await?;
    if versions.is_empty() && !loaders.is_empty() {
        versions = fetch_project_versions(
            client,
            id_or_slug,
            Some(inst.game_version.as_str()),
            &[],
        )
        .await?;
        versions.retain(|v| version_matches_loader(v, inst.loader, project_type));
    }
    versions.sort_by(|a, b| {
        b.date_published
            .cmp(&a.date_published)
            .then_with(|| b.id.cmp(&a.id))
    });
    Ok(versions)
}

async fn pick_compatible_version(
    client: &reqwest::Client,
    inst: &Instance,
    id_or_slug: &str,
    project_type: &str,
    project: &ModrinthProject,
    published_before: Option<&str>,
) -> Result<ModrinthVersion> {
    let versions = compatible_versions(client, inst, id_or_slug, project_type).await?;
    let chosen = if let Some(before) = published_before.map(str::trim).filter(|s| !s.is_empty())
    {
        versions
            .iter()
            .find(|v| v.date_published.is_empty() || v.date_published.as_str() <= before)
            .cloned()
            .or_else(|| versions.first().cloned())
    } else {
        versions.first().cloned()
    };
    chosen.ok_or_else(|| {
        let loader = inst.loader.as_str();
        Error::msg(format!(
            "Brak wersji „{}” pasującej do {} {}.",
            display_title(project),
            loader,
            inst.game_version
        ))
    })
}

fn map_content_version(
    v: ModrinthVersion,
    meta: &HashMap<String, ModrinthProject>,
) -> ModrinthContentVersion {
    ModrinthContentVersion {
        id: v.id,
        version_number: if v.version_number.is_empty() {
            v.name.clone()
        } else {
            v.version_number.clone()
        },
        version_name: v.name,
        version_type: if v.version_type.is_empty() {
            "release".into()
        } else {
            v.version_type
        },
        game_versions: v.game_versions,
        loaders: v.loaders,
        date_published: if v.date_published.is_empty() {
            None
        } else {
            Some(v.date_published)
        },
        downloads: v.downloads,
        dependencies: v
            .dependencies
            .iter()
            .map(|d| map_content_dep(d, meta))
            .collect(),
    }
}

fn map_content_dep(
    d: &ModrinthDependency,
    meta: &HashMap<String, ModrinthProject>,
) -> ModrinthContentDepInfo {
    let pid = dep_project_id(d).map(str::to_string);
    let info = pid.as_deref().and_then(|id| {
        meta.get(id).or_else(|| {
            meta.values().find(|p| p.id == id || p.slug == id)
        })
    });
    ModrinthContentDepInfo {
        project_id: pid,
        version_id: d
            .version_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        dependency_type: d.dependency_type.clone(),
        title: info.map(display_title),
        slug: info.and_then(|p| {
            if p.slug.is_empty() {
                None
            } else {
                Some(p.slug.clone())
            }
        }),
    }
}

async fn fetch_projects_meta(
    client: &reqwest::Client,
    ids: &[String],
) -> HashMap<String, ModrinthProject> {
    let mut out = HashMap::new();
    for chunk in ids.chunks(40) {
        if chunk.is_empty() {
            continue;
        }
        let encoded = match serde_json::to_string(chunk) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let Ok(resp) = client
            .get("https://api.modrinth.com/v2/projects")
            .query(&[("ids", encoded)])
            .send()
            .await
        else {
            continue;
        };
        if !resp.status().is_success() {
            continue;
        }
        let Ok(projects) = resp.json::<Vec<ModrinthProject>>().await else {
            continue;
        };
        for p in projects {
            if !p.slug.is_empty() {
                out.insert(p.slug.clone(), p.clone());
            }
            if !p.id.is_empty() {
                out.insert(p.id.clone(), p);
            }
        }
    }
    out
}

async fn fetch_project_versions(
    client: &reqwest::Client,
    id_or_slug: &str,
    game_version: Option<&str>,
    loaders: &[String],
) -> Result<Vec<ModrinthVersion>> {
    let mut req = client.get(format!(
        "https://api.modrinth.com/v2/project/{id_or_slug}/version"
    ));
    if let Some(ver) = game_version.map(str::trim).filter(|s| !s.is_empty()) {
        let encoded = serde_json::to_string(&[ver]).unwrap_or_else(|_| format!(r#"["{ver}"]"#));
        req = req.query(&[("game_versions", encoded)]);
    }
    if !loaders.is_empty() {
        let encoded =
            serde_json::to_string(loaders).unwrap_or_else(|_| format!("[{}]", loaders.join(",")));
        req = req.query(&[("loaders", encoded)]);
    }
    req.send()
        .await?
        .error_for_status()
        .map_err(|_| Error::msg("Nie udało się pobrać wersji z Modrinth."))?
        .json()
        .await
        .map_err(|_| Error::msg("Modrinth zwrócił niepoprawną listę wersji."))
}

fn version_loaders(loader: Loader, project_type: &str) -> Vec<String> {
    if project_type != "mod" {
        return Vec::new();
    }
    match loader {
        Loader::Fabric => vec!["fabric".into()],
        Loader::Quilt => vec!["quilt".into(), "fabric".into()],
        Loader::Forge => vec!["forge".into()],
        Loader::Neoforge => vec!["neoforge".into()],
        Loader::Vanilla => Vec::new(),
    }
}

fn version_matches_loader(version: &ModrinthVersion, loader: Loader, project_type: &str) -> bool {
    if project_type != "mod" || loader == Loader::Vanilla {
        return true;
    }
    if version.loaders.is_empty() {
        return true;
    }
    let wanted = version_loaders(loader, project_type);
    version
        .loaders
        .iter()
        .any(|l| wanted.iter().any(|w| w.eq_ignore_ascii_case(l)))
}

fn pick_content_file<'a>(
    files: &'a [ModrinthFile],
    project_type: &str,
) -> Option<&'a ModrinthFile> {
    let prefer_jar = project_type == "mod";
    let ok = |f: &ModrinthFile| file_ext_ok(&content_filename_raw(f), project_type, prefer_jar);
    files
        .iter()
        .find(|f| f.primary && ok(f))
        .or_else(|| files.iter().find(|f| ok(f)))
        .or_else(|| files.iter().find(|f| f.primary))
        .or_else(|| files.first())
}

fn file_ext_ok(name: &str, project_type: &str, prefer_jar: bool) -> bool {
    let n = name.to_ascii_lowercase();
    let jar = n.ends_with(".jar");
    let zip = n.ends_with(".zip");
    match project_type {
        "mod" if prefer_jar => jar || zip,
        "shader" | "resourcepack" | "datapack" => zip || jar,
        _ => jar || zip,
    }
}

fn content_filename_raw(file: &ModrinthFile) -> String {
    if !file.filename.is_empty() {
        return Path::new(&file.filename)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(file.filename.as_str())
            .to_string();
    }
    file.url
        .split('?')
        .next()
        .unwrap_or(&file.url)
        .rsplit('/')
        .next()
        .unwrap_or("download.bin")
        .to_string()
}

fn content_filename(file: &ModrinthFile) -> Result<String> {
    let name = content_filename_raw(file);
    if name.is_empty() || name == "download.bin" {
        return Err(Error::msg("Brak nazwy pliku w wersji Modrinth."));
    }
    Ok(name)
}

fn display_title(project: &ModrinthProject) -> String {
    if !project.title.is_empty() {
        project.title.clone()
    } else if !project.slug.is_empty() {
        project.slug.clone()
    } else {
        project.id.clone()
    }
}

fn type_label(project_type: &str) -> &'static str {
    match project_type {
        "mod" => "mod",
        "shader" => "shader",
        "resourcepack" => "paczka zasobów",
        "datapack" => "datapack",
        "modpack" => "modpack",
        _ => "projekt",
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentUpdate {
    pub name: String,
    pub kind: ContentKind,
    pub file_name: String,
    pub slug: Option<String>,
    pub project_title: Option<String>,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
}

pub async fn check_content_updates(
    client: &reqwest::Client,
    dirs: &Dirs,
    inst: &Instance,
) -> Result<Vec<ContentUpdate>> {
    let files = instances::list_content(dirs, &inst.id)?;
    let mut out = Vec::new();
    for file in files {
        if !file.enabled {
            continue;
        }
        let path = dirs
            .game_dir(&inst.id)
            .join(file.kind.dir_name())
            .join(&file.name);
        if !path.is_file() {
            continue;
        }
        let hash = match crate::download::file_sha1_sync(&path) {
            Ok(h) => h,
            Err(_) => continue,
        };
        let version: ModrinthVersion = match client
            .get(format!("https://api.modrinth.com/v2/version_file/{hash}"))
            .query(&[("algorithm", "sha1")])
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => match resp.json().await {
                Ok(v) => v,
                Err(_) => continue,
            },
            _ => continue,
        };
        let project_id = match version_project_id(&version) {
            Some(id) => id,
            None => continue,
        };
        let project: ModrinthProject = match download::download_json(
            client,
            &format!("https://api.modrinth.com/v2/project/{project_id}"),
        )
        .await
        {
            Ok(p) => p,
            Err(_) => continue,
        };
        let ptype = normalize_project_type(&project.project_type).unwrap_or("mod");
        let latest = match pick_compatible_version(client, inst, &project_id, ptype, &project, None)
            .await
        {
            Ok(v) => v,
            Err(_) => continue,
        };
        if latest.id == version.id {
            continue;
        }
        out.push(ContentUpdate {
            name: file.display_name.clone(),
            kind: file.kind,
            file_name: file.name,
            slug: Some(if project.slug.is_empty() {
                project_id
            } else {
                project.slug.clone()
            }),
            project_title: Some(display_title(&project)),
            current_version: Some(if version.version_number.is_empty() {
                version.id
            } else {
                version.version_number
            }),
            latest_version: Some(if latest.version_number.is_empty() {
                latest.id
            } else {
                latest.version_number
            }),
        });
    }
    Ok(out)
}

fn version_project_id(version: &ModrinthVersion) -> Option<String> {
    let id = version.project_id.trim();
    if id.is_empty() {
        None
    } else {
        Some(id.to_string())
    }
}

pub async fn export_mrpack(dirs: &Dirs, inst: &Instance, dest: &Path) -> Result<PathBuf> {
    let game_dir = dirs.game_dir(&inst.id);
    if !game_dir.exists() {
        return Err(Error::msg("Folder gry instancji jest pusty."));
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(dest)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut deps = HashMap::new();
    deps.insert("minecraft".to_string(), inst.game_version.clone());
    match inst.loader {
        Loader::Fabric => {
            if let Some(v) = &inst.loader_version {
                deps.insert("fabric-loader".into(), v.clone());
            }
        }
        Loader::Quilt => {
            if let Some(v) = &inst.loader_version {
                deps.insert("quilt-loader".into(), v.clone());
            }
        }
        Loader::Forge => {
            if let Some(v) = &inst.loader_version {
                deps.insert("forge".into(), v.clone());
            }
        }
        Loader::Neoforge => {
            if let Some(v) = &inst.loader_version {
                deps.insert("neoforge".into(), v.clone());
            }
        }
        Loader::Vanilla => {}
    }
    let index = serde_json::json!({
        "formatVersion": 1,
        "game": "minecraft",
        "versionId": "exported",
        "name": inst.name,
        "summary": format!("Eksport z Lumen ({})", inst.game_version),
        "files": [],
        "dependencies": deps,
    });
    zip.start_file("modrinth.index.json", opts)?;
    std::io::Write::write_all(&mut zip, serde_json::to_string_pretty(&index)?.as_bytes())?;

    let skip = [
        "saves",
        "logs",
        "crash-reports",
        "natives",
        ".cache",
        "debug",
        "data",
    ];
    for entry in walkdir::WalkDir::new(&game_dir).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = match entry.path().strip_prefix(&game_dir) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if rel_str.contains("..") {
            continue;
        }
        let top = rel_str.split('/').next().unwrap_or("");
        if skip.iter().any(|s| top.eq_ignore_ascii_case(s)) {
            continue;
        }
        let name = rel
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if name.eq_ignore_ascii_case("session.lock")
            || name.eq_ignore_ascii_case("usernamecache.json")
        {
            continue;
        }
        let zip_path = format!("overrides/{rel_str}");
        zip.start_file(&zip_path, opts)?;
        let bytes = std::fs::read(entry.path())?;
        std::io::Write::write_all(&mut zip, &bytes)?;
    }
    zip.finish()?;
    Ok(dest.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_slug_rejects_file_imports() {
        assert!(is_catalog_pack_slug("fabulously-optimized"));
        assert!(is_catalog_pack_slug("https://modrinth.com/modpack/cobblemon"));
        assert!(!is_catalog_pack_slug("mrpack"));
        assert!(!is_catalog_pack_slug("curseforge"));
        assert!(!is_catalog_pack_slug(""));
        assert!(!is_catalog_pack_slug("packs/foo.mrpack"));
    }
}
