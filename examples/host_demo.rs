//! Host Integration Demo: Simulates how a video editor engine discovers,
//! configures, and invokes the motion blur plugin.

use motion_blur_rs::*;
use std::collections::HashMap;

fn main() {
    println!("=== Video Editor Plugin Host Engine Simulator ===");

    // 1. Discover and instantiate the plugin
    let plugin = create_motion_blur_plugin();
    println!("Loaded Plugin: {} [{}]", plugin.display_name(), plugin.plugin_id());
    println!("Category:      {}", plugin.category());

    // 2. Enumerate parameters for the UI inspector
    println!("\nRegistered UI Parameters:");
    for param in plugin.describe_parameters() {
        println!("  - [{}] {}: {}", param.id, param.name, param.description);
    }

    // 3. Negotiate temporal dependencies at timeline position t = 1.0s (30 fps)
    let current_time = TimeCode::new(1.0, 30.0);
    let dependencies = plugin.get_frame_dependencies(current_time);
    println!("\nTemporal Frame Requests for timeline t={:.2}s:", current_time.seconds);
    for req in &dependencies {
        println!(
            "  * Slot '{}' -> Offset {} frames ({:.3}s)",
            req.label, req.frame_offset, req.target_time.seconds
        );
    }

    // 4. Synthesize test video frames: a white circle translating horizontally
    let width = 128;
    let height = 128;

    let mut frame_prev = FrameBuffer::new(width, height, PixelFormat::Rgba8);
    let mut frame_curr = FrameBuffer::new(width, height, PixelFormat::Rgba8);
    let mut frame_next = FrameBuffer::new(width, height, PixelFormat::Rgba8);

    draw_circle(&mut frame_prev, 35, 64, 15, ColorRgba::WHITE); // x = 35
    draw_circle(&mut frame_curr, 64, 64, 15, ColorRgba::WHITE); // x = 64 (center)
    draw_circle(&mut frame_next, 93, 64, 15, ColorRgba::WHITE); // x = 93

    println!("\nSynthesized 3 consecutive frames with horizontal motion (dx = +29 px/frame).");

    // 5. Configure user parameters
    let mut params = PluginParameters::new();
    params.set(
        MotionBlurPlugin::PARAM_BLUR_AMOUNT,
        ParameterValue::Float(0.8), // 0.8 shutter length
    );
    params.set(
        MotionBlurPlugin::PARAM_SAMPLES,
        ParameterValue::Int(16),
    );
    params.set(
        MotionBlurPlugin::PARAM_SHUTTER_CURVE,
        ParameterValue::Choice {
            index: 1, // Gaussian shutter
            options: vec!["Box".into(), "Gaussian".into(), "Triangular".into(), "Trapezoidal".into()],
        },
    );

    // 6. Assemble input frame map
    let mut inputs: FrameInputMap = HashMap::new();
    inputs.insert(MotionBlurPlugin::LABEL_PREVIOUS, &frame_prev);
    inputs.insert(MotionBlurPlugin::LABEL_CURRENT, &frame_curr);
    inputs.insert(MotionBlurPlugin::LABEL_NEXT, &frame_next);

    // 7. Execute render pipeline
    let context = RenderContext {
        current_time,
        width,
        height,
        backend: ExecutionBackend::CpuMultithreaded,
    };

    let mut output = FrameBuffer::new(width, height, PixelFormat::Rgba8);

    let start = std::time::Instant::now();
    match plugin.render(&context, &params, &inputs, &mut output) {
        Ok(()) => {
            let elapsed = start.elapsed();
            println!(
                "\n[SUCCESS] Rendered motion blur frame ({}x{}) in {:.2} ms!",
                width,
                height,
                elapsed.as_secs_f64() * 1000.0
            );

            // Verify blurring along the horizontal axis
            let center_pixel = output.get_pixel_rgba(64, 64);
            let edge_pixel = output.get_pixel_rgba(50, 64);
            println!("Center Pixel (64, 64) Alpha/Luma: {:.2}", center_pixel.luminance());
            println!("Trail Pixel  (50, 64) Alpha/Luma: {:.2}", edge_pixel.luminance());
        }
        Err(e) => {
            eprintln!("[ERROR] Plugin render failed: {}", e);
        }
    }
}

/// Helper to rasterize a solid circle for testing.
fn draw_circle(buffer: &mut FrameBuffer, cx: usize, cy: usize, radius: usize, color: ColorRgba) {
    let r_sq = (radius * radius) as isize;
    for y in 0..buffer.height {
        let dy = y as isize - cy as isize;
        for x in 0..buffer.width {
            let dx = x as isize - cx as isize;
            if dx * dx + dy * dy <= r_sq {
                buffer.set_pixel_rgba(x, y, color);
            }
        }
    }
}
