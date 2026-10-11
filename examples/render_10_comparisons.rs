use motion_blur_rs::*;
use std::collections::HashMap;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rendering 10 Multi-Point Comparisons Across Video ===");
    let video_path = "/home/kabir/Desktop/exe/RSMB7AE/extracted_RSMBPro/motion_blur_rs/aftermotion_export_1791711678.mp4";
    let width = 1920;
    let height = 1080;
    let frame_bytes = width * height * 4;

    // 10 sample points distributed across the 9-second video (274 frames total)
    let sample_frames = [15, 42, 70, 98, 125, 155, 185, 215, 242, 265];

    let plugin = create_motion_blur_plugin();
    let mut params = PluginParameters::new();
    params.set(MotionBlurPlugin::PARAM_BLUR_AMOUNT, ParameterValue::Float(3.5)); // Full heavy blur
    params.set(MotionBlurPlugin::PARAM_SAMPLES, ParameterValue::Int(24));
    params.set(
        MotionBlurPlugin::PARAM_SHUTTER_CURVE,
        ParameterValue::Choice {
            index: 0, // Box Shutter for maximum visible blur trails
            options: vec!["Box".into(), "Gaussian".into(), "Triangular".into(), "Trapezoidal".into()],
        },
    );

    let out_dir = "/home/kabir/Desktop/exe/RSMB7AE/extracted_RSMBPro/motion_blur_rs/output";

    for (idx, &frame_no) in sample_frames.iter().enumerate() {
        let sec = (frame_no as f64 - 1.0) / 30.0;
        println!(
            "\n[{}/{}] Processing Frame {} (~{:.2}s)...",
            idx + 1,
            sample_frames.len(),
            frame_no,
            sec
        );

        // Extract 3 consecutive frames around frame_no: [frame_no - 1, frame_no, frame_no + 1]
        let extract = Command::new("ffmpeg")
            .args([
                "-y",
                "-ss", &format!("{:.3}", sec),
                "-i", video_path,
                "-vframes", "3",
                "-f", "rawvideo",
                "-pix_fmt", "rgba",
                "/tmp/triplet.raw",
            ])
            .output()?;

        if !extract.status.success() {
            eprintln!("Failed to extract triplet for frame {}", frame_no);
            continue;
        }

        let raw = std::fs::read("/tmp/triplet.raw")?;
        if raw.len() < frame_bytes * 3 {
            eprintln!("Triplet too small for frame {}", frame_no);
            continue;
        }

        let mut frame_prev = FrameBuffer::new(width, height, PixelFormat::Rgba8);
        let mut frame_curr = FrameBuffer::new(width, height, PixelFormat::Rgba8);
        let mut frame_next = FrameBuffer::new(width, height, PixelFormat::Rgba8);

        frame_prev.data = raw[0..frame_bytes].to_vec();
        frame_curr.data = raw[frame_bytes..2 * frame_bytes].to_vec();
        frame_next.data = raw[2 * frame_bytes..3 * frame_bytes].to_vec();

        let mut inputs = HashMap::new();
        inputs.insert(MotionBlurPlugin::LABEL_PREVIOUS, &frame_prev);
        inputs.insert(MotionBlurPlugin::LABEL_CURRENT, &frame_curr);
        inputs.insert(MotionBlurPlugin::LABEL_NEXT, &frame_next);

        let context = RenderContext {
            current_time: TimeCode::new(sec, 30.0),
            width,
            height,
            backend: ExecutionBackend::CpuMultithreaded,
        };

        let mut output_frame = FrameBuffer::new(width, height, PixelFormat::Rgba8);
        plugin.render(&context, &params, &inputs, &mut output_frame)?;

        // Construct split screen: Left = Original, Right = Motion Blurred, Yellow Divider
        let mut split = FrameBuffer::new(width, height, PixelFormat::Rgba8);
        let split_x = width / 2;

        for y in 0..height {
            for x in 0..width {
                let col = if x < split_x {
                    frame_curr.get_pixel_rgba(x, y)
                } else if x == split_x || x == split_x + 1 {
                    ColorRgba::new(1.0, 0.85, 0.0, 1.0) // Gold yellow line
                } else {
                    output_frame.get_pixel_rgba(x, y)
                };
                split.set_pixel_rgba(x, y, col);
            }
        }

        let out_png = format!("{}/compare_{:02}_frame_{}.png", out_dir, idx + 1, frame_no);
        std::fs::write("/tmp/split_temp.raw", &split.data)?;

        Command::new("ffmpeg")
            .args([
                "-y",
                "-f", "rawvideo",
                "-pix_fmt", "rgba",
                "-s", "1920x1080",
                "-i", "/tmp/split_temp.raw",
                &out_png,
            ])
            .output()?;

        println!("  -> Saved: {}", out_png);
    }

    println!("\n[ALL COMPLETE] Generated 10 comparison images in {}", out_dir);
    Ok(())
}
