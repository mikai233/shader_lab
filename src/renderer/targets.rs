use std::collections::HashMap;

use winit::dpi::PhysicalSize;

pub struct RenderTarget {
    pub _texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

pub struct RenderTargets {
    format: wgpu::TextureFormat,
    size: PhysicalSize<u32>,
    targets: HashMap<String, RenderTarget>,
}

impl RenderTargets {
    pub fn new(format: wgpu::TextureFormat, size: PhysicalSize<u32>) -> Self {
        Self {
            format,
            size,
            targets: HashMap::new(),
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, size: PhysicalSize<u32>, names: &[&str]) {
        self.size = size;
        self.targets.clear();
        for name in names {
            self.targets
                .insert((*name).to_string(), self.create_target(device, name));
        }
    }

    pub fn view(&self, name: &str) -> &wgpu::TextureView {
        &self.targets[name].view
    }

    fn create_target(&self, device: &wgpu::Device, label: &str) -> RenderTarget {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: self.size.width.max(1),
                height: self.size.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        RenderTarget {
            _texture: texture,
            view,
        }
    }
}
