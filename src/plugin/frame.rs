//! Frame and pixel buffer representations for video processing pipelines.

/// Supported pixel formats in video editor plugin buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// 8-bit per channel RGBA (32 bits total per pixel)
    Rgba8,
    /// 32-bit floating point per channel RGBA (128 bits total per pixel, HDR / Linear Color)
    Rgba32F,
    /// 32-bit floating point single-channel Luminance / Matte buffer
    Luminance32F,
}

/// Generic RGBA float color representation [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorRgba {
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

    #[inline(always)]
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Rec. 709 perceived luminance
    #[inline(always)]
    pub fn luminance(&self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    #[inline(always)]
    pub fn clamp(&self) -> Self {
        Self {
            r: self.r.clamp(0.0, 1.0),
            g: self.g.clamp(0.0, 1.0),
            b: self.b.clamp(0.0, 1.0),
            a: self.a.clamp(0.0, 1.0),
        }
    }
}

/// In-memory CPU video frame buffer.
#[derive(Debug, Clone)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub format: PixelFormat,
    /// Raw byte storage for interleaved pixels
    pub data: Vec<u8>,
}

impl FrameBuffer {
    /// Allocates an empty frame buffer initialized with zeros.
    pub fn new(width: usize, height: usize, format: PixelFormat) -> Self {
        let bpp = match format {
            PixelFormat::Rgba8 => 4,
            PixelFormat::Rgba32F => 16,
            PixelFormat::Luminance32F => 4,
        };
        let size = width * height * bpp;
        Self {
            width,
            height,
            format,
            data: vec![0u8; size],
        }
    }

    /// Retrieves an RGBA pixel normalized to float values.
    #[inline(always)]
    pub fn get_pixel_rgba(&self, x: usize, y: usize) -> ColorRgba {
        if x >= self.width || y >= self.height {
            return ColorRgba::TRANSPARENT;
        }

        let idx = y * self.width + x;
        match self.format {
            PixelFormat::Rgba8 => {
                let offset = idx * 4;
                ColorRgba {
                    r: self.data[offset] as f32 / 255.0,
                    g: self.data[offset + 1] as f32 / 255.0,
                    b: self.data[offset + 2] as f32 / 255.0,
                    a: self.data[offset + 3] as f32 / 255.0,
                }
            }
            PixelFormat::Rgba32F => {
                let offset = idx * 16;
                let r = f32::from_ne_bytes(self.data[offset..offset + 4].try_into().unwrap());
                let g = f32::from_ne_bytes(self.data[offset + 4..offset + 8].try_into().unwrap());
                let b = f32::from_ne_bytes(self.data[offset + 8..offset + 12].try_into().unwrap());
                let a = f32::from_ne_bytes(self.data[offset + 12..offset + 16].try_into().unwrap());
                ColorRgba { r, g, b, a }
            }
            PixelFormat::Luminance32F => {
                let offset = idx * 4;
                let l = f32::from_ne_bytes(self.data[offset..offset + 4].try_into().unwrap());
                ColorRgba { r: l, g: l, b: l, a: 1.0 }
            }
        }
    }

    /// Sets an RGBA pixel from float values.
    #[inline(always)]
    pub fn set_pixel_rgba(&mut self, x: usize, y: usize, color: ColorRgba) {
        if x >= self.width || y >= self.height {
            return;
        }

        let idx = y * self.width + x;
        match self.format {
            PixelFormat::Rgba8 => {
                let offset = idx * 4;
                self.data[offset] = (color.r.clamp(0.0, 1.0) * 255.0).round() as u8;
                self.data[offset + 1] = (color.g.clamp(0.0, 1.0) * 255.0).round() as u8;
                self.data[offset + 2] = (color.b.clamp(0.0, 1.0) * 255.0).round() as u8;
                self.data[offset + 3] = (color.a.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            PixelFormat::Rgba32F => {
                let offset = idx * 16;
                self.data[offset..offset + 4].copy_from_slice(&color.r.to_ne_bytes());
                self.data[offset + 4..offset + 8].copy_from_slice(&color.g.to_ne_bytes());
                self.data[offset + 8..offset + 12].copy_from_slice(&color.b.to_ne_bytes());
                self.data[offset + 12..offset + 16].copy_from_slice(&color.a.to_ne_bytes());
            }
            PixelFormat::Luminance32F => {
                let offset = idx * 4;
                let l = color.luminance();
                self.data[offset..offset + 4].copy_from_slice(&l.to_ne_bytes());
            }
        }
    }

    /// Extracts a single-channel grayscale luminance buffer (normalized f32).
    pub fn to_luminance_plane(&self) -> Vec<f32> {
        let count = self.width * self.height;
        let mut plane = Vec::with_capacity(count);

        match self.format {
            PixelFormat::Rgba8 => {
                for i in 0..count {
                    let off = i * 4;
                    let r = self.data[off] as f32 / 255.0;
                    let g = self.data[off + 1] as f32 / 255.0;
                    let b = self.data[off + 2] as f32 / 255.0;
                    plane.push(0.2126 * r + 0.7152 * g + 0.0722 * b);
                }
            }
            PixelFormat::Rgba32F => {
                for i in 0..count {
                    let off = i * 16;
                    let r = f32::from_ne_bytes(self.data[off..off + 4].try_into().unwrap());
                    let g = f32::from_ne_bytes(self.data[off + 4..off + 8].try_into().unwrap());
                    let b = f32::from_ne_bytes(self.data[off + 8..off + 12].try_into().unwrap());
                    plane.push(0.2126 * r + 0.7152 * g + 0.0722 * b);
                }
            }
            PixelFormat::Luminance32F => {
                for i in 0..count {
                    let off = i * 4;
                    let l = f32::from_ne_bytes(self.data[off..off + 4].try_into().unwrap());
                    plane.push(l);
                }
            }
        }

        plane
    }
}
