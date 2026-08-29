use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipArchive;
use zip::ZipWriter;

const MAGIC: &[u8; 8] = b"LUMENPK1";

pub struct Payload {
    pub stub_len: u64,
    pub zip_len: u64,
}

pub fn read_info(exe: &Path) -> io::Result<Option<Payload>> {
    let mut f = File::open(exe)?;
    let len = f.metadata()?.len();
    if len < 16 {
        return Ok(None);
    }
    f.seek(SeekFrom::End(-16))?;
    let mut footer = [0u8; 16];
    f.read_exact(&mut footer)?;
    if &footer[8..] != MAGIC {
        return Ok(None);
    }
    let zip_len = u64::from_le_bytes(footer[..8].try_into().unwrap());
    if zip_len + 16 > len {
        return Ok(None);
    }
    Ok(Some(Payload {
        stub_len: len - zip_len - 16,
        zip_len,
    }))
}

pub fn extract_stub(exe: &Path) -> io::Result<Vec<u8>> {
    match read_info(exe)? {
        Some(info) => {
            let mut f = File::open(exe)?;
            let mut buf = vec![0u8; info.stub_len as usize];
            f.read_exact(&mut buf)?;
            Ok(buf)
        }
        None => fs::read(exe),
    }
}

pub fn open_zip(exe: &Path) -> Result<ZipArchive<io::Cursor<Vec<u8>>>, String> {
    let info = read_info(exe)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| {
            "Brak pakietu instalacyjnego. Zbuduj instalator poleceniem npm run installer:pack."
                .to_string()
        })?;
    let mut f = File::open(exe).map_err(|e| e.to_string())?;
    f.seek(SeekFrom::Start(info.stub_len))
        .map_err(|e| e.to_string())?;
    let mut zip_bytes = vec![0u8; info.zip_len as usize];
    f.read_exact(&mut zip_bytes).map_err(|e| e.to_string())?;
    ZipArchive::new(io::Cursor::new(zip_bytes)).map_err(|e| e.to_string())
}

pub fn extract_to(
    exe: &Path,
    dest: &Path,
    mut on_progress: impl FnMut(f32, &str),
) -> Result<(), String> {
    let mut archive = open_zip(exe)?;
    let total = archive.len().max(1);
    fs::create_dir_all(dest).map_err(|e| format!("Nie można utworzyć folderu: {e}"))?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let rel = file
            .enclosed_name()
            .ok_or_else(|| "Nieprawidłowa ścieżka w pakiecie.".to_string())?;
        let out = dest.join(rel);
        let name = out
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "plik".into());
        on_progress(i as f32 / total as f32, &format!("Kopiowanie: {name}"));
        if file.is_dir() {
            fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut dest_f = File::create(&out).map_err(|e| {
            format!("Nie można zapisać {}: {e}", out.display())
        })?;
        io::copy(&mut file, &mut dest_f).map_err(|e| e.to_string())?;
    }
    on_progress(1.0, "Pliki skopiowane.");
    Ok(())
}

pub fn make_sfx(stub: &Path, payload_dir: &Path, out: &Path) -> Result<(), String> {
    if !stub.is_file() {
        return Err(format!("Nie znaleziono programu instalatora: {}", stub.display()));
    }
    if !payload_dir.is_dir() {
        return Err(format!("Brak folderu pakietu: {}", payload_dir.display()));
    }
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let zip_tmp = out.with_extension("payload.zip.tmp");
    write_zip(payload_dir, &zip_tmp)?;

    let mut out_f = File::create(out).map_err(|e| e.to_string())?;
    let mut stub_f = File::open(stub).map_err(|e| e.to_string())?;
    io::copy(&mut stub_f, &mut out_f).map_err(|e| e.to_string())?;
    let mut zip_f = File::open(&zip_tmp).map_err(|e| e.to_string())?;
    let zip_len = zip_f.metadata().map_err(|e| e.to_string())?.len();
    io::copy(&mut zip_f, &mut out_f).map_err(|e| e.to_string())?;
    out_f
        .write_all(&zip_len.to_le_bytes())
        .map_err(|e| e.to_string())?;
    out_f.write_all(MAGIC).map_err(|e| e.to_string())?;
    out_f.flush().map_err(|e| e.to_string())?;
    drop(out_f);
    drop(zip_f);
    let _ = fs::remove_file(&zip_tmp);
    Ok(())
}

fn write_zip(payload_dir: &Path, zip_path: &Path) -> Result<(), String> {
    let file = File::create(zip_path).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut files: Vec<PathBuf> = walkdir::WalkDir::new(payload_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect();
    files.sort();

    if files.is_empty() {
        return Err("Folder pakietu jest pusty (brak octra.exe).".into());
    }

    for path in files {
        let rel = path
            .strip_prefix(payload_dir)
            .map_err(|_| "Ścieżka poza folderem pakietu.".to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        if rel.is_empty() {
            continue;
        }
        let opts = if already_compressed(&rel) {
            stored
        } else {
            deflated
        };
        zip.start_file(&rel, opts).map_err(|e| e.to_string())?;
        let mut f = File::open(&path).map_err(|e| e.to_string())?;
        io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
        eprintln!("  + {rel}");
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn already_compressed(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.ends_with(".mrpack")
        || n.ends_with(".zip")
        || n.ends_with(".png")
        || n.ends_with(".jpg")
        || n.ends_with(".7z")
}
