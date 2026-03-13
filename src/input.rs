use winit::dpi::PhysicalSize;

#[derive(Debug, Clone, Copy)]
pub struct InputState {
    pub mouse_position: [f32; 2],
    pub resolution: [f32; 2],
    pub drag_compare: bool,
}

impl InputState {
    pub fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            mouse_position: [size.width as f32 * 0.5, size.height as f32 * 0.5],
            resolution: [size.width as f32, size.height as f32],
            drag_compare: false,
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
        }
    }
}
