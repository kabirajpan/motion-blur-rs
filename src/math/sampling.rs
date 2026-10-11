//! Sub-pixel image interpolation routines (Bilinear & Bicubic).

use super::vector::Vec2;
use crate::plugin::frame::{ColorRgba, FrameBuffer};

/// Bilinear sampling of a single-channel float array.
#[inline(always)]
pub fn sample_bilinear_plane(plane: &[f32], width: usize, height: usize, pos: Vec2) -> f32 {
    let x = pos.x.clamp(0.0, (width - 1) as f32);
    let y = pos.y.clamp(0.0, (height - 1) as f32);

    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);

    let fx = x - x0 as f32;
    let fy = y - y0 as f32;

    let p00 = plane[y0 * width + x0];
    let p10 = plane[y0 * width + x1];
    let p01 = plane[y1 * width + x0];
    let p11 = plane[y1 * width + x1];

    let top = p00 * (1.0 - fx) + p10 * fx;
    let bottom = p01 * (1.0 - fx) + p11 * fx;

    top * (1.0 - fy) + bottom * fy
}

/// Bilinear sampling of an RGBA FrameBuffer.
#[inline(always)]
pub fn sample_bilinear_rgba(frame: &FrameBuffer, pos: Vec2) -> ColorRgba {
    let width = frame.width;
    let height = frame.height;

    let x = pos.x.clamp(0.0, (width - 1) as f32);
    let y = pos.y.clamp(0.0, (height - 1) as f32);

    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);

    let fx = x - x0 as f32;
    let fy = y - y0 as f32;

    let c00 = frame.get_pixel_rgba(x0, y0);
    let c10 = frame.get_pixel_rgba(x1, y0);
    let c01 = frame.get_pixel_rgba(x0, y1);
    let c11 = frame.get_pixel_rgba(x1, y1);

    let top_r = c00.r * (1.0 - fx) + c10.r * fx;
    let top_g = c00.g * (1.0 - fx) + c10.g * fx;
    let top_b = c00.b * (1.0 - fx) + c10.b * fx;
    let top_a = c00.a * (1.0 - fx) + c10.a * fx;

    let bot_r = c01.r * (1.0 - fx) + c11.r * fx;
    let bot_g = c01.g * (1.0 - fx) + c11.g * fx;
    let bot_b = c01.b * (1.0 - fx) + c11.b * fx;
    let bot_a = c01.a * (1.0 - fx) + c11.a * fx;

    ColorRgba {
        r: top_r * (1.0 - fy) + bot_r * fy,
        g: top_g * (1.0 - fy) + bot_g * fy,
        b: top_b * (1.0 - fy) + bot_b * fy,
        a: top_a * (1.0 - fy) + bot_a * fy,
    }
}
