use motion_blur_rs::*;
use std::collections::HashMap;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rendering 2X Intense Motion Blur (Blur Amount = 7.0, 32 Samples) ===");
    let video_path = "/home/kabir/Desktop/exe/RSMB7AE/extracted_RSMBPro/motion_blur_rs/aftermotion_export_1791711678.mp4";
    let width = 1920;
    let height = 1080;
    let frame_bytes = width * height * 4;

    // 10 new distinct sample frames across the video
    let sample_frames = [25, 55, 85, 110, 140, 170, 200, 230, 250, 270];

    let plugin = create_motion_blur_plugin();
    let mut params = PluginParameters::new();
    params.set(MotionBlurPlugin::PARAM_BLUR_AMOUNT, ParameterValue::Float(7.0)); // 2X INCREASE: 7.0!
    params.set(MotionBlurPlugin::PARAM_SAMPLES, ParameterValue::Int(32)); // 32 samples for ultra-smooth trails
    params.set(
        MotionBlurPlugin::PARAM_SHUTTER_CURVE,
        ParameterValue::Choice {
            index: 0, // Box Shutter
            options: vec!["Box".into(), "Gaussian".into(), "Triangular".into(), "Trapezoidal".into()],
        },
    );

    let out_dir = "/home/kabir/Desktop/exe/RSMB7AE/extracted_RSMBPro/motion_blur_rs/output";

    for (idx, &frame_no) in sample_frames.iter().enumerate() {
        let sec = (frame_no as f64 - 1.0) / 30.0;
        println!(
            "\n[{}/{}] Processing Frame {} (~{:.2}s) with 2x Blur (7.0)...",
            idx + 1,
            sample_frames.len(),
            frame_no,
            sec
        );

        let extract = Command::new("ffmpeg")
            .args([
                "-y",
                "-ss", &format!("{:.3}", sec),
                "-i", video_path,
                "-vframes", "3",
                "-f", "rawvideo",
                "-pix_fmt", "rgba",
                "/tmp/triplet_2x.raw",
            ])
            .output()?;

        if !extract.status.success() {
            eprintln!("Failed to extract triplet for frame {}", frame_no);
            continue;
        }

        let raw = std::fs::read("/tmp/triplet_2x.raw")?;
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

        // Construct split screen: Left = Original, Right = 2x Motion Blur, Yellow Divider
        let mut split = FrameBuffer::new(width, height, PixelFormat::Rgba8);
        let split_x = width / 2;

        for y in 0..height {
            for x in 0..width {
                let col = if x < split_x {
                    frame_curr.get_pixel_rgba(x, y)
                } else if x == split_x || x == split_x + 1 {
                    ColorRgba::new(1.0, 0.85, 0.0, 1.0)
                } else {
                    output_frame.get_pixel_rgba(x, y)
                };
                split.set_pixel_rgba(x, y, col);
            }
        }

        let out_png = format!("{}/compare_2x_{:02}_frame_{}.png", out_dir, idx + 1, frame_no);
        std::fs::write("/tmp/split_2x_temp.raw", &split.data)?;

        Command::new("ffmpeg")
            .args([
                "-y",
                "-f", "rawvideo",
                "-pix_fmt", "rgba",
                "-s", "1920x1080",
                "-i", "/tmp/split_2x_temp.raw",
                &out_png,
            ])
            .output()?;

        println!("  -> Saved: {}", out_png);
    }

    println!("\n[ALL COMPLETE] Generated 10 2X-Blur comparison images in {}", out_dir);
    Ok(())
}
