#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use vice::installer::{build_install_plan_from_source, install_and_replace_from_plan};

// Embedded icon
const VICE_ICON: &[u8] = include_bytes!("../../assets/vice_icon.png");

type InstallResult = Result<Vec<PathBuf>, String>;
type SharedInstallResult = Arc<Mutex<Option<InstallResult>>>;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 400.0])
            .with_decorations(false) // Frameless window
            .with_transparent(true),
        ..Default::default()
    };

    eframe::run_native(
        "VICE Installer",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(ViceInstallerApp::new(cc)))
        }),
    )
}

#[derive(PartialEq)]
enum AppState {
    Intro,
    ReadyToInstall,
    Installing,
    Done,
    Error(String),
}

struct ViceInstallerApp {
    state: AppState,
    start_time: Instant,
    progress: f32,
    old_version_detected: bool,
    target_dir: Option<PathBuf>,
    source_dir: PathBuf,
    install_thread: Option<thread::JoinHandle<InstallResult>>,
    install_result: SharedInstallResult,
}

impl ViceInstallerApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Simple heuristic for source directory
        let source_dir = env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."));

        // Check for old version
        let mut old_version_detected = false;
        if let Some(user_dirs) = directories::UserDirs::new() {
            let local_data = if cfg!(target_os = "windows") {
                PathBuf::from(env::var("LOCALAPPDATA").unwrap_or_else(|_| "".to_string()))
            } else {
                user_dirs.home_dir().join(".local").join("share")
            };
            let vice_dir = local_data.join("VICE");
            if vice_dir.exists() {
                old_version_detected = true;
            }
        }

        Self {
            state: AppState::Intro,
            start_time: Instant::now(),
            progress: 0.0,
            old_version_detected,
            target_dir: None, // Default will be resolved by installer core
            source_dir,
            install_thread: None,
            install_result: Arc::new(Mutex::new(None)),
        }
    }

    fn start_installation(&mut self) {
        self.state = AppState::Installing;
        self.progress = 0.0;

        let source = self.source_dir.clone();
        let target = self.target_dir.clone();
        let result_arc = self.install_result.clone();

        self.install_thread = Some(thread::spawn(move || {
            // Fake delay for premium feel
            thread::sleep(std::time::Duration::from_millis(1500));

            let plan = match build_install_plan_from_source(&source, target) {
                Ok(p) => p,
                Err(e) => {
                    *result_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                        Some(Err(format!("Plan error: {}", e)));
                    return Err(e.to_string());
                }
            };

            match install_and_replace_from_plan(&plan) {
                Ok(paths) => {
                    *result_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Ok(paths.clone()));
                    Ok(paths)
                }
                Err(e) => {
                    *result_arc
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                        Some(Err(format!("Install error: {}", e)));
                    Err(e.to_string())
                }
            }
        }));
    }
}

impl eframe::App for ViceInstallerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make it transparent
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Continuous repaint for animations
        ctx.request_repaint();

        let t = self.start_time.elapsed().as_secs_f32();

        // Intro transition
        if self.state == AppState::Intro && t > 2.0 {
            self.state = AppState::ReadyToInstall;
        }

        // Install progress transition
        if self.state == AppState::Installing {
            self.progress += 0.01;
            if self.progress >= 1.0 {
                self.progress = 1.0;
            }
            // Check thread
            let res = self
                .install_result
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take();
            if let Some(result) = res {
                match result {
                    Ok(_) => self.state = AppState::Done,
                    Err(err) => self.state = AppState::Error(err),
                }
            }
        }

        custom_window_frame(ui, "VICE Installer", |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);

                // Pulse animation for logo
                let pulse = (t * 3.0).sin() * 0.05 + 1.0;
                let size = if self.state == AppState::Intro {
                    egui::vec2(150.0 * pulse, 150.0 * pulse)
                } else {
                    egui::vec2(100.0, 100.0)
                };

                let image = egui::Image::from_bytes("bytes://vice_icon.png", VICE_ICON)
                    .fit_to_exact_size(size)
                    .corner_radius(16.0);

                ui.add(image);
                ui.add_space(20.0);

                // Dynamic UI based on state
                match self.state {
                    AppState::Intro => {
                        ui.heading(
                            egui::RichText::new("VICE Engine")
                                .color(egui::Color32::from_rgb(0, 229, 255))
                                .size(32.0),
                        );
                        ui.label(
                            egui::RichText::new("Inicjalizacja...").color(egui::Color32::GRAY),
                        );
                    }
                    AppState::ReadyToInstall => {
                        ui.heading(
                            egui::RichText::new("Gotowy do instalacji")
                                .color(egui::Color32::WHITE)
                                .size(24.0),
                        );

                        ui.add_space(10.0);
                        if self.old_version_detected {
                            ui.label(
                                egui::RichText::new(
                                    "Wykryto poprzednią wersję. Zostanie zaktualizowana.",
                                )
                                .color(egui::Color32::from_rgb(138, 43, 226)),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new("Nowa instalacja.")
                                    .color(egui::Color32::LIGHT_GRAY),
                            );
                        }

                        ui.add_space(30.0);

                        let btn_text = if self.old_version_detected {
                            "AKTUALIZUJ VICE"
                        } else {
                            "ZAINSTALUJ VICE"
                        };

                        // Custom nice button
                        let btn = egui::Button::new(
                            egui::RichText::new(btn_text)
                                .size(20.0)
                                .color(egui::Color32::BLACK),
                        )
                        .fill(egui::Color32::from_rgb(0, 229, 255))
                        .min_size(egui::vec2(200.0, 50.0))
                        .corner_radius(8.0);

                        if ui.add(btn).clicked() {
                            self.start_installation();
                        }
                    }
                    AppState::Installing => {
                        ui.heading(
                            egui::RichText::new("Instalowanie...")
                                .color(egui::Color32::WHITE)
                                .size(24.0),
                        );
                        ui.add_space(20.0);

                        // Custom progress bar
                        let progress_bar = egui::ProgressBar::new(self.progress)
                            .fill(egui::Color32::from_rgb(138, 43, 226))
                            .animate(true);
                        ui.add(progress_bar);
                    }
                    AppState::Done => {
                        ui.heading(
                            egui::RichText::new("Zakończono Pomyślnie!")
                                .color(egui::Color32::from_rgb(0, 255, 128))
                                .size(28.0),
                        );
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new("VICE jest gotowy do działania.")
                                .color(egui::Color32::WHITE),
                        );
                        ui.add_space(30.0);

                        let btn = egui::Button::new(
                            egui::RichText::new("ZAMKNIJ")
                                .size(18.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(40, 40, 40))
                        .min_size(egui::vec2(150.0, 40.0));

                        if ui.add(btn).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                    AppState::Error(ref err) => {
                        ui.heading(
                            egui::RichText::new("Wystąpił Błąd")
                                .color(egui::Color32::RED)
                                .size(24.0),
                        );
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(err).color(egui::Color32::LIGHT_RED));

                        ui.add_space(30.0);
                        if ui.button("ZAMKNIJ").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                }
            });
        });
    }
}

fn custom_window_frame(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    let ctx = ui.ctx().clone();
    let panel_frame = egui::Frame {
        fill: egui::Color32::from_rgb(11, 12, 16), // Dark premium background #0B0C10
        corner_radius: 12.0.into(),
        stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(30, 30, 35)),
        outer_margin: 0.5.into(),
        inner_margin: egui::Margin::same(10),
        shadow: egui::epaint::Shadow {
            blur: 16,
            spread: 0,
            offset: [0, 8],
            color: egui::Color32::from_black_alpha(150),
        },
    };

    egui::CentralPanel::default()
        .frame(panel_frame)
        .show_inside(ui, |ui| {
            // Custom title bar
            let title_bar_height = 32.0;
            let title_bar_rect = {
                let mut rect = ui.max_rect();
                rect.max.y = rect.min.y + title_bar_height;
                rect
            };

            let mut title_bar_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(title_bar_rect)
                    .layout(*ui.layout())
                    .id_salt("title_bar"),
            );

            title_bar_ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new(title).color(egui::Color32::GRAY));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });

            // Drag window area
            let title_bar_response = ui.interact(
                title_bar_rect,
                ui.id().with("title_bar_drag"),
                egui::Sense::click_and_drag(),
            );
            if title_bar_response.dragged() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            ui.add_space(title_bar_height);

            // Content
            add_contents(ui);
        });
}
