// @include ../shared/common.wgsl
// @include ../shared/fullscreen.wgsl

@fragment
fn fs_main(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let uv = in.uv;
    
    // Create some interesting shapes to detect edges on
    let t = globals.time;
    
    // Moving circles
    let c1 = length(uv - vec2f(0.5 + 0.2 * cos(t), 0.5 + 0.2 * sin(t))) - 0.1;
    let c2 = length(uv - vec2f(0.5 + 0.3 * cos(t * 0.7), 0.5 + 0.3 * sin(t * 1.3))) - 0.08;
    
    // A square
    let q_pos = abs(uv - 0.5) - 0.2;
    let square = max(q_pos.x, q_pos.y);
    
    // Combine shapes
    let dist = min(min(c1, c2), square);
    
    // Smooth stepping for anti-aliased base shapes
    let base = smoothstep(0.01, 0.0, dist);
    
    // Add some color variation
    let color = mix(
        vec3f(0.1, 0.2, 0.4),
        vec3f(0.8, 0.9, 1.0),
        base
    );
    
    // Add a moving gradient
    let grad = 0.5 + 0.5 * sin(uv.x * 10.0 + t);
    
    return vec4f(color * (0.8 + 0.2 * grad), 1.0);
}
