mod app;
mod theme;
mod ui;
mod conversion;

use eframe::NativeOptions;
use egui::ViewportBuilder;

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("RFileMaster")
            .with_inner_size([900.0, 620.0])
            .with_min_inner_size([700.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "RFileMaster",
        options,
        Box::new(|cc| Box::new(app::TransmogrifyApp::new(cc))),
    )
}
