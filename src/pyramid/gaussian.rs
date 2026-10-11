//! Separable Gaussian filter downsampling (5-tap binomial kernel [1, 4, 6, 4, 1] / 16).

/// Applies a 1D horizontal Gaussian pass with 2x downsampling.
pub fn downsample_horizontal(input: &[f32], width: usize, height: usize) -> (Vec<f32>, usize) {
    let out_width = (width + 1) / 2;
    let mut out = vec![0.0f32; out_width * height];

    for y in 0..height {
        let row_in = &input[y * width..(y + 1) * width];
        let row_out = &mut out[y * out_width..(y + 1) * out_width];

        for ox in 0..out_width {
            let ix = ox * 2;
            let p_m2 = if ix >= 2 { row_in[ix - 2] } else { row_in[0] };
            let p_m1 = if ix >= 1 { row_in[ix - 1] } else { row_in[0] };
            let p_0 = row_in[ix];
            let p_p1 = if ix + 1 < width { row_in[ix + 1] } else { row_in[width - 1] };
            let p_p2 = if ix + 2 < width { row_in[ix + 2] } else { row_in[width - 1] };

            row_out[ox] = (p_m2 + 4.0 * p_m1 + 6.0 * p_0 + 4.0 * p_p1 + p_p2) / 16.0;
        }
    }

    (out, out_width)
}

/// Applies a 1D vertical Gaussian pass with 2x downsampling.
pub fn downsample_vertical(input: &[f32], width: usize, height: usize) -> (Vec<f32>, usize) {
    let out_height = (height + 1) / 2;
    let mut out = vec![0.0f32; width * out_height];

    for oy in 0..out_height {
        let iy = oy * 2;
        let ym2 = if iy >= 2 { iy - 2 } else { 0 };
        let ym1 = if iy >= 1 { iy - 1 } else { 0 };
        let y0 = iy;
        let yp1 = (iy + 1).min(height - 1);
        let yp2 = (iy + 2).min(height - 1);

        for x in 0..width {
            let p_m2 = input[ym2 * width + x];
            let p_m1 = input[ym1 * width + x];
            let p_0 = input[y0 * width + x];
            let p_p1 = input[yp1 * width + x];
            let p_p2 = input[yp2 * width + x];

            out[oy * width + x] = (p_m2 + 4.0 * p_m1 + 6.0 * p_0 + 4.0 * p_p1 + p_p2) / 16.0;
        }
    }

    (out, out_height)
}

/// Full 2D Gaussian downsample by a factor of 2.
pub fn downsample_2x(input: &[f32], width: usize, height: usize) -> (Vec<f32>, usize, usize) {
    let (h_pass, out_w) = downsample_horizontal(input, width, height);
    let (v_pass, out_h) = downsample_vertical(&h_pass, out_w, height);
    (v_pass, out_w, out_h)
}
