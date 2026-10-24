use std::thread;
use std::time::Duration;
use std::{
    collections::HashMap,
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[derive(Debug, Clone, PartialEq)]
pub struct UiWindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub color: [u8; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub color: [u8; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiLabel {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub color: [u8; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiButton {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub color: [u8; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiImageCmd {
    pub path: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub source_rect: Option<[f32; 4]>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiRenderCommand {
    Window {
        title: String,
        width: u32,
        height: u32,
    },
    Frame,
    Panel {
        id: String,
        title: String,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    EndPanel,
    Layout {
        kind: String,
        id: String,
    },
    End,
    ScrollArea {
        id: String,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    EndScroll,
    Rect(UiRect),
    Line {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        color: [u8; 4],
    },
    Circle {
        x: i32,
        y: i32,
        radius: u32,
        color: [u8; 4],
    },
    Text(UiLabel),
    Button(UiButton),
    Image(UiImageCmd),
    Widget {
        kind: String,
        id: String,
    },
    Table {
        id: String,
        columns: u32,
    },
    Theme {
        name: String,
    },
    StylePush,
    StylePop,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiStats {
    pub frames: u64,
    pub rects: u64,
    pub images: u64,
    pub labels: u64,
    pub buttons: u64,
    pub draw_calls: u64,
    pub recorded_commands: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct UiFrame {
    width: u32,
    height: u32,
    background: [u8; 4],
    rects: Vec<UiRect>,
    images: Vec<UiImageCmd>,
    labels: Vec<UiLabel>,
    buttons: Vec<UiButton>,
    commands: Vec<UiRenderCommand>,
}

#[derive(Debug)]
pub(crate) struct UiShared {
    open: AtomicBool,
    has_error: AtomicBool,
    frame_front: Mutex<UiFrame>,
    frame_back: Mutex<UiFrame>,
    recorded_commands: Mutex<Vec<UiRenderCommand>>,
    recording_enabled: AtomicBool,
    frame_pacing_enabled: AtomicBool,
    input_enabled: AtomicBool,
    frame_limit: Mutex<Option<u64>>,
    stats_frames: AtomicU64,
    stats_rects: AtomicU64,
    stats_images: AtomicU64,
    stats_labels: AtomicU64,
    stats_buttons: AtomicU64,
    stats_draw_calls: AtomicU64,
    panel_depth: Mutex<usize>,
    layout_depth: Mutex<usize>,
    scroll_depth: Mutex<usize>,
    pressed_keys: Mutex<HashSet<String>>,
    released_keys: Mutex<HashMap<String, u8>>,
    mouse_pos: Mutex<(f64, f64)>,
    mouse_down: AtomicBool,
    mouse_clicked_winit: AtomicBool,
}

#[derive(Debug, Clone)]
pub struct UiRuntime {
    shared: Arc<UiShared>,
    mouse_clicked_this_frame: Arc<AtomicBool>,
}

// ----------------------------------------------------------------------------
// Windows UI Handle
// ----------------------------------------------------------------------------

#[cfg(target_os = "windows")]
mod windows_ui {
    use super::*;
    use once_cell::sync::OnceCell;
    use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};

    pub struct UiHandle(pub(crate) Arc<UiShared>);

    pub enum UiCommand {
        Open(UiWindowConfig, UiHandle),
        Shutdown,
    }

    pub(crate) static UI_SENDER: OnceCell<Sender<UiCommand>> = OnceCell::new();

    pub fn set_global_sender(tx: Sender<UiCommand>) -> bool {
        UI_SENDER.set(tx).is_ok()
    }

    pub fn run_ui_manager(rx: Receiver<UiCommand>, finished: Arc<AtomicBool>) {
        const POLL_INTERVAL: Duration = Duration::from_millis(16);

        loop {
            match rx.recv_timeout(POLL_INTERVAL) {
                Ok(UiCommand::Open(config, UiHandle(shared))) => {
                    run_window(shared, config);
                }
                Ok(UiCommand::Shutdown) => return,
                Err(RecvTimeoutError::Timeout) => {
                    if finished.load(Ordering::SeqCst) {
                        return;
                    }
                }
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::sync::mpsc;

        #[test]
        fn manager_honors_shutdown_without_opening_a_window() {
            let (tx, rx) = mpsc::channel();
            tx.send(UiCommand::Shutdown).unwrap();

            run_ui_manager(rx, Arc::new(AtomicBool::new(false)));
        }

        #[test]
        fn manager_exits_after_the_program_finishes() {
            let (_tx, rx) = mpsc::channel();
            let finished = Arc::new(AtomicBool::new(true));

            run_ui_manager(rx, finished);
        }
    }
}
#[cfg(target_os = "windows")]
pub use windows_ui::{run_ui_manager, set_global_sender, UiCommand, UiHandle};

pub static GLOBAL_UI: once_cell::sync::Lazy<UiRuntime> = once_cell::sync::Lazy::new(UiRuntime::new);
pub const MAX_RECORDED_COMMANDS: usize = 8192;

// ----------------------------------------------------------------------------
// Vertex definitions
// ----------------------------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct TextureVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl TextureVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextureVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

// ----------------------------------------------------------------------------
// UiRuntime Main
// ----------------------------------------------------------------------------

impl UiRuntime {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(UiShared {
                open: AtomicBool::new(false),
                has_error: AtomicBool::new(false),
                frame_front: Mutex::new(UiFrame {
                    width: 800,
                    height: 600,
                    background: [32, 32, 32, 255],
                    rects: Vec::new(),
                    images: Vec::new(),
                    labels: Vec::new(),
                    buttons: Vec::new(),
                    commands: Vec::new(),
                }),
                frame_back: Mutex::new(UiFrame {
                    width: 800,
                    height: 600,
                    background: [32, 32, 32, 255],
                    rects: Vec::new(),
                    images: Vec::new(),
                    labels: Vec::new(),
                    buttons: Vec::new(),
                    commands: Vec::new(),
                }),
                recorded_commands: Mutex::new(Vec::new()),
                recording_enabled: AtomicBool::new(true),
                frame_pacing_enabled: AtomicBool::new(true),
                input_enabled: AtomicBool::new(true),
                frame_limit: Mutex::new(None),
                stats_frames: AtomicU64::new(0),
                stats_rects: AtomicU64::new(0),
                stats_images: AtomicU64::new(0),
                stats_labels: AtomicU64::new(0),
                stats_buttons: AtomicU64::new(0),
                stats_draw_calls: AtomicU64::new(0),
                panel_depth: Mutex::new(0),
                layout_depth: Mutex::new(0),
                scroll_depth: Mutex::new(0),
                pressed_keys: Mutex::new(HashSet::new()),
                released_keys: Mutex::new(HashMap::new()),
                mouse_pos: Mutex::new((0.0, 0.0)),
                mouse_down: AtomicBool::new(false),
                mouse_clicked_winit: AtomicBool::new(false),
            }),
            mouse_clicked_this_frame: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn open_window(&self, config: UiWindowConfig) {
        {
            let mut front = self
                .shared
                .frame_front
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let mut back = self
                .shared
                .frame_back
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            front.width = config.width.max(1);
            front.height = config.height.max(1);
            front.background = config.color;
            front.rects.clear();
            front.images.clear();
            front.labels.clear();
            front.buttons.clear();
            front.commands.clear();
            back.width = config.width.max(1);
            back.height = config.height.max(1);
            back.background = config.color;
            back.rects.clear();
            back.images.clear();
            back.labels.clear();
            back.buttons.clear();
            back.commands.clear();
        }
        self.record(UiRenderCommand::Window {
            title: config.title.clone(),
            width: config.width,
            height: config.height,
        });

        if self.shared.open.swap(true, Ordering::SeqCst) {
            return;
        }

        self.shared.has_error.store(false, Ordering::SeqCst);

        #[cfg(target_os = "windows")]
        {
            if let Some(tx) = windows_ui::UI_SENDER.get() {
                let _ = tx.send(windows_ui::UiCommand::Open(
                    config,
                    windows_ui::UiHandle(self.shared.clone()),
                ));
                return;
            }
        }

        let shared = self.shared.clone();
        thread::spawn(move || run_window(shared, config));
    }

    pub fn begin_frame(&self) {
        let frame = self.shared.stats_frames.fetch_add(1, Ordering::SeqCst) + 1;
        if self
            .shared
            .frame_limit
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .is_some_and(|limit| frame > limit)
        {
            self.shared.open.store(false, Ordering::SeqCst);
        }
        let clicked = self
            .shared
            .mouse_clicked_winit
            .swap(false, Ordering::SeqCst);
        self.mouse_clicked_this_frame
            .store(clicked, Ordering::SeqCst);
        self.decay_released_keys();

        if let (Ok(mut front), Ok(mut back)) = (
            self.shared.frame_front.lock(),
            self.shared.frame_back.lock(),
        ) {
            std::mem::swap(&mut front.rects, &mut back.rects);
            std::mem::swap(&mut front.images, &mut back.images);
            std::mem::swap(&mut front.labels, &mut back.labels);
            std::mem::swap(&mut front.buttons, &mut back.buttons);
            std::mem::swap(&mut front.commands, &mut back.commands);
            front.width = back.width;
            front.height = back.height;
            front.background = back.background;

            back.rects.clear();
            back.images.clear();
            back.labels.clear();
            back.buttons.clear();
            back.commands.clear();
        }
        self.record(UiRenderCommand::Frame);
    }

    pub fn set_recording_enabled(&self, enabled: bool) {
        self.shared
            .recording_enabled
            .store(enabled, Ordering::SeqCst);
    }

    pub fn set_frame_limit(&self, limit: Option<u64>) {
        *self
            .shared
            .frame_limit
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = limit;
    }

    pub fn set_frame_pacing_enabled(&self, enabled: bool) {
        self.shared
            .frame_pacing_enabled
            .store(enabled, Ordering::SeqCst);
    }

    pub fn set_input_enabled(&self, enabled: bool) {
        self.shared.input_enabled.store(enabled, Ordering::SeqCst);
        if !enabled {
            self.shared.mouse_down.store(false, Ordering::SeqCst);
            self.shared
                .mouse_clicked_winit
                .store(false, Ordering::SeqCst);
            self.mouse_clicked_this_frame.store(false, Ordering::SeqCst);
            self.shared
                .pressed_keys
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clear();
            self.shared
                .released_keys
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clear();
        }
    }

    pub fn reset_stats(&self) {
        self.shared.stats_frames.store(0, Ordering::SeqCst);
        self.shared.stats_rects.store(0, Ordering::SeqCst);
        self.shared.stats_images.store(0, Ordering::SeqCst);
        self.shared.stats_labels.store(0, Ordering::SeqCst);
        self.shared.stats_buttons.store(0, Ordering::SeqCst);
        self.shared.stats_draw_calls.store(0, Ordering::SeqCst);
    }

    pub fn stats(&self) -> UiStats {
        UiStats {
            frames: self.shared.stats_frames.load(Ordering::SeqCst),
            rects: self.shared.stats_rects.load(Ordering::SeqCst),
            images: self.shared.stats_images.load(Ordering::SeqCst),
            labels: self.shared.stats_labels.load(Ordering::SeqCst),
            buttons: self.shared.stats_buttons.load(Ordering::SeqCst),
            draw_calls: self.shared.stats_draw_calls.load(Ordering::SeqCst),
            recorded_commands: self
                .shared
                .recorded_commands
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .len(),
        }
    }

    fn decay_released_keys(&self) {
        let mut expired = Vec::new();
        if let Ok(mut released) = self.shared.released_keys.lock() {
            for (key, frames) in released.iter_mut() {
                if *frames == 0 {
                    expired.push(key.clone());
                } else {
                    *frames -= 1;
                }
            }
            for key in &expired {
                released.remove(key);
            }
        }
        if !expired.is_empty() {
            if let Ok(mut pressed) = self.shared.pressed_keys.lock() {
                for key in expired {
                    pressed.remove(&key);
                }
            }
        }
    }

    pub fn frame(&self) -> bool {
        self.is_open()
    }

    pub fn add_image(
        &self,
        path: &str,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        source_rect: Option<[f32; 4]>,
    ) {
        if let Ok(mut back) = self.shared.frame_back.lock() {
            let command = UiImageCmd {
                path: path.to_string(),
                x,
                y,
                width,
                height,
                source_rect,
            };
            back.images.push(command.clone());
            back.commands.push(UiRenderCommand::Image(command.clone()));
            drop(back);
            self.shared.stats_images.fetch_add(1, Ordering::SeqCst);
            self.record(UiRenderCommand::Image(command));
        }
    }

    pub fn add_rect(&self, rect: UiRect) {
        if let Ok(mut frame) = self.shared.frame_back.lock() {
            frame.rects.push(rect.clone());
            frame.commands.push(UiRenderCommand::Rect(rect.clone()));
            drop(frame);
            self.shared.stats_rects.fetch_add(1, Ordering::SeqCst);
            self.record(UiRenderCommand::Rect(rect));
        }
    }

    pub fn add_label(&self, label: UiLabel) {
        if let Ok(mut frame) = self.shared.frame_back.lock() {
            frame.labels.push(label.clone());
            frame.commands.push(UiRenderCommand::Text(label.clone()));
            drop(frame);
            self.shared.stats_labels.fetch_add(1, Ordering::SeqCst);
            self.record(UiRenderCommand::Text(label));
        }
    }

    pub fn add_button(&self, button: UiButton) {
        if let Ok(mut frame) = self.shared.frame_back.lock() {
            frame.buttons.push(button.clone());
            frame.commands.push(UiRenderCommand::Button(button.clone()));
            drop(frame);
            self.shared.stats_buttons.fetch_add(1, Ordering::SeqCst);
            self.record(UiRenderCommand::Button(button));
        }
    }

    pub fn record(&self, command: UiRenderCommand) {
        if !self.shared.recording_enabled.load(Ordering::SeqCst) {
            return;
        }
        let mut commands = self
            .shared
            .recorded_commands
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        commands.push(command);
        if commands.len() > MAX_RECORDED_COMMANDS {
            let excess = commands.len() - MAX_RECORDED_COMMANDS;
            commands.drain(0..excess);
        }
    }

    pub fn recorded_commands(&self) -> Vec<UiRenderCommand> {
        self.shared
            .recorded_commands
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    pub fn clear_recorded_commands(&self) {
        self.shared
            .recorded_commands
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
    }

    pub fn push_panel(&self) {
        *self
            .shared
            .panel_depth
            .lock()
            .unwrap_or_else(|error| error.into_inner()) += 1;
    }

    pub fn pop_panel(&self) -> bool {
        let mut depth = self
            .shared
            .panel_depth
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if *depth == 0 {
            false
        } else {
            *depth -= 1;
            true
        }
    }

    pub fn push_layout(&self) {
        *self
            .shared
            .layout_depth
            .lock()
            .unwrap_or_else(|error| error.into_inner()) += 1;
    }

    pub fn pop_layout(&self) -> bool {
        let mut depth = self
            .shared
            .layout_depth
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if *depth == 0 {
            false
        } else {
            *depth -= 1;
            true
        }
    }

    pub fn push_scroll(&self) {
        *self
            .shared
            .scroll_depth
            .lock()
            .unwrap_or_else(|error| error.into_inner()) += 1;
    }

    pub fn pop_scroll(&self) -> bool {
        let mut depth = self
            .shared
            .scroll_depth
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if *depth == 0 {
            false
        } else {
            *depth -= 1;
            true
        }
    }

    pub fn is_open(&self) -> bool {
        if self.shared.open.load(Ordering::SeqCst) {
            if self.shared.frame_pacing_enabled.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(15));
            }
            self.begin_frame();
        }
        self.shared.open.load(Ordering::SeqCst)
    }

    pub fn has_error(&self) -> bool {
        self.shared.has_error.load(Ordering::SeqCst)
    }

    pub fn is_key_pressed(&self, key: &str) -> bool {
        if !self.shared.input_enabled.load(Ordering::SeqCst) {
            return false;
        }
        let key = normalize_key_name(key);
        let keys = self
            .shared
            .pressed_keys
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        keys.contains(key.as_str())
    }

    pub fn get_mouse_state(&self) -> ((f64, f64), bool, bool) {
        if !self.shared.input_enabled.load(Ordering::SeqCst) {
            return ((0.0, 0.0), false, false);
        }
        let pos = *self
            .shared
            .mouse_pos
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let down = self.shared.mouse_down.load(Ordering::SeqCst);
        let clicked = self.mouse_clicked_this_frame.load(Ordering::SeqCst);
        (pos, down, clicked)
    }
}

fn normalize_key_name(key: &str) -> String {
    let lower = key.to_ascii_lowercase();
    match lower.as_str() {
        "arrowup" | "up" => "arrowup".to_string(),
        "arrowdown" | "down" => "arrowdown".to_string(),
        "arrowleft" | "left" => "arrowleft".to_string(),
        "arrowright" | "right" => "arrowright".to_string(),
        "escape" | "esc" => "escape".to_string(),
        "enter" | "return" => "enter".to_string(),
        "space" | "spacebar" => "space".to_string(),
        _ if lower.starts_with("key") && lower.len() == 4 => lower[3..].to_string(),
        _ => lower,
    }
}

impl Default for UiRuntime {
    fn default() -> Self {
        Self::new()
    }
}

fn run_window(shared: Arc<UiShared>, config: UiWindowConfig) {
    let mut builder = EventLoop::builder();

    #[cfg(target_os = "linux")]
    {
        use winit::platform::x11::EventLoopBuilderExtX11;
        EventLoopBuilderExtX11::with_any_thread(&mut builder, true);
    }

    #[cfg(target_os = "windows")]
    {
        use winit::platform::windows::EventLoopBuilderExtWindows;
        builder.with_any_thread(true);
    }

    let event_loop = match builder.build() {
        Ok(event_loop) => event_loop,
        Err(_) => {
            shared.has_error.store(true, Ordering::SeqCst);
            shared.open.store(false, Ordering::SeqCst);
            return;
        }
    };

    let mut app = UiApp::new(shared.clone(), config);
    if event_loop.run_app(&mut app).is_err() {
        shared.has_error.store(true, Ordering::SeqCst);
    }
    shared.open.store(false, Ordering::SeqCst);
}

// ----------------------------------------------------------------------------
// Wgpu State
// ----------------------------------------------------------------------------

struct CachedTexture {
    bind_group: Arc<wgpu::BindGroup>,
    width: u32,
    height: u32,
}

struct WgpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    image_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    texture_cache: std::collections::HashMap<String, CachedTexture>,
    glyph_brush: wgpu_glyph::GlyphBrush<()>,
    staging_belt: wgpu::util::StagingBelt,
}

impl WgpuState {
    async fn new(window: &'static Window) -> Option<Self> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).ok()?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok()?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .ok()?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let image_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Image Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                immediate_size: 0,
            });

        let image_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Image Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("texture_shader.wgsl").into()),
        });

        let image_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Image Render Pipeline"),
            layout: Some(&image_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &image_shader,
                entry_point: Some("vs_main"),
                buffers: &[TextureVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &image_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let font = ab_glyph::FontArc::try_from_slice(include_bytes!("../assets/font.ttf")).ok()?;

        let glyph_brush =
            wgpu_glyph::GlyphBrushBuilder::using_font(font).build(&device, surface_format);

        let staging_belt = wgpu::util::StagingBelt::new(device.clone(), 1024);

        Some(Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            image_pipeline,
            texture_bind_group_layout,
            sampler,
            texture_cache: std::collections::HashMap::new(),
            glyph_brush,
            staging_belt,
        })
    }

    fn load_texture(&mut self, path: &str) -> Option<(u32, u32, Arc<wgpu::BindGroup>)> {
        if let Some(cached) = self.texture_cache.get(path) {
            return Some((cached.width, cached.height, cached.bind_group.clone()));
        }

        let img = match image::open(path) {
            Ok(i) => i.into_rgba8(),
            Err(e) => {
                println!("Failed to load image '{}': {}", path, e);
                return None;
            }
        };
        let dimensions = img.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(path),
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let bind_group_arc = Arc::new(bind_group);
        let cached = CachedTexture {
            bind_group: bind_group_arc.clone(),
            width: dimensions.0,
            height: dimensions.1,
        };
        self.texture_cache.insert(path.to_string(), cached);

        Some((dimensions.0, dimensions.1, bind_group_arc))
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn map_color(&self, c: [u8; 4]) -> wgpu::Color {
        wgpu::Color {
            r: c[0] as f64 / 255.0,
            g: c[1] as f64 / 255.0,
            b: c[2] as f64 / 255.0,
            a: c[3] as f64 / 255.0,
        }
    }

    fn render(&mut self, frame_state: &UiFrame) -> Result<u64, wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut draw_calls = 0u64;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut vertices = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        let logical_w = frame_state.width as f32;
        let logical_h = frame_state.height as f32;
        let phys_w = self.size.width as f32;
        let phys_h = self.size.height as f32;

        let scale = f32::min(phys_w / logical_w, phys_h / logical_h);
        let offset_x = (phys_w - logical_w * scale) / 2.0;
        let offset_y = (phys_h - logical_h * scale) / 2.0;

        let mut add_quad = |x: f32, y: f32, width: f32, height: f32, color: [u8; 4]| {
            let px = offset_x + x * scale;
            let py = offset_y + y * scale;
            let pw = width * scale;
            let ph = height * scale;

            let cx1 = (px / phys_w) * 2.0 - 1.0;
            let cy1 = 1.0 - (py / phys_h) * 2.0;
            let cx2 = ((px + pw) / phys_w) * 2.0 - 1.0;
            let cy2 = 1.0 - ((py + ph) / phys_h) * 2.0;

            let c = [
                color[0] as f32 / 255.0,
                color[1] as f32 / 255.0,
                color[2] as f32 / 255.0,
                color[3] as f32 / 255.0,
            ];

            let base = vertices.len() as u16;
            vertices.push(Vertex {
                position: [cx1, cy1],
                color: c,
            });
            vertices.push(Vertex {
                position: [cx1, cy2],
                color: c,
            });
            vertices.push(Vertex {
                position: [cx2, cy2],
                color: c,
            });
            vertices.push(Vertex {
                position: [cx2, cy1],
                color: c,
            });

            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);
            indices.push(base);
            indices.push(base + 2);
            indices.push(base + 3);
        };

        for rect in &frame_state.rects {
            add_quad(
                rect.x as f32,
                rect.y as f32,
                rect.width as f32,
                rect.height as f32,
                rect.color,
            );
        }

        for btn in &frame_state.buttons {
            add_quad(
                btn.x as f32,
                btn.y as f32,
                btn.width as f32,
                btn.height as f32,
                btn.color,
            );
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.map_color(frame_state.background)),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            if !vertices.is_empty() {
                draw_calls += 1;
                let vertex_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                let index_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(&indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            }
        }

        // Draw Images
        if frame_state.images.is_empty() && frame_state.rects.is_empty() {
            // println!("Frame is totally empty!"); // to avoid spam
        }
        for img in &frame_state.images {
            if let Some((tex_width, tex_height, bind_group)) = self.load_texture(&img.path) {
                let px = offset_x + img.x as f32 * scale;
                let py = offset_y + img.y as f32 * scale;
                let pw = img.width as f32 * scale;
                let ph = img.height as f32 * scale;

                let cx1 = (px / phys_w) * 2.0 - 1.0;
                let cy1 = 1.0 - (py / phys_h) * 2.0;
                let cx2 = ((px + pw) / phys_w) * 2.0 - 1.0;
                let cy2 = 1.0 - ((py + ph) / phys_h) * 2.0;

                let (uv_x1, uv_y1, uv_x2, uv_y2) = if let Some([sx, sy, sw, sh]) = img.source_rect {
                    let tex_w = tex_width as f32;
                    let tex_h = tex_height as f32;
                    (sx / tex_w, sy / tex_h, (sx + sw) / tex_w, (sy + sh) / tex_h)
                } else {
                    (0.0, 0.0, 1.0, 1.0)
                };

                let vertices: &[TextureVertex] = &[
                    TextureVertex {
                        position: [cx1, cy1],
                        uv: [uv_x1, uv_y1],
                    },
                    TextureVertex {
                        position: [cx1, cy2],
                        uv: [uv_x1, uv_y2],
                    },
                    TextureVertex {
                        position: [cx2, cy2],
                        uv: [uv_x2, uv_y2],
                    },
                    TextureVertex {
                        position: [cx2, cy1],
                        uv: [uv_x2, uv_y1],
                    },
                ];
                let tex_indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

                let vertex_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Tex Vertex Buffer"),
                            contents: bytemuck::cast_slice(vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                let index_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Tex Index Buffer"),
                            contents: bytemuck::cast_slice(&tex_indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Image Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

                render_pass.set_pipeline(&self.image_pipeline);
                render_pass.set_bind_group(0, &*bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                draw_calls += 1;
                render_pass.draw_indexed(0..6, 0, 0..1);
            }
        }

        for label in &frame_state.labels {
            self.glyph_brush.queue(wgpu_glyph::Section {
                screen_position: (
                    offset_x + label.x as f32 * scale,
                    offset_y + label.y as f32 * scale,
                ),
                bounds: (phys_w, phys_h),
                text: vec![wgpu_glyph::Text::new(&label.text)
                    .with_color([
                        label.color[0] as f32 / 255.0,
                        label.color[1] as f32 / 255.0,
                        label.color[2] as f32 / 255.0,
                        label.color[3] as f32 / 255.0,
                    ])
                    .with_scale(24.0 * scale)],
                ..wgpu_glyph::Section::default()
            });
        }

        for btn in &frame_state.buttons {
            self.glyph_brush.queue(wgpu_glyph::Section {
                screen_position: (
                    offset_x + (btn.x as f32 + 5.0) * scale,
                    offset_y + (btn.y as f32 + (btn.height as f32 / 2.0) - 12.0) * scale,
                ),
                bounds: (phys_w, phys_h),
                text: vec![wgpu_glyph::Text::new(&btn.text)
                    .with_color([1.0, 1.0, 1.0, 1.0])
                    .with_scale(20.0 * scale)],
                ..wgpu_glyph::Section::default()
            });
        }

        if !frame_state.labels.is_empty() || !frame_state.buttons.is_empty() {
            draw_calls += 1;
        }

        if let Err(e) = self.glyph_brush.draw_queued(
            &self.device,
            &mut self.staging_belt,
            &mut encoder,
            &view,
            self.size.width,
            self.size.height,
        ) {
            eprintln!("UI render error: {}", e);
        }

        self.staging_belt.finish();
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.staging_belt.recall();

        Ok(draw_calls)
    }
}

// ----------------------------------------------------------------------------
// UiApp
// ----------------------------------------------------------------------------

struct UiApp {
    shared: Arc<UiShared>,
    config: UiWindowConfig,
    window_id: Option<WindowId>,
    window: Option<&'static Window>,
    wgpu_state: Option<WgpuState>,
}

impl UiApp {
    fn new(shared: Arc<UiShared>, config: UiWindowConfig) -> Self {
        Self {
            shared,
            config,
            window_id: None,
            window: None,
            wgpu_state: None,
        }
    }
}

impl ApplicationHandler for UiApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title(self.config.title.clone())
            .with_inner_size(LogicalSize::new(
                self.config.width.max(1) as f64,
                self.config.height.max(1) as f64,
            ));

        let window = match event_loop.create_window(attrs) {
            Ok(window) => window,
            Err(_) => {
                self.shared.has_error.store(true, Ordering::SeqCst);
                self.shared.open.store(false, Ordering::SeqCst);
                event_loop.exit();
                return;
            }
        };

        let window: &'static Window = Box::leak(Box::new(window));

        self.window_id = Some(window.id());
        self.window = Some(window);

        let wgpu_state = pollster::block_on(WgpuState::new(window));
        if let Some(state) = wgpu_state {
            self.wgpu_state = Some(state);
        } else {
            self.shared.has_error.store(true, Ordering::SeqCst);
            self.shared.open.store(false, Ordering::SeqCst);
            event_loop.exit();
            return;
        }

        window.focus_window();
        window.request_redraw();
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if !self.shared.open.load(Ordering::SeqCst) {
            event_loop.exit();
            return;
        }
        if let Some(window) = self.window {
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if Some(window_id) != self.window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                self.shared.open.store(false, Ordering::SeqCst);
                event_loop.exit();
            }
            WindowEvent::Focused(false) => {
                let mut keys = self
                    .shared
                    .pressed_keys
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                keys.clear();
                self.shared
                    .released_keys
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .clear();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let mut key_name = None;
                if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                    key_name =
                        match code {
                            winit::keyboard::KeyCode::ArrowUp | winit::keyboard::KeyCode::KeyW => {
                                Some("arrowup".to_string())
                            }
                            winit::keyboard::KeyCode::ArrowDown
                            | winit::keyboard::KeyCode::KeyS => Some("arrowdown".to_string()),
                            winit::keyboard::KeyCode::ArrowLeft
                            | winit::keyboard::KeyCode::KeyA => Some("arrowleft".to_string()),
                            winit::keyboard::KeyCode::ArrowRight
                            | winit::keyboard::KeyCode::KeyD => Some("arrowright".to_string()),
                            winit::keyboard::KeyCode::Escape => Some("escape".to_string()),
                            winit::keyboard::KeyCode::Enter => Some("enter".to_string()),
                            winit::keyboard::KeyCode::Space => Some("space".to_string()),
                            _ => None,
                        };
                }
                if key_name.is_none() {
                    if let winit::keyboard::Key::Character(c) = &event.logical_key {
                        key_name = Some(c.to_string().to_lowercase());
                    }
                }
                if let Some(key_name) = key_name {
                    let mut keys = self
                        .shared
                        .pressed_keys
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    if event.state == winit::event::ElementState::Pressed {
                        keys.insert(key_name.clone());
                        drop(keys);
                        self.shared
                            .released_keys
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .remove(&key_name);
                    } else {
                        drop(keys);
                        self.shared
                            .released_keys
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .insert(key_name, 5);
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let phys_w = self
                    .wgpu_state
                    .as_ref()
                    .map(|s| s.size.width as f64)
                    .unwrap_or(800.0);
                let phys_h = self
                    .wgpu_state
                    .as_ref()
                    .map(|s| s.size.height as f64)
                    .unwrap_or(600.0);
                let logical_w = self
                    .shared
                    .frame_front
                    .lock()
                    .map(|f| f.width as f64)
                    .unwrap_or(800.0);
                let logical_h = self
                    .shared
                    .frame_front
                    .lock()
                    .map(|f| f.height as f64)
                    .unwrap_or(600.0);

                let scale = f64::min(phys_w / logical_w, phys_h / logical_h);
                let offset_x = (phys_w - logical_w * scale) / 2.0;
                let offset_y = (phys_h - logical_h * scale) / 2.0;

                let mx = (position.x - offset_x) / scale;
                let my = (position.y - offset_y) / scale;

                if let Ok(mut pos) = self.shared.mouse_pos.lock() {
                    *pos = (mx, my);
                }
            }
            WindowEvent::MouseInput {
                state,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                let is_pressed = state == winit::event::ElementState::Pressed;
                self.shared.mouse_down.store(is_pressed, Ordering::SeqCst);
                if is_pressed {
                    self.shared
                        .mouse_clicked_winit
                        .store(true, Ordering::SeqCst);
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(wgpu_state) = self.wgpu_state.as_mut() {
                    wgpu_state.resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                if !self.shared.open.load(Ordering::SeqCst) {
                    event_loop.exit();
                    return;
                }
                if let Some(wgpu_state) = self.wgpu_state.as_mut() {
                    let frame_state = self
                        .shared
                        .frame_front
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .clone();
                    match wgpu_state.render(&frame_state) {
                        Ok(draw_calls) => {
                            self.shared
                                .stats_draw_calls
                                .fetch_add(draw_calls, Ordering::SeqCst);
                        }
                        Err(wgpu::SurfaceError::Lost) => {
                            let size = wgpu_state.size;
                            wgpu_state.resize(size);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            self.shared.has_error.store(true, Ordering::SeqCst);
                            self.shared.open.store(false, Ordering::SeqCst);
                            event_loop.exit();
                        }
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn parse_hex_color(text: &str) -> Option<[u8; 4]> {
    let hex = text.strip_prefix('#').unwrap_or(text);
    if !(hex.len() == 6 || hex.len() == 8) || !hex.is_ascii() {
        return None;
    }
    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    let alpha = if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16).ok()?
    } else {
        255
    };
    Some([red, green, blue, alpha])
}
