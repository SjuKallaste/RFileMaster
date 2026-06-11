use egui::{Color32, FontId, Response, RichText, Sense, Stroke, Ui, Vec2};
use crate::theme;

pub fn section_label(ui: &mut Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(text).font(theme::label_font()).color(theme::p().text_primary));
    ui.add_space(2.0);
}

pub fn accent_button(ui: &mut Ui, label: &str, enabled: bool) -> Response {
    let p = theme::p();
    let size = Vec2::new(ui.available_width().min(200.0), 34.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let (bg, fg, stroke_col) = if !enabled {
            (p.surface, p.text_muted, p.base_dark)
        } else if response.is_pointer_button_down_on() {
            (p.accent_dim, p.accent_bright, p.accent)
        } else if response.hovered() {
            (p.accent, p.base_darker, p.accent)
        } else {
            (p.accent_dim, p.accent_bright, p.accent)
        };
        ui.painter().rect(rect, theme::ROUNDING_SM, bg, Stroke::new(1.5, stroke_col));
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            FontId::new(13.0, egui::FontFamily::Proportional),
            fg,
        );
    }
    if enabled { response } else { response.clone() }
}

pub fn ghost_button(ui: &mut Ui, label: &str) -> Response {
    let p = theme::p();
    let size = Vec2::new(ui.available_width().min(160.0), 28.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let (bg, fg) = if response.hovered() {
            (p.surface_raised, p.text_primary)
        } else {
            (Color32::TRANSPARENT, p.text_secondary)
        };
        ui.painter().rect(rect, theme::ROUNDING_SM, bg, Stroke::new(1.0, p.base));
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            FontId::new(12.0, egui::FontFamily::Proportional),
            fg,
        );
    }
    response
}

pub fn status_badge(ui: &mut Ui, label: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        FontId::new(10.0, egui::FontFamily::Proportional),
        color,
    );
    let pad = Vec2::new(7.0, 3.0);
    let size = galley.size() + pad * 2.0;
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(rect) {
        let bg = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 30);
        ui.painter().rect(rect, theme::ROUNDING_SM, bg, Stroke::new(1.0, color));
        ui.painter().galley(rect.min + pad, galley, color);
    }
}

pub fn divider(ui: &mut Ui) {
    let p = theme::p();
    ui.add_space(6.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter().hline(rect.x_range(), rect.center().y, Stroke::new(1.0, p.surface_raised));
    ui.add_space(6.0);
}