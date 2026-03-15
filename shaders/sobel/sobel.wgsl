// @include ../shared/common.wgsl
// @include ../shared/fullscreen.wgsl

@group(1) @binding(0)
var linear_sampler: sampler;
@group(1) @binding(1)
var source_texture: texture_2d<f32>;

@fragment
fn fs_main(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let uv = in.uv;
    let res = safe_resolution();
    let texel_size = 1.0 / res;

    // Sobel kernels
    // Gx = [-1  0  1]
    //      [-2  0  2]
    //      [-1  0  1]
    //
    // Gy = [-1 -2 -1]
    //      [ 0  0  0]
    //      [ 1  2  1]

    var intensity: array<f32, 9>;
    for (var i = 0; i < 3; i = i + 1) {
        for (var j = 0; j < 3; j = j + 1) {
            let offset = vec2(f32(i - 1), f32(j - 1)) * texel_size;
            let sample = textureSample(source_texture, linear_sampler, uv + offset).rgb;
            // Convert to grayscale for edge detection
            intensity[i * 3 + j] = dot(sample, vec3(0.299, 0.587, 0.114));
        }
    }

    let gx = -1.0 * intensity[0] + 1.0 * intensity[2]
           - 2.0 * intensity[3] + 2.0 * intensity[5]
           - 1.0 * intensity[6] + 1.0 * intensity[8];

    let gy = -1.0 * intensity[0] - 2.0 * intensity[1] - 1.0 * intensity[2]
           + 1.0 * intensity[6] + 2.0 * intensity[7] + 1.0 * intensity[8];

    let g = sqrt(gx * gx + gy * gy);
    
    // Smooth the edges slightly
    let edge = smoothstep(0.1, 0.4, g);
    
    let source_color = textureSample(source_texture, linear_sampler, uv).rgb;
    let edge_color = vec3(1.0, 1.0, 0.0); // Yellow edges for visibility
    
    var final_color: vec3f;
    if (globals.compare_enabled != 0u) {
        if (uv.x < globals.compare_split) {
            final_color = source_color;
        } else {
            final_color = mix(vec3(0.0), edge_color, edge);
        }
        
        // Split line
        if (abs(uv.x - globals.compare_split) < 0.002) {
            final_color = vec3(1.0);
        }
    } else {
        // Overlay edges on source
        final_color = mix(source_color, edge_color, edge);
    }

    return vec4(final_color, 1.0);
}
