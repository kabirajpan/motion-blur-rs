// 4D Temporal Shutter Accumulator Compute Shader (WGSL)

struct ShutterUniforms {
    shutter_amount: f32,
    num_samples: i32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var flow_tex: texture_2d<f32>;
@group(0) @binding(2) var out_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var<uniform> config: ShutterUniforms;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = vec2<i32>(global_id.xy);
    let dims = textureDimensions(src_tex);

    if (pos.x >= i32(dims.x) || pos.y >= i32(dims.y)) {
        return;
    }

    let velocity = textureLoad(flow_tex, pos, 0).xy;
    let speed_sq = dot(velocity, velocity);

    // If no motion or zero shutter, copy pixel directly
    if (speed_sq < 1e-4 || config.shutter_amount < 1e-4) {
        let base_col = textureLoad(src_tex, pos, 0);
        textureStore(out_tex, pos, base_col);
        return;
    }

    var accum = vec4<f32>(0.0);
    var total_weight = 0.0;
    let n = max(config.num_samples, 2);

    for (var i: i32 = 0; i < n; i = i + 1) {
        let t = (f32(i) / f32(n - 1)) - 0.5; // range [-0.5, 0.5]
        
        // Gaussian shutter profile: exp(-0.5 * (t / sigma)^2)
        let sigma = 0.22;
        let w = exp(-0.5 * (t / sigma) * (t / sigma));

        let offset = velocity * (t * config.shutter_amount);
        let sample_pos = vec2<f32>(f32(pos.x) + offset.x, f32(pos.y) + offset.y);

        let clamped_pos = vec2<i32>(
            clamp(i32(round(sample_pos.x)), 0, i32(dims.x) - 1),
            clamp(i32(round(sample_pos.y)), 0, i32(dims.y) - 1)
        );

        let col = textureLoad(src_tex, clamped_pos, 0);
        accum = accum + col * w;
        total_weight = total_weight + w;
    }

    let final_color = accum / total_weight;
    textureStore(out_tex, pos, final_color);
}
