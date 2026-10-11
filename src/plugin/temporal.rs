//! Temporal frame negotiation and dependency tracking.

/// High-precision timeline time representation in seconds.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TimeCode {
    pub seconds: f64,
    pub frame_rate: f64,
}

impl TimeCode {
    pub fn new(seconds: f64, frame_rate: f64) -> Self {
        Self { seconds, frame_rate }
    }

    pub fn frame_index(&self) -> i64 {
        (self.seconds * self.frame_rate).round() as i64
    }

    pub fn offset_by_frames(&self, frames: i64) -> Self {
        let frame_duration = 1.0 / self.frame_rate;
        Self {
            seconds: self.seconds + (frames as f64 * frame_duration),
            frame_rate: self.frame_rate,
        }
    }
}

/// Request for an auxiliary frame on the timeline needed to render the current frame.
#[derive(Debug, Clone, PartialEq)]
pub struct TemporalFrameRequest {
    /// Identifier label (e.g., "current", "previous", "next", "matte")
    pub label: &'static str,
    /// Absolute time code requested from the host
    pub target_time: TimeCode,
    /// Relative offset from current frame
    pub frame_offset: i64,
}

impl TemporalFrameRequest {
    pub fn new(label: &'static str, target_time: TimeCode, frame_offset: i64) -> Self {
        Self {
            label,
            target_time,
            frame_offset,
        }
    }
}
