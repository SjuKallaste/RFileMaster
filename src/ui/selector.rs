use egui::{Color32, ComboBox, FontId, RichText, Sense, Stroke, Ui, Vec2};
use crate::conversion::formats::{FileFormat, FormatCategory, FormatRegistry};
use crate::theme;
use crate::ui::widgets;

pub struct SelectorState {
    pub active_category: Option<FormatCategory>,
    pub source_format: Option<String>,
    pub target_format: Option<String>,
    pub input_paths: Vec<std::path::PathBuf>,
    pub output_dir: Option<std::path::PathBuf>,
    pub merge: bool,
    pub use_youtube: bool,
    pub youtube_url: String,
}

impl SelectorState {
    pub fn new() -> Self {
        Self {
            active_category: None,
            source_format: None,
            target_format: None,
            input_paths: Vec::new(),
            output_dir: None,
            merge: false,
            use_youtube: false,
            youtube_url: String::new(),
        }
    }

    pub fn reset_formats(&mut self) {
        self.source_format = None;
        self.target_format = None;
        self.input_paths.clear();
        self.merge = false;
        self.use_youtube = false;
        self.youtube_url.clear();
    }

    pub fn supports_merge(&self) -> bool {
        match (&self.source_format, &self.target_format) {
            (Some(s), Some(t)) => crate::conversion::engine::supports_merge(s, t),
            _ => false,
        }
    }

    pub fn allows_youtube(&self) -> bool {
        matches!(self.active_category, Some(FormatCategory::Audio) | Some(FormatCategory::Video))
    }

    pub fn is_ready(&self) -> bool {
        if self.use_youtube {
            self.target_format.is_some() && !self.youtube_url.trim().is_empty()
        } else {
            self.source_format.is_some() && self.target_format.is_some() && !self.input_paths.is_empty()
        }
    }

    pub fn detect_format_from_paths(&mut self, registry: &FormatRegistry) {
        let ext = self.input_paths.first()
            .and_then(|path| path.extension())
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        let ext = match ext {
            Some(e) => e,
            None => return,
        };
        let fmt = registry.find(&ext).or_else(|| {
            if ext == "jpeg" { registry.find("jpg") } else { None }
        });
        if let Some(fmt) = fmt {
            if self.source_format.as_deref() != Some(fmt.id) {
                self.source_format = Some(fmt.id.to_string());
                self.target_format = None;
                self.active_category = Some(fmt.category.clone());
            }
        }
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
                Self::draw_file_row(ui, state, registry, main_w);
            });

            ui.add_space(20.0);
        });
    }

    fn draw_category_sidebar(ui: &mut Ui, state: &mut SelectorState, _registry: &FormatRegistry, w: f32) {
        widgets::section_label(ui, "Category");
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
        let p = theme::p();
        let size = Vec2::new(width, 30.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        if ui.is_rect_visible(rect) {
            let (bg, fg) = if selected {
                (p.accent, p.text_primary)
            } else if response.hovered() {
                (p.surface_high, p.text_primary)
            } else {
                (Color32::TRANSPARENT, p.text_secondary)
            };
            ui.painter().rect(rect, theme::ROUNDING_SM, bg, Stroke::NONE);
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
        let p = theme::p();
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
                ui.label(RichText::new("From").font(theme::label_font()).color(p.text_primary));
                ui.add_space(6.0);
                ui.add_enabled_ui(!state.use_youtube, |ui| {
                    ComboBox::from_id_source("source_fmt")
                        .selected_text(
                            state.source_format.as_deref()
                                .and_then(|id| registry.find(id))
                                .map(|f| f.display())
                                .unwrap_or_else(|| if state.use_youtube { "YouTube (detected automatically)".to_string() } else { "Select source format".to_string() }),
                        )
                        .width(col_w)
                        .show_ui(ui, |ui| {
                            ui.style_mut().visuals.widgets.noninteractive.bg_fill = p.surface_high;
                            for fmt in &formats {
                                let selected = state.source_format.as_deref() == Some(fmt.id);
                                if ui.selectable_label(selected, fmt.display()).clicked() {
                                    if state.source_format.as_deref() != Some(fmt.id) {
                                        state.target_format = None;
                                        state.input_paths.clear();
                                    }
                                    state.source_format = Some(fmt.id.to_string());
                                }
                            }
                        });
                });
            });

            ui.add_space(gap);

            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.label(RichText::new("To").font(theme::label_font()).color(p.text_primary));
                ui.add_space(6.0);

                let targets: Vec<&FileFormat> = if state.use_youtube {
                    if let Some(cat) = &state.active_category {
                        registry.formats_in_category(cat)
                    } else {
                        vec![]
                    }
                } else if let Some(src) = &state.source_format {
                    registry.targets_for(src)
                } else {
                    vec![]
                };

                let placeholder = if !state.use_youtube && state.source_format.is_none() {
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
                                if state.target_format.as_deref() != Some(fmt.id) {
                                    state.input_paths.clear();
                                }
                                state.target_format = Some(fmt.id.to_string());
                            }
                        }
                    });
            });
        });

        if let (Some(tgt_id), true) = (&state.target_format, state.use_youtube) {
            ui.add_space(8.0);
            let tgt_label = registry.find(tgt_id).map(|f| f.label).unwrap_or(tgt_id.as_str());
            ui.label(
                RichText::new(format!("YouTube -> {}", tgt_label))
                    .font(theme::small_font())
                    .color(p.success),
            );
        } else if let (Some(src_id), Some(tgt_id)) = (&state.source_format, &state.target_format) {
            ui.add_space(8.0);
            let src_label = registry.find(src_id).map(|f| f.label).unwrap_or(src_id.as_str());
            let tgt_label = registry.find(tgt_id).map(|f| f.label).unwrap_or(tgt_id.as_str());
            let suffix = if state.supports_merge() { "  -  multiple files supported" } else { "" };
            ui.label(
                RichText::new(format!("{} -> {}{}", src_label, tgt_label, suffix))
                    .font(theme::small_font())
                    .color(p.success),
            );
        }
    }

    fn draw_file_row(ui: &mut Ui, state: &mut SelectorState, registry: &FormatRegistry, main_w: f32) {
        let p = theme::p();
        let gap = 16.0;
        let col_w = (main_w - gap) / 2.0;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(col_w);

                if state.allows_youtube() {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut state.use_youtube,
                                    RichText::new("Download from YouTube URL instead")
                                        .font(theme::small_font())
                                        .color(p.text_secondary),
                        );
                    });
                    ui.add_space(6.0);
                }

                if state.use_youtube {
                    ui.label(RichText::new("YouTube URL").font(theme::label_font()).color(p.text_primary));
                    ui.add_space(6.0);
                    Self::youtube_url_field(ui, state, col_w);
                } else {
                    let multi = state.input_paths.len() > 1 || state.supports_merge();
                    let label = if multi { "Input Files" } else { "Input File" };
                    ui.label(RichText::new(label).font(theme::label_font()).color(p.text_primary));
                    ui.add_space(6.0);
                    Self::drop_zone(ui, state, col_w, registry);
                    if state.supports_merge() && state.input_paths.len() > 1 {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut state.merge,
                                        RichText::new("Merge into one file")
                                            .font(theme::small_font())
                                            .color(p.text_secondary),
                            );
                        });
                    }
                }
            });

            ui.add_space(gap);

            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.label(RichText::new("Output Directory (optional)").font(theme::label_font()).color(p.text_primary));
                ui.add_space(6.0);
                Self::output_dir_picker(ui, state, col_w);
            });
        });
    }

    fn youtube_url_field(ui: &mut Ui, state: &mut SelectorState, width: f32) {
        let p = theme::p();
        let height = 110.0;
        egui::Frame::none()
            .fill(p.surface_raised)
            .stroke(Stroke::new(1.0, p.base))
            .rounding(theme::ROUNDING_SM)
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.set_width(width - 24.0);
                ui.set_height(height - 24.0);
                ui.vertical_centered_justified(|ui| {
                    ui.add_space(height / 2.0 - 30.0);
                    let edit = egui::TextEdit::singleline(&mut state.youtube_url)
                        .hint_text("https://www.youtube.com/watch?v=...")
                        .font(theme::label_font());
                    ui.add(edit);
                });
            });
    }

    fn drop_zone(ui: &mut Ui, state: &mut SelectorState, width: f32, registry: &FormatRegistry) {
        let p = theme::p();
        let has_files = !state.input_paths.is_empty();
        let height = if has_files && state.input_paths.len() > 1 {
            (40.0 + state.input_paths.len() as f32 * 18.0).clamp(110.0, 200.0)
        } else {
            110.0
        };

        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
        let is_drag = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());

        let (bg, stroke_col) = if is_drag {
            (Color32::from_rgba_unmultiplied(p.accent.r(), p.accent.g(), p.accent.b(), 22), p.accent)
        } else if response.hovered() {
            (p.surface_high, p.base_light)
        } else {
            (p.surface_raised, p.base)
        };

        if ui.is_rect_visible(rect) {
            ui.painter().rect(rect, theme::ROUNDING_SM, bg, Stroke::new(1.0, stroke_col));

            if state.input_paths.is_empty() {
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Drop files here or click to browse",
                    FontId::new(12.0, egui::FontFamily::Proportional),
                    p.text_muted,
                );
            } else if state.input_paths.len() == 1 {
                let name = state.input_paths[0].file_name().and_then(|n| n.to_str()).unwrap_or("file selected");
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    name,
                    FontId::new(13.0, egui::FontFamily::Proportional),
                    p.text_primary,
                );
            } else {
                let top = rect.min.y + 10.0;
                ui.painter().text(
                    egui::pos2(rect.min.x + 10.0, top),
                    egui::Align2::LEFT_TOP,
                    format!("{} files selected", state.input_paths.len()),
                    FontId::new(11.0, egui::FontFamily::Proportional),
                    p.accent_bright,
                );
                for (i, path_entry) in state.input_paths.iter().enumerate() {
                    let name = path_entry.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                    let y = top + 20.0 + i as f32 * 18.0;
                    if y + 16.0 > rect.max.y { break; }
                    ui.painter().text(
                        egui::pos2(rect.min.x + 14.0, y),
                        egui::Align2::LEFT_TOP,
                        name,
                        FontId::new(11.0, egui::FontFamily::Proportional),
                        p.text_secondary,
                    );
                }
            }
        }

        if response.clicked() {
            let mut dialog = rfd::FileDialog::new();
            if let Some(src) = &state.source_format {
                dialog = dialog.add_filter(src.to_uppercase().as_str(), &[src.as_str()]);
            }
            if let Some(paths) = dialog.pick_files() {
                state.input_paths = paths;
                state.detect_format_from_paths(registry);
                if state.input_paths.len() > 1 && state.supports_merge() {
                    state.merge = true;
                }
            }
        }

        let dropped = ui.ctx().input(|i| i.raw.dropped_files.clone());
        if !dropped.is_empty() {
            let paths: Vec<_> = dropped.into_iter().filter_map(|f| f.path).collect();
            state.input_paths.extend(paths);
            state.detect_format_from_paths(registry);
            if state.input_paths.len() > 1 && state.supports_merge() {
                state.merge = true;
            }
        }
    }

    fn output_dir_picker(ui: &mut Ui, state: &mut SelectorState, width: f32) {
        let p = theme::p();
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 110.0), Sense::click());

        let (bg, stroke_col) = if response.hovered() {
            (p.surface_high, p.base_light)
        } else {
            (p.surface_raised, p.base)
        };

        if ui.is_rect_visible(rect) {
            ui.painter().rect(rect, theme::ROUNDING_SM, bg, Stroke::new(1.0, stroke_col));
            let text = if let Some(dir) = &state.output_dir {
                dir.to_string_lossy().to_string()
            } else {
                "Same as input (default)".to_string()
            };
            let color = if state.output_dir.is_some() { p.text_primary } else { p.text_muted };
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