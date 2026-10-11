//! Motion boundary detection and alpha boost preconditioning.

use crate::optical_flow::FlowField;

/// Detects motion boundaries by measuring spatial gradients across the velocity field.
pub fn detect_motion_boundaries(flow: &FlowField, threshold: f32) -> Vec<f32> {
    let width = flow.width;
    let height = flow.height;
    let mut boundaries = vec![0.0f32; width * height];

    for y in 0..height {
        let ym1 = if y > 0 { y - 1 } else { 0 };
        let yp1 = (y + 1).min(height - 1);

        for x in 0..width {
            let xm1 = if x > 0 { x - 1 } else { 0 };
            let xp1 = (x + 1).min(width - 1);

            let vx_dx = flow.get(xp1, y).x - flow.get(xm1, y).x;
            let vy_dx = flow.get(xp1, y).y - flow.get(xm1, y).y;
            let vx_dy = flow.get(x, yp1).x - flow.get(x, ym1).x;
            let vy_dy = flow.get(x, yp1).y - flow.get(x, ym1).y;

            let gradient_mag = (vx_dx * vx_dx + vy_dx * vy_dx + vx_dy * vx_dy + vy_dy * vy_dy).sqrt();
            boundaries[y * width + x] = if gradient_mag > threshold { 1.0 } else { 0.0 };
        }
    }

    boundaries
}

/// Applies Alpha Boost on foreground transparent boundaries (RSMB Pro technique).
pub fn apply_alpha_boost(matte: &mut [f32], boost_factor: f32) {
    if boost_factor <= 1.0 {
        return;
    }

    for val in matte.iter_mut() {
        if *val > 0.01 && *val < 0.99 {
            *val = (*val * boost_factor).min(1.0);
        }
    }
}
