struct Globals {
    resolution: vec2f,
    mouse: vec2f,
    time: f32,
    delta_time: f32,
    frame_index: u32,
    compare_enabled: u32,
    compare_split: f32,
    _padding: f32,
};

@group(0) @binding(0)
var<uniform> globals: Globals;

fn safe_resolution() -> vec2f {
    return max(globals.resolution, vec2f(1.0, 1.0));
}
