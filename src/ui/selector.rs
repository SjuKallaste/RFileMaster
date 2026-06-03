use egui::{Color32, ComboBox, FontId, RichText, Sense, Stroke, Ui, Vec2};
use crate::conversion::formats::{FileFormat, FormatCategory, FormatRegistry};
use crate::theme;
use crate::ui::widgets;

pub struct SelectorState {
    pub active_category: Option<FormatCategory>,
    pub source_format: Option<String>,
    pub target_format: Option<String>,
    pub input_path: Option<std::path::PathBuf>,
    pub output_dir: Option<std::path::PathBuf>,
}

impl SelectorState {
    pub fn new() -> Self {
        Self {
            active_category: None,
            source_format: None,
            target_format: None,
            input_path: None,
            output_dir: None,
        }
    }

    pub fn reset_formats(&mut self) {
        self.source_format = None;
        self.target_format = None;
    }

    pub fn is_ready(&self) -> bool {
        self.source_format.is_some() && self.target_format.is_some() && self.input_path.is_some()
    }
}

pub struct SelectorPanel;

impl SelectorPanel {
    pub fn show(ui: &mut Ui, state: &mut SelectorState, registry: &FormatRegistry) {
        let available = ui.available_size();
        let sidebar_w = (available.x * 0.16).clamp(110.0, 200.0);
        let gutter = 24.0;
        let main_w = available.x - sidebar_w - gutter - 80.0;

        ui.add_space(20.0);

        ui.horizontal(|ui| {
            ui.add_space(20.0);

            ui.vertical(|ui| {
                ui.set_width(sidebar_w);
                Self::draw_category_sidebar(ui, state, registry, sidebar_w);
            });

            ui.add_space(gutter);

            ui.vertical(|ui| {
                ui.set_width(main_w);
                Self::draw_format_row(ui, state, registry, main_w);
                ui.add_space(20.0);
                widgets::divider(ui);
                Self::draw_file_row(ui, state, main_w);
            });

            ui.add_space(20.0);
        });
    }

    fn draw_category_sidebar(ui: &mut Ui, state: &mut SelectorState, _registry: &FormatRegistry, w: f32) {
        widgets::section_label(ui, "CATEGORY");
        ui.add_space(6.0);
        ui.spacing_mut().item_spacing = Vec2::new(0.0, 4.0);
        for cat in FormatCategory::all() {
            let selected = state.active_category.as_ref() == Some(&cat);
            if Self::sidebar_button(ui, cat.label(), selected, w).clicked() {
                if selected {
                    state.active_category = None;
                } else {
                    state.active_category = Some(cat);
                }
                state.reset_formats();
            }
        }
    }

    fn sidebar_button(ui: &mut Ui, label: &str, selected: bool, width: f32) -> egui::Response {
        let size = Vec2::new(width, 30.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        if ui.is_rect_visible(rect) {
            let (bg, fg, stroke) = if selected {
                (theme::ACCENT, theme::TEXT_PRIMARY, Stroke::NONE)
            } else if response.hovered() {
                (theme::SURFACE_HIGH, theme::TEXT_PRIMARY, Stroke::NONE)
            } else {
                (Color32::TRANSPARENT, theme::TEXT_SECONDARY, Stroke::NONE)
            };
            ui.painter().rect(rect, theme::ROUNDING_SM, bg, stroke);
            ui.painter().text(
                egui::pos2(rect.min.x + 10.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                label,
                FontId::new(13.0, egui::FontFamily::Proportional),
                fg,
            );
        }
        response
    }

    fn draw_format_row(ui: &mut Ui, state: &mut SelectorState, registry: &FormatRegistry, main_w: f32) {
        let formats: Vec<&FileFormat> = if let Some(cat) = &state.active_category {
            registry.formats_in_category(cat)
        } else {
            registry.formats.iter().collect()
        };

        let gap = 16.0;
        let col_w = (main_w - gap) / 2.0;

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.label(RichText::new("FROM").font(theme::label_font()).color(theme::TEXT_MUTED));
                ui.add_space(6.0);
                ComboBox::from_id_source("source_fmt")
                    .selected_text(
                        state.source_format.as_deref()
                            .and_then(|id| registry.find(id))
                            .map(|f| f.display())
                            .unwrap_or_else(|| "Select source format".to_string()),
                    )
                    .width(col_w)
                    .show_ui(ui, |ui| {
                        ui.style_mut().visuals.widgets.noninteractive.bg_fill = theme::SURFACE_HIGH;
                        for fmt in &formats {
                            let selected = state.source_format.as_deref() == Some(fmt.id);
                            if ui.selectable_label(selected, fmt.display()).clicked() {
                                if state.source_format.as_deref() != Some(fmt.id) {
                                    state.target_format = None;
                                }
                                state.source_format = Some(fmt.id.to_string());
                            }
                        }
                    });
            });

            ui.add_space(gap);

            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.label(RichText::new("TO").font(theme::label_font()).color(theme::TEXT_MUTED));
                ui.add_space(6.0);

                let targets: Vec<&FileFormat> = if let Some(src) = &state.source_format {
                    registry.targets_for(src)
                } else {
                    vec![]
                };

                let placeholder = if state.source_format.is_none() {
                    "Select source first".to_string()
                } else if targets.is_empty() {
                    "No targets available".to_string()
                } else {
                    state.target_format.as_deref()
                        .and_then(|id| registry.find(id))
                        .map(|f| f.display())
                        .unwrap_or_else(|| "Select target format".to_string())
                };

                ComboBox::from_id_source("target_fmt")
                    .selected_text(placeholder)
                    .width(col_w)
                    .show_ui(ui, |ui| {
                        for fmt in &targets {
                            let selected = state.target_format.as_deref() == Some(fmt.id);
                            if ui.selectable_label(selected, fmt.display()).clicked() {
                                state.target_format = Some(fmt.id.to_string());
                            }
                        }
                    });
            });
        });

        if let (Some(src_id), Some(tgt_id)) = (&state.source_format, &state.target_format) {
            ui.add_space(8.0);
            let src_label = registry.find(src_id).map(|f| f.label).unwrap_or(src_id.as_str());
            let tgt_label = registry.find(tgt_id).map(|f| f.label).unwrap_or(tgt_id.as_str());
            ui.label(
                RichText::new(format!("{} -> {}", src_label, tgt_label))
                    .font(theme::small_font())
                    .color(theme::SUCCESS),
            );
        }
    }

    fn draw_file_row(ui: &mut Ui, state: &mut SelectorState, main_w: f32) {
        let gap = 16.0;
        let col_w = (main_w - gap) / 2.0;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.label(RichText::new("INPUT FILE").font(theme::label_font()).color(theme::TEXT_MUTED));
                ui.add_space(6.0);
                Self::drop_zone(ui, state, col_w);
            });

            ui.add_space(gap);

            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.label(RichText::new("OUTPUT DIRECTORY (optional)").font(theme::label_font()).color(theme::TEXT_MUTED));
                ui.add_space(6.0);
                Self::output_dir_picker(ui, state, col_w);
            });
        });
    }

    fn drop_zone(ui: &mut Ui, state: &mut SelectorState, width: f32) {
        let height = 110.0;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
        let is_drag = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());

        let (bg, stroke_col) = if is_drag {
            (Color32::from_rgba_unmultiplied(0xb0, 0x7e, 0xe8, 18), theme::ACCENT)
        } else if response.hovered() {
            (theme::SURFACE_HIGH, theme::BASE_LIGHT)
        } else {
            (theme::SURFACE_RAISED, theme::BASE)
        };

        if ui.is_rect_visible(rect) {
            ui.painter().rect(rect, theme::ROUNDING_SM, bg, Stroke::new(1.0, stroke_col));
            let (text, color, font_size) = if let Some(p) = &state.input_path {
                (p.file_name().and_then(|n| n.to_str()).unwrap_or("file selected").to_string(), theme::TEXT_PRIMARY, 13.0)
            } else {
                ("Drop file here or click to browse".to_string(), theme::TEXT_MUTED, 13.0)
            };
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                FontId::new(font_size, egui::FontFamily::Proportional),
                color,
            );
        }

        if response.clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                state.input_path = Some(path);
            }
        }

        let dropped = ui.ctx().input(|i| i.raw.dropped_files.clone());
        if !dropped.is_empty() {
            if let Some(first) = dropped.into_iter().next() {
                if let Some(p) = first.path {
                    state.input_path = Some(p);
                }
            }
        }
    }

    fn output_dir_picker(ui: &mut Ui, state: &mut SelectorState, width: f32) {
        let height = 110.0;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());

        let (bg, stroke_col) = if response.hovered() {
            (theme::SURFACE_HIGH, theme::BASE_LIGHT)
        } else {
            (theme::SURFACE_RAISED, theme::BASE)
        };

        if ui.is_rect_visible(rect) {
            ui.painter().rect(rect, theme::ROUNDING_SM, bg, Stroke::new(1.0, stroke_col));
            let text = if let Some(p) = &state.output_dir {
                p.to_string_lossy().to_string()
            } else {
                "Same as input (default)".to_string()
            };
            let color = if state.output_dir.is_some() { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED };
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                FontId::new(13.0, egui::FontFamily::Proportional),
                color,
            );
        }

        if response.clicked() {
            if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                state.output_dir = Some(dir);
            }
        }
    }
}