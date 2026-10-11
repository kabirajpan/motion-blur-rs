use motion_blur_rs::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("/tmp/raw3frames.bin")?;
    let width = 1920;
    let height = 1080;
    let frame_bytes = width * height * 4;

    let mut buf0 = vec![0u8; frame_bytes];
    let mut buf1 = vec![0u8; frame_bytes];
    let mut buf2 = vec![0u8; frame_bytes];

    file.read_exact(&mut buf0)?;
    file.read_exact(&mut buf1)?;
    file.read_exact(&mut buf2)?;

    let mut frame_prev = FrameBuffer::new(width, height, PixelFormat::Rgba8);
    let mut frame_curr = FrameBuffer::new(width, height, PixelFormat::Rgba8);
    let mut frame_next = FrameBuffer::new(width, height, PixelFormat::Rgba8);

    frame_prev.data = buf0;
    frame_curr.data = buf1;
    frame_next.data = buf2;

    let p0 = frame_prev.to_luminance_plane();
    let p1 = frame_curr.to_luminance_plane();
    let p2 = frame_next.to_luminance_plane();

    let options = optical_flow::FlowOptions {
        pyramid_levels: 4,
        window_radius: 2,
        sensitivity: 0.05,
        iterations: 3,
    };

    println!("Calculating optical flow...");
    let flow_fwd = optical_flow::compute_optical_flow(&p1, &p2, width, height, options);
    let flow_bwd = optical_flow::compute_optical_flow(&p1, &p0, width, height, options);

    let mut samples_checked = 0;
    for y in (100..height-100).step_by(200) {
        for x in (100..width-100).step_by(300) {
            let vf = flow_fwd.get(x, y);
            let vb = flow_bwd.get(x, y);
            let blended = vf * 0.5 - vb * 0.5;
            println!("Pixel ({}, {}): vf = ({:.2}, {:.2}), vb = ({:.2}, {:.2}) -> blended = ({:.2}, {:.2})",
                x, y, vf.x, vf.y, vb.x, vb.y, blended.x, blended.y);
            samples_checked += 1;
            if samples_checked >= 8 { break; }
        }
        if samples_checked >= 8 { break; }
    }

    let plugin = create_motion_blur_plugin();
    let mut params = PluginParameters::new();
    params.set(MotionBlurPlugin::PARAM_BLUR_AMOUNT, ParameterValue::Float(1.5));
    params.set(MotionBlurPlugin::PARAM_SAMPLES, ParameterValue::Int(16));

    let mut inputs = HashMap::new();
    inputs.insert(MotionBlurPlugin::LABEL_CURRENT, &frame_curr);
    inputs.insert(MotionBlurPlugin::LABEL_PREVIOUS, &frame_prev);
    inputs.insert(MotionBlurPlugin::LABEL_NEXT, &frame_next);

    let mut output = FrameBuffer::new(width, height, PixelFormat::Rgba8);
    let ctx = RenderContext {
        current_time: TimeCode::new(0.0, 30.0),
        width,
        height,
        backend: ExecutionBackend::CpuMultithreaded,
    };

    plugin.render(&ctx, &params, &inputs, &mut output)?;

    // Check difference between input and output
    let mut diff_sum = 0.0;
    let mut max_diff = 0u8;
    for i in 0..frame_curr.data.len() {
        let d = (frame_curr.data[i] as i32 - output.data[i] as i32).abs() as u8;
        diff_sum += d as f64;
        if d > max_diff {
            max_diff = d;
        }
    }
    let mean_diff = diff_sum / frame_curr.data.len() as f64;
    println!("Mean diff between curr and output: {:.3}, Max diff: {}", mean_diff, max_diff);

    Ok(())
}
