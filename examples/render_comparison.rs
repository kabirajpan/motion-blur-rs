use motion_blur_rs::*;
use std::collections::HashMap;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rendering Original vs Full Motion Blur Comparison ===");
    let video_path = "/home/kabir/Desktop/exe/RSMB7AE/extracted_RSMBPro/motion_blur_rs/aftermotion_export_1791711678.mp4";
    let width = 1920;
    let height = 1080;

    // 1. Extract frames 35, 36, 37 (high motion section)
    // Frame 35 = 35 / 30 = 1.167s
    println!("Extracting frames 35, 36, 37...");
    let extract_cmd = Command::new("ffmpeg")
        .args([
            "-y",
            "-ss", "00:00:01.16",
            "-i", video_path,
            "-vframes", "3",
            "-f", "rawvideo",
            "-pix_fmt", "rgba",
            "/tmp/comparison_3frames.raw",
        ])
        .output()?;

    if !extract_cmd.status.success() {
        return Err("Failed to extract frames with ffmpeg".into());
    }

    let raw_bytes = std::fs::read("/tmp/comparison_3frames.raw")?;
    let frame_size = width * height * 4;

    let mut frame_prev = FrameBuffer::new(width, height, PixelFormat::Rgba8);
    let mut frame_curr = FrameBuffer::new(width, height, PixelFormat::Rgba8);
    let mut frame_next = FrameBuffer::new(width, height, PixelFormat::Rgba8);

    frame_prev.data = raw_bytes[0..frame_size].to_vec();
    frame_curr.data = raw_bytes[frame_size..2 * frame_size].to_vec();
    frame_next.data = raw_bytes[2 * frame_size..3 * frame_size].to_vec();

    // 2. Configure Plugin with FULL BLUR AMOUNT and Box Shutter
    let plugin = create_motion_blur_plugin();
    let mut params = PluginParameters::new();
    params.set(MotionBlurPlugin::PARAM_BLUR_AMOUNT, ParameterValue::Float(3.5)); // Full heavy blur!
    params.set(MotionBlurPlugin::PARAM_SAMPLES, ParameterValue::Int(24));
    params.set(
        MotionBlurPlugin::PARAM_SHUTTER_CURVE,
        ParameterValue::Choice {
            index: 0, // 0 = Box Shutter (Maximum visible streak!)
            options: vec!["Box".into(), "Gaussian".into(), "Triangular".into(), "Trapezoidal".into()],
        },
    );

    let mut inputs = HashMap::new();
    inputs.insert(MotionBlurPlugin::LABEL_PREVIOUS, &frame_prev);
    inputs.insert(MotionBlurPlugin::LABEL_CURRENT, &frame_curr);
    inputs.insert(MotionBlurPlugin::LABEL_NEXT, &frame_next);

    let context = RenderContext {
        current_time: TimeCode::new(1.2, 30.0),
        width,
        height,
        backend: ExecutionBackend::CpuMultithreaded,
    };

    let mut output_frame = FrameBuffer::new(width, height, PixelFormat::Rgba8);

    println!("Rendering frame with FULL motion blur (Blur Amount = 3.5, 24 samples)...");
    plugin.render(&context, &params, &inputs, &mut output_frame)?;

    // 3. Create Side-by-Side Split Frame:
    // Left half: Original (None), Right half: Implemented Motion Blur!
    let mut split_frame = FrameBuffer::new(width, height, PixelFormat::Rgba8);
    let split_x = width / 2;

    for y in 0..height {
        for x in 0..width {
            let col = if x < split_x {
                frame_curr.get_pixel_rgba(x, y)
            } else if x == split_x || x == split_x + 1 {
                // Divider line (Bright Yellow)
                ColorRgba::new(1.0, 0.9, 0.0, 1.0)
            } else {
                output_frame.get_pixel_rgba(x, y)
            };
            split_frame.set_pixel_rgba(x, y, col);
        }
    }

    // 4. Save PNG files using ffmpeg
    std::fs::write("/tmp/orig.raw", &frame_curr.data)?;
    std::fs::write("/tmp/blurred.raw", &output_frame.data)?;
    std::fs::write("/tmp/split.raw", &split_frame.data)?;

    let out_dir = "/home/kabir/Desktop/exe/RSMB7AE/extracted_RSMBPro/motion_blur_rs/output";

    let save_png = |raw_path: &str, png_name: &str| -> Result<(), Box<dyn std::error::Error>> {
        let dest = format!("{}/{}", out_dir, png_name);
        Command::new("ffmpeg")
            .args([
                "-y",
                "-f", "rawvideo",
                "-pix_fmt", "rgba",
                "-s", "1920x1080",
                "-i", raw_path,
                &dest,
            ])
            .output()?;
        println!("Saved: {}", dest);
        Ok(())
    };

    save_png("/tmp/orig.raw", "frame_none_original.png")?;
    save_png("/tmp/blurred.raw", "frame_full_motion_blur.png")?;
    save_png("/tmp/split.raw", "frame_comparison_split.png")?;

    println!("[SUCCESS] All comparison images generated in {}!", out_dir);
    Ok(())
}
