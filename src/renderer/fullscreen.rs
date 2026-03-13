use anyhow::{Context, bail};

use crate::renderer::ShaderCompiler;

pub struct FullscreenPipeline {
    pub pipeline: wgpu::RenderPipeline,
}

impl FullscreenPipeline {
    pub fn new(
        device: &wgpu::Device,
        compiler: &ShaderCompiler,
        label: &str,
        shader_path: &str,
        layouts: &[&wgpu::BindGroupLayout],
        target_format: wgpu::TextureFormat,
    ) -> anyhow::Result<Self> {
        let shader = compiler
            .compile(device, shader_path)
            .with_context(|| format!("failed to compile shader {shader_path}"))?;

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{label}_layout")),
            bind_group_layouts: layouts,
            immediate_size: 0,
        });

        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        if let Some(err) = pollster::block_on(error_scope.pop()) {
            bail!("pipeline validation failed for {label}: {err}");
        }

        Ok(Self { pipeline })
    }
}
