//! Bidirectional consistency checking for occlusion detection.

use crate::optical_flow::FlowField;

/// Mask representing per-pixel occlusion confidence (0.0 = occluded / invalid, 1.0 = valid motion).
#[derive(Debug, Clone)]
pub struct OcclusionMask {
    pub width: usize,
    pub height: usize,
    pub confidence: Vec<f32>,
}

impl OcclusionMask {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            confidence: vec![1.0f32; width * height],
        }
    }

    #[inline(always)]
    pub fn is_valid(&self, x: usize, y: usize) -> bool {
        self.confidence[y * self.width + x] > 0.5
    }
}

/// Evaluates bidirectional consistency between forward and backward flow fields.
pub fn compute_occlusion_mask(
    flow_forward: &FlowField,
    flow_backward: &FlowField,
    tolerance_threshold: f32,
) -> OcclusionMask {
    let width = flow_forward.width;
    let height = flow_forward.height;
    let mut mask = OcclusionMask::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let vf = flow_forward.get(x, y);

            // Position where forward flow takes pixel (x, y)
            let tx = ((x as f32 + vf.x).round() as isize).clamp(0, width as isize - 1) as usize;
            let ty = ((y as f32 + vf.y).round() as isize).clamp(0, height as isize - 1) as usize;

            let vb = flow_backward.get(tx, ty);

            // Inconsistency vector: should sum to zero if motion is true & consistent
            let inconsistency = (vf + vb).length();

            // Soft confidence penalty
            let confidence = if inconsistency > tolerance_threshold {
                (1.0 - (inconsistency - tolerance_threshold) / tolerance_threshold).max(0.0)
            } else {
                1.0
            };

            mask.confidence[y * width + x] = confidence;
        }
    }

    mask
}
