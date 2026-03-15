mod noise_blur;
mod sobel;
mod voronoi;

use crate::renderer::Renderer;

pub use noise_blur::NoiseBlurScene;
pub use sobel::SobelScene;
pub use voronoi::VoronoiScene;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneType {
    NoiseBlur,
    Sobel,
    Voronoi,
}

impl SceneType {
    pub fn all() -> &'static [Self] {
        &[Self::NoiseBlur, Self::Sobel, Self::Voronoi]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::NoiseBlur => "Noise Blur",
            Self::Sobel => "Sobel Edge Detection",
            Self::Voronoi => "Voronoi Noise",
        }
    }

    pub fn create(&self, renderer: &Renderer) -> anyhow::Result<Box<dyn LabScene>> {
        match self {
            Self::NoiseBlur => Ok(Box::new(NoiseBlurScene::new(renderer)?)),
            Self::Sobel => Ok(Box::new(SobelScene::new(renderer)?)),
            Self::Voronoi => Ok(Box::new(VoronoiScene::new(renderer)?)),
        }
    }
}

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
