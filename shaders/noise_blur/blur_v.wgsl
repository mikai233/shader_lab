// @include ../shared/common.wgsl
// @include ../shared/fullscreen.wgsl

@group(1) @binding(0)
var linear_sampler: sampler;

@group(1) @binding(1)
var source_tex: texture_2d<f32>;

@fragment
fn fs_main(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let texel = vec2f(0.0, 1.0 / safe_resolution().y);
    let uv = in.uv;

    var color = textureSample(source_tex, linear_sampler, uv) * 0.29411765;
    color = color + textureSample(source_tex, linear_sampler, uv - texel * 1.3333334) * 0.35294118;
    color = color + textureSample(source_tex, linear_sampler, uv + texel * 1.3333334) * 0.35294118;
    return color;
}
