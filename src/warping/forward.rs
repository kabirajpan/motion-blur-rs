//! Forward splatting warp with coverage weight accumulation.

use crate::optical_flow::FlowField;
use crate::plugin::frame::{ColorRgba, FrameBuffer};

/// Forward splats pixels from `source` along forward `flow` scaled by `time_offset`.
pub fn warp_forward_with_coverage(
    source: &FrameBuffer,
    flow: &FlowField,
    time_offset: f32,
    output: &mut FrameBuffer,
) {
    let width = source.width;
    let height = source.height;

    let mut accum_r = vec![0.0f32; width * height];
    let mut accum_g = vec![0.0f32; width * height];
    let mut accum_b = vec![0.0f32; width * height];
    let mut accum_a = vec![0.0f32; width * height];
    let mut coverage = vec![0.0f32; width * height];

    for y in 0..height {
        for x in 0..width {
            let v = flow.get(x, y);
            let target_x = x as f32 + v.x * time_offset;
            let target_y = y as f32 + v.y * time_offset;

            if target_x >= 0.0 && target_x < (width - 1) as f32 && target_y >= 0.0 && target_y < (height - 1) as f32 {
                let x0 = target_x.floor() as usize;
                let y0 = target_y.floor() as usize;
                let x1 = x0 + 1;
                let y1 = y0 + 1;

                let fx = target_x - x0 as f32;
                let fy = target_y - y0 as f32;

                let c = source.get_pixel_rgba(x, y);

                // Distribute splat weights across 4 neighboring pixels
                let w00 = (1.0 - fx) * (1.0 - fy);
                let w10 = fx * (1.0 - fy);
                let w01 = (1.0 - fx) * fy;
                let w11 = fx * fy;

                let neighbors = [(x0, y0, w00), (x1, y0, w10), (x0, y1, w01), (x1, y1, w11)];

                for (nx, ny, weight) in neighbors {
                    if nx < width && ny < height {
                        let idx = ny * width + nx;
                        accum_r[idx] += c.r * weight;
                        accum_g[idx] += c.g * weight;
                        accum_b[idx] += c.b * weight;
                        accum_a[idx] += c.a * weight;
                        coverage[idx] += weight;
                    }
                }
            }
        }
    }

    // Normalize splatted pixels by coverage weights
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let cov = coverage[idx];
            if cov > 1e-4 {
                let inv = 1.0 / cov;
                output.set_pixel_rgba(
                    x,
                    y,
                    ColorRgba::new(
                        accum_r[idx] * inv,
                        accum_g[idx] * inv,
                        accum_b[idx] * inv,
                        accum_a[idx] * inv,
                    ),
                );
            } else {
                // Disocclusion hole: preserve base color from source
                output.set_pixel_rgba(x, y, source.get_pixel_rgba(x, y));
            }
        }
    }
}
