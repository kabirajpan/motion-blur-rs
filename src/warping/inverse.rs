//! Inverse image warping with sub-pixel bilinear interpolation.

use crate::math::sampling::sample_bilinear_rgba;
use crate::math::vector::Vec2;
use crate::optical_flow::FlowField;
use crate::plugin::frame::FrameBuffer;

/// Warps `source` backward along `flow` vectors scaled by `time_offset`.
pub fn warp_inverse(
    source: &FrameBuffer,
    flow: &FlowField,
    time_offset: f32,
    output: &mut FrameBuffer,
) {
    let width = source.width;
    let height = source.height;

    for y in 0..height {
        for x in 0..width {
            let v = flow.get(x, y);
            let sample_pos = Vec2::new(
                x as f32 + v.x * time_offset,
                y as f32 + v.y * time_offset,
            );

            let color = sample_bilinear_rgba(source, sample_pos);
            output.set_pixel_rgba(x, y, color);
        }
    }
}
