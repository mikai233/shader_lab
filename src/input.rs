use winit::dpi::PhysicalSize;

#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub active: bool,
    pub query: String,
}

#[derive(Debug, Clone)]
pub struct InputState {
    pub mouse_position: [f32; 2],
    pub resolution: [f32; 2],
    pub drag_compare: bool,
    pub search: SearchState,
}

impl InputState {
    pub fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            mouse_position: [size.width as f32 * 0.5, size.height as f32 * 0.5],
            resolution: [size.width as f32, size.height as f32],
            drag_compare: false,
            search: SearchState::default(),
        }
    }

    pub fn set_resolution(&mut self, size: PhysicalSize<u32>) {
        self.resolution = [size.width as f32, size.height as f32];
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse_position: [0.0, 0.0],
            resolution: [1.0, 1.0],
            drag_compare: false,
            search: SearchState::default(),
        }
    }
}
