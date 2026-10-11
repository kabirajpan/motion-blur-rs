//! # motion_blur_rs
//!
//! A modular, production-grade motion estimation and temporal shutter blurring
//! plugin crate for Rust-powered video editing engines (mobile and desktop).

pub mod gpu;
pub mod math;
pub mod motion_blur_plugin;
pub mod occlusion;
pub mod optical_flow;
pub mod plugin;
pub mod pyramid;
pub mod shutter;
pub mod warping;

// Re-export primary interfaces
pub use motion_blur_plugin::MotionBlurPlugin;
pub use plugin::*;
pub use shutter::ShutterCurve;

/// Factory function to instantiate the motion blur plugin.
pub fn create_motion_blur_plugin() -> Box<dyn VideoPlugin> {
    Box::new(MotionBlurPlugin::default())
}
