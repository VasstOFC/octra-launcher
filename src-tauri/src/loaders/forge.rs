use std::collections::HashMap;
use std::io::{BufRead, Read};
use std::path::{Path, PathBuf};
use std::process::Stdio;

use serde::{Deserialize, Serialize};

use crate::download::{self, download_file, download_json, download_text};
use crate::error::{Error, Result};
use crate::install::{self, emit_progress, library_files, InstallProgress};
use crate::instances::Loader;
use crate::meta::{self, VersionMeta};
use crate::paths::Dirs;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct InstallProfile {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    json: Option<String>,
    #[serde(default)]
    data: HashMap<String, SideValue>,
    #[serde(default)]
    libraries: Vec<meta::Library>,
    #[serde(default)]
    processors: Vec<Processor>,
    #[serde(default)]
    minecraft: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SideValue {
    #[serde(default)]
    client: Option<String>,
    #[serde(default)]
    server: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Processor {
    #[serde(default)]
    jar: String,
    #[serde(default)]
    classpath: Vec<String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    outputs: HashMap<String, String>,
    #[serde(default)]
    sides: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderVersion {
    pub version: String,
    pub recommended: bool,
}

pub async fn list_forge(client: &reqwest::Client, game: &str) -> Result<Vec<LoaderVersion>> {
    let url = "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";
    let map: HashMap<String, Vec<String>> = match download_json(client, url).await {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };
    let mut recommended = None;
    if let Ok(promo) = download_json::<serde_json::Value>(
        client,
        "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json",
    )
    .await
    {
        recommended = promo
            .get("promos")
            .and_then(|p| p.get(format!("{game}-recommended")))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                promo
                    .get("promos")
                    .and_then(|p| p.get(format!("{game}-latest")))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });
    }
    let mut versions: Vec<LoaderVersion> = map
        .get(game)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .rev()
        .map(|version| {
            let version = version
                .strip_prefix(&format!("{game}-"))
                .unwrap_or(&version)
                .to_string();
            LoaderVersion {
                recommended: recommended.as_ref() == Some(&version),
                version,
            }
        })
        .collect();
    if let Some(rec) = &recommended {
        if let Some(idx) = versions.iter().position(|v| &v.version == rec) {
            let item = versions.remove(idx);
            versions.insert(0, item);
        }
    }
    Ok(versions)
}

pub async fn list_neoforge(client: &reqwest::Client, game: &str) -> Result<Vec<LoaderVersion>> {
    let xml = download_text(
        client,
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml",
    )
    .await?;
    let prefix = match game.strip_prefix("1.") {
        Some(rest) => format!("{rest}."),
        None => return Ok(Vec::new()),
    };
    let re = regex::Regex::new(r"<version>([^<]+)</version>").unwrap();
    let mut versions = Vec::new();
    for cap in re.captures_iter(&xml) {
        let v = cap[1].to_string();
        if v.starts_with(&prefix) && !v.contains("beta") {
            versions.push(LoaderVersion {
                version: v,
                recommended: false,
            });
        }
    }
    versions.reverse();
    if let Some(first) = versions.first_mut() {
        first.recommended = true;
    }
    Ok(versions)
}

pub async fn install_modded(
    client: &reqwest::Client,
    app: &tauri::AppHandle,
    dirs: &Dirs,
    instance_id: &str,
    loader: Loader,
    game_version: &str,
    loader_version: &str,
    java: &Path,
    cancel: &tokio_util::sync::CancellationToken,
) -> Result<String> {
    install::install_vanilla(client, app, dirs, instance_id, game_version, cancel).await?;
    let (installer_url, installer_name) = match loader {
        Loader::Forge => {
            let ver = format!("{game_version}-{loader_version}");
            (
                format!(
                    "https://maven.minecraftforge.net/net/minecraftforge/forge/{ver}/forge-{ver}-installer.jar"
                ),
                format!("forge-{ver}-installer.jar"),
            )
        }
        Loader::Neoforge => (
            format!(
                "https://maven.neoforged.net/releases/net/neoforged/neoforge/{loader_version}/neoforge-{loader_version}-installer.jar"
            ),
            format!("neoforge-{loader_version}-installer.jar"),
        ),
        _ => return Err(Error::msg("Nieobsługiwany loader.")),
    };
    emit_progress(
        app,
        InstallProgress {
            instance_id: instance_id.into(),
            stage: "loader".into(),
            current: 0,
            total: 1,
            file: Some(installer_name.clone()),
            message: "Pobieranie instalatora".into(),
        },
    );
    let installer = dirs.cache.join(&installer_name);
    download_file(client, &installer_url, &installer, None, None, None)
        .await
        .map_err(|_| {
            Error::msg(format!(
                "Nie udało się pobrać instalatora ({installer_name}). Sprawdź wersję loadera."
            ))
        })?;

    match run_processors(client, app, dirs, instance_id, game_version, &installer, java, cancel)
        .await
    {
        Ok(id) => Ok(id),
        Err(proc_err) => {
            emit_progress(
                app,
                InstallProgress {
                    instance_id: instance_id.into(),
                    stage: "loader".into(),
                    current: 0,
                    total: 1,
                    file: None,
                    message: "Zapasowo: oficjalny instalator…".into(),
                },
            );
            match run_official_installer(app, dirs, instance_id, &installer, java, game_version, loader, loader_version)
                .await
            {
                Ok(id) => Ok(id),
                Err(e) => Err(Error::msg(format!(
                    "Instalacja loadera nie powiodła się.\nProcesory: {proc_err}\nInstalator: {e}"
                ))),
            }
        }
    }
}

async fn run_processors(
    client: &reqwest::Client,
    app: &tauri::AppHandle,
    dirs: &Dirs,
    instance_id: &str,
    game_version: &str,
    installer: &Path,
    java: &Path,
    cancel: &tokio_util::sync::CancellationToken,
) -> Result<String> {
    let profile_raw = download::read_zip_text(installer, "install_profile.json")?;
    let profile: InstallProfile = serde_json::from_str(&profile_raw)?;
    let version_json_inner = profile
        .json
        .clone()
        .unwrap_or_else(|| "/version.json".into());
    let version_raw = download::read_zip_text(installer, &version_json_inner)
        .or_else(|_| download::read_zip_text(installer, "version.json"))?;
    let version_meta: VersionMeta = serde_json::from_str(&version_raw)?;
    let version_id = version_meta.id.clone();
    std::fs::create_dir_all(dirs.version_dir(&version_id))?;
    std::fs::write(dirs.version_json(&version_id), serde_json::to_string_pretty(&version_meta)?)?;

    let mut libs = profile.libraries.clone();
    libs.extend(version_meta.libraries.clone());
    let mut files = Vec::new();
    for lib in &libs {
        files.extend(library_files(dirs, lib));
    }
    install::download_libraries(client, app, instance_id, files, cancel).await?;

    let work = dirs.cache.join(format!("install-{version_id}"));
    std::fs::create_dir_all(&work)?;

    let mut data: HashMap<String, String> = HashMap::new();
    data.insert(
        "MINECRAFT_JAR".into(),
        dirs.version_jar(game_version).to_string_lossy().to_string(),
    );
    data.insert("SIDE".into(), "client".into());
    data.insert("ROOT".into(), dirs.root.to_string_lossy().to_string());
    data.insert("INSTALLER".into(), installer.to_string_lossy().to_string());
    data.insert(
        "LIBRARY_DIR".into(),
        dirs.libraries.to_string_lossy().to_string(),
    );
    data.insert("MINECRAFT_VERSION".into(), game_version.into());
    for (key, side) in &profile.data {
        if let Some(val) = side.client.as_ref() {
            data.insert(key.clone(), resolve_data_token(val, dirs, installer, &work)?);
        }
    }

    let client_processors: Vec<&Processor> = profile
        .processors
        .iter()
        .filter(|p| p.sides.is_empty() || p.sides.iter().any(|s| s.eq_ignore_ascii_case("client")))
        .collect();
    let total = client_processors.len() as u64;
    for (i, proc) in client_processors.iter().enumerate() {
        if cancel.is_cancelled() {
            return Err(Error::msg("Instalacja anulowana."));
        }
        if outputs_ok(proc, &data) {
            continue;
        }
        emit_progress(
            app,
            InstallProgress {
                instance_id: instance_id.into(),
                stage: "processors".into(),
                current: i as u64 + 1,
                total: total.max(1),
                file: Some(proc.jar.clone()),
                message: format!("Procesor Forge {}/{}", i + 1, total),
            },
        );
        run_one_processor(java, dirs, proc, &data)?;
    }
    Ok(version_id)
}

fn outputs_ok(proc: &Processor, data: &HashMap<String, String>) -> bool {
    if proc.outputs.is_empty() {
        return false;
    }
    for (path_tok, sha) in &proc.outputs {
        let path = subst(path_tok, data);
        let p = PathBuf::from(&path);
        if !p.exists() {
            return false;
        }
        if !sha.is_empty() {
            if let Ok(got) = download::file_sha1_sync(&p) {
                let expected = sha.trim_matches(|c: char| c == '\'' || c == '"');
                if got != expected.to_lowercase() && got != sha.to_lowercase() {
                    return false;
                }
            }
        }
    }
    true
}

fn run_one_processor(
    java: &Path,
    dirs: &Dirs,
    proc: &Processor,
    data: &HashMap<String, String>,
) -> Result<()> {
    let main = main_class_of(&dirs.library_file(&meta::maven_path(&proc.jar)))?;
    let mut cp = vec![dirs.library_file(&meta::maven_path(&proc.jar))];
    for c in &proc.classpath {
        cp.push(dirs.library_file(&meta::maven_path(c)));
    }
    let cp_str = cp
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(crate::launch::classpath_sep());
    let args: Vec<String> = proc.args.iter().map(|a| subst(a, data)).collect();
    let mut cmd = std::process::Command::new(java);
    cmd.arg("-cp").arg(&cp_str).arg(&main).args(&args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let out = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| Error::msg(format!("Nie uruchomiono procesora {}: {e}", proc.jar)))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        return Err(Error::msg(format!(
            "Procesor {} zakończył się błędem.\n{stdout}\n{stderr}",
            proc.jar
        )));
    }
    Ok(())
}

fn subst(s: &str, data: &HashMap<String, String>) -> String {
    let mut out = s.to_string();
    for (k, v) in data {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    if out.starts_with('[') && out.ends_with(']') {
        // leftover maven token
        let inner = &out[1..out.len() - 1];
        return inner.to_string();
    }
    out
}

fn resolve_data_token(val: &str, dirs: &Dirs, installer: &Path, work: &Path) -> Result<String> {
    let val = val.trim();
    if val.starts_with('[') && val.ends_with(']') {
        let coord = &val[1..val.len() - 1];
        return Ok(dirs
            .library_file(&meta::maven_path(coord))
            .to_string_lossy()
            .to_string());
    }
    if (val.starts_with('\'') && val.ends_with('\'')) || (val.starts_with('"') && val.ends_with('"'))
    {
        return Ok(val[1..val.len() - 1].to_string());
    }
    if val.starts_with('/') || val.starts_with("data/") {
        let inner = val.trim_start_matches('/');
        let dest = work.join(inner.replace('/', "_"));
        download::extract_zip_file(installer, inner, &dest)?;
        return Ok(dest.to_string_lossy().to_string());
    }
    Ok(val.to_string())
}

fn main_class_of(jar: &Path) -> Result<String> {
    let file = std::fs::File::open(jar)
        .map_err(|_| Error::msg(format!("Brak biblioteki procesora {}", jar.display())))?;
    let mut zip = zip::ZipArchive::new(file)?;
    let mut mf = zip
        .by_name("META-INF/MANIFEST.MF")
        .map_err(|_| Error::msg(format!("Brak MANIFEST.MF w {}", jar.display())))?;
    let mut text = String::new();
    mf.read_to_string(&mut text)?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("Main-Class:") {
            return Ok(rest.trim().to_string());
        }
    }
    Err(Error::msg(format!(
        "Brak Main-Class w manifeście {}",
        jar.display()
    )))
}

async fn run_official_installer(
    app: &tauri::AppHandle,
    dirs: &Dirs,
    instance_id: &str,
    installer: &Path,
    java: &Path,
    game_version: &str,
    loader: Loader,
    loader_version: &str,
) -> Result<String> {
    emit_progress(
        app,
        InstallProgress {
            instance_id: instance_id.into(),
            stage: "loader".into(),
            current: 0,
            total: 1,
            file: None,
            message: "Uruchamianie instalatora Forge/NeoForge".into(),
        },
    );
    let mut cmd = std::process::Command::new(java);
    cmd.arg("-jar")
        .arg(installer)
        .arg("--installClient")
        .arg(&dirs.root)
        .current_dir(&dirs.root);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let out = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| Error::msg(format!("Nie uruchomiono instalatora: {e}")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        return Err(Error::msg(format!("{stdout}\n{stderr}")));
    }
    guess_version_id(dirs, game_version, loader, loader_version)
}

fn guess_version_id(
    dirs: &Dirs,
    game_version: &str,
    loader: Loader,
    loader_version: &str,
) -> Result<String> {
    let candidates = match loader {
        Loader::Forge => vec![
            format!("{game_version}-forge-{loader_version}"),
            format!("{game_version}-forge{loader_version}"),
            format!("forge-{game_version}-{loader_version}"),
        ],
        Loader::Neoforge => vec![
            format!("neoforge-{loader_version}"),
            format!("{game_version}-neoforge-{loader_version}"),
        ],
        _ => vec![],
    };
    for id in &candidates {
        if dirs.version_json(id).exists() {
            return Ok(id.clone());
        }
    }
    // Scan versions folder for a newly written json that mentions the loader version.
    if let Ok(rd) = std::fs::read_dir(&dirs.versions) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains(loader_version) {
                return Ok(name);
            }
        }
    }
    Err(Error::msg(
        "Instalator zakończył pracę, ale nie znaleziono profilu wersji.",
    ))
}

#[allow(dead_code)]
fn _read_lines(path: &Path) -> Result<Vec<String>> {
    let f = std::fs::File::open(path)?;
    Ok(std::io::BufReader::new(f)
        .lines()
        .filter_map(|l| l.ok())
        .collect())
}
