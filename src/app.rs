use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use log::{error, info, warn};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::hot_reload::ShaderHotReload;
use crate::input::InputState;
use crate::renderer::{GlobalsData, RenderError, Renderer};
use crate::scene::{LabScene, create_default_scene};

pub fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = ShaderLabApp::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}

struct ShaderLabApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    scene: Option<Box<dyn LabScene>>,
    hot_reload: Option<ShaderHotReload>,
    input: InputState,
    start_time: Instant,
    last_frame_time: Instant,
    frame_index: u32,
    compare_enabled: bool,
    compare_split: f32,
}

impl Default for ShaderLabApp {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            window: None,
            renderer: None,
            scene: None,
            hot_reload: None,
            input: InputState::default(),
            start_time: now,
            last_frame_time: now,
            frame_index: 0,
            compare_enabled: true,
            compare_split: 0.5,
        }
    }
}

impl ShaderLabApp {
    fn init(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        if self.window.is_some() {
            return Ok(());
        }

        let window = Arc::new(
            event_loop.create_window(
                WindowAttributes::default()
                    .with_title("shader_lab | C toggle compare | left-drag split | Esc to quit")
                    .with_inner_size(PhysicalSize::new(1280, 720))
                    .with_resizable(true),
            )?,
        );

        let renderer = pollster::block_on(Renderer::new(window.clone()))
            .context("failed to create renderer")?;
        let scene = create_default_scene(&renderer).context("failed to create default scene")?;
        let hot_reload = ShaderHotReload::new(PathBuf::from("shaders"))
            .ok()
            .flatten();

        self.input = InputState::new(renderer.size());
        self.window = Some(window);
        self.renderer = Some(renderer);
        self.scene = Some(scene);
        self.hot_reload = hot_reload;
        self.start_time = Instant::now();
        self.last_frame_time = self.start_time;
        self.frame_index = 0;

        info!("shader_lab initialized");
        Ok(())
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        let (Some(window), Some(renderer), Some(scene)) = (
            self.window.as_ref(),
            self.renderer.as_mut(),
            self.scene.as_mut(),
        ) else {
            return;
        };

        if let Some(hot_reload) = &self.hot_reload {
            let changed = hot_reload.drain();
            if !changed.is_empty() {
                match scene.reload(renderer) {
                    Ok(()) => info!("reloaded shaders after {} file changes", changed.len()),
                    Err(err) => error!("shader reload failed, keeping previous pipelines: {err:#}"),
                }
            }
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.start_time).as_secs_f32();
        let delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        renderer.update_globals(GlobalsData {
            resolution: [renderer.size().width as f32, renderer.size().height as f32],
            mouse: self.input.mouse_position,
            time: elapsed,
            delta_time,
            frame_index: self.frame_index,
            compare_enabled: u32::from(self.compare_enabled),
            compare_split: self.compare_split,
            _padding: 0.0,
        });

        self.frame_index = self.frame_index.wrapping_add(1);

        match renderer.render(scene.as_mut()) {
            Ok(()) => {}
            Err(RenderError::Surface(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated)) => {
                renderer.resize(renderer.size());
                scene.resize(renderer);
                window.request_redraw();
            }
            Err(RenderError::Surface(wgpu::SurfaceError::OutOfMemory)) => {
                error!("wgpu surface ran out of memory");
                event_loop.exit();
            }
            Err(RenderError::Surface(wgpu::SurfaceError::Timeout)) => {
                warn!("surface timeout");
            }
            Err(RenderError::Surface(wgpu::SurfaceError::Other)) => {
                warn!("surface returned an unspecified error");
            }
            Err(RenderError::Other(err)) => {
                error!("render error: {err:#}");
            }
        }
    }
}

impl ApplicationHandler for ShaderLabApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(err) = self.init(event_loop) {
            error!("failed to initialize app: {err:#}");
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    self.input.set_resolution(size);
                    if let (Some(renderer), Some(scene)) =
                        (self.renderer.as_mut(), self.scene.as_mut())
                    {
                        renderer.resize(size);
                        scene.resize(renderer);
                    }
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                let size = window.inner_size();
                self.input.set_resolution(size);
                if let (Some(renderer), Some(scene)) = (self.renderer.as_mut(), self.scene.as_mut())
                {
                    renderer.resize(size);
                    scene.resize(renderer);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input.mouse_position = [position.x as f32, position.y as f32];
                if self.input.drag_compare {
                    let width = self.input.resolution[0].max(1.0);
                    self.compare_split = (position.x as f32 / width).clamp(0.05, 0.95);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    self.input.drag_compare = state == ElementState::Pressed;
                    if self.input.drag_compare {
                        let width = self.input.resolution[0].max(1.0);
                        self.compare_split =
                            (self.input.mouse_position[0] / width).clamp(0.05, 0.95);
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed() {
                    match event.logical_key {
                        Key::Named(NamedKey::Escape) => event_loop.exit(),
                        Key::Character(ref text) if text.eq_ignore_ascii_case("c") => {
                            self.compare_enabled = !self.compare_enabled;
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::RedrawRequested => self.redraw(event_loop),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
