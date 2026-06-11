use egui::{Frame, Margin, ProgressBar, RichText, Stroke, Ui, Vec2};
use crate::conversion::job::JobStatus;
use crate::conversion::runner::ConversionRunner;
use crate::theme;
use crate::ui::widgets;

pub struct QueuePanel;

impl QueuePanel {
    pub fn show(ui: &mut Ui, runner: &ConversionRunner) {
        let jobs = runner.jobs.lock().unwrap().clone();

        ui.add_space(20.0);

        let available_w = ui.available_width();
        let panel_w = (available_w - 80.0).min(720.0).max(400.0);

        ui.allocate_ui_with_layout(
            Vec2::new(available_w, ui.available_height()),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.set_width(panel_w);

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Conversion Queue")
                            .font(theme::heading_font())
                            .color(theme::p().text_primary),
                    );
                    ui.add_space(8.0);
                    if !jobs.is_empty() {
                        widgets::status_badge(ui, &format!("{} jobs", jobs.len()), theme::p().accent);
                    }
                });

                ui.add_space(12.0);

                if jobs.is_empty() {
                    Frame::none()
                        .fill(theme::p().surface)
                        .rounding(theme::ROUNDING_LG)
                        .stroke(Stroke::new(1.0, theme::p().base_dark))
                        .inner_margin(Margin::same(32.0))
                        .show(ui, |ui| {
                            ui.set_width(panel_w - 2.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new("No conversions in progress")
                                        .color(theme::p().text_muted)
                                        .font(theme::label_font()),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new("Set up a conversion on the Convert tab and press Convert.")
                                        .color(theme::p().text_muted)
                                        .font(theme::small_font()),
                                );
                            });
                        });
                    return;
                }

                let mut to_remove: Vec<u64> = vec![];

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(panel_w);
                    for job in &jobs {
                        Frame::none()
                            .fill(theme::p().surface)
                            .rounding(theme::ROUNDING_MD)
                            .stroke(Stroke::new(1.0, theme::p().base_dark))
                            .inner_margin(Margin::symmetric(16.0, 12.0))
                            .show(ui, |ui| {
                                ui.set_width(panel_w - 2.0);

                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.set_min_width(panel_w - 120.0);
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(job.display_name())
                                                    .font(theme::label_font())
                                                    .color(theme::p().text_primary),
                                            );
                                            ui.add_space(6.0);
                                            ui.label(
                                                RichText::new(format!(
                                                    "{} -> {}",
                                                    job.source_format.to_uppercase(),
                                                    job.target_format.to_uppercase()
                                                ))
                                                    .font(theme::small_font())
                                                    .color(theme::p().text_muted),
                                            );
                                        });

                                        ui.add_space(6.0);

                                        match &job.status {
                                            JobStatus::Queued => {
                                                ui.label(
                                                    RichText::new("Waiting...")
                                                        .font(theme::small_font())
                                                        .color(theme::p().text_muted),
                                                );
                                            }
                                            JobStatus::Running(progress) => {
                                                ui.add(
                                                    ProgressBar::new(*progress)
                                                        .fill(theme::p().accent)
                                                        .desired_width(panel_w - 160.0)
                                                        .animate(true),
                                                );
                                            }
                                            JobStatus::Done(out) => {
                                                ui.horizontal(|ui| {
                                                    widgets::status_badge(ui, "Done", theme::p().success);
                                                    ui.add_space(6.0);
                                                    ui.label(
                                                        RichText::new(
                                                            out.file_name()
                                                                .and_then(|n| n.to_str())
                                                                .unwrap_or("output"),
                                                        )
                                                            .font(theme::small_font())
                                                            .color(theme::p().text_secondary),
                                                    );
                                                });
                                            }                                            JobStatus::Failed(e) => {
                                                ui.horizontal(|ui| {
                                                    widgets::status_badge(ui, "Failed", theme::p().error);
                                                    ui.add_space(6.0);
                                                    ui.label(
                                                        RichText::new(e.as_str())
                                                            .font(theme::small_font())
                                                            .color(theme::p().error),
                                                    );
                                                });
                                            }
                                        }
                                    });

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if widgets::ghost_button(ui, "Remove").clicked() {
                                            to_remove.push(job.id);
                                        }
                                    });
                                });
                            });
                        ui.add_space(8.0);
                    }
                });

                ui.add_space(8.0);
                if jobs.iter().any(|j| j.status.is_terminal()) {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::ghost_button(ui, "Clear finished").clicked() {
                            runner.clear_finished();
                        }
                    });
                }

                drop(jobs);
                for id in to_remove {
                    runner.remove(id);
                }
            },
        );
    }
}