//! Winscroller Mouse Custom Cursor Forge — pure Rust SVG generator & CUR/ANI encoder.
//! Supports 14 arrows, 30 surfaces, 3 fills, 4 textures, 3 depths, 22 lines, 18 ornaments,
//! 5 shadows, accent caps, details, and hand/beam/busy/resize cursor shapes.

pub mod arrows;
pub mod geo;
pub mod glyphs;
pub mod lines;
pub mod ornaments;
pub mod surfaces;
pub mod svg;

/// Rasterize an SVG string at `size x size` into RGBA8888 (straight alpha) bytes.
pub fn rasterize_svg(svg_str: &str, size: u32) -> Option<Vec<u8>> {
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg_str, &opt).ok()?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size)?;
    let sx = (size as f32) / tree.size().width();
    let sy = (size as f32) / tree.size().height();
    let transform = resvg::tiny_skia::Transform::from_scale(sx, sy);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let mut rgba = pixmap.take();
    for px in rgba.as_chunks_mut::<4>().0 {
        let a = px[3] as u32;
        if a > 0 && a < 255 {
            px[0] = ((px[0] as u32 * 255 + a / 2) / a).min(255) as u8;
            px[1] = ((px[1] as u32 * 255 + a / 2) / a).min(255) as u8;
            px[2] = ((px[2] as u32 * 255 + a / 2) / a).min(255) as u8;
        }
    }
    Some(rgba)
}

/// Encode RGBA pixels into a PNG byte buffer.
pub fn encode_png(rgba: &[u8], w: u32, h: u32) -> Option<Vec<u8>> {
    let mut buf = Vec::with_capacity((w * h * 2) as usize);
    {
        let mut enc = png::Encoder::new(&mut buf, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().ok()?;
        writer.write_image_data(rgba).ok()?;
    }
    Some(buf)
}

/// Encode a single RGBA image + hotspot into a valid Windows `.cur` file (PNG-in-CUR).
pub fn encode_cur(rgba: &[u8], size: u32, hot_x: u16, hot_y: u16) -> Option<Vec<u8>> {
    let png_bytes = encode_png(rgba, size, size)?;
    let mut out = Vec::with_capacity(22 + png_bytes.len());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes()); // type = 2 (CUR)
    out.extend_from_slice(&1u16.to_le_bytes()); // count = 1
    let dim = if size >= 256 { 0u8 } else { size as u8 };
    out.push(dim);
    out.push(dim);
    out.push(0);
    out.push(0);
    out.extend_from_slice(&hot_x.to_le_bytes());
    out.extend_from_slice(&hot_y.to_le_bytes());
    out.extend_from_slice(&(png_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(&22u32.to_le_bytes());
    out.extend_from_slice(&png_bytes);
    Some(out)
}

/// Encode multiple `.cur` frames into a valid Windows animated `.ani` file (RIFF ACON).
pub fn encode_ani(cur_frames: &[Vec<u8>], delay_ms: u32) -> Vec<u8> {
    let n = cur_frames.len() as u32;
    let jiffies = ((delay_ms as f32 / 16.6667).round() as u32).max(1);

    let mut fram_body = Vec::new();
    fram_body.extend_from_slice(b"fram");
    for cur in cur_frames {
        fram_body.extend_from_slice(b"icon");
        fram_body.extend_from_slice(&(cur.len() as u32).to_le_bytes());
        fram_body.extend_from_slice(cur);
        if !cur.len().is_multiple_of(2) {
            fram_body.push(0);
        }
    }

    let mut anih = Vec::with_capacity(44);
    anih.extend_from_slice(b"anih");
    anih.extend_from_slice(&36u32.to_le_bytes());
    anih.extend_from_slice(&36u32.to_le_bytes());
    anih.extend_from_slice(&n.to_le_bytes());
    anih.extend_from_slice(&n.to_le_bytes());
    anih.extend_from_slice(&0u32.to_le_bytes());
    anih.extend_from_slice(&0u32.to_le_bytes());
    anih.extend_from_slice(&0u32.to_le_bytes());
    anih.extend_from_slice(&1u32.to_le_bytes());
    anih.extend_from_slice(&jiffies.to_le_bytes());
    anih.extend_from_slice(&1u32.to_le_bytes());

    let list_total = 8 + fram_body.len();
    let riff_size = (4 + anih.len() + list_total) as u32;

    let mut out = Vec::with_capacity(8 + riff_size as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_size.to_le_bytes());
    out.extend_from_slice(b"ACON");
    out.extend_from_slice(&anih);
    out.extend_from_slice(b"LIST");
    out.extend_from_slice(&(fram_body.len() as u32).to_le_bytes());
    out.extend_from_slice(&fram_body);
    out
}

/// Encode multiple PNG images into a multi-resolution Windows `.ico` file.
pub fn encode_ico(entries: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let count = entries.len() as u16;
    let header_len = 6 + entries.len() * 16;
    let total_png: usize = entries.iter().map(|(_, p)| p.len()).sum();
    let mut out = Vec::with_capacity(header_len + total_png);
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // type = 1 (ICO)
    out.extend_from_slice(&count.to_le_bytes());

    let mut offset = header_len as u32;
    for (size, png_bytes) in entries {
        let dim = if *size >= 256 { 0u8 } else { *size as u8 };
        out.push(dim);
        out.push(dim);
        out.push(0);
        out.push(0);
        out.extend_from_slice(&1u16.to_le_bytes()); // planes
        out.extend_from_slice(&32u16.to_le_bytes()); // bit count
        out.extend_from_slice(&(png_bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += png_bytes.len() as u32;
    }
    for (_, png_bytes) in entries {
        out.extend_from_slice(png_bytes);
    }
    out
}
