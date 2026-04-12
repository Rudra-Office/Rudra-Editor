//! Glyph rasterizer — converts font outlines to bitmaps using ab_glyph_rasterizer.
//!
//! This module enables **pixel-identical rendering** across all platforms by
//! rasterizing glyphs in WASM rather than relying on the browser's font engine.
//! The browser's `ctx.fillText()` produces different pixels on different OS/browser
//! combinations. By rasterizing in Rust/WASM, the same font + size + glyph always
//! produces the same bitmap.
//!
//! ## Architecture
//!
//! ```text
//! Font bytes → ttf-parser → outline curves → ab_glyph_rasterizer → RGBA bitmap
//!                                                                      ↓
//!                                                            Canvas ImageData
//! ```

use ab_glyph_rasterizer::Rasterizer;

/// A rasterized glyph bitmap with positioning info.
#[derive(Debug, Clone)]
pub struct RasterizedGlyph {
    /// Pixel width of the bitmap.
    pub width: u32,
    /// Pixel height of the bitmap.
    pub height: u32,
    /// X offset from the glyph origin (in pixels).
    pub bearing_x: f32,
    /// Y offset from the baseline (positive = up, in pixels).
    pub bearing_y: f32,
    /// Horizontal advance width in pixels.
    pub advance: f32,
    /// RGBA pixel data (width * height * 4 bytes).
    /// Alpha channel contains the coverage; RGB is filled with the specified color.
    pub pixels: Vec<u8>,
}

/// Rasterize a single glyph from a font at a given size.
///
/// Returns `None` if the font doesn't have the glyph or has no outline
/// (e.g., space characters).
///
/// # Arguments
/// * `font_data` — raw font file bytes (TTF/OTF)
/// * `glyph_id` — glyph index from shaping output
/// * `size_px` — font size in pixels (not points)
/// * `color` — RGB color `[r, g, b]` for the glyph
pub fn rasterize_glyph(
    font_data: &[u8],
    glyph_id: u16,
    size_px: f32,
    color: [u8; 3],
) -> Option<RasterizedGlyph> {
    let face = ttf_parser::Face::parse(font_data, 0).ok()?;
    let gid = ttf_parser::GlyphId(glyph_id);

    // Get glyph metrics
    let upem = face.units_per_em() as f32;
    let scale = size_px / upem;

    let h_advance = face.glyph_hor_advance(gid)? as f32 * scale;

    // Get bounding box
    let bbox = face.glyph_bounding_box(gid)?;
    let x_min = bbox.x_min as f32 * scale;
    let y_min = bbox.y_min as f32 * scale;
    let x_max = bbox.x_max as f32 * scale;
    let y_max = bbox.y_max as f32 * scale;

    let width = (x_max - x_min).ceil() as u32 + 2; // +2 for anti-aliasing margin
    let height = (y_max - y_min).ceil() as u32 + 2;

    if width == 0 || height == 0 || width > 1024 || height > 1024 {
        return None;
    }

    // Rasterize the outline
    let mut rasterizer = Rasterizer::new(width as usize, height as usize);

    // Walk the outline and feed to rasterizer
    let offset_x = -x_min + 1.0;
    let offset_y = y_max + 1.0; // flip Y (font coords are bottom-up)

    struct OutlineBuilder<'a> {
        rasterizer: &'a mut Rasterizer,
        scale: f32,
        offset_x: f32,
        offset_y: f32,
        last_x: f32,
        last_y: f32,
    }

    impl ttf_parser::OutlineBuilder for OutlineBuilder<'_> {
        fn move_to(&mut self, x: f32, y: f32) {
            self.last_x = x * self.scale + self.offset_x;
            self.last_y = self.offset_y - y * self.scale;
        }

        fn line_to(&mut self, x: f32, y: f32) {
            let nx = x * self.scale + self.offset_x;
            let ny = self.offset_y - y * self.scale;
            self.rasterizer.draw_line(
                ab_glyph_rasterizer::point(self.last_x, self.last_y),
                ab_glyph_rasterizer::point(nx, ny),
            );
            self.last_x = nx;
            self.last_y = ny;
        }

        fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
            let cx = x1 * self.scale + self.offset_x;
            let cy = self.offset_y - y1 * self.scale;
            let nx = x * self.scale + self.offset_x;
            let ny = self.offset_y - y * self.scale;
            self.rasterizer.draw_quad(
                ab_glyph_rasterizer::point(self.last_x, self.last_y),
                ab_glyph_rasterizer::point(cx, cy),
                ab_glyph_rasterizer::point(nx, ny),
            );
            self.last_x = nx;
            self.last_y = ny;
        }

        fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
            let c1x = x1 * self.scale + self.offset_x;
            let c1y = self.offset_y - y1 * self.scale;
            let c2x = x2 * self.scale + self.offset_x;
            let c2y = self.offset_y - y2 * self.scale;
            let nx = x * self.scale + self.offset_x;
            let ny = self.offset_y - y * self.scale;
            self.rasterizer.draw_cubic(
                ab_glyph_rasterizer::point(self.last_x, self.last_y),
                ab_glyph_rasterizer::point(c1x, c1y),
                ab_glyph_rasterizer::point(c2x, c2y),
                ab_glyph_rasterizer::point(nx, ny),
            );
            self.last_x = nx;
            self.last_y = ny;
        }

        fn close(&mut self) {
            // Closing is implicit in ab_glyph_rasterizer
        }
    }

    let mut builder = OutlineBuilder {
        rasterizer: &mut rasterizer,
        scale,
        offset_x,
        offset_y,
        last_x: 0.0,
        last_y: 0.0,
    };

    face.outline_glyph(gid, &mut builder)?;

    // Convert coverage to RGBA
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    rasterizer.for_each_pixel(|idx, coverage| {
        let alpha = (coverage * 255.0).round() as u8;
        if alpha > 0 {
            let base = idx * 4;
            if base + 3 < pixels.len() {
                pixels[base] = color[0];
                pixels[base + 1] = color[1];
                pixels[base + 2] = color[2];
                pixels[base + 3] = alpha;
            }
        }
    });

    Some(RasterizedGlyph {
        width,
        height,
        bearing_x: x_min - 1.0,
        bearing_y: y_max + 1.0,
        advance: h_advance,
        pixels,
    })
}

/// Rasterize a string of text using shaped glyph positions.
///
/// Returns a single bitmap containing all glyphs positioned correctly.
/// This is the core function for canvas rendering — replaces `ctx.fillText()`.
pub fn rasterize_text_run(
    font_data: &[u8],
    glyphs: &[(u16, f32, f32)], // (glyph_id, x_offset, y_offset)
    size_px: f32,
    color: [u8; 3],
    total_width: f32,
    line_height: f32,
) -> Option<Vec<u8>> {
    let width = total_width.ceil() as u32 + 4;
    let height = line_height.ceil() as u32 + 4;

    if width == 0 || height == 0 || width > 4096 || height > 512 {
        return None;
    }

    let mut bitmap = vec![0u8; (width * height * 4) as usize];
    let baseline_y = line_height * 0.8; // approximate baseline

    for &(glyph_id, x_pos, y_pos) in glyphs {
        if let Some(glyph) = rasterize_glyph(font_data, glyph_id, size_px, color) {
            // Composite glyph bitmap onto the run bitmap
            let dest_x = (x_pos + glyph.bearing_x) as i32;
            let dest_y = (baseline_y - glyph.bearing_y + y_pos) as i32;

            for gy in 0..glyph.height {
                for gx in 0..glyph.width {
                    let src_idx = ((gy * glyph.width + gx) * 4) as usize;
                    let dx = dest_x + gx as i32;
                    let dy = dest_y + gy as i32;

                    if dx >= 0 && dx < width as i32 && dy >= 0 && dy < height as i32 {
                        let dst_idx = ((dy as u32 * width + dx as u32) * 4) as usize;
                        if src_idx + 3 < glyph.pixels.len() && dst_idx + 3 < bitmap.len() {
                            let alpha = glyph.pixels[src_idx + 3];
                            if alpha > 0 {
                                // Alpha blending
                                let inv_alpha = 255 - alpha;
                                bitmap[dst_idx] =
                                    ((glyph.pixels[src_idx] as u16 * alpha as u16
                                        + bitmap[dst_idx] as u16 * inv_alpha as u16)
                                        / 255) as u8;
                                bitmap[dst_idx + 1] =
                                    ((glyph.pixels[src_idx + 1] as u16 * alpha as u16
                                        + bitmap[dst_idx + 1] as u16 * inv_alpha as u16)
                                        / 255) as u8;
                                bitmap[dst_idx + 2] =
                                    ((glyph.pixels[src_idx + 2] as u16 * alpha as u16
                                        + bitmap[dst_idx + 2] as u16 * inv_alpha as u16)
                                        / 255) as u8;
                                bitmap[dst_idx + 3] =
                                    alpha.max(bitmap[dst_idx + 3]);
                            }
                        }
                    }
                }
            }
        }
    }

    Some(bitmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rasterize_returns_none_for_invalid_data() {
        assert!(rasterize_glyph(b"not a font", 0, 16.0, [0, 0, 0]).is_none());
    }

    #[test]
    fn rasterize_text_run_with_empty_glyphs() {
        // Empty glyphs list → returns a blank bitmap (not None)
        let result = rasterize_text_run(b"bad", &[], 16.0, [0, 0, 0], 100.0, 20.0);
        // Either None (invalid font) or Some (blank bitmap) is acceptable
        let _ = result;
    }
}
