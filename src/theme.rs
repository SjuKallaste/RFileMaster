use egui::{Color32, FontId, FontFamily, Rounding, Stroke, Style, Visuals};

pub const ROUNDING_SM: Rounding = Rounding::same(3.0);
pub const ROUNDING_MD: Rounding = Rounding::same(5.0);
pub const ROUNDING_LG: Rounding = Rounding::same(7.0);

#[derive(Clone)]
pub struct Palette {
    pub base: Color32,
    pub base_dark: Color32,
    pub base_darker: Color32,
    pub base_light: Color32,
    pub titlebar: Color32,
    pub accent: Color32,
    pub accent_dim: Color32,
    pub accent_bright: Color32,
    pub surface: Color32,
    pub surface_raised: Color32,
    pub surface_high: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub success: Color32,
    pub error: Color32,
}

pub fn dark() -> Palette {
    Palette {
        base: Color32::from_rgb(0x78, 0x72, 0x76),
        base_dark: Color32::from_rgb(0x38, 0x35, 0x38),
        base_darker: Color32::from_rgb(0x1e, 0x1c, 0x1f),
        base_light: Color32::from_rgb(0xa8, 0xa2, 0xa5),
        titlebar: Color32::from_rgb(0x10, 0x0e, 0x13),
        accent: Color32::from_rgb(0x9b, 0x6e, 0xd8),
        accent_dim: Color32::from_rgb(0x4e, 0x2d, 0x7a),
        accent_bright: Color32::from_rgb(0xd4, 0xb4, 0xff),
        surface: Color32::from_rgb(0x28, 0x25, 0x2c),
        surface_raised: Color32::from_rgb(0x34, 0x30, 0x3a),
        surface_high: Color32::from_rgb(0x42, 0x3d, 0x4a),
        text_primary: Color32::from_rgb(0xf5, 0xf2, 0xff),
        text_secondary: Color32::from_rgb(0xc0, 0xba, 0xcc),
        text_muted: Color32::from_rgb(0x78, 0x72, 0x84),
        success: Color32::from_rgb(0x5e, 0xc4, 0x8a),
        error: Color32::from_rgb(0xe0, 0x60, 0x60),
    }
}

pub fn light() -> Palette {
    Palette {
        base: Color32::from_rgb(0xd4, 0xb8, 0xc8),
        base_dark: Color32::from_rgb(0xe8, 0xd4, 0xe0),
        base_darker: Color32::from_rgb(0xf8, 0xf0, 0xf4),
        base_light: Color32::from_rgb(0xb8, 0x98, 0xac),
        titlebar: Color32::from_rgb(0xf0, 0xe0, 0xea),
        accent: Color32::from_rgb(0xd4, 0x60, 0x9a),
        accent_dim: Color32::from_rgb(0xf2, 0xc0, 0xd8),
        accent_bright: Color32::from_rgb(0xa0, 0x30, 0x68),
        surface: Color32::from_rgb(0xff, 0xf8, 0xfb),
        surface_raised: Color32::from_rgb(0xf8, 0xec, 0xf2),
        surface_high: Color32::from_rgb(0xee, 0xdc, 0xe6),
        text_primary: Color32::from_rgb(0x1e, 0x10, 0x18),
        text_secondary: Color32::from_rgb(0x50, 0x38, 0x46),
        text_muted: Color32::from_rgb(0x90, 0x70, 0x80),
        success: Color32::from_rgb(0x28, 0x8a, 0x50),
        error: Color32::from_rgb(0xc0, 0x28, 0x28),
    }
}

pub fn apply(ctx: &egui::Context, theme: &Theme) {
    let p = match theme {
        Theme::Dark => dark(),
        Theme::Light => light(),
        Theme::HighContrast => high_contrast(),
    };
    let mut style = Style::default();
    let mut visuals = if matches!(theme, Theme::Light) { Visuals::light() } else { Visuals::dark() };

    visuals.window_fill = p.surface;
    visuals.panel_fill = p.base_darker;
    visuals.faint_bg_color = p.surface;
    visuals.extreme_bg_color = if matches!(theme, Theme::Light) {
        Color32::from_rgb(0xff, 0xff, 0xff)
    } else if matches!(theme, Theme::HighContrast) {
        Color32::from_rgb(0x00, 0x00, 0x00)
    } else {
        Color32::from_rgb(0x18, 0x16, 0x1c)
    };

    let border_w = if matches!(theme, Theme::HighContrast) { 1.5 } else { 1.0 };
    let inactive_fill = if matches!(theme, Theme::HighContrast) { Color32::TRANSPARENT } else { p.surface_raised };

    visuals.widgets.noninteractive.bg_fill = inactive_fill;
    visuals.widgets.noninteractive.weak_bg_fill = p.surface;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(border_w, p.text_secondary);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(border_w, p.base_dark);
    visuals.widgets.noninteractive.rounding = ROUNDING_SM;

    visuals.widgets.inactive.bg_fill = inactive_fill;
    visuals.widgets.inactive.weak_bg_fill = p.surface;
    visuals.widgets.inactive.fg_stroke = Stroke::new(border_w, p.text_secondary);
    visuals.widgets.inactive.bg_stroke = Stroke::new(border_w, p.base);
    visuals.widgets.inactive.rounding = ROUNDING_SM;

    visuals.widgets.hovered.bg_fill = p.surface_high;
    visuals.widgets.hovered.weak_bg_fill = p.surface_raised;
    visuals.widgets.hovered.fg_stroke = Stroke::new(border_w + 0.5, p.text_primary);
    visuals.widgets.hovered.bg_stroke = Stroke::new(border_w + 0.5, p.accent);
    visuals.widgets.hovered.rounding = ROUNDING_SM;

    visuals.widgets.active.bg_fill = p.accent_dim;
    visuals.widgets.active.weak_bg_fill = p.surface_high;
    visuals.widgets.active.fg_stroke = Stroke::new(border_w + 1.0, p.text_primary);
    visuals.widgets.active.bg_stroke = Stroke::new(border_w + 1.0, p.accent);
    visuals.widgets.active.rounding = ROUNDING_SM;

    visuals.widgets.open.bg_fill = p.surface_high;
    visuals.widgets.open.fg_stroke = Stroke::new(border_w, p.accent);
    visuals.widgets.open.bg_stroke = Stroke::new(border_w, p.accent);
    visuals.widgets.open.rounding = ROUNDING_SM;

    visuals.selection.bg_fill = p.accent_dim;
    visuals.selection.stroke = Stroke::new(1.0, p.accent);
    visuals.window_rounding = ROUNDING_MD;
    visuals.window_stroke = Stroke::new(if matches!(theme, Theme::HighContrast) { 2.0 } else { 1.0 }, p.base_dark);
    visuals.popup_shadow = egui::epaint::Shadow::NONE;
    visuals.window_shadow = egui::epaint::Shadow::NONE;

    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(14.0, 7.0);
    style.spacing.menu_margin = egui::Margin::same(4.0);
    style.spacing.combo_height = 300.0;

    ctx.set_style(style);
}

pub fn heading_font() -> FontId {
    FontId::new(20.0, FontFamily::Proportional)
}

pub fn label_font() -> FontId {
    FontId::new(13.0, FontFamily::Proportional)
}

pub fn small_font() -> FontId {
    FontId::new(11.0, FontFamily::Proportional)
}

pub fn high_contrast() -> Palette {
    Palette {
        base: Color32::from_rgb(0xff, 0xff, 0xff),
        base_dark: Color32::from_rgb(0x6f, 0x6f, 0x6f),
        base_darker: Color32::from_rgb(0x00, 0x00, 0x00),
        base_light: Color32::from_rgb(0xff, 0xff, 0xff),
        titlebar: Color32::from_rgb(0x00, 0x00, 0x00),
        accent: Color32::from_rgb(0x00, 0xb7, 0xff),
        accent_dim: Color32::from_rgb(0x00, 0x40, 0x6f),
        accent_bright: Color32::from_rgb(0x00, 0xb7, 0xff),
        surface: Color32::from_rgb(0x00, 0x00, 0x00),
        surface_raised: Color32::from_rgb(0x0a, 0x0a, 0x0a),
        surface_high: Color32::from_rgb(0x1a, 0x1a, 0x1a),
        text_primary: Color32::from_rgb(0xff, 0xff, 0xff),
        text_secondary: Color32::from_rgb(0xcc, 0xcc, 0xcc),
        text_muted: Color32::from_rgb(0x99, 0x99, 0x99),
        success: Color32::from_rgb(0x4e, 0xc9, 0x94),
        error: Color32::from_rgb(0xf4, 0x85, 0x71),
    }
}

use crate::ui::settings::Theme;

thread_local! {
    static CURRENT: std::cell::RefCell<Palette> = std::cell::RefCell::new(dark());
}

pub fn set(theme: &Theme) {
    CURRENT.with(|c| *c.borrow_mut() = match theme {
        Theme::Dark => dark(),
        Theme::Light => light(),
        Theme::HighContrast => high_contrast(),
    });
}

pub fn p() -> Palette {
    CURRENT.with(|c| c.borrow().clone())
}