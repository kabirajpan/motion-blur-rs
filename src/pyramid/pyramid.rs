//! Multi-level Image Pyramid for coarse-to-fine processing.

use super::gaussian::downsample_2x;

/// A single resolution level of an image pyramid.
#[derive(Debug, Clone)]
pub struct PyramidLevel {
    pub level_index: usize,
    pub width: usize,
    pub height: usize,
    pub plane: Vec<f32>,
}

/// Gaussian multi-scale image pyramid.
#[derive(Debug, Clone)]
pub struct ImagePyramid {
    pub levels: Vec<PyramidLevel>,
}

impl ImagePyramid {
    /// Builds a multi-resolution pyramid from base full-resolution luminance plane.
    pub fn build(base_plane: &[f32], width: usize, height: usize, max_levels: usize) -> Self {
        let mut levels = Vec::with_capacity(max_levels);

        // Level 0 is original resolution
        levels.push(PyramidLevel {
            level_index: 0,
            width,
            height,
            plane: base_plane.to_vec(),
        });

        let mut curr_plane = base_plane.to_vec();
        let mut curr_w = width;
        let mut curr_h = height;

        for lvl in 1..max_levels {
            // Stop if resolution is too small for meaningful convolution
            if curr_w < 16 || curr_h < 16 {
                break;
            }

            let (next_plane, next_w, next_h) = downsample_2x(&curr_plane, curr_w, curr_h);
            levels.push(PyramidLevel {
                level_index: lvl,
                width: next_w,
                height: next_h,
                plane: next_plane.clone(),
            });

            curr_plane = next_plane;
            curr_w = next_w;
            curr_h = next_h;
        }

        Self { levels }
    }

    pub fn num_levels(&self) -> usize {
        self.levels.len()
    }

    pub fn get_level(&self, index: usize) -> Option<&PyramidLevel> {
        self.levels.get(index)
    }
}
