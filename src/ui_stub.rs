use once_cell::sync::Lazy;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};

pub const MAX_RECORDED_COMMANDS: usize = 8192;

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

#[derive(Debug, Default)]
pub struct UiRuntime {
    open: AtomicBool,
    commands: Mutex<Vec<UiRenderCommand>>,
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
}

impl UiRuntime {
    pub fn new() -> Self {
        Self {
            open: AtomicBool::new(false),
            commands: Mutex::new(Vec::new()),
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
        }
    }

    pub fn open_window(&self, config: UiWindowConfig) {
        self.open.store(true, Ordering::SeqCst);
        self.record(UiRenderCommand::Window {
            title: config.title,
            width: config.width,
            height: config.height,
        });
    }

    pub fn begin_frame(&self) {
        let frame = self.stats_frames.fetch_add(1, Ordering::SeqCst) + 1;
        if self
            .frame_limit
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .is_some_and(|limit| frame > limit)
        {
            self.open.store(false, Ordering::SeqCst);
        }
        self.record(UiRenderCommand::Frame);
    }

    pub fn frame(&self) -> bool {
        self.begin_frame();
        self.open.load(Ordering::SeqCst)
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
        self.record(UiRenderCommand::Image(UiImageCmd {
            path: path.to_string(),
            x,
            y,
            width,
            height,
            source_rect,
        }));
        self.stats_images.fetch_add(1, Ordering::SeqCst);
    }

    pub fn add_rect(&self, rect: UiRect) {
        self.record(UiRenderCommand::Rect(rect));
        self.stats_rects.fetch_add(1, Ordering::SeqCst);
    }

    pub fn add_label(&self, label: UiLabel) {
        self.record(UiRenderCommand::Text(label));
        self.stats_labels.fetch_add(1, Ordering::SeqCst);
    }

    pub fn add_button(&self, button: UiButton) {
        self.record(UiRenderCommand::Button(button));
        self.stats_buttons.fetch_add(1, Ordering::SeqCst);
    }

    pub fn record(&self, command: UiRenderCommand) {
        if !self.recording_enabled.load(Ordering::SeqCst) {
            return;
        }
        let mut commands = self
            .commands
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        commands.push(command);
        if commands.len() > MAX_RECORDED_COMMANDS {
            let excess = commands.len() - MAX_RECORDED_COMMANDS;
            commands.drain(0..excess);
        }
    }

    pub fn set_recording_enabled(&self, enabled: bool) {
        self.recording_enabled.store(enabled, Ordering::SeqCst);
    }

    pub fn set_frame_limit(&self, limit: Option<u64>) {
        *self
            .frame_limit
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = limit;
    }

    pub fn set_frame_pacing_enabled(&self, enabled: bool) {
        self.frame_pacing_enabled.store(enabled, Ordering::SeqCst);
    }

    pub fn set_input_enabled(&self, enabled: bool) {
        self.input_enabled.store(enabled, Ordering::SeqCst);
    }

    pub fn reset_stats(&self) {
        self.stats_frames.store(0, Ordering::SeqCst);
        self.stats_rects.store(0, Ordering::SeqCst);
        self.stats_images.store(0, Ordering::SeqCst);
        self.stats_labels.store(0, Ordering::SeqCst);
        self.stats_buttons.store(0, Ordering::SeqCst);
        self.stats_draw_calls.store(0, Ordering::SeqCst);
    }

    pub fn stats(&self) -> UiStats {
        UiStats {
            frames: self.stats_frames.load(Ordering::SeqCst),
            rects: self.stats_rects.load(Ordering::SeqCst),
            images: self.stats_images.load(Ordering::SeqCst),
            labels: self.stats_labels.load(Ordering::SeqCst),
            buttons: self.stats_buttons.load(Ordering::SeqCst),
            draw_calls: self.stats_draw_calls.load(Ordering::SeqCst),
            recorded_commands: self
                .commands
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .len(),
        }
    }

    pub fn recorded_commands(&self) -> Vec<UiRenderCommand> {
        self.commands
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    pub fn clear_recorded_commands(&self) {
        self.commands
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
    }

    pub fn push_panel(&self) {
        *self
            .panel_depth
            .lock()
            .unwrap_or_else(|error| error.into_inner()) += 1;
    }

    pub fn pop_panel(&self) -> bool {
        let mut depth = self
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
            .layout_depth
            .lock()
            .unwrap_or_else(|error| error.into_inner()) += 1;
    }

    pub fn pop_layout(&self) -> bool {
        let mut depth = self
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
            .scroll_depth
            .lock()
            .unwrap_or_else(|error| error.into_inner()) += 1;
    }

    pub fn pop_scroll(&self) -> bool {
        let mut depth = self
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
        self.open.load(Ordering::SeqCst)
    }

    pub fn has_error(&self) -> bool {
        false
    }

    pub fn is_key_pressed(&self, _key: &str) -> bool {
        if !self.input_enabled.load(Ordering::SeqCst) {
            return false;
        }
        false
    }

    pub fn get_mouse_state(&self) -> ((f64, f64), bool, bool) {
        ((0.0, 0.0), false, false)
    }
}

pub static GLOBAL_UI: Lazy<UiRuntime> = Lazy::new(UiRuntime::new);

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
