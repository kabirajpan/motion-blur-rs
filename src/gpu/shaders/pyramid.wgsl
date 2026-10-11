// Separable 5-tap Binomial Gaussian Downsampling Compute Shader (WGSL)

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var output_tex: texture_storage_2d<r32float, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let out_pos = vec2<i32>(global_id.xy);
    let dims = textureDimensions(input_tex);

    let in_x = out_pos.x * 2;
    let in_y = out_pos.y * 2;

    if (in_x >= i32(dims.x) || in_y >= i32(dims.y)) {
        return;
    }

    // 5x5 separable kernel approximation: [1, 4, 6, 4, 1] / 16
    let weights = array<f32, 5>(1.0 / 16.0, 4.0 / 16.0, 6.0 / 16.0, 4.0 / 16.0, 1.0 / 16.0);
    var accum = 0.0;

    for (var dy: i32 = -2; dy <= 2; dy = dy + 1) {
        let sample_y = clamp(in_y + dy, 0, i32(dims.y) - 1);
        let wy = weights[dy + 2];

        for (var dx: i32 = -2; dx <= 2; dx = dx + 1) {
            let sample_x = clamp(in_x + dx, 0, i32(dims.x) - 1);
            let wx = weights[dx + 2];

            let pixel = textureLoad(input_tex, vec2<i32>(sample_x, sample_y), 0).r;
            accum = accum + pixel * (wx * wy);
        }
    }

    textureStore(output_tex, out_pos, vec4<f32>(accum, 0.0, 0.0, 1.0));
}
