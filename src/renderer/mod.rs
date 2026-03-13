mod context;
mod fullscreen;
mod pipelines;
mod targets;
mod uniforms;

pub use context::RendererContext;
pub use fullscreen::FullscreenPipeline;
pub use pipelines::ShaderCompiler;
pub use targets::RenderTargets;
pub use uniforms::{Globals, GlobalsData};

use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::scene::{FrameContext, LabScene};

pub struct Renderer {
    context: RendererContext,
    globals: Globals,
}

pub enum RenderError {
    Surface(wgpu::SurfaceError),
    Other(anyhow::Error),
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let context = RendererContext::new(window).await?;
        let globals = Globals::new(&context.device);
        Ok(Self { context, globals })
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.context.size
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.context.surface_format
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.context.device
    }

    pub fn globals_layout(&self) -> &wgpu::BindGroupLayout {
        &self.globals.layout
    }

    pub fn shader_root(&self) -> &std::path::Path {
        &self.context.shader_root
    }

    pub fn update_globals(&self, data: GlobalsData) {
        self.globals.update(&self.context.queue, data);
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.context.resize(size);
    }

    pub fn render(&mut self, scene: &mut dyn LabScene) -> Result<(), RenderError> {
        let surface_texture = self
            .context
            .surface
            .get_current_texture()
            .map_err(RenderError::Surface)?;

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("frame_encoder"),
                });

        scene
            .render(FrameContext {
                encoder: &mut encoder,
                surface_view: &surface_view,
                globals_bind_group: &self.globals.bind_group,
            })
            .map_err(RenderError::Other)?;

        self.context.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
        Ok(())
    }
}
