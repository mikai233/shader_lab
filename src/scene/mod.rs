mod noise_blur;

use crate::renderer::Renderer;

pub use noise_blur::NoiseBlurScene;

pub struct FrameContext<'a> {
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub surface_view: &'a wgpu::TextureView,
    pub globals_bind_group: &'a wgpu::BindGroup,
}

pub trait LabScene {
    fn resize(&mut self, renderer: &Renderer);
    fn reload(&mut self, renderer: &Renderer) -> anyhow::Result<()>;
    fn render(&mut self, frame: FrameContext<'_>) -> anyhow::Result<()>;
}

pub fn create_default_scene(renderer: &Renderer) -> anyhow::Result<Box<dyn LabScene>> {
    Ok(Box::new(NoiseBlurScene::new(renderer)?))
}
