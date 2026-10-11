//! Optical flow data structures and options.

use crate::math::vector::Vec2;

/// Dense 2D motion vector field representing per-pixel displacements.
#[derive(Debug, Clone)]
pub struct FlowField {
    pub width: usize,
    pub height: usize,
    /// Velocity vector for each pixel in row-major order: index = y * width + x
    pub vectors: Vec<Vec2>,
}

impl FlowField {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            vectors: vec![Vec2::ZERO; width * height],
        }
    }

    #[inline(always)]
    pub fn get(&self, x: usize, y: usize) -> Vec2 {
        if x < self.width && y < self.height {
            self.vectors[y * self.width + x]
        } else {
            Vec2::ZERO
        }
    }

    #[inline(always)]
    pub fn set(&mut self, x: usize, y: usize, vec: Vec2) {
        if x < self.width && y < self.height {
            self.vectors[y * self.width + x] = vec;
        }
    }

    /// Rescales and upsamples flow field to target dimensions (bilinear upsampling).
    pub fn upsample_to(&self, target_w: usize, target_h: usize) -> Self {
        let mut upsampled = Self::new(target_w, target_h);
        let scale_x = target_w as f32 / self.width as f32;
        let scale_y = target_h as f32 / self.height as f32;

        for ty in 0..target_h {
            let src_y = ty as f32 / scale_y;
            let sy0 = (src_y.floor() as usize).min(self.height - 1);
            let sy1 = (sy0 + 1).min(self.height - 1);
            let fy = src_y - sy0 as f32;

            for tx in 0..target_w {
                let src_x = tx as f32 / scale_x;
                let sx0 = (src_x.floor() as usize).min(self.width - 1);
                let sx1 = (sx0 + 1).min(self.width - 1);
                let fx = src_x - sx0 as f32;

                let v00 = self.get(sx0, sy0);
                let v10 = self.get(sx1, sy0);
                let v01 = self.get(sx0, sy1);
                let v11 = self.get(sx1, sy1);

                let top = v00 * (1.0 - fx) + v10 * fx;
                let bot = v01 * (1.0 - fx) + v11 * fx;
                let interpolated = top * (1.0 - fy) + bot * fy;

                // Scale the velocity vectors to match target pixel space
                upsampled.set(
                    tx,
                    ty,
                    Vec2::new(interpolated.x * scale_x, interpolated.y * scale_y),
                );
            }
        }

        upsampled
    }
}

/// Algorithmic parameters for optical flow estimation.
#[derive(Debug, Clone, Copy)]
pub struct FlowOptions {
    /// Number of pyramid levels to compute (typically 3 to 5)
    pub pyramid_levels: usize,
    /// Window radius for spatial gradient integration (e.g., radius 2 = 5x5 window)
    pub window_radius: usize,
    /// Motion sensitivity threshold (regularization lambda against noise)
    pub sensitivity: f32,
    /// Iteration count at each pyramid stage
    pub iterations: usize,
}

impl Default for FlowOptions {
    fn default() -> Self {
        Self {
            pyramid_levels: 4,
            window_radius: 2,
            sensitivity: 0.05,
            iterations: 3,
        }
    }
}
