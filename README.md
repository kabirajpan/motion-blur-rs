# 🎬 motion_blur_rs — Professional Video Editor Motion Blur Plugin

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-iOS%20%7C%20Android%20%7C%20macOS%20%7C%20Windows%20%7C%20Linux-blue.svg)]()
[![Backend](https://img.shields.io/badge/backend-CPU%20(Rayon)%20%7C%20GPU%20(WGSL)-brightgreen.svg)]()
[![Resolution](https://img.shields.io/badge/support-1080p%20%7C%204K%20%7C%20HDR-purple.svg)]()

A high-performance, modular, cross-platform motion estimation and temporal shutter blurring plugin crate written in Rust. Designed specifically to be hosted by video editing engines targeting **mobile (iOS / Android)** and **desktop (macOS / Windows / Linux)**.

---

## ⚡ Quick Start: How to Use the CLI

Process any video file directly using high-speed multi-threaded FFmpeg streaming pipes (with audio stream preservation):

### Prerequisites
Make sure `ffmpeg` and `ffprobe` are installed on your system:
- **macOS:** `brew install ffmpeg`
- **Ubuntu/Debian:** `sudo apt install ffmpeg`
- **Windows:** `winget install Gyan.FFmpeg`

### Command Syntax
```bash
cargo run --release --example process_video -- <input_video> <output_video> [blur_amount] [samples] [frame_limit] [curve]
```

### Common Commands
```bash
# 1. Standard Cinematic Motion Blur
cargo run --release --example process_video -- input.mp4 output_blurred.mp4 3.5 24

# 2. 2X Intense Motion Blur (Ultra-prominent trails)
cargo run --release --example process_video -- input.mp4 output_2x_blurred.mp4 7.0 32

# 3. Quick 30-Frame / 1-Second Preview Test
cargo run --release --example process_video -- input.mp4 preview.mp4 3.5 20 30
```

### CLI Options Reference
| Parameter | Type | Default | Recommended | Description |
| :--- | :---: | :---: | :---: | :--- |
| `input_video` | `path` | *Required* | — | Path to source video (`.mp4`, `.mov`, `.mkv`, etc.) |
| `output_video` | `path` | *Required* | — | Destination path for the motion-blurred video |
| `blur_amount` | `f32` | `0.5` | `3.5` (Normal) / `7.0` (2X) | Shutter length multiplier scaling velocity integration |
| `samples` | `i32` | `16` | `24` – `32` | Sub-frame integration steps (higher = zero stepping artifacts) |
| `frame_limit` | `usize` | `None` | `30` (Preview) | Maximum frames to render (omit to process entire video) |
| `curve` | `usize` | `0` | `0` (Box) / `1` (Gaussian) | Shutter shape: `0 = Box` (constant trail), `1 = Gaussian`, `2 = Triangular` |

---

## 📸 Visual Showcase: Before vs After

### 1. Full-Frame Comparison: None vs Applied

Uncropped 1080p frame comparing the raw camera capture against the reconstructed motion-blurred frame.

| Left: Original Frame (None — No Blur) | Right: Motion Blurred (Applied — Blur Amount: 3.5) |
| :---: | :---: |
| ![Original Frame None](./output/frame_none_original.png) | ![Full Motion Blurred Frame](./output/frame_full_motion_blur.png) |
| *Discrete camera capture (staccato motion)* | *Reconstructed temporal shutter blur (natural motion streaks)* |

---

### 2. Normal Motion Blur Comparisons (3 Highlights)

Split-screen frames (*Left of Gold line = Original / None*, *Right of Gold line = Applied Blur Amount 3.5, 24 samples*):

#### Frame 42 (~1.37s)
![Frame 42](./output/compare_02_frame_42.png)

#### Frame 98 (~3.23s)
![Frame 98](./output/compare_04_frame_98.png)

#### Frame 185 (~6.13s)
![Frame 185](./output/compare_07_frame_185.png)

---

### 3. 2X Intense Motion Blur Comparisons (3 Highlights)

Split-screen frames (*Left of Gold line = Original / None*, *Right of Gold line = 2X Blur Amount 7.0, 32 samples*):

#### Frame 85 (~2.80s)
![Frame 85 2X](./output/compare_2x_03_frame_85.png)

#### Frame 140 (~4.63s)
![Frame 140 2X](./output/compare_2x_05_frame_140.png)

#### Frame 200 (~6.63s)
![Frame 200 2X](./output/compare_2x_07_frame_200.png)

---

## 🧠 Architecture Overview

```
[ Video Editor Host Engine ]
            │
            ▼ 1. Enumerate parameters & inspector controls
  [ Plugin Registry ] -> instantiate VideoPlugin
            │
            ▼ 2. Query temporal dependencies at time T: frames (T-1, T, T+1)
  [ Timeline Frame Fetcher ]
            │
            ▼ 3. Submit frames & parameters to plugin.render()
  ┌────────────────────────────────────────────────────────┐
  │  motion_blur_rs Engine                                 │
  │                                                        │
  │  Stage 1: Multi-Scale Image Pyramids                   │
  │           Separable 5-tap Gaussian downsampling        │
  │                                                        │
  │  Stage 2: Differential Optical Flow Solver             │
  │           Coarse-to-fine regularized structure tensor  │
  │                                                        │
  │  Stage 3: Bidirectional Occlusion & Alpha Boost        │
  │           Boundary detection & transparent edge boost  │
  │                                                        │
  │  Stage 4: 4D Temporal Shutter Accumulator              │
  │           Sub-frame integration along vector paths     │
  └────────────────────────────────────────────────────────┘
            │
            ▼ 4. Write blurred pixels into output FrameBuffer
[ Timeline Compositor / Preview Viewport ]
```

---

## 📁 Directory & Module Organization

```
motion_blur_rs/
├── Cargo.toml
├── README.md
├── SHOWCASE.md
├── src/
│   ├── lib.rs                  # Public exports and factory method (create_motion_blur_plugin)
│   ├── motion_blur_plugin.rs   # Concrete MotionBlurPlugin implementing VideoPlugin
│   │
│   ├── plugin/                 # Universal Video Plugin Architecture & Contract
│   │   ├── interface.rs        # VideoPlugin trait, RenderContext, ExecutionBackend
│   │   ├── frame.rs            # FrameBuffer, ColorRgba, PixelFormat (Rgba8, Rgba32F, Luma)
│   │   ├── params.rs           # PluginParameters, ParameterDescriptor, UI controls
│   │   └── temporal.rs         # TimeCode, TemporalFrameRequest (multi-frame negotiation)
│   │
│   ├── math/                   # Core Mathematical Foundations
│   │   ├── vector.rs           # 2D Motion vector operations (Vec2)
│   │   ├── sampling.rs         # Bilinear & bicubic sub-pixel interpolation
│   │   └── gradient.rs         # 2D spatial (Ix, Iy) and temporal (It) gradients
│   │
│   ├── pyramid/                # Multi-Scale Pyramid Hierarchy
│   │   ├── gaussian.rs         # Separable 1D Gaussian 2x downsampling
│   │   └── pyramid.rs          # ImagePyramid generation & level management
│   │
│   ├── optical_flow/           # Dense Motion Vector Estimation
│   │   ├── types.rs            # FlowField and FlowOptions
│   │   ├── solver.rs           # Regularized Lucas-Kanade / Horn-Schunck structure tensor
│   │   └── coarse_to_fine.rs   # Pyramidal optical flow with vector upsampling
│   │
│   ├── occlusion/              # Consistency & Boundary Filtering
│   │   ├── consistency.rs      # Forward-backward consistency check (OcclusionMask)
│   │   └── boundary.rs         # Motion boundary detector & Foreground Alpha Boost
│   │
│   ├── warping/                # Image Warping Pipelines
│   │   ├── inverse.rs          # Inverse warping with edge clamping
│   │   └── forward.rs          # Forward splatting with coverage accumulation
│   │
│   ├── shutter/                # 4D Temporal Shutter Simulation
│   │   ├── curves.rs           # Camera shutter profiles (Box, Gaussian, Triangular)
│   │   └── accumulator.rs      # Multi-threaded sub-frame integration (Rayon)
│   │
│   └── gpu/                    # Modern GPU Compute Pipelines (WGSL)
│       └── shaders/
│           ├── pyramid.wgsl    # Downsampling compute shader
│           ├── flow.wgsl       # Optical flow solver compute shader
│           └── shutter.wgsl    # 4D temporal shutter accumulation compute shader
│
└── examples/
    ├── host_demo.rs            # Host simulator demonstrating discovery and rendering
    └── process_video.rs        # FFmpeg video file batch stream processor
```

---

## 📦 How to Integrate into Your Video Editor Engine

Add `motion_blur_rs` to your video editor's `Cargo.toml`:

```toml
[dependencies]
motion_blur_rs = { path = "../motion_blur_rs" }
```

In your editor's render graph / node pipeline:

```rust
use motion_blur_rs::{
    create_motion_blur_plugin, FrameBuffer, PixelFormat, RenderContext,
    TimeCode, PluginParameters, ParameterValue, MotionBlurPlugin, ExecutionBackend
};
use std::collections::HashMap;

// 1. Instantiate plugin
let plugin = create_motion_blur_plugin();

// 2. Query temporal frame requests at current timeline timestamp
let current_time = TimeCode::new(1.0, 30.0);
let dependencies = plugin.get_frame_dependencies(current_time);

// 3. Assemble required frames from timeline cache
let mut inputs = HashMap::new();
inputs.insert(MotionBlurPlugin::LABEL_PREVIOUS, &frame_prev);
inputs.insert(MotionBlurPlugin::LABEL_CURRENT, &frame_curr);
inputs.insert(MotionBlurPlugin::LABEL_NEXT, &frame_next);

// 4. Configure parameters
let mut params = PluginParameters::new();
params.set(MotionBlurPlugin::PARAM_BLUR_AMOUNT, ParameterValue::Float(3.5));
params.set(MotionBlurPlugin::PARAM_SAMPLES, ParameterValue::Int(24));

// 5. Render to output buffer
let context = RenderContext {
    current_time,
    width: 1920,
    height: 1080,
    backend: ExecutionBackend::CpuMultithreaded,
};

let mut output = FrameBuffer::new(1920, 1080, PixelFormat::Rgba8);
plugin.render(&context, &params, &inputs, &mut output)?;
```

---

## 📄 License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
