pub mod boundary;
pub mod consistency;

pub use boundary::{apply_alpha_boost, detect_motion_boundaries};
pub use consistency::{compute_occlusion_mask, OcclusionMask};
