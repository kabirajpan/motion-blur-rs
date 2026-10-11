//! Coarse-to-fine pyramidal optical flow estimation.

use super::solver::solve_flow_level;
use super::types::{FlowField, FlowOptions};
use crate::pyramid::ImagePyramid;

/// Computes dense optical flow from `plane_source` (frame t) to `plane_target` (frame t +/- 1).
pub fn compute_optical_flow(
    plane_source: &[f32],
    plane_target: &[f32],
    width: usize,
    height: usize,
    options: FlowOptions,
) -> FlowField {
    // 1. Build multi-resolution image pyramids for source and target frames
    let pyr_source = ImagePyramid::build(plane_source, width, height, options.pyramid_levels);
    let pyr_target = ImagePyramid::build(plane_target, width, height, options.pyramid_levels);

    let num_levels = pyr_source.num_levels().min(pyr_target.num_levels());
    if num_levels == 0 {
        return FlowField::new(width, height);
    }

    // 2. Start from coarsest level (lowest resolution)
    let coarsest_level = &pyr_source.levels[num_levels - 1];
    let mut current_flow = FlowField::new(coarsest_level.width, coarsest_level.height);

    for lvl_idx in (0..num_levels).rev() {
        let src_lvl = &pyr_source.levels[lvl_idx];
        let tgt_lvl = &pyr_target.levels[lvl_idx];

        // Upsample flow field if moving from coarser to finer level
        if current_flow.width != src_lvl.width || current_flow.height != src_lvl.height {
            current_flow = current_flow.upsample_to(src_lvl.width, src_lvl.height);
        }

        // Adaptive iteration count: high resolutions require fewer iterations after upsampling
        let iters = if src_lvl.width >= 1280 {
            1
        } else {
            options.iterations
        };

        for _ in 0..iters {
            current_flow = solve_flow_level(
                &src_lvl.plane,
                &tgt_lvl.plane,
                src_lvl.width,
                src_lvl.height,
                &current_flow,
                options.window_radius,
                options.sensitivity,
            );
        }
    }

    current_flow
}
