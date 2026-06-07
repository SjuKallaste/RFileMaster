#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod theme;
mod ui;
mod conversion;

use eframe::NativeOptions;
use egui::ViewportBuilder;

fn load_icon() -> Option<egui::IconData> {
    let icon_path = std::path::Path::new("icon/icon.png");
    if !icon_path.exists() {
        return None;
    }
    let img = image::open(icon_path).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    Some(egui::IconData {
        rgba: img.into_raw(),
        width: w,
        height: h,
    })
}

fn main() -> eframe::Result<()> {
    let mut viewport = ViewportBuilder::default()
        .with_title("RFileMaster")
        .with_inner_size([900.0, 620.0])
        .with_min_inner_size([700.0, 480.0]);

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    let options = NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "RFileMaster",
        options,
        Box::new(|cc| Box::new(app::TransmogrifyApp::new(cc))),
    )
}
