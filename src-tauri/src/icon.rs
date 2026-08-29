//! Ikona instancji (plik w katalogu instancji) i kolory „backend LED”.

use std::collections::VecDeque;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use base64::Engine as _;
use image::{Rgba, RgbaImage};

use crate::error::Result;
use crate::instances::Instance;
use crate::paths::Dirs;

pub const DEFAULT_LED: &str = "#c4a7ff";
pub const DEFAULT_LED_2: &str = "#a78bfa";
const MAX_ICON_BYTES: usize = 2 * 1024 * 1024;

const GAME_ICON_NAMES: &[&str] = &[
    "icon.png",
    "icon.jpg",
    "icon.jpeg",
    "icon.webp",
    "icon.gif",
    "icon.svg",
    "pack.png",
    "logo.png",
];

pub fn icon_abs_path(dirs: &Dirs, inst: &Instance) -> Option<PathBuf> {
    let rel = inst.icon_path.as_deref()?.trim();
    if rel.is_empty() || rel.contains("..") || rel.contains('/') || rel.contains('\\') {
        return None;
    }
    let path = dirs.instance_dir(&inst.id).join(rel);
    path.is_file().then_some(path)
}

pub fn install_icon_bytes(dirs: &Dirs, inst: &mut Instance, bytes: &[u8]) -> Result<()> {
    if bytes.is_empty() || bytes.len() > MAX_ICON_BYTES {
        return Ok(());
    }
    let prepared = prepare_icon_png(bytes);
    let (store, ext) = match &prepared {
        Some(png) => (png.as_slice(), "png"),
        None => (bytes, sniff_ext(bytes)),
    };
    let name = format!("icon.{ext}");
    let dir = dirs.instance_dir(&inst.id);
    std::fs::create_dir_all(&dir)?;
    let dest = dir.join(&name);
    std::fs::write(&dest, store)?;
    remove_other_icons(&dir, &name);
    inst.icon_path = Some(name);
    let (led, led2) = palette_from_bytes(store).unwrap_or_else(default_led);
    inst.led_color = led.clone();
    inst.led_color_2 = led2;
    if inst.icon_color.trim().is_empty() {
        inst.icon_color = led;
    }
    Ok(())
}

pub fn adopt_from_game_dir(dirs: &Dirs, inst: &mut Instance) -> Result<bool> {
    if icon_abs_path(dirs, inst).is_some() {
        return Ok(false);
    }
    let game = dirs.game_dir(&inst.id);
    for name in GAME_ICON_NAMES {
        let src = game.join(name);
        if !src.is_file() {
            continue;
        }
        let bytes = match std::fs::read(&src) {
            Ok(b) if !b.is_empty() && b.len() <= MAX_ICON_BYTES => b,
            _ => continue,
        };
        install_icon_bytes(dirs, inst, &bytes)?;
        return Ok(inst.icon_path.is_some());
    }
    Ok(false)
}

/// Usuwa zainstalowane `icon.*` z katalogu instancji (np. po zastąpieniu logo glifem).
pub fn remove_installed_icons(dirs: &Dirs, inst_id: &str) {
    let dir = dirs.instance_dir(inst_id);
    for _ in 0..4 {
        let mut leftover = false;
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for entry in rd.flatten() {
                let name = entry.file_name();
                let Some(s) = name.to_str() else { continue };
                if is_managed_icon_file(s) {
                    let path = entry.path();
                    remove_file_force(&path);
                    if path.exists() {
                        leftover = true;
                    }
                }
            }
        }
        if !leftover {
            break;
        }
    }
}

fn is_managed_icon_file(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.starts_with("icon.")
        && matches!(
            lower.rsplit('.').next(),
            Some("png" | "jpg" | "jpeg" | "webp" | "gif" | "svg" | "ico")
        )
}

fn remove_file_force(path: &Path) {
    if !path.exists() {
        return;
    }
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        if perms.readonly() {
            perms.set_readonly(false);
            let _ = std::fs::set_permissions(path, perms);
        }
    }
    let _ = std::fs::remove_file(path);
}

pub fn read_data_url(dirs: &Dirs, inst: &Instance) -> Option<String> {
    // Tylko plik z `icon_path`. Luźne `icon.png` po odłączeniu paczki nie wraca
    // jako logo — inaczej glif / nowy obraz nigdy nie wygra.
    let path = icon_abs_path(dirs, inst)?;
    let bytes = std::fs::read(&path).ok()?;
    if bytes.is_empty() || bytes.len() > MAX_ICON_BYTES {
        return None;
    }
    let mime = mime_for_ext(
        path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png"),
    );
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Some(format!("data:{mime};base64,{b64}"))
}

pub fn extract_from_zip(pack: &Path) -> Option<Vec<u8>> {
    let file = std::fs::File::open(pack).ok()?;
    let mut zip = zip::ZipArchive::new(file).ok()?;
    let mut best: Option<(i32, usize, String)> = None;
    for i in 0..zip.len() {
        let name = {
            let entry = zip.by_index(i).ok()?;
            if entry.is_dir() {
                continue;
            }
            entry
                .name()
                .replace('\\', "/")
                .trim_start_matches('/')
                .to_string()
        };
        let score = icon_entry_score(&name);
        if score < 0 {
            continue;
        }
        let better = match &best {
            None => true,
            Some((s, _, _)) => score > *s,
        };
        if better {
            best = Some((score, i, name));
        }
    }
    let (_, idx, _) = best?;
    let mut entry = zip.by_index(idx).ok()?;
    if entry.size() > MAX_ICON_BYTES as u64 {
        return None;
    }
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf).ok()?;
    if buf.is_empty() || buf.len() > MAX_ICON_BYTES {
        return None;
    }
    Some(buf)
}

fn icon_entry_score(name: &str) -> i32 {
    let n = name.to_ascii_lowercase();
    if n.contains("..")
        || n.contains("mods/")
        || n.contains("resourcepacks/")
        || n.contains("shaderpacks/")
        || n.contains("config/")
    {
        return -1;
    }
    let file = n.rsplit('/').next().unwrap_or("");
    let Some((stem, ext)) = file.rsplit_once('.') else {
        return -1;
    };
    let is_img = matches!(
        ext,
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "ico" | "svg"
    );
    if !is_img {
        return -1;
    }
    let in_overrides = n.starts_with("overrides/") || n.starts_with("client-overrides/");
    let depth = n.matches('/').count();
    let name_score = match stem {
        "icon" => 50,
        "packicon" | "pack_icon" | "pack-icon" => 45,
        "pack" => 40,
        "logo" => 35,
        _ => 0,
    };
    if name_score == 0 && !(in_overrides && depth <= 1) {
        return -1;
    }
    let loc = if in_overrides && depth == 1 {
        40
    } else if in_overrides && depth <= 2 {
        20
    } else if depth == 0 {
        25
    } else {
        0
    };
    name_score + loc
}

fn remove_other_icons(dir: &Path, keep: &str) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let name = entry.file_name();
            let Some(s) = name.to_str() else { continue };
            if s == keep {
                continue;
            }
            if is_managed_icon_file(s) {
                remove_file_force(&entry.path());
            }
        }
    }
}

fn sniff_ext(bytes: &[u8]) -> &'static str {
    if bytes.len() >= 8 && bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        return "png";
    }
    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        return "jpg";
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return "webp";
    }
    if bytes.len() >= 6 && (bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a")) {
        return "gif";
    }
    let trimmed = bytes
        .iter()
        .skip_while(|b| b.is_ascii_whitespace())
        .copied()
        .take(64)
        .collect::<Vec<_>>();
    let head = String::from_utf8_lossy(&trimmed).to_ascii_lowercase();
    if head.starts_with("<svg") || (head.starts_with("<?xml") && head.contains("svg")) {
        return "svg";
    }
    "png"
}

fn mime_for_ext(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        _ => "image/png",
    }
}

fn default_led() -> (String, String) {
    (DEFAULT_LED.into(), DEFAULT_LED_2.into())
}

pub fn palette_from_bytes(bytes: &[u8]) -> Option<(String, String)> {
    let img = image::load_from_memory(bytes).ok()?;
    let small = img.thumbnail(48, 48).to_rgba8();
    let mut acc_r = 0u64;
    let mut acc_g = 0u64;
    let mut acc_b = 0u64;
    let mut wsum = 0u64;
    let mut best_sat = -1.0f32;
    let mut sat_rgb = (0xc4u8, 0xa7, 0xff);
    for p in small.pixels() {
        let [r, g, b, a] = p.0;
        if a < 40 {
            continue;
        }
        let rf = r as f32;
        let gf = g as f32;
        let bf = b as f32;
        let max = rf.max(gf).max(bf);
        let min = rf.min(gf).min(bf);
        let lum = 0.2126 * rf + 0.7152 * gf + 0.0722 * bf;
        if lum < 14.0 || lum > 248.0 {
            continue;
        }
        let sat = if max > 1.0 { (max - min) / max } else { 0.0 };
        let w = (((sat * 2.4) + 0.3) * (a as f32 / 255.0) * 100.0) as u64;
        if w == 0 {
            continue;
        }
        acc_r += r as u64 * w;
        acc_g += g as u64 * w;
        acc_b += b as u64 * w;
        wsum += w;
        if sat > best_sat && lum > 36.0 && lum < 230.0 {
            best_sat = sat;
            sat_rgb = (r, g, b);
        }
    }
    if wsum == 0 {
        return Some(default_led());
    }
    let avg = (
        (acc_r / wsum) as u8,
        (acc_g / wsum) as u8,
        (acc_b / wsum) as u8,
    );
    let glow = if best_sat > 0.12 { sat_rgb } else { avg };
    let glow = boost_sat(glow, 1.22);
    let dim = darken(glow, 0.42);
    Some((to_hex(glow), to_hex(dim)))
}

fn boost_sat(rgb: (u8, u8, u8), amount: f32) -> (u8, u8, u8) {
    let (r, g, b) = (rgb.0 as f32, rgb.1 as f32, rgb.2 as f32);
    let avg = (r + g + b) / 3.0;
    let mix = |c: f32| (avg + (c - avg) * amount).clamp(0.0, 255.0) as u8;
    (mix(r), mix(g), mix(b))
}

fn darken(rgb: (u8, u8, u8), f: f32) -> (u8, u8, u8) {
    (
        (rgb.0 as f32 * f) as u8,
        (rgb.1 as f32 * f) as u8,
        (rgb.2 as f32 * f) as u8,
    )
}

fn to_hex(rgb: (u8, u8, u8)) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb.0, rgb.1, rgb.2)
}

fn prepare_icon_png(bytes: &[u8]) -> Option<Vec<u8>> {
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let punched = knock_out_matte(img)?;
    let mut out = Cursor::new(Vec::new());
    punched.write_to(&mut out, image::ImageFormat::Png).ok()?;
    let stored = out.into_inner();
    if stored.is_empty() || stored.len() > MAX_ICON_BYTES {
        return None;
    }
    Some(stored)
}

fn knock_out_matte(src: RgbaImage) -> Option<RgbaImage> {
    let (w, h) = src.dimensions();
    if w < 8 || h < 8 {
        return None;
    }
    let corners = [
        *src.get_pixel(1, 1),
        *src.get_pixel(w - 2, 1),
        *src.get_pixel(1, h - 2),
        *src.get_pixel(w - 2, h - 2),
    ];
    if corners.iter().any(|p| p[3] < 40) {
        return None;
    }
    let sr = corners.iter().map(|p| p[0] as u32).sum::<u32>() / 4;
    let sg = corners.iter().map(|p| p[1] as u32).sum::<u32>() / 4;
    let sb = corners.iter().map(|p| p[2] as u32).sum::<u32>() / 4;
    let sample = [sr as u8, sg as u8, sb as u8];
    let lum = 0.2126 * sr as f32 + 0.7152 * sg as f32 + 0.0722 * sb as f32;
    if lum > 48.0 {
        return None;
    }

    let mut img = src;
    let mut seen = vec![false; (w * h) as usize];
    let mut q = VecDeque::new();
    let is_matte = |p: Rgba<u8>| {
        if p[3] < 40 {
            return true;
        }
        let l = 0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32;
        let dist = p[0].abs_diff(sample[0]) as u16
            + p[1].abs_diff(sample[1]) as u16
            + p[2].abs_diff(sample[2]) as u16;
        l < 46.0 && dist < 48
    };
    for y in 0..h {
        for x in 0..w {
            if x > 2 && y > 2 && x < w - 3 && y < h - 3 {
                continue;
            }
            if is_matte(*img.get_pixel(x, y)) {
                let idx = (y * w + x) as usize;
                seen[idx] = true;
                q.push_back((x, y));
            }
        }
    }
    let mut punched = 0u32;
    while let Some((x, y)) = q.pop_front() {
        img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
        punched += 1;
        for (nx, ny) in [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ] {
            if nx >= w || ny >= h {
                continue;
            }
            let idx = (ny * w + nx) as usize;
            if seen[idx] {
                continue;
            }
            if !is_matte(*img.get_pixel(nx, ny)) {
                continue;
            }
            seen[idx] = true;
            q.push_back((nx, ny));
        }
    }
    let total = w * h;
    if punched < total / 50 || punched > (total * 3) / 4 {
        return None;
    }
    Some(img)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instances::test_instance;
    use crate::paths::Dirs;
    use uuid::Uuid;

    fn png_stub() -> Vec<u8> {
        let mut b = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        b.resize(32, 0);
        b
    }

    #[test]
    fn read_data_url_ignores_loose_file_without_icon_path() {
        let root = std::env::temp_dir().join(format!("lumen-icon-read-{}", Uuid::new_v4()));
        let dirs = Dirs::from_root(root.clone());
        let inst = test_instance("i1");
        std::fs::create_dir_all(dirs.instance_dir(&inst.id)).unwrap();
        std::fs::write(dirs.instance_dir(&inst.id).join("icon.png"), png_stub()).unwrap();
        assert!(read_data_url(&dirs, &inst).is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn read_data_url_uses_icon_path() {
        let root = std::env::temp_dir().join(format!("lumen-icon-read2-{}", Uuid::new_v4()));
        let dirs = Dirs::from_root(root.clone());
        let mut inst = test_instance("i2");
        inst.icon_path = Some("icon.png".into());
        std::fs::create_dir_all(dirs.instance_dir(&inst.id)).unwrap();
        std::fs::write(dirs.instance_dir(&inst.id).join("icon.png"), png_stub()).unwrap();
        let url = read_data_url(&dirs, &inst).expect("icon");
        assert!(url.starts_with("data:image/png;base64,"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn read_data_url_does_not_fallback_to_other_file() {
        let root = std::env::temp_dir().join(format!("lumen-icon-read3-{}", Uuid::new_v4()));
        let dirs = Dirs::from_root(root.clone());
        let mut inst = test_instance("i4");
        inst.icon_path = Some("icon.webp".into());
        std::fs::create_dir_all(dirs.instance_dir(&inst.id)).unwrap();
        std::fs::write(dirs.instance_dir(&inst.id).join("icon.png"), png_stub()).unwrap();
        assert!(read_data_url(&dirs, &inst).is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn remove_installed_icons_clears_readonly() {
        let root = std::env::temp_dir().join(format!("lumen-icon-rm-{}", Uuid::new_v4()));
        let dirs = Dirs::from_root(root.clone());
        let id = "i3";
        let dir = dirs.instance_dir(id);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("icon.png");
        std::fs::write(&file, png_stub()).unwrap();
        let mut perms = std::fs::metadata(&file).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&file, perms).unwrap();
        remove_installed_icons(&dirs, id);
        assert!(!file.exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn knock_out_matte_clears_black_corners() {
        let mut img = image::RgbaImage::from_pixel(32, 32, image::Rgba([0, 0, 0, 255]));
        for y in 8..24 {
            for x in 8..24 {
                img.put_pixel(x, y, image::Rgba([255, 40, 160, 255]));
            }
        }
        let out = knock_out_matte(img).expect("punch");
        assert_eq!(out.get_pixel(1, 1)[3], 0);
        assert_eq!(out.get_pixel(16, 16)[3], 255);
        assert_eq!(out.get_pixel(16, 16)[0], 255);
    }
}
