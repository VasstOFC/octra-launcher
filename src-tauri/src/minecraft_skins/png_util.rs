//! Normalizacja PNG skinów Minecraft — adaptacja z Modrinth App (GPLv3).
//! Źródło: packages/app-lib/src/api/minecraft_skins/png_util.rs

use std::io::Cursor;

use base64::Engine as _;

use crate::error::{Error, Result};

#[derive(Clone, Copy)]
struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

const PNG_SIGNATURE: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

pub fn is_png(data: &[u8]) -> bool {
    data.starts_with(PNG_SIGNATURE)
}

pub fn blob_to_data_url(png_data: &[u8]) -> Option<String> {
    is_png(png_data).then(|| {
        format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(png_data)
        )
    })
}

/// Normalizuje teksturę skina do 64×64 (legacy 64×32, Notch hack, wewnętrzna nieprzezroczystość).
pub fn normalize_skin_texture_bytes(texture_data: &[u8]) -> Result<Vec<u8>> {
    let mut png_reader = {
        let mut decoder = png::Decoder::new(Cursor::new(texture_data));
        decoder.set_transformations(png::Transformations::normalize_to_color8());
        decoder
            .read_info()
            .map_err(|e| Error::msg(format!("Niepoprawny PNG skina: {e}")))?
    };

    if png_reader.info().width != 64 || ![64, 32].contains(&png_reader.info().height) {
        return Err(Error::msg(
            "Skin Minecraft musi mieć wymiary 64×64 lub 64×32.",
        ));
    }

    let is_legacy = png_reader.info().height == 32;
    let mut texture_buf = get_skin_texture_buffer(&mut png_reader, is_legacy)?;
    if is_legacy {
        convert_legacy_skin_texture(&mut texture_buf, png_reader.info());
        notch_transparency_hack(&mut texture_buf, png_reader.info());
    }
    make_inner_parts_opaque(&mut texture_buf, png_reader.info());

    let mut encoded_png = Vec::new();
    let mut png_encoder = png::Encoder::new(&mut encoded_png, 64, 64);
    png_encoder.set_color(png::ColorType::Rgba);
    png_encoder.set_depth(png::BitDepth::Eight);
    png_encoder.set_filter(png::FilterType::NoFilter);
    png_encoder.set_compression(png::Compression::Fast);

    if let Some(ch) = png_reader.info().source_chromaticities {
        png_encoder.set_source_chromaticities(ch);
    }
    if let Some(gamma) = png_reader.info().source_gamma {
        png_encoder.set_source_gamma(gamma);
    }
    if let Some(srgb) = png_reader.info().srgb {
        png_encoder.set_source_srgb(srgb);
    }

    let flat: Vec<u8> = texture_buf
        .iter()
        .flat_map(|p| [p.r, p.g, p.b, p.a])
        .collect();
    let mut writer = png_encoder
        .write_header()
        .map_err(|e| Error::msg(format!("PNG encode: {e}")))?;
    writer
        .write_image_data(&flat)
        .map_err(|e| Error::msg(format!("PNG encode: {e}")))?;
    writer
        .finish()
        .map_err(|e| Error::msg(format!("PNG encode: {e}")))?;

    Ok(encoded_png)
}

fn get_skin_texture_buffer(
    png_reader: &mut png::Reader<std::io::Cursor<&[u8]>>,
    is_legacy: bool,
) -> Result<Vec<Rgba>> {
    let output_buffer_size = png_reader.output_buffer_size();
    let mut png_buf = if is_legacy {
        vec![0; output_buffer_size * 2]
    } else {
        vec![0; output_buffer_size]
    };
    png_reader
        .next_frame(&mut png_buf)
        .map_err(|e| Error::msg(format!("Nie udało się odczytać PNG: {e}")))?;

    let mut texture_buf: Vec<Rgba> = match png_reader.output_color_type().0 {
        png::ColorType::Grayscale => png_buf
            .iter()
            .map(|&v| Rgba {
                r: v,
                g: v,
                b: v,
                a: 255,
            })
            .collect::<Vec<_>>(),
        png::ColorType::GrayscaleAlpha => png_buf
            .chunks_exact(2)
            .map(|c| Rgba {
                r: c[0],
                g: c[0],
                b: c[0],
                a: c[1],
            })
            .collect::<Vec<_>>(),
        png::ColorType::Rgb => png_buf
            .chunks_exact(3)
            .map(|c| Rgba {
                r: c[0],
                g: c[1],
                b: c[2],
                a: 255,
            })
            .collect::<Vec<_>>(),
        png::ColorType::Rgba => png_buf
            .chunks_exact(4)
            .map(|c| Rgba {
                r: c[0],
                g: c[1],
                b: c[2],
                a: c[3],
            })
            .collect::<Vec<_>>(),
        _ => return Err(Error::msg("Nieobsługiwany format PNG skina.")),
    };

    if is_legacy {
        set_alpha(&mut texture_buf, png_reader.info(), 0, 32, 64, 64, 0);
    }

    Ok(texture_buf)
}

fn convert_legacy_skin_texture(texture_buf: &mut [Rgba], info: &png::Info<'_>) {
    const FACES: &[(usize, usize, isize, isize, usize, usize)] = &[
        (4, 16, 16, 32, 4, 4),
        (8, 16, 16, 32, 4, 4),
        (0, 20, 24, 32, 4, 12),
        (4, 20, 16, 32, 4, 12),
        (8, 20, 8, 32, 4, 12),
        (12, 20, 16, 32, 4, 12),
        (44, 16, -8, 32, 4, 4),
        (48, 16, -8, 32, 4, 4),
        (40, 20, 0, 32, 4, 12),
        (44, 20, -8, 32, 4, 12),
        (48, 20, -16, 32, 4, 12),
        (52, 20, -8, 32, 4, 12),
    ];
    for &(x, y, ox, oy, w, h) in FACES {
        copy_rect_mirror_h(texture_buf, info, x, y, ox, oy, w, h);
    }
}

fn notch_transparency_hack(texture_buf: &mut [Rgba], info: &png::Info<'_>) {
    let (x1, y1, x2, y2) = (32, 0, 64, 32);
    for y in y1..y2 {
        for x in x1..x2 {
            if texture_buf[x + y * info.width as usize].a < 128 {
                return;
            }
        }
    }
    set_alpha(texture_buf, info, x1, y1, x2, y2, 0);
}

fn make_inner_parts_opaque(texture_buf: &mut [Rgba], info: &png::Info<'_>) {
    for &(x1, y1, x2, y2) in &[(0, 0, 32, 16), (0, 16, 64, 32), (16, 48, 48, 64)] {
        set_alpha(texture_buf, info, x1, y1, x2, y2, 255);
    }
}

fn copy_rect_mirror_h(
    buf: &mut [Rgba],
    info: &png::Info<'_>,
    x: usize,
    y: usize,
    off_x: isize,
    off_y: isize,
    width: usize,
    height: usize,
) {
    let w = info.width as usize;
    for row in 0..height {
        for col in 0..width {
            let src_x = x + col;
            let src_y = y + row;
            let dst_x = (x as isize + off_x) as usize + (width - 1 - col);
            let dst_y = (y as isize + off_y) as usize + row;
            buf[dst_x + dst_y * w] = buf[src_x + src_y * w];
        }
    }
}

fn set_alpha(
    buf: &mut [Rgba],
    info: &png::Info<'_>,
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
    alpha: u8,
) {
    let w = info.width as usize;
    for y in y1..y2 {
        for x in x1..x2 {
            buf[x + y * w].a = alpha;
        }
    }
}
