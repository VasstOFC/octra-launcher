use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

static MAX_DOWNLOADS: AtomicU32 = AtomicU32::new(10);
static ACTIVE_DOWNLOADS: AtomicU32 = AtomicU32::new(0);

pub fn set_max_concurrent_downloads(n: u32) {
    MAX_DOWNLOADS.store(n.clamp(1, 16), Ordering::Relaxed);
}

async fn acquire_download_slot() {
    loop {
        let limit = MAX_DOWNLOADS.load(Ordering::Relaxed).max(1);
        let cur = ACTIVE_DOWNLOADS.load(Ordering::Relaxed);
        if cur < limit
            && ACTIVE_DOWNLOADS
                .compare_exchange(cur, cur + 1, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
        {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

fn release_download_slot() {
    ACTIVE_DOWNLOADS.fetch_sub(1, Ordering::AcqRel);
}

use sha1::{Digest, Sha1};
use tokio::io::AsyncWriteExt;

use crate::error::{Error, Result};

pub async fn sha1_file(path: &Path) -> Result<String> {
    let data = tokio::fs::read(path).await?;
    Ok(sha1_bytes(&data))
}

pub fn sha1_bytes(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn file_sha1_sync(path: &Path) -> Result<String> {
    let data = std::fs::read(path)?;
    Ok(sha1_bytes(&data))
}

#[derive(Clone)]
pub struct ByteProgress {
    pub current: Arc<AtomicU64>,
    pub total: Arc<AtomicU64>,
}

impl ByteProgress {
    pub fn new() -> Self {
        Self {
            current: Arc::new(AtomicU64::new(0)),
            total: Arc::new(AtomicU64::new(0)),
        }
    }
}

pub async fn download_file(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    sha1: Option<&str>,
    size: Option<u64>,
    progress: Option<&ByteProgress>,
) -> Result<()> {
    download_file_with_headers(client, url, dest, sha1, size, progress, &[]).await
}

pub async fn download_file_with_headers(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    sha1: Option<&str>,
    size: Option<u64>,
    progress: Option<&ByteProgress>,
    extra_headers: &[(&str, &str)],
) -> Result<()> {
    if dest.exists() {
        if let Some(expected) = sha1 {
            if sha1_file(dest).await.unwrap_or_default() == expected.to_lowercase() {
                if let (Some(p), Some(sz)) = (progress, size) {
                    p.current.fetch_add(sz, Ordering::Relaxed);
                }
                return Ok(());
            }
        } else if let Some(sz) = size {
            if tokio::fs::metadata(dest).await?.len() == sz {
                if let Some(p) = progress {
                    p.current.fetch_add(sz, Ordering::Relaxed);
                }
                return Ok(());
            }
        }
    }
    acquire_download_slot().await;
    let result =
        download_file_inner(client, url, dest, sha1, size, progress, extra_headers).await;
    release_download_slot();
    result
}

async fn download_file_inner(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    sha1: Option<&str>,
    _size: Option<u64>,
    progress: Option<&ByteProgress>,
    extra_headers: &[(&str, &str)],
) -> Result<()> {
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let tmp = dest.with_extension("part");
    let mut req = client.get(url);
    for (k, v) in extra_headers {
        req = req.header(*k, *v);
    }
    let resp = req.send().await?.error_for_status()?;
    let mut stream = resp.bytes_stream();
    let mut file = tokio::fs::File::create(&tmp).await?;
    let mut hasher = Sha1::new();
    use futures::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        hasher.update(&chunk);
        file.write_all(&chunk).await?;
        if let Some(p) = progress {
            p.current.fetch_add(chunk.len() as u64, Ordering::Relaxed);
        }
    }
    file.flush().await?;
    drop(file);
    let got = hex::encode(hasher.finalize());
    if let Some(expected) = sha1 {
        if got != expected.to_lowercase() {
            let _ = tokio::fs::remove_file(&tmp).await;
            return Err(Error::msg(format!(
                "Niezgodny skrót SHA-1 dla {} (oczekiwano {expected}, otrzymano {got})",
                dest.display()
            )));
        }
    }
    if dest.exists() {
        tokio::fs::remove_file(dest).await.ok();
    }
    tokio::fs::rename(&tmp, dest).await?;
    Ok(())
}

pub async fn download_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
) -> Result<T> {
    let resp = client.get(url).send().await?.error_for_status()?;
    let value = resp.json::<T>().await?;
    Ok(value)
}

pub async fn download_text(client: &reqwest::Client, url: &str) -> Result<String> {
    let resp = client.get(url).send().await?.error_for_status()?;
    Ok(resp.text().await?)
}

/// ZIP EOCD (End of Central Directory) lives in the last 64 KiB + 22 bytes.
const EOCD_SIG: [u8; 4] = [0x50, 0x4B, 0x05, 0x06];
const EOCD_SEARCH: u64 = 65_557;

pub fn zip_is_complete(path: &Path) -> bool {
    validate_zip_archive(path).is_ok()
}

/// Fail-fast check: PK header + EOCD. Does not extract the archive.
pub fn validate_zip_archive(path: &Path) -> Result<()> {
    let mut file = std::fs::File::open(path)?;
    let len = file.metadata()?.len();
    if len < 22 {
        return Err(incomplete_zip_error());
    }
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    if magic[0] != b'P' || magic[1] != b'K' {
        return Err(Error::msg(format!(
            "Plik „{}” nie jest archiwum ZIP.",
            path.display()
        )));
    }
    let search = len.min(EOCD_SEARCH);
    let mut buf = vec![0u8; search as usize];
    file.seek(SeekFrom::End(-(search as i64)))?;
    file.read_exact(&mut buf)?;
    if !buf.windows(4).any(|w| w == EOCD_SIG) {
        return Err(incomplete_zip_error());
    }
    Ok(())
}

fn incomplete_zip_error() -> Error {
    Error::msg(
        "Archiwum ZIP jest niekompletne lub uszkodzone (ucięty plik — brak końca archiwum).",
    )
}

pub fn extract_zip(archive: &Path, dest: &Path, exclude_prefixes: &[&str]) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let name = entry.name().replace('\\', "/");
        if name.contains("..") {
            continue;
        }
        if exclude_prefixes
            .iter()
            .any(|p| name.starts_with(p) || name.contains("/META-INF/"))
        {
            continue;
        }
        if name.starts_with("META-INF/") {
            continue;
        }
        let out_path = dest.join(name);
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut outfile = std::fs::File::create(&out_path)?;
        std::io::copy(&mut entry, &mut outfile)?;
    }
    Ok(())
}

pub fn extract_zip_prefix(archive: &Path, dest: &Path, prefix: &str) -> Result<()> {
    let prefix = prefix.trim_matches('/').to_string() + "/";
    std::fs::create_dir_all(dest)?;
    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let name = entry.name().replace('\\', "/");
        if name.contains("..") {
            continue;
        }
        let Some(rel) = name.strip_prefix(&prefix) else {
            continue;
        };
        if rel.is_empty() {
            continue;
        }
        let out_path = dest.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut outfile = std::fs::File::create(&out_path)?;
        std::io::copy(&mut entry, &mut outfile)?;
    }
    Ok(())
}

pub fn extract_zip_file(archive: &Path, inner: &str, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    let want = inner.trim_start_matches('/').replace('\\', "/");
    let mut found = None;
    for i in 0..zip.len() {
        let name = {
            let entry = zip.by_index(i)?;
            entry.name().replace('\\', "/").trim_start_matches('/').to_string()
        };
        if name == want {
            found = Some(i);
            break;
        }
    }
    let mut entry = zip.by_index(found.ok_or_else(|| Error::msg(format!("Brak pliku {inner} w instalatorze")))?)?;
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut outfile = std::fs::File::create(dest)?;
    std::io::copy(&mut entry, &mut outfile)?;
    Ok(())
}

pub fn zip_has_file(archive: &Path, inner: &str) -> bool {
    zip_locate(archive, inner).is_some()
}

/// Exact path first, then a unique basename match (`manifest.json` at archive root or in a folder).
pub fn zip_locate(archive: &Path, inner: &str) -> Option<usize> {
    let file = std::fs::File::open(archive).ok()?;
    let mut zip = zip::ZipArchive::new(file).ok()?;
    let want = inner.trim_start_matches('/').replace('\\', "/");
    let base = want.rsplit('/').next().unwrap_or(&want);
    let mut fallback: Option<usize> = None;
    let mut fallback_count = 0usize;
    for i in 0..zip.len() {
        let name = {
            let entry = zip.by_index(i).ok()?;
            entry
                .name()
                .replace('\\', "/")
                .trim_start_matches('/')
                .to_string()
        };
        if name.eq_ignore_ascii_case(&want) {
            return Some(i);
        }
        if name.rsplit('/').next().is_some_and(|b| b.eq_ignore_ascii_case(base)) {
            fallback = Some(i);
            fallback_count += 1;
        }
    }
    if fallback_count == 1 {
        fallback
    } else {
        None
    }
}

pub fn read_zip_text(archive: &Path, inner: &str) -> Result<String> {
    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    let want = inner.trim_start_matches('/').replace('\\', "/");
    let mut found = None;
    for i in 0..zip.len() {
        let name = {
            let entry = zip.by_index(i)?;
            entry.name().replace('\\', "/").trim_start_matches('/').to_string()
        };
        if name == want {
            found = Some(i);
            break;
        }
    }
    let idx = match found.or_else(|| zip_locate(archive, inner)) {
        Some(i) => i,
        None => {
            return Err(Error::msg(format!("Brak pliku {inner} w archiwum")));
        }
    };
    let mut entry = zip.by_index(idx)?;
    let mut buf = String::new();
    std::io::Read::read_to_string(&mut entry, &mut buf)?;
    Ok(buf)
}
