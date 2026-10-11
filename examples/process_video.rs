//! Video File Processor: Applies the MotionBlurPlugin to real video files using ffmpeg streams.
//!
//! Usage:
//!   cargo run --release --example process_video -- <input_video> <output_video> [blur_amount] [samples]

use motion_blur_rs::*;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: cargo run --release --example process_video -- <input_video> <output_video> [blur_amount=0.5] [samples=16]");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let blur_amount: f32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(3.0);
    let samples: i32 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(20);
    let max_frames: Option<usize> = args.get(5).and_then(|s| s.parse().ok());
    let curve_idx: usize = args.get(6).and_then(|s| s.parse().ok()).unwrap_or(0); // 0 = Box for full visible trail

    println!("=== Motion Blur Video File Processor ===");
    println!("Input Video:   {}", input_path);
    println!("Output Video:  {}", output_path);
    println!("Blur Amount:   {}", blur_amount);
    println!("Samples:       {}", samples);
    if let Some(mf) = max_frames {
        println!("Frame Limit:   {} frames", mf);
    }

    // 1. Probe input video metadata using ffprobe
    let probe_cmd = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height,r_frame_rate,nb_frames",
            "-of", "csv=p=0:s=x",
            input_path,
        ])
        .output()?;

    let probe_output = String::from_utf8_lossy(&probe_cmd.stdout).trim().to_string();
    let parts: Vec<&str> = probe_output.split('x').collect();
    if parts.len() < 3 {
        return Err(format!("Failed to parse video format from ffprobe: '{}'", probe_output).into());
    }

    let width: usize = parts[0].parse()?;
    let height: usize = parts[1].parse()?;
    
    // Parse frame rate (e.g., "30/1" or "29.97")
    let fps_str = parts[2];
    let fps: f64 = if fps_str.contains('/') {
        let frac: Vec<&str> = fps_str.split('/').collect();
        let num: f64 = frac[0].parse().unwrap_or(30.0);
        let den: f64 = frac[1].parse().unwrap_or(1.0);
        num / den
    } else {
        fps_str.parse().unwrap_or(30.0)
    };

    println!("Resolution:    {}x{}", width, height);
    println!("Frame Rate:    {:.2} fps", fps);

    // 2. Instantiate Motion Blur Plugin
    let plugin = create_motion_blur_plugin();
    let mut params = PluginParameters::new();
    params.set(MotionBlurPlugin::PARAM_BLUR_AMOUNT, ParameterValue::Float(blur_amount));
    params.set(MotionBlurPlugin::PARAM_SAMPLES, ParameterValue::Int(samples));
    params.set(
        MotionBlurPlugin::PARAM_SHUTTER_CURVE,
        ParameterValue::Choice {
            index: curve_idx,
            options: vec!["Box".into(), "Gaussian".into(), "Triangular".into(), "Trapezoidal".into()],
        },
    );

    // 3. Start ffmpeg decoder pipe (extracts raw RGBA frames)
    let mut decoder = Command::new("ffmpeg")
        .args([
            "-i", input_path,
            "-f", "rawvideo",
            "-pix_fmt", "rgba",
            "pipe:1",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let mut decoder_stdout = decoder.stdout.take().ok_or("Failed to open decoder stdout pipe")?;

    // 4. Start ffmpeg encoder pipe (writes encoded H.264 video)
    let fps_arg = format!("{:.2}", fps);
    let size_arg = format!("{}x{}", width, height);
    let mut encoder = Command::new("ffmpeg")
        .args([
            "-y",
            "-f", "rawvideo",
            "-pix_fmt", "rgba",
            "-s", &size_arg,
            "-r", &fps_arg,
            "-i", "pipe:0",
            "-i", input_path,
            "-map", "0:v:0",
            "-map", "1:a:0?",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-preset", "veryfast",
            "-crf", "18",
            "-c:a", "copy",
            "-shortest",
            output_path,
        ])
        .stdin(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let mut encoder_stdin = encoder.stdin.take().ok_or("Failed to open encoder stdin pipe")?;

    let _frame_bytes = width * height * 4;
    let mut read_frame = || -> Option<FrameBuffer> {
        let mut buffer = FrameBuffer::new(width, height, PixelFormat::Rgba8);
        if decoder_stdout.read_exact(&mut buffer.data).is_ok() {
            Some(buffer)
        } else {
            None
        }
    };

    println!("\nStreaming and processing frames through MotionBlurPlugin pipeline...");

    let mut frame_index: i64 = 0;
    let mut output_frame = FrameBuffer::new(width, height, PixelFormat::Rgba8);

    // Sliding window buffer: [previous, current, next]
    let mut prev_opt: Option<FrameBuffer> = None;
    let mut curr_opt: Option<FrameBuffer> = read_frame();
    let mut next_opt: Option<FrameBuffer> = read_frame();

    let start_time = std::time::Instant::now();

    while let Some(curr_frame) = curr_opt {
        let current_time = TimeCode::new(frame_index as f64 / fps, fps);
        let context = RenderContext {
            current_time,
            width,
            height,
            backend: ExecutionBackend::CpuMultithreaded,
        };

        let mut inputs: FrameInputMap = HashMap::new();
        inputs.insert(MotionBlurPlugin::LABEL_CURRENT, &curr_frame);
        if let Some(ref p) = prev_opt {
            inputs.insert(MotionBlurPlugin::LABEL_PREVIOUS, p);
        }
        if let Some(ref n) = next_opt {
            inputs.insert(MotionBlurPlugin::LABEL_NEXT, n);
        }

        // Apply motion blur plugin
        plugin.render(&context, &params, &inputs, &mut output_frame)?;

        // Write processed frame to ffmpeg encoder
        encoder_stdin.write_all(&output_frame.data)?;

        frame_index += 1;
        if let Some(limit) = max_frames {
            if frame_index >= limit as i64 {
                println!("\nReached requested limit of {} frames.", limit);
                break;
            }
        }
        if frame_index % 10 == 0 || frame_index == 1 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let proc_fps = frame_index as f64 / elapsed.max(0.001);
            print!("\rProcessed frame {:4} ({:.2} fps)", frame_index, proc_fps);
            std::io::stdout().flush().ok();
        }

        // Advance sliding window
        prev_opt = Some(curr_frame);
        curr_opt = next_opt;
        next_opt = read_frame();
    }

    drop(encoder_stdin);
    let _ = encoder.wait()?;
    let _ = decoder.wait()?;

    let total_elapsed = start_time.elapsed().as_secs_f64();
    println!(
        "\n[DONE] Successfully processed {} frames in {:.2}s ({:.2} fps)!",
        frame_index, total_elapsed, frame_index as f64 / total_elapsed
    );
    println!("Output saved to: {}", output_path);

    Ok(())
}
