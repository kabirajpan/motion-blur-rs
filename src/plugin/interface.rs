//! Professional Video Plugin Trait Definition.

use super::frame::FrameBuffer;
use super::params::{ParameterDescriptor, PluginParameters};
use super::temporal::{TemporalFrameRequest, TimeCode};
use std::collections::HashMap;

/// Hardware backend used to execute the plugin logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionBackend {
    /// Multi-threaded CPU with SIMD acceleration
    CpuMultithreaded,
    /// Hardware GPU Compute (Metal / Vulkan / DX12 via wgpu)
    GpuCompute,
}

/// Execution environment and rendering metadata passed by the host video engine.
#[derive(Debug, Clone)]
pub struct RenderContext {
    pub current_time: TimeCode,
    pub width: usize,
    pub height: usize,
    pub backend: ExecutionBackend,
}

/// Map containing the input frames provided by the host satisfying the plugin's requests.
pub type FrameInputMap<'a> = HashMap<&'static str, &'a FrameBuffer>;

/// Core interface that every video plugin in the video editor engine must implement.
pub trait VideoPlugin: Send + Sync {
    /// Unique machine-readable identifier (e.g., "com.editor.plugins.motion_blur")
    fn plugin_id(&self) -> &'static str;

    /// Human-readable plugin name (e.g., "Pro Motion Blur")
    fn display_name(&self) -> &'static str;

    /// Plugin category for host UI grouping
    fn category(&self) -> &'static str;

    /// Returns the schema of user-tunable parameters for UI rendering.
    fn describe_parameters(&self) -> Vec<ParameterDescriptor>;

    /// Declares temporal frame dependencies required to render the frame at `current_time`.
    /// E.g., Motion Blur requires frame(t - 1), frame(t), and frame(t + 1).
    fn get_frame_dependencies(&self, current_time: TimeCode) -> Vec<TemporalFrameRequest>;

    /// Executes the rendering pipeline.
    /// Reads inputs from `inputs` and writes the resulting frame to `output`.
    fn render(
        &self,
        context: &RenderContext,
        params: &PluginParameters,
        inputs: &FrameInputMap,
        output: &mut FrameBuffer,
    ) -> Result<(), PluginError>;
}

/// Error conditions encountered during plugin execution.
#[derive(Debug, Clone, PartialEq)]
pub enum PluginError {
    MissingInputFrame(&'static str),
    DimensionMismatch { expected: (usize, usize), found: (usize, usize) },
    UnsupportedPixelFormat,
    ComputeError(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingInputFrame(lbl) => write!(f, "Missing required input frame: '{}'", lbl),
            Self::DimensionMismatch { expected, found } => {
                write!(f, "Frame dimension mismatch: expected {:?}, got {:?}", expected, found)
            }
            Self::UnsupportedPixelFormat => write!(f, "Unsupported pixel format in buffer"),
            Self::ComputeError(msg) => write!(f, "Plugin compute error: {}", msg),
        }
    }
}

impl std::error::Error for PluginError {}
