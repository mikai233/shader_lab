use anyhow::Context;

use crate::renderer::{FullscreenPipeline, RenderTargets, Renderer, ShaderCompiler};
use crate::scene::{FrameContext, LabScene};

const TARGET_NAMES: &[&str] = &["noise_target", "blur_h_target", "blur_v_target"];

pub struct NoiseBlurScene {
    shader_compiler: ShaderCompiler,
    targets: RenderTargets,
    pipelines: Pipelines,
    linear_sampler: wgpu::Sampler,
    single_texture_layout: wgpu::BindGroupLayout,
    composite_layout: wgpu::BindGroupLayout,
    blur_h_bind_group: wgpu::BindGroup,
    blur_v_bind_group: wgpu::BindGroup,
    composite_bind_group: wgpu::BindGroup,
}

struct Pipelines {
    noise: FullscreenPipeline,
    blur_h: FullscreenPipeline,
    blur_v: FullscreenPipeline,
    composite: FullscreenPipeline,
}

impl NoiseBlurScene {
    pub fn new(renderer: &Renderer) -> anyhow::Result<Self> {
        let shader_compiler = ShaderCompiler::new(renderer.shader_root().to_path_buf());
        let single_texture_layout = create_single_texture_layout(renderer.device());
        let composite_layout = create_composite_layout(renderer.device());
        let linear_sampler = renderer.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("linear_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let pipelines = build_pipelines(
            renderer,
            &shader_compiler,
            &single_texture_layout,
            &composite_layout,
        )?;

        let mut targets = RenderTargets::new(renderer.surface_format(), renderer.size());
        targets.resize(renderer.device(), renderer.size(), TARGET_NAMES);

        let (blur_h_bind_group, blur_v_bind_group, composite_bind_group) = create_bind_groups(
            renderer.device(),
            &targets,
            &linear_sampler,
            &single_texture_layout,
            &composite_layout,
        );

        Ok(Self {
            shader_compiler,
            targets,
            pipelines,
            linear_sampler,
            single_texture_layout,
            composite_layout,
            blur_h_bind_group,
            blur_v_bind_group,
            composite_bind_group,
        })
    }

    fn rebuild_targets_and_bind_groups(&mut self, renderer: &Renderer) {
        self.targets
            .resize(renderer.device(), renderer.size(), TARGET_NAMES);
        let (blur_h_bind_group, blur_v_bind_group, composite_bind_group) = create_bind_groups(
            renderer.device(),
            &self.targets,
            &self.linear_sampler,
            &self.single_texture_layout,
            &self.composite_layout,
        );
        self.blur_h_bind_group = blur_h_bind_group;
        self.blur_v_bind_group = blur_v_bind_group;
        self.composite_bind_group = composite_bind_group;
    }
}

impl LabScene for NoiseBlurScene {
    fn resize(&mut self, renderer: &Renderer) {
        self.rebuild_targets_and_bind_groups(renderer);
    }

    fn reload(&mut self, renderer: &Renderer) -> anyhow::Result<()> {
        let pipelines = build_pipelines(
            renderer,
            &self.shader_compiler,
            &self.single_texture_layout,
            &self.composite_layout,
        )
        .context("failed to rebuild scene pipelines")?;
        self.pipelines = pipelines;
        Ok(())
    }

    fn render(&mut self, frame: FrameContext<'_>) -> anyhow::Result<()> {
        {
            let mut pass = frame
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("noise_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: self.targets.view("noise_target"),
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
            pass.set_pipeline(&self.pipelines.noise.pipeline);
            pass.set_bind_group(0, frame.globals_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        {
            let mut pass = frame
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("blur_h_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: self.targets.view("blur_h_target"),
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
            pass.set_pipeline(&self.pipelines.blur_h.pipeline);
            pass.set_bind_group(0, frame.globals_bind_group, &[]);
            pass.set_bind_group(1, &self.blur_h_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        {
            let mut pass = frame
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("blur_v_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: self.targets.view("blur_v_target"),
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
            pass.set_pipeline(&self.pipelines.blur_v.pipeline);
            pass.set_bind_group(0, frame.globals_bind_group, &[]);
            pass.set_bind_group(1, &self.blur_v_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        {
            let mut pass = frame
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("composite_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: frame.surface_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.015,
                                g: 0.02,
                                b: 0.03,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });
            pass.set_pipeline(&self.pipelines.composite.pipeline);
            pass.set_bind_group(0, frame.globals_bind_group, &[]);
            pass.set_bind_group(1, &self.composite_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        Ok(())
    }
}

fn build_pipelines(
    renderer: &Renderer,
    shader_compiler: &ShaderCompiler,
    single_texture_layout: &wgpu::BindGroupLayout,
    composite_layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<Pipelines> {
    Ok(Pipelines {
        noise: FullscreenPipeline::new(
            renderer.device(),
            shader_compiler,
            "noise_pipeline",
            "noise_blur/noise.wgsl",
            &[renderer.globals_layout()],
            renderer.surface_format(),
        )?,
        blur_h: FullscreenPipeline::new(
            renderer.device(),
            shader_compiler,
            "blur_h_pipeline",
            "noise_blur/blur_h.wgsl",
            &[renderer.globals_layout(), single_texture_layout],
            renderer.surface_format(),
        )?,
        blur_v: FullscreenPipeline::new(
            renderer.device(),
            shader_compiler,
            "blur_v_pipeline",
            "noise_blur/blur_v.wgsl",
            &[renderer.globals_layout(), single_texture_layout],
            renderer.surface_format(),
        )?,
        composite: FullscreenPipeline::new(
            renderer.device(),
            shader_compiler,
            "composite_pipeline",
            "noise_blur/composite.wgsl",
            &[renderer.globals_layout(), composite_layout],
            renderer.surface_format(),
        )?,
    })
}

fn create_bind_groups(
    device: &wgpu::Device,
    targets: &RenderTargets,
    linear_sampler: &wgpu::Sampler,
    single_texture_layout: &wgpu::BindGroupLayout,
    composite_layout: &wgpu::BindGroupLayout,
) -> (wgpu::BindGroup, wgpu::BindGroup, wgpu::BindGroup) {
    let blur_h_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("blur_h_bind_group"),
        layout: single_texture_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(linear_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(targets.view("noise_target")),
            },
        ],
    });

    let blur_v_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("blur_v_bind_group"),
        layout: single_texture_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(linear_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(targets.view("blur_h_target")),
            },
        ],
    });

    let composite_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("composite_bind_group"),
        layout: composite_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(linear_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(targets.view("noise_target")),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(targets.view("blur_v_target")),
            },
        ],
    });

    (blur_h_bind_group, blur_v_bind_group, composite_bind_group)
}

fn create_single_texture_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("single_texture_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    })
}

fn create_composite_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("composite_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    })
}
