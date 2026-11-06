//! Standalone Taskotto views inspired by two historical task-manager design
//! languages. They share the existing procfs sampler, process actions, and
//! settings; only drawing/navigation is specialized.

mod applications;
mod chrome;
mod details;
mod performance;
mod process;
mod users;

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

pub(super) fn networking(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    performance::networking_page(app, ui);
}

pub(super) fn users(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    users::page(app, ui);
}

pub(super) fn xp_details(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    details::xp_page(app, ui);
}

pub(super) fn applications(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    applications::page(app, ui);
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
