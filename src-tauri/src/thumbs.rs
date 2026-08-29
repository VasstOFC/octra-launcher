//! Miniatury obrazów — cache w `.octralauncher/cache/thumbs/`.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::GenericImageView;
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};
use crate::paths::Dirs;

fn thumbs_dir(dirs: &Dirs) -> PathBuf {
    dirs.cache.join("thumbs")
}

fn cache_key(source: &Path, max_width: u32) -> Result<String> {
    let meta = fs::metadata(source)?;
    let mtime = meta
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut hasher = Sha256::new();
    hasher.update(source.to_string_lossy().as_bytes());
    hasher.update(mtime.to_le_bytes());
    hasher.update(meta.len().to_le_bytes());
    hasher.update(max_width.to_le_bytes());
    Ok(hex::encode(hasher.finalize()))
}

fn resize_to_jpeg(source: &Path, dest: &Path, max_width: u32) -> Result<()> {
    let bytes = fs::read(source)?;
    if bytes.is_empty() {
        return Err(Error::msg("Plik obrazu jest pusty."));
    }
    let img = image::load_from_memory(&bytes).map_err(|e| Error::msg(e.to_string()))?;
    let (w, h) = img.dimensions();
    let out = if w <= max_width {
        img
    } else {
        let nh = (h as f32 * (max_width as f32 / w as f32)).round() as u32;
        img.resize(max_width, nh.max(1), FilterType::Triangle)
    };
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut buf = Cursor::new(Vec::new());
    out.write_to(&mut buf, image::ImageFormat::Jpeg)
        .map_err(|e| Error::msg(e.to_string()))?;
    fs::write(dest, buf.into_inner())?;
    Ok(())
}

/// Zwraca ścieżkę do miniatury (generuje cache jeśli brak).
pub fn ensure_thumb(dirs: &Dirs, source: &Path, max_width: u32) -> Result<PathBuf> {
    if !source.is_file() {
        return Err(Error::msg("Plik obrazu nie istnieje."));
    }
    let key = cache_key(source, max_width)?;
    let dest = thumbs_dir(dirs).join(format!("{key}.jpg"));
    if dest.is_file() {
        if let (Ok(src_m), Ok(dst_m)) = (source.metadata().and_then(|m| m.modified()), dest.metadata().and_then(|m| m.modified())) {
            if dst_m >= src_m {
                return Ok(dest);
            }
        } else {
            return Ok(dest);
        }
    }
    resize_to_jpeg(source, &dest, max_width)?;
    Ok(dest)
}

pub fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}
