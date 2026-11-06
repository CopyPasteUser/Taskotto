//! Standalone Taskotto views inspired by three historical task-manager design
//! languages. They share the existing procfs sampler, process actions, and
//! settings; only drawing/navigation is specialized.

mod applications;
mod chrome;
mod details;
mod performance;
mod process;
mod seven;
mod system;
mod users;

pub(super) use system::SystemPages;

use super::*;

pub(super) fn supports(theme: Theme) -> bool {
    matches!(theme, Theme::XpLuna | Theme::Windows7 | Theme::Windows10)
}

pub(super) fn topbar(app: &mut TaskottoApp, ctx: &egui::Context) {
    if app.settings.theme == Theme::Windows7 {
        return seven::topbar(app, ctx);
    }
    chrome::topbar(app, ctx);
}

pub(super) fn navigation(app: &mut TaskottoApp, ctx: &egui::Context) {
    if app.settings.theme == Theme::Windows7 {
        return seven::navigation(app, ctx);
    }
    chrome::navigation(app, ctx);
}

pub(super) fn vertical_navigation(app: &mut TaskottoApp, ctx: &egui::Context) {
    chrome::vertical_navigation(app, ctx);
}

pub(super) fn statusbar(app: &mut TaskottoApp, ctx: &egui::Context) {
    if app.settings.theme == Theme::Windows7 {
        return seven::statusbar(app, ctx);
    }
    chrome::statusbar(app, ctx);
}

pub(super) fn processes(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    if app.settings.theme == Theme::Windows7 {
        return seven::processes(app, ui);
    }
    process::page(app, ui);
}

pub(super) fn performance(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    if app.settings.theme == Theme::Windows7 {
        return seven::performance(app, ui);
    }
    performance::page(app, ui);
}

pub(super) fn networking(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    if app.settings.theme == Theme::Windows7 {
        return seven::networking(app, ui);
    }
    performance::networking_page(app, ui);
}

pub(super) fn users(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    if app.settings.theme == Theme::Windows7 {
        return seven::users(app, ui);
    }
    users::page(app, ui);
}

pub(super) fn xp_details(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    details::xp_page(app, ui);
}

pub(super) fn applications(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    if app.settings.theme == Theme::Windows7 {
        return seven::applications(app, ui);
    }
    applications::page(app, ui);
}

pub(super) fn app_history(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    system::app_history(app, ui);
}

pub(super) fn startup(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    system::startup(app, ui);
}

pub(super) fn services(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    if app.settings.theme == Theme::Windows7 {
        return seven::services(app, ui);
    }
    system::services(app, ui);
}

pub(super) fn seven_properties_dialog(app: &mut TaskottoApp, ctx: &egui::Context) {
    seven::properties_dialog(app, ctx);
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
