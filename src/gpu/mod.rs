//! Modern GPU Compute Shaders (WGSL) for mobile and desktop video pipelines.

pub const PYRAMID_SHADER_WGSL: &str = include_str!("shaders/pyramid.wgsl");
pub const FLOW_SHADER_WGSL: &str = include_str!("shaders/flow.wgsl");
pub const SHUTTER_SHADER_WGSL: &str = include_str!("shaders/shutter.wgsl");
