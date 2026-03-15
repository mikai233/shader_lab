// @include ../shared/common.wgsl
// @include ../shared/fullscreen.wgsl

fn hash22(p: vec2f) -> vec2f {
    var p3 = fract(p.xyx * vec3f(0.1031, 0.1030, 0.0973));
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

@fragment
fn fs_main(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let uv = in.uv;
    let res = safe_resolution();
    let aspect = res.x / res.y;
    
    var p = (uv * 2.0 - 1.0) * vec2f(aspect, 1.0);
    p = p * 4.0; // Scale
    
    let i_p = floor(p);
    let f_p = fract(p);
    
    var min_dist = 8.0;
    var closest_cell = vec2f(0.0);
    
    let t = globals.time * 0.5;
    
    for (var y = -1; y <= 1; y = y + 1) {
        for (var x = -1; x <= 1; x = x + 1) {
            let neighbor = vec2f(f32(x), f32(y));
            var point = hash22(i_p + neighbor);
            
            // Animate points
            point = 0.5 + 0.5 * sin(t + 6.2831 * point);
            
            let diff = neighbor + point - f_p;
            let dist = length(diff);
            
            if (dist < min_dist) {
                min_dist = dist;
                closest_cell = i_p + neighbor;
            }
        }
    }
    
    let cell_hash = hash22(closest_cell);
    let color_cells = vec3f(cell_hash.x, cell_hash.y, 0.5 + 0.5 * sin(t));
    let color_dist = vec3f(1.0 - min_dist);
    
    var final_color: vec3f;
    if (globals.compare_enabled != 0u) {
        if (uv.x < globals.compare_split) {
            final_color = color_cells;
        } else {
            final_color = color_dist;
        }
        
        if (abs(uv.x - globals.compare_split) < 0.002) {
            final_color = vec3f(1.0);
        }
    } else {
        final_color = color_cells * (0.8 + 0.2 * color_dist);
    }
    
    return vec4f(final_color, 1.0);
}
