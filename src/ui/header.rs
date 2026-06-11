use egui::{RichText, Ui};
use crate::theme;

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Convert,
    Queue,
    Settings,
}

pub struct HeaderBar;

impl HeaderBar {
    pub fn show(ui: &mut Ui, active_tab: &mut Tab, job_count: usize) {
        let p = theme::p();
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(
                RichText::new("RFileMaster")
                    .font(theme::heading_font())
                    .color(p.accent_bright)
                    .strong(),
            );

            ui.add_space(ui.available_width() - 280.0);

            let tabs = [
                (Tab::Convert, "Convert"),
                (Tab::Queue, "Queue"),
                (Tab::Settings, "Settings"),
            ];

            for (tab, label) in &tabs {
                let is_active = active_tab == tab;
                let display = if matches!(tab, Tab::Queue) && job_count > 0 {
                    format!("{} ({})", label, job_count)
                } else {
                    label.to_string()
                };

                let text = RichText::new(display)
                    .font(theme::label_font())
                    .color(if is_active { p.accent_bright } else { p.text_secondary });

                let response = ui.selectable_label(is_active, text);
                if response.clicked() {
                    *active_tab = tab.clone();
                }
                ui.add_space(8.0);
            }
            ui.add_space(16.0);
        });
    }
}