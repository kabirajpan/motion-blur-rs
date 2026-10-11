//! Image spatial and temporal gradient calculations.

/// Gradients computed at a specific pixel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageGradients {
    /// Horizontal spatial gradient (dI/dx)
    pub ix: f32,
    /// Vertical spatial gradient (dI/dy)
    pub iy: f32,
    /// Temporal gradient (dI/dt)
    pub it: f32,
}

/// Computes spatial gradients on plane0 and temporal gradient between plane0 and plane1.
#[inline(always)]
pub fn compute_gradient_at(
    plane0: &[f32],
    plane1: &[f32],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
) -> ImageGradients {
    let x_prev = if x > 0 { x - 1 } else { 0 };
    let x_next = (x + 1).min(width - 1);
    let y_prev = if y > 0 { y - 1 } else { 0 };
    let y_next = (y + 1).min(height - 1);

    // Central difference for spatial gradients
    let ix = 0.5 * (plane0[y * width + x_next] - plane0[y * width + x_prev]);
    let iy = 0.5 * (plane0[y_next * width + x] - plane0[y_prev * width + x]);

    // Temporal difference: I(t+1) - I(t)
    let it = plane1[y * width + x] - plane0[y * width + x];

    ImageGradients { ix, iy, it }
}
