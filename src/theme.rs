use egui::{Color32, FontId, FontFamily, Rounding, Stroke, Style, Visuals};

pub const BASE: Color32 = Color32::from_rgb(0x78, 0x72, 0x76);
pub const BASE_DARK: Color32 = Color32::from_rgb(0x38, 0x35, 0x38);
pub const BASE_DARKER: Color32 = Color32::from_rgb(0x1e, 0x1c, 0x1f);
pub const BASE_LIGHT: Color32 = Color32::from_rgb(0xa8, 0xa2, 0xa5);
pub const BASE_LIGHTER: Color32 = Color32::from_rgb(0xd4, 0xd0, 0xd2);
pub const TITLEBAR: Color32 = Color32::from_rgb(0x10, 0x0e, 0x13);
pub const ACCENT: Color32 = Color32::from_rgb(0x9b, 0x6e, 0xd8);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(0x4e, 0x2d, 0x7a);
pub const ACCENT_BRIGHT: Color32 = Color32::from_rgb(0xd4, 0xb4, 0xff);
pub const SURFACE: Color32 = Color32::from_rgb(0x28, 0x25, 0x2c);
pub const SURFACE_RAISED: Color32 = Color32::from_rgb(0x34, 0x30, 0x3a);
pub const SURFACE_HIGH: Color32 = Color32::from_rgb(0x42, 0x3d, 0x4a);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xf5, 0xf2, 0xff);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0xc0, 0xba, 0xcc);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x78, 0x72, 0x84);
pub const SUCCESS: Color32 = Color32::from_rgb(0x5e, 0xc4, 0x8a);
pub const ERROR: Color32 = Color32::from_rgb(0xe0, 0x60, 0x60);
pub const WARN: Color32 = Color32::from_rgb(0xe0, 0xb8, 0x50);

pub const ROUNDING_NONE: Rounding = Rounding::ZERO;
pub const ROUNDING_SM: Rounding = Rounding::same(3.0);
pub const ROUNDING_MD: Rounding = Rounding::same(5.0);
pub const ROUNDING_LG: Rounding = Rounding::same(7.0);

pub fn apply(ctx: &egui::Context) {
    let mut style = Style::default();
    let mut visuals = Visuals::dark();

    visuals.window_fill = SURFACE;
    visuals.panel_fill = BASE_DARKER;
    visuals.faint_bg_color = SURFACE;
    visuals.extreme_bg_color = Color32::from_rgb(0x18, 0x16, 0x1c);

    visuals.widgets.noninteractive.bg_fill = SURFACE_RAISED;
    visuals.widgets.noninteractive.weak_bg_fill = SURFACE;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BASE_DARK);
    visuals.widgets.noninteractive.rounding = ROUNDING_SM;

    visuals.widgets.inactive.bg_fill = SURFACE_RAISED;
    visuals.widgets.inactive.weak_bg_fill = SURFACE;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BASE);
    visuals.widgets.inactive.rounding = ROUNDING_SM;

    visuals.widgets.hovered.bg_fill = SURFACE_HIGH;
    visuals.widgets.hovered.weak_bg_fill = SURFACE_RAISED;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, TEXT_PRIMARY);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, ACCENT);
    visuals.widgets.hovered.rounding = ROUNDING_SM;

    visuals.widgets.active.bg_fill = ACCENT_DIM;
    visuals.widgets.active.weak_bg_fill = SURFACE_HIGH;
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, TEXT_PRIMARY);
    visuals.widgets.active.bg_stroke = Stroke::new(2.0, ACCENT);
    visuals.widgets.active.rounding = ROUNDING_SM;

    visuals.widgets.open.bg_fill = SURFACE_HIGH;
    visuals.widgets.open.fg_stroke = Stroke::new(1.5, ACCENT);
    visuals.widgets.open.bg_stroke = Stroke::new(1.5, ACCENT);
    visuals.widgets.open.rounding = ROUNDING_SM;

    visuals.selection.bg_fill = ACCENT_DIM;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.window_rounding = ROUNDING_MD;
    visuals.window_stroke = Stroke::new(1.0, BASE_DARK);
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

pub fn mono_font() -> FontId {
    FontId::new(12.0, FontFamily::Monospace)
}
