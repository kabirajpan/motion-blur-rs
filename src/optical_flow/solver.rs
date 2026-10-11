//! Local differential optical flow solver with structure tensor regularization.

use super::types::FlowField;
use crate::math::gradient::compute_gradient_at;
use crate::math::sampling::sample_bilinear_plane;
use crate::math::vector::Vec2;
use rayon::prelude::*;

/// Solves residual optical flow at a single pyramid level given an initial motion estimate.
pub fn solve_flow_level(
    plane0: &[f32],
    plane1: &[f32],
    width: usize,
    height: usize,
    initial_flow: &FlowField,
    window_radius: usize,
    sensitivity: f32,
) -> FlowField {
    let mut refined_flow = FlowField::new(width, height);
    let r = window_radius as isize;
    let lambda = sensitivity.max(1e-4);

    // Warp plane1 backward by initial_flow to compute residual temporal differences
    let mut warped_plane1 = vec![0.0f32; width * height];
    for y in 0..height {
        for x in 0..width {
            let v = initial_flow.get(x, y);
            let sample_pos = Vec2::new(x as f32 + v.x, y as f32 + v.y);
            warped_plane1[y * width + x] = sample_bilinear_plane(plane1, width, height, sample_pos);
        }
    }

    // Process scanlines in parallel via Rayon
    refined_flow
        .vectors
        .par_chunks_mut(width)
        .enumerate()
        .for_each(|(y, row)| {
            for x in 0..width {
                let mut sum_ix2 = 0.0f32;
                let mut sum_iy2 = 0.0f32;
                let mut sum_ix_iy = 0.0f32;
                let mut sum_ix_it = 0.0f32;
                let mut sum_iy_it = 0.0f32;

                // Accumulate spatial and temporal gradients over local integration window
                for dy in -r..=r {
                    let wy = (y as isize + dy).clamp(0, height as isize - 1) as usize;
                    for dx in -r..=r {
                        let wx = (x as isize + dx).clamp(0, width as isize - 1) as usize;

                        let grad = compute_gradient_at(
                            plane0,
                            &warped_plane1,
                            width,
                            height,
                            wx,
                            wy,
                        );

                        sum_ix2 += grad.ix * grad.ix;
                        sum_iy2 += grad.iy * grad.iy;
                        sum_ix_iy += grad.ix * grad.iy;
                        sum_ix_it += grad.ix * grad.it;
                        sum_iy_it += grad.iy * grad.it;
                    }
                }

                // Regularized Structure Tensor: G = [sum_ix2, sum_ix_iy; sum_ix_iy, sum_iy2] + lambda * I
                let g00 = sum_ix2 + lambda;
                let g01 = sum_ix_iy;
                let g10 = sum_ix_iy;
                let g11 = sum_iy2 + lambda;

                let det = g00 * g11 - g01 * g10;
                let residual_v = if det.abs() > 1e-6 {
                    let inv_det = 1.0 / det;
                    let delta_u = inv_det * (g11 * (-sum_ix_it) - g01 * (-sum_iy_it));
                    let delta_v = inv_det * (-g10 * (-sum_ix_it) + g00 * (-sum_iy_it));
                    Vec2::new(delta_u, delta_v).clamp_length(5.0)
                } else {
                    Vec2::ZERO
                };

                let base_v = initial_flow.get(x, y);
                row[x] = base_v + residual_v;
            }
        });

    refined_flow
}
