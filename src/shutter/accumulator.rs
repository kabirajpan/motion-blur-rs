//! 4D Temporal sub-frame shutter accumulation.

use super::curves::ShutterCurve;
use crate::math::sampling::sample_bilinear_rgba;
use crate::math::vector::Vec2;
use crate::optical_flow::FlowField;
use crate::plugin::frame::{ColorRgba, FrameBuffer};
use rayon::prelude::*;

/// Accumulates temporal sub-frame slices along motion vectors to simulate physical shutter exposure.
pub fn accumulate_shutter_blur(
    source: &FrameBuffer,
    flow: &FlowField,
    shutter_amount: f32,
    num_samples: usize,
    curve: ShutterCurve,
    output: &mut FrameBuffer,
) {
    let width = source.width;
    let _height = source.height;
    let samples = num_samples.max(2);

    // Compute sample weights and offsets beforehand
    let mut time_steps = Vec::with_capacity(samples);
    let mut total_weight = 0.0f32;

    for i in 0..samples {
        let t = (i as f32 / (samples - 1) as f32) - 0.5; // [-0.5, 0.5]
        let w = curve.weight(t);
        time_steps.push((t, w));
        total_weight += w;
    }

    let inv_weight = if total_weight > 1e-5 { 1.0 / total_weight } else { 1.0 };

    // Parallel scanline accumulation across CPU cores
    let format = output.format;
    let bpp = match format {
        crate::plugin::PixelFormat::Rgba8 => 4,
        crate::plugin::PixelFormat::Rgba32F => 16,
        crate::plugin::PixelFormat::Luminance32F => 4,
    };

    output
        .data
        .par_chunks_mut(width * bpp)
        .enumerate()
        .for_each(|(y, row_slice)| {
            for x in 0..width {
                let v = flow.get(x, y);

                let col = if v.length_squared() < 1e-4 || shutter_amount <= 1e-4 {
                    source.get_pixel_rgba(x, y)
                } else {
                    let mut accum_r = 0.0f32;
                    let mut accum_g = 0.0f32;
                    let mut accum_b = 0.0f32;
                    let mut accum_a = 0.0f32;

                    for &(t, w) in &time_steps {
                        let offset_t = t * shutter_amount;
                        let sample_pos = Vec2::new(
                            x as f32 + v.x * offset_t,
                            y as f32 + v.y * offset_t,
                        );

                        let c = sample_bilinear_rgba(source, sample_pos);
                        accum_r += c.r * w;
                        accum_g += c.g * w;
                        accum_b += c.b * w;
                        accum_a += c.a * w;
                    }

                    ColorRgba::new(
                        accum_r * inv_weight,
                        accum_g * inv_weight,
                        accum_b * inv_weight,
                        accum_a * inv_weight,
                    )
                };

                // Write pixel into slice
                match format {
                    crate::plugin::PixelFormat::Rgba8 => {
                        let off = x * 4;
                        row_slice[off] = (col.r.clamp(0.0, 1.0) * 255.0).round() as u8;
                        row_slice[off + 1] = (col.g.clamp(0.0, 1.0) * 255.0).round() as u8;
                        row_slice[off + 2] = (col.b.clamp(0.0, 1.0) * 255.0).round() as u8;
                        row_slice[off + 3] = (col.a.clamp(0.0, 1.0) * 255.0).round() as u8;
                    }
                    crate::plugin::PixelFormat::Rgba32F => {
                        let off = x * 16;
                        row_slice[off..off + 4].copy_from_slice(&col.r.to_ne_bytes());
                        row_slice[off + 4..off + 8].copy_from_slice(&col.g.to_ne_bytes());
                        row_slice[off + 8..off + 12].copy_from_slice(&col.b.to_ne_bytes());
                        row_slice[off + 12..off + 16].copy_from_slice(&col.a.to_ne_bytes());
                    }
                    crate::plugin::PixelFormat::Luminance32F => {
                        let off = x * 4;
                        let l = col.luminance();
                        row_slice[off..off + 4].copy_from_slice(&l.to_ne_bytes());
                    }
                }
            }
        });
}
