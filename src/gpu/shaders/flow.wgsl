// Regularized Structure Tensor Optical Flow Solver (WGSL)

struct Uniforms {
    lambda: f32,
    window_radius: i32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var tgt_tex: texture_2d<f32>;
@group(0) @binding(2) var out_flow: texture_storage_2d<rg32float, write>;
@group(0) @binding(3) var<uniform> config: Uniforms;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = vec2<i32>(global_id.xy);
    let dims = textureDimensions(src_tex);

    if (pos.x >= i32(dims.x) || pos.y >= i32(dims.y)) {
        return;
    }

    var sum_ix2 = 0.0;
    var sum_iy2 = 0.0;
    var sum_ix_iy = 0.0;
    var sum_ix_it = 0.0;
    var sum_iy_it = 0.0;

    let r = config.window_radius;

    for (var dy: i32 = -r; dy <= r; dy = dy + 1) {
        let wy = clamp(pos.y + dy, 0, i32(dims.y) - 1);
        for (var dx: i32 = -r; dx <= r; dx = dx + 1) {
            let wx = clamp(pos.x + dx, 0, i32(dims.x) - 1);

            let p_xp = textureLoad(src_tex, vec2<i32>(min(wx + 1, i32(dims.x) - 1), wy), 0).r;
            let p_xm = textureLoad(src_tex, vec2<i32>(max(wx - 1, 0), wy), 0).r;
            let p_yp = textureLoad(src_tex, vec2<i32>(wx, min(wy + 1, i32(dims.y) - 1)), 0).r;
            let p_ym = textureLoad(src_tex, vec2<i32>(wx, max(wy - 1, 0)), 0).r;

            let p_src = textureLoad(src_tex, vec2<i32>(wx, wy), 0).r;
            let p_tgt = textureLoad(tgt_tex, vec2<i32>(wx, wy), 0).r;

            let ix = 0.5 * (p_xp - p_xm);
            let iy = 0.5 * (p_yp - p_ym);
            let it = p_tgt - p_src;

            sum_ix2 = sum_ix2 + ix * ix;
            sum_iy2 = sum_iy2 + iy * iy;
            sum_ix_iy = sum_ix_iy + ix * iy;
            sum_ix_it = sum_ix_it + ix * it;
            sum_iy_it = sum_iy_it + iy * it;
        }
    }

    let g00 = sum_ix2 + config.lambda;
    let g01 = sum_ix_iy;
    let g10 = sum_ix_iy;
    let g11 = sum_iy2 + config.lambda;

    let det = g00 * g11 - g01 * g10;
    var velocity = vec2<f32>(0.0, 0.0);

    if (abs(det) > 1e-6) {
        let inv_det = 1.0 / det;
        let u = inv_det * (g11 * (-sum_ix_it) - g01 * (-sum_iy_it));
        let v = inv_det * (-g10 * (-sum_ix_it) + g00 * (-sum_iy_it));
        velocity = vec2<f32>(u, v);
    }

    textureStore(out_flow, pos, vec4<f32>(velocity.x, velocity.y, 0.0, 1.0));
}
