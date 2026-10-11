//! Physical camera shutter curves and exposure weighting functions.

/// Camera shutter exposure profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutterCurve {
    /// Rectangular / Box shutter (constant exposure over the open interval)
    Box,
    /// Gaussian shutter (smooth bell curve falloff, eliminates stepping artifacts)
    Gaussian,
    /// Triangular shutter
    Triangular,
    /// Mechanical rotary disc trapezoidal shutter
    Trapezoidal,
}

impl ShutterCurve {
    /// Computes exposure weight for normalized shutter time `t` in `[-0.5, 0.5]`.
    #[inline(always)]
    pub fn weight(&self, t: f32) -> f32 {
        let abs_t = t.abs().min(0.5);
        match self {
            Self::Box => 1.0,
            Self::Triangular => 1.0 - 2.0 * abs_t,
            Self::Gaussian => {
                let sigma = 0.22;
                (-0.5 * (t / sigma).powi(2)).exp()
            }
            Self::Trapezoidal => {
                if abs_t < 0.3 {
                    1.0
                } else {
                    (0.5 - abs_t) / 0.2
                }
            }
        }
    }
}
