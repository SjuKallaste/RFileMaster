use eframe::CreationContext;
use egui::{CentralPanel, Frame, Margin, TopBottomPanel};
use crate::conversion::formats::{init_registry, FormatRegistry, REGISTRY};
use crate::conversion::runner::ConversionRunner;
use crate::theme;
use crate::ui::header::{HeaderBar, Tab};
use crate::ui::queue::QueuePanel;
use crate::ui::selector::{SelectorPanel, SelectorState};
use crate::ui::settings::{AppSettings, SettingsPanel};
use crate::ui::widgets;

pub struct TransmogrifyApp {
    active_tab: Tab,
    selector: SelectorState,
    runner: ConversionRunner,
    settings: AppSettings,
    registry: &'static FormatRegistry,
}

impl TransmogrifyApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);
        let registry = REGISTRY.get_or_init(init_registry);
        Self {
            active_tab: Tab::Convert,
            selector: SelectorState::new(),
            runner: ConversionRunner::new(),
            settings: AppSettings::default(),
            registry,
        }
    }
}

impl eframe::App for TransmogrifyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.runner.tick();
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        let job_count = self.runner.jobs.lock().unwrap().len();

        TopBottomPanel::top("header")
            .frame(
                Frame::none()
                    .fill(theme::TITLEBAR)
                    .inner_margin(Margin::symmetric(0.0, 14.0)),
            )
            .show(ctx, |ui| {
                HeaderBar::show(ui, &mut self.active_tab, job_count);
            });

        if self.active_tab == Tab::Convert {
            let mut convert_clicked = false;
            TopBottomPanel::bottom("action_bar")
                .frame(
                    Frame::none()
                        .fill(theme::BASE_DARKER)
                        .inner_margin(Margin::symmetric(32.0, 16.0)),
                )
                .show(ctx, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let ready = self.selector.is_ready();
                        if widgets::accent_button(ui, "Convert", ready).clicked() && ready {
                            convert_clicked = true;
                        }
                        ui.add_space(8.0);
                        if widgets::ghost_button(ui, "Clear").clicked() {
                            self.selector = SelectorState::new();
                        }
                    });
                });

            if convert_clicked {
                if let (Some(src), Some(tgt), Some(input)) = (
                    self.selector.source_format.clone(),
                    self.selector.target_format.clone(),
                    self.selector.input_path.clone(),
                ) {
                    let out_dir = self.selector.output_dir.clone()
                        .or_else(|| self.settings.default_output_dir.clone())
                        .or_else(|| input.parent().map(|p| p.to_path_buf()));
                    self.runner.enqueue(input, src, tgt, out_dir);
                    self.active_tab = Tab::Queue;
                }
            }
        }

        CentralPanel::default()
            .frame(
                Frame::none()
                    .fill(theme::BASE_DARKER)
                    .inner_margin(Margin::same(0.0)),
            )
            .show(ctx, |ui| {
                match self.active_tab {
                    Tab::Convert => {
                        SelectorPanel::show(ui, &mut self.selector, self.registry);
                    }
                    Tab::Queue => {
                        QueuePanel::show(ui, &self.runner);
                    }
                    Tab::Settings => {
                        SettingsPanel::show(ui, &mut self.settings);
                    }
                }
            });
    }
}
