use egui::{Color32, FontId, Response, RichText, Sense, Stroke, Ui, Vec2};
use crate::theme;

pub fn section_label(ui: &mut Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(text).font(theme::small_font()).color(theme::TEXT_MUTED));
    ui.add_space(2.0);
}

pub fn accent_button(ui: &mut Ui, label: &str, enabled: bool) -> Response {
    let size = Vec2::new(ui.available_width().min(200.0), 34.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let (bg, fg, stroke_col) = if !enabled {
            (theme::SURFACE, theme::TEXT_MUTED, theme::BASE_DARK)
        } else if response.is_pointer_button_down_on() {
            (theme::ACCENT_DIM, theme::ACCENT_BRIGHT, theme::ACCENT)
        } else if response.hovered() {
            (theme::ACCENT, theme::BASE_DARKER, theme::ACCENT)
        } else {
            (theme::ACCENT_DIM, theme::ACCENT_BRIGHT, theme::ACCENT)
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
    let size = Vec2::new(ui.available_width().min(160.0), 28.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let (bg, fg) = if response.hovered() {
            (theme::SURFACE_RAISED, theme::TEXT_PRIMARY)
        } else {
            (Color32::TRANSPARENT, theme::TEXT_SECONDARY)
        };
        ui.painter().rect(rect, theme::ROUNDING_SM, bg, Stroke::new(1.0, theme::BASE));
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
    ui.add_space(6.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter().hline(rect.x_range(), rect.center().y, Stroke::new(1.0, theme::SURFACE_RAISED));
    ui.add_space(6.0);
}

pub fn format_chip(ui: &mut Ui, label: &str, selected: bool) -> Response {
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        FontId::new(12.0, egui::FontFamily::Proportional),
        Color32::WHITE,
    );
    let pad = Vec2::new(10.0, 4.0);
    let size = galley.size() + pad * 2.0;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let (bg, fg, stroke) = if selected {
            (theme::ACCENT_DIM, theme::ACCENT_BRIGHT, Stroke::new(1.5, theme::ACCENT))
        } else if response.hovered() {
            (theme::SURFACE_HIGH, theme::TEXT_PRIMARY, Stroke::new(1.0, theme::BASE_LIGHT))
        } else {
            (theme::SURFACE_RAISED, theme::TEXT_SECONDARY, Stroke::new(1.0, theme::BASE))
        };
        ui.painter().rect(rect, theme::ROUNDING_SM, bg, stroke);
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

pub fn arrow_divider(ui: &mut Ui) {
    ui.add_space(4.0);
    let size = Vec2::new(48.0, 24.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(rect) {
        let c = rect.center();
        let stroke = Stroke::new(2.0, theme::ACCENT);
        ui.painter().arrow(egui::pos2(c.x - 16.0, c.y), Vec2::new(32.0, 0.0), stroke);
    }
    ui.add_space(4.0);
}
