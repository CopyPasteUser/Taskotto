//! Standalone Taskotto views inspired by two historical task-manager design
//! languages. They share the existing procfs sampler, process actions, and
//! settings; only drawing/navigation is specialized.

mod chrome;
mod performance;
mod process;

use super::*;

pub(super) fn supports(theme: Theme) -> bool {
    matches!(theme, Theme::XpLuna | Theme::Windows10)
}

pub(super) fn topbar(app: &mut TaskottoApp, ctx: &egui::Context) {
    chrome::topbar(app, ctx);
}

pub(super) fn navigation(app: &mut TaskottoApp, ctx: &egui::Context) {
    chrome::navigation(app, ctx);
}

pub(super) fn statusbar(app: &mut TaskottoApp, ctx: &egui::Context) {
    chrome::statusbar(app, ctx);
}

pub(super) fn processes(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    process::page(app, ui);
}

pub(super) fn performance(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    performance::page(app, ui);
}

fn is_xp(app: &TaskottoApp) -> bool {
    app.settings.theme == Theme::XpLuna
}

fn accent(app: &TaskottoApp) -> Color32 {
    ThemeStyle::from_settings(&app.settings).accent
}

fn subtle_separator(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, color);
}
