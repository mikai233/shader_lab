use crate::renderer::{FullscreenPipeline, Renderer, ShaderCompiler};
use crate::scene::{FrameContext, LabScene};

pub struct VoronoiScene {
    shader_compiler: ShaderCompiler,
    pipeline: FullscreenPipeline,
}

impl VoronoiScene {
    pub fn new(renderer: &Renderer) -> anyhow::Result<Self> {
        let shader_compiler = ShaderCompiler::new(renderer.shader_root().to_path_buf());
        let pipeline = FullscreenPipeline::new(
            renderer.device(),
            &shader_compiler,
            "voronoi_pipeline",
            "voronoi/voronoi.wgsl",
            &[renderer.globals_layout()],
            renderer.surface_format(),
        )?;

        Ok(Self {
            shader_compiler,
            pipeline,
        })
    }
}

impl LabScene for VoronoiScene {
    fn resize(&mut self, _renderer: &Renderer) {}

    fn reload(&mut self, renderer: &Renderer) -> anyhow::Result<()> {
        self.pipeline = FullscreenPipeline::new(
            renderer.device(),
            &self.shader_compiler,
            "voronoi_pipeline",
            "voronoi/voronoi.wgsl",
            &[renderer.globals_layout()],
            renderer.surface_format(),
        )?;
        Ok(())
    }

    fn render(&mut self, frame: FrameContext<'_>) -> anyhow::Result<()> {
        let mut pass = frame
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("voronoi_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame.surface_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        pass.set_pipeline(&self.pipeline.pipeline);
        pass.set_bind_group(0, frame.globals_bind_group, &[]);
        pass.draw(0..3, 0..1);
        Ok(())
    }
}
