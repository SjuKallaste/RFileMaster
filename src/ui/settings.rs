use egui::{ComboBox, Frame, Margin, RichText, Stroke, Ui, Vec2};
use crate::theme;
use crate::ui::widgets;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    HighContrast,
}

impl Theme {
    pub fn label(&self) -> &'static str {
        match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
            Theme::HighContrast => "High Contrast",
        }
    }
    pub fn all() -> [Theme; 3] {
        [Theme::Dark, Theme::Light, Theme::HighContrast]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub auto_open_output: bool,
    pub overwrite_existing: bool,
    pub default_output_dir: Option<std::path::PathBuf>,
    pub max_concurrent_jobs: usize,
    pub theme: Theme,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_open_output: false,
            overwrite_existing: false,
            default_output_dir: None,
            max_concurrent_jobs: (std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4) / 2).max(1),
            theme: detect_system_theme(),
        }
    }
}

pub fn detect_system_theme() -> Theme {
    match dark_light::detect() {
        dark_light::Mode::Dark | dark_light::Mode::Default => Theme::Dark,
        dark_light::Mode::Light => Theme::Light,
    }
}

pub struct SettingsPanel;

impl SettingsPanel {
    pub fn show(ui: &mut Ui, settings: &mut AppSettings) {
        ui.add_space(20.0);
        let available_w = ui.available_width();
        let panel_w = (available_w - 80.0).min(720.0).max(400.0);

        ui.allocate_ui_with_layout(
            Vec2::new(available_w, ui.available_height()),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.set_width(panel_w);

                ui.label(
                    RichText::new("Settings")
                        .font(theme::heading_font())
                        .color(theme::p().text_primary),
                );
                ui.add_space(16.0);

                Frame::none()
                    .fill(theme::p().surface)
                    .rounding(theme::ROUNDING_MD)
                    .stroke(Stroke::new(1.0, theme::p().base_dark))
                    .inner_margin(Margin::same(20.0))
                    .show(ui, |ui| {
                        ui.set_width(panel_w - 2.0);

                        widgets::section_label(ui, "Appearance");
                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Theme").font(theme::label_font()).color(theme::p().text_secondary));
                            ui.add_space(8.0);
                            ComboBox::from_id_source("theme_select")
                                .selected_text(settings.theme.label())
                                .width(160.0)
                                .show_ui(ui, |ui| {
                                    for t in Theme::all() {
                                        let selected = settings.theme == t;
                                        if ui.selectable_label(selected, t.label()).clicked() {
                                            settings.theme = t;
                                        }
                                    }
                                });
                        });

                        widgets::divider(ui);
                        widgets::section_label(ui, "Output behaviour");
                        ui.add_space(4.0);

                        ui.checkbox(
                            &mut settings.auto_open_output,
                            RichText::new("Open output folder when conversion finishes")
                                .font(theme::label_font())
                                .color(theme::p().text_primary),
                        );
                        ui.add_space(4.0);
                        ui.checkbox(
                            &mut settings.overwrite_existing,
                            RichText::new("Overwrite existing files without prompting")
                                .font(theme::label_font())
                                .color(theme::p().text_primary),
                        );

                        widgets::divider(ui);
                        widgets::section_label(ui, "Default output directory");
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            let dir_label = settings.default_output_dir
                                .as_ref()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Same directory as input file".to_string());

                            ui.label(
                                RichText::new(dir_label)
                                    .font(theme::label_font())
                                    .color(if settings.default_output_dir.is_some() {
                                        theme::p().text_primary
                                    } else {
                                        theme::p().text_muted
                                    }),
                            );
                            ui.add_space(8.0);
                            if widgets::ghost_button(ui, "Change").clicked() {
                                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                                    settings.default_output_dir = Some(dir);
                                }
                            }
                            if settings.default_output_dir.is_some() {
                                if widgets::ghost_button(ui, "Clear").clicked() {
                                    settings.default_output_dir = None;
                                }
                            }
                        });

                        widgets::divider(ui);
                        widgets::section_label(ui, "Concurrency");
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Max simultaneous conversions:")
                                    .font(theme::label_font())
                                    .color(theme::p().text_secondary),
                            );
                            ui.add_space(8.0);
                            let mut val = settings.max_concurrent_jobs as u32;
                            if ui.add(egui::DragValue::new(&mut val).clamp_range(1u32..=8u32)).changed() {
                                settings.max_concurrent_jobs = val as usize;
                            }
                        });
                    });
            },
        );
    }
}