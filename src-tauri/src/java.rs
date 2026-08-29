use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::paths::Dirs;
use crate::settings::Settings;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaRuntime {
    pub path: String,
    pub major: u32,
    pub vendor: String,
    pub source: String,
}

fn parse_major(output: &str) -> Option<u32> {
    for line in output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("java.specification.version = ") {
            let rest = rest.trim();
            if let Some(stripped) = rest.strip_prefix("1.") {
                return stripped.parse().ok();
            }
            return rest.parse().ok();
        }
    }
    // java -version stderr: `openjdk version "21.0.4"`
    for line in output.lines() {
        if let Some(idx) = line.find("version \"") {
            let rest = &line[idx + 9..];
            let ver = rest.split('"').next()?;
            if let Some(stripped) = ver.strip_prefix("1.") {
                return stripped.split('.').next()?.parse().ok();
            }
            return ver.split('.').next()?.parse().ok();
        }
    }
    None
}

pub fn probe_java(exe: &Path) -> Option<JavaRuntime> {
    let mut cmd = std::process::Command::new(exe);
    cmd.args(["-XshowSettings:properties", "-version"]);
    crate::winhide::hide_std(&mut cmd);
    let output = cmd.output().ok()?;
    let mut text = String::new();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    text.push_str(&String::from_utf8_lossy(&output.stdout));
    let major = parse_major(&text)?;
    let vendor = text
        .lines()
        .find_map(|l| l.trim().strip_prefix("java.vendor = "))
        .unwrap_or("unknown")
        .trim()
        .to_string();
    Some(JavaRuntime {
        path: exe.to_string_lossy().to_string(),
        major,
        vendor,
        source: "system".into(),
    })
}

fn collect_from_dir(dir: &Path, out: &mut Vec<JavaRuntime>, source: &str) {
    if !dir.exists() {
        return;
    }
    let walker = walkdir::WalkDir::new(dir).max_depth(5);
    for entry in walker.into_iter().flatten() {
        let path = entry.path();
        if path.file_name().and_then(|s| s.to_str()) == Some("java.exe")
            || path.file_name().and_then(|s| s.to_str()) == Some("java")
        {
            // skip debug / jre/bin duplicate by preferring bin/java
            if !path
                .parent()
                .and_then(|p| p.file_name())
                .is_some_and(|n| n == "bin")
            {
                continue;
            }
            if let Some(mut rt) = probe_java(path) {
                rt.source = source.into();
                if !out.iter().any(|e| e.path == rt.path) {
                    out.push(rt);
                }
            }
        }
    }
}

pub fn scan(dirs: &Dirs, settings: &Settings) -> Vec<JavaRuntime> {
    let mut found = Vec::new();
    collect_from_dir(&dirs.runtime, &mut found, "lumen");
    if let Some(manual) = &settings.java_path {
        let p = PathBuf::from(manual);
        if let Some(rt) = probe_java(&p) {
            if !found.iter().any(|e| e.path == rt.path) {
                found.insert(0, rt);
            }
        }
    }
    if let Ok(home) = std::env::var("JAVA_HOME") {
        let exe = PathBuf::from(&home).join("bin").join(java_exe_name());
        if let Some(rt) = probe_java(&exe) {
            if !found.iter().any(|e| e.path == rt.path) {
                found.push(rt);
            }
        }
    }
    let program_files = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
    for rel in [
        "Eclipse Adoptium",
        "Java",
        "Microsoft",
        "Amazon Corretto",
        "Zulu",
        "BellSoft",
        "AdoptOpenJDK",
        "Temurin",
        "OpenJDK",
    ] {
        collect_from_dir(&PathBuf::from(&program_files).join(rel), &mut found, "system");
    }
    found.sort_by(|a, b| b.major.cmp(&a.major));
    found
}

fn java_exe_name() -> &'static str {
    if cfg!(windows) {
        "java.exe"
    } else {
        "java"
    }
}

pub fn pick(runtimes: &[JavaRuntime], required_major: u32, settings: &Settings) -> Result<JavaRuntime> {
    if settings.java_mode == "manual" {
        if let Some(path) = &settings.java_path {
            if let Some(rt) = probe_java(Path::new(path)) {
                if rt.major >= required_major {
                    return Ok(rt);
                }
                return Err(Error::msg(format!(
                    "Wybrana Java {} jest za stara (wymagana {required_major}+)",
                    rt.major
                )));
            }
            return Err(Error::msg("Nie udało się uruchomić wskazanej Javy."));
        }
    }
    pick_compatible(runtimes, required_major, settings)
}

/// Dobiera Javę po wymaganym majorze (ścieżki z ustawień + skan), bez trybu „ręczna”.
pub fn pick_compatible(
    runtimes: &[JavaRuntime],
    required_major: u32,
    settings: &Settings,
) -> Result<JavaRuntime> {
    let majors: Vec<u32> = match required_major {
        8 => vec![8, 17, 21, 25],
        17 => vec![17, 21, 25],
        21 => vec![21, 25],
        25 => vec![25],
        n => vec![n],
    };
    for major in majors {
        if let Some(path) = settings.java_path_for_major(major) {
            if let Some(rt) = probe_java(Path::new(path)) {
                if rt.major >= required_major {
                    return Ok(rt);
                }
            }
        }
    }
    if let Some(rt) = runtimes
        .iter()
        .find(|r| r.major == required_major)
        .cloned()
        .or_else(|| {
            runtimes
                .iter()
                .filter(|r| r.major >= required_major)
                .min_by_key(|r| r.major)
                .cloned()
        })
    {
        return Ok(rt);
    }
    Err(Error::msg(format!(
        "Brak Javy {required_major}. Pobierz ją w Ustawieniach."
    )))
}

pub async fn download_temurin(
    client: &reqwest::Client,
    dirs: &Dirs,
    major: u32,
    emit: impl Fn(&str),
) -> Result<JavaRuntime> {
    emit(&format!("Pobieranie Temurin {major}…"));
    let os = if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    };
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x64"
    };
    let url = format!(
        "https://api.adoptium.net/v3/binary/latest/{major}/ga/{os}/{arch}/jre/hotspot/normal/eclipse?project=jdk"
    );
    let dest_root = dirs.runtime.join(major.to_string());
    if dest_root.exists() {
        let mut found = Vec::new();
        collect_from_dir(&dest_root, &mut found, "lumen");
        if let Some(rt) = found.into_iter().find(|r| r.major >= major) {
            return Ok(rt);
        }
    }
    let archive = dirs.cache.join(format!("temurin-{major}.zip"));
    crate::download::download_file(client, &url, &archive, None, None, None).await?;
    emit("Rozpakowywanie Javy…");
    if dest_root.exists() {
        tokio::fs::remove_dir_all(&dest_root).await.ok();
    }
    tokio::fs::create_dir_all(&dest_root).await?;
    let archive_clone = archive.clone();
    let dest_clone = dest_root.clone();
    tokio::task::spawn_blocking(move || {
        crate::download::extract_zip(&archive_clone, &dest_clone, &[])
    })
    .await
    .map_err(|e| Error::msg(e.to_string()))??;
    let mut found = Vec::new();
    collect_from_dir(&dest_root, &mut found, "lumen");
    found
        .into_iter()
        .find(|r| r.major >= major)
        .ok_or_else(|| Error::msg("Nie znaleziono java.exe po pobraniu Temurin."))
}

pub fn system_memory_mb() -> u64 {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    sys.total_memory() / 1024 / 1024
}
