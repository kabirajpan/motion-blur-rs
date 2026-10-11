//! Motion Blur Plugin implementing the professional VideoPlugin interface.

use crate::occlusion::apply_alpha_boost;
use crate::optical_flow::{compute_optical_flow, FlowField, FlowOptions};
use crate::plugin::*;
use crate::shutter::{accumulate_shutter_blur, ShutterCurve};

/// Production-ready Motion Blur Video Plugin.
#[derive(Debug, Default)]
pub struct MotionBlurPlugin;

impl MotionBlurPlugin {
    pub const PARAM_BLUR_AMOUNT: &'static str = "blur_amount";
    pub const PARAM_SENSITIVITY: &'static str = "motion_sensitivity";
    pub const PARAM_TRACKING_BALANCE: &'static str = "tracking_balance";
    pub const PARAM_SAMPLES: &'static str = "samples";
    pub const PARAM_SHUTTER_CURVE: &'static str = "shutter_curve";
    pub const PARAM_ALPHA_BOOST: &'static str = "alpha_boost";

    pub const LABEL_CURRENT: &'static str = "current";
    pub const LABEL_PREVIOUS: &'static str = "previous";
    pub const LABEL_NEXT: &'static str = "next";
    pub const LABEL_MATTE: &'static str = "matte";
}

impl VideoPlugin for MotionBlurPlugin {
    fn plugin_id(&self) -> &'static str {
        "com.editor.plugins.motion_blur"
    }

    fn display_name(&self) -> &'static str {
        "Pro Motion Blur"
    }

    fn category(&self) -> &'static str {
        "Time & Motion"
    }

    fn describe_parameters(&self) -> Vec<ParameterDescriptor> {
        vec![
            ParameterDescriptor {
                id: Self::PARAM_BLUR_AMOUNT,
                name: "Blur Amount",
                description: "Scales the motion blur length (0.5 = 180 deg shutter angle)",
                default_value: ParameterValue::Float(0.5),
                min_value: Some(0.0),
                max_value: Some(5.0),
            },
            ParameterDescriptor {
                id: Self::PARAM_SENSITIVITY,
                name: "Motion Sensitivity",
                description: "Noise tolerance threshold for optical flow estimation",
                default_value: ParameterValue::Float(0.05),
                min_value: Some(0.001),
                max_value: Some(0.5),
            },
            ParameterDescriptor {
                id: Self::PARAM_TRACKING_BALANCE,
                name: "Tracking Balance",
                description: "Weight between previous frame (0.0) and next frame (1.0)",
                default_value: ParameterValue::Float(0.5),
                min_value: Some(0.0),
                max_value: Some(1.0),
            },
            ParameterDescriptor {
                id: Self::PARAM_SAMPLES,
                name: "Sub-Frame Samples",
                description: "Number of integration steps along the motion trajectory",
                default_value: ParameterValue::Int(16),
                min_value: Some(2.0),
                max_value: Some(64.0),
            },
            ParameterDescriptor {
                id: Self::PARAM_SHUTTER_CURVE,
                name: "Shutter Profile",
                description: "Temporal exposure weighting curve",
                default_value: ParameterValue::Choice {
                    index: 1, // Gaussian by default
                    options: vec![
                        "Box".to_string(),
                        "Gaussian".to_string(),
                        "Triangular".to_string(),
                        "Trapezoidal".to_string(),
                    ],
                },
                min_value: None,
                max_value: None,
            },
            ParameterDescriptor {
                id: Self::PARAM_ALPHA_BOOST,
                name: "Foreground Alpha Boost",
                description: "Boosts low-alpha edges on transparent moving objects",
                default_value: ParameterValue::Float(1.0),
                min_value: Some(1.0),
                max_value: Some(10.0),
            },
        ]
    }

    fn get_frame_dependencies(&self, current_time: TimeCode) -> Vec<TemporalFrameRequest> {
        vec![
            TemporalFrameRequest::new(Self::LABEL_CURRENT, current_time, 0),
            TemporalFrameRequest::new(Self::LABEL_PREVIOUS, current_time.offset_by_frames(-1), -1),
            TemporalFrameRequest::new(Self::LABEL_NEXT, current_time.offset_by_frames(1), 1),
        ]
    }

    fn render(
        &self,
        _context: &RenderContext,
        params: &PluginParameters,
        inputs: &FrameInputMap,
        output: &mut FrameBuffer,
    ) -> Result<(), PluginError> {
        // 1. Fetch required input frames
        let frame_curr = inputs
            .get(Self::LABEL_CURRENT)
            .ok_or(PluginError::MissingInputFrame(Self::LABEL_CURRENT))?;

        let width = frame_curr.width;
        let height = frame_curr.height;

        if output.width != width || output.height != height {
            return Err(PluginError::DimensionMismatch {
                expected: (width, height),
                found: (output.width, output.height),
            });
        }

        // 2. Unpack parameter values
        let blur_amount = params.get_float(Self::PARAM_BLUR_AMOUNT, 0.5);
        let sensitivity = params.get_float(Self::PARAM_SENSITIVITY, 0.05);
        let tracking_balance = params.get_float(Self::PARAM_TRACKING_BALANCE, 0.5);
        let num_samples = params.get_int(Self::PARAM_SAMPLES, 16) as usize;
        let curve_idx = params.get_choice_index(Self::PARAM_SHUTTER_CURVE, 1);
        let alpha_boost = params.get_float(Self::PARAM_ALPHA_BOOST, 1.0);

        let shutter_curve = match curve_idx {
            0 => ShutterCurve::Box,
            1 => ShutterCurve::Gaussian,
            2 => ShutterCurve::Triangular,
            _ => ShutterCurve::Trapezoidal,
        };

        // 3. Early return if blur amount is negligible
        if blur_amount <= 1e-4 {
            for y in 0..height {
                for x in 0..width {
                    output.set_pixel_rgba(x, y, frame_curr.get_pixel_rgba(x, y));
                }
            }
            return Ok(());
        }

        // 4. Extract luminance representation for motion estimation
        let mut curr_plane = frame_curr.to_luminance_plane();

        // Apply alpha boost if matte is present
        if let Some(frame_matte) = inputs.get(Self::LABEL_MATTE) {
            let mut matte_plane = frame_matte.to_luminance_plane();
            apply_alpha_boost(&mut matte_plane, alpha_boost);
            for i in 0..curr_plane.len() {
                curr_plane[i] *= matte_plane[i];
            }
        }

        let flow_options = FlowOptions {
            pyramid_levels: 4,
            window_radius: 2,
            sensitivity,
            iterations: 3,
        };

        // 5. Calculate motion vectors
        let mut blended_flow = FlowField::new(width, height);

        let has_prev = inputs.get(Self::LABEL_PREVIOUS);
        let has_next = inputs.get(Self::LABEL_NEXT);

        match (has_prev, has_next) {
            (Some(frame_prev), Some(frame_next)) => {
                let prev_plane = frame_prev.to_luminance_plane();
                let next_plane = frame_next.to_luminance_plane();

                // Compute backward and forward optical flow
                let flow_bwd = compute_optical_flow(&curr_plane, &prev_plane, width, height, flow_options);
                let flow_fwd = compute_optical_flow(&curr_plane, &next_plane, width, height, flow_options);

                // Blend forward and backward flow fields
                let w_next = tracking_balance;
                let w_prev = 1.0 - tracking_balance;

                for y in 0..height {
                    for x in 0..width {
                        let vb = flow_bwd.get(x, y);
                        let vf = flow_fwd.get(x, y);
                        // Forward motion points in direction of next frame; backward motion is negated
                        let motion = vf * w_next - vb * w_prev;
                        blended_flow.set(x, y, motion);
                    }
                }
            }
            (Some(frame_prev), None) => {
                let prev_plane = frame_prev.to_luminance_plane();
                let flow_bwd = compute_optical_flow(&curr_plane, &prev_plane, width, height, flow_options);
                for y in 0..height {
                    for x in 0..width {
                        blended_flow.set(x, y, flow_bwd.get(x, y) * (-1.0));
                    }
                }
            }
            (None, Some(frame_next)) => {
                let next_plane = frame_next.to_luminance_plane();
                blended_flow = compute_optical_flow(&curr_plane, &next_plane, width, height, flow_options);
            }
            (None, None) => {
                // No adjacent frames: cannot calculate motion
                for y in 0..height {
                    for x in 0..width {
                        output.set_pixel_rgba(x, y, frame_curr.get_pixel_rgba(x, y));
                    }
                }
                return Ok(());
            }
        }

        // 6. Perform 4D temporal shutter accumulation
        accumulate_shutter_blur(
            frame_curr,
            &blended_flow,
            blur_amount,
            num_samples,
            shutter_curve,
            output,
        );

        Ok(())
    }
}
