// @include ../shared/common.wgsl
// @include ../shared/fullscreen.wgsl

@group(1) @binding(0)
var linear_sampler: sampler;

@group(1) @binding(1)
var base_tex: texture_2d<f32>;

@group(1) @binding(2)
var bloom_tex: texture_2d<f32>;

@fragment
fn fs_main(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let compare_x = clamp(globals.compare_split, 0.05, 0.95);
    let original = textureSample(base_tex, linear_sampler, in.uv).rgb;
    let bloom = textureSample(bloom_tex, linear_sampler, in.uv).rgb;
    var color = original + bloom * 1.2;

    if (globals.compare_enabled != 0u) {
        if (in.uv.x < compare_x) {
            color = original;
        }

        let split = smoothstep(0.0, 0.003, abs(in.uv.x - compare_x));
        color = mix(vec3f(1.0), color, split);
    }

    let grid = step(0.995, fract(in.uv.y * 18.0)) * 0.03;
    color = color + vec3f(grid);

    return vec4f(color, 1.0);
}
