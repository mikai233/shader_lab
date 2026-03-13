// @include ../shared/common.wgsl
// @include ../shared/fullscreen.wgsl

fn hash21(p: vec2f) -> f32 {
    return fract(sin(dot(p, vec2f(127.1, 311.7))) * 43758.5453123);
}

fn value_noise(p: vec2f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let a = hash21(i);
    let b = hash21(i + vec2f(1.0, 0.0));
    let c = hash21(i + vec2f(0.0, 1.0));
    let d = hash21(i + vec2f(1.0, 1.0));
    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p0: vec2f) -> f32 {
    var p = p0;
    var sum = 0.0;
    var amp = 0.55;
    for (var i = 0; i < 5; i = i + 1) {
        sum = sum + value_noise(p) * amp;
        p = p * 2.03 + vec2f(19.7, 13.4);
        amp = amp * 0.5;
    }
    return sum;
}

@fragment
fn fs_main(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let resolution = safe_resolution();
    let uv = in.uv;
    let aspect = resolution.x / resolution.y;
    let mouse = globals.mouse / resolution;

    var p = (uv * 2.0 - 1.0) * vec2f(aspect, 1.0);
    p = p * 2.7 + vec2f(globals.time * 0.12, globals.time * -0.04);
    p = p + (mouse - 0.5) * vec2f(2.5, 1.6);

    let n = fbm(p);
    let glow = smoothstep(0.55, 0.92, n);
    let base = mix(vec3f(0.02, 0.04, 0.08), vec3f(0.15, 0.46, 0.96), n);
    let accent = vec3f(1.0, 0.83, 0.42) * glow;
    let vignette = smoothstep(1.35, 0.2, length(uv * 2.0 - 1.0));

    return vec4f((base + accent * 0.7) * vignette, 1.0);
}
