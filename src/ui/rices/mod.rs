//! Standalone Taskotto views inspired by historical task-manager design
//! languages. They share the existing procfs sampler, process actions, and
//! settings; only drawing/navigation is specialized.

mod aero;
mod applications;
mod chrome;
mod chromeos;
mod classic98;
mod declarative;
mod details;
mod macos;
mod performance;
mod process;
mod seven;
mod system;
mod users;

pub(super) use declarative::render as declarative_processes;
pub(super) use system::SystemPages;

use super::*;

pub(super) fn supports(theme: Theme) -> bool {
    matches!(
        theme,
        Theme::Windows98
            | Theme::XpLuna
            | Theme::Windows7
            | Theme::Windows10
            | Theme::Chromeos
            | Theme::FrutigerAero
            | Theme::Macos
    )
}

pub(super) fn topbar(app: &mut TaskottoApp, ctx: &egui::Context) {
    if app.settings.theme == Theme::FrutigerAero {
        return aero::topbar(app, ctx);
    }
    if app.settings.theme == Theme::Windows98 {
        return;
    }
    if app.settings.theme == Theme::Macos {
        return macos::topbar(app, ctx);
    }
    if app.settings.theme == Theme::Chromeos {
        return chromeos::topbar(app, ctx);
    }
    if app.settings.theme == Theme::Windows7 {
        return seven::topbar(app, ctx);
    }
    chrome::topbar(app, ctx);
}

pub(super) fn classic98_navigation(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    classic98::navigation(app, ui);
}

pub(super) fn navigation(app: &mut TaskottoApp, ctx: &egui::Context) {
    if app.settings.theme == Theme::FrutigerAero {
        return aero::navigation(app, ctx);
    }
    if app.settings.theme == Theme::Windows98 {
        return;
    }
    if app.settings.theme == Theme::Macos {
        return macos::navigation(app, ctx);
    }
    if app.settings.theme == Theme::Chromeos {
        return chromeos::navigation(app, ctx);
    }
    if app.settings.theme == Theme::Windows7 {
        return seven::navigation(app, ctx);
    }
    chrome::navigation(app, ctx);
}

pub(super) fn vertical_navigation(app: &mut TaskottoApp, ctx: &egui::Context) {
    if app.settings.theme == Theme::FrutigerAero {
        return aero::sidebar(app, ctx);
    }
    if app.settings.theme == Theme::Macos {
        return macos::vertical_navigation(app, ctx);
    }
    if app.settings.theme == Theme::Chromeos {
        return chromeos::vertical_navigation(app, ctx);
    }
    chrome::vertical_navigation(app, ctx);
}

pub(super) fn statusbar(app: &mut TaskottoApp, ctx: &egui::Context) {
    if app.settings.theme == Theme::FrutigerAero {
        return aero::statusbar(app, ctx);
    }
    if app.settings.theme == Theme::Windows98 {
        return classic98::statusbar(app, ctx);
    }
    if app.settings.theme == Theme::Macos {
        return macos::statusbar(app, ctx);
    }
    if app.settings.theme == Theme::Chromeos {
        return chromeos::statusbar(app, ctx);
    }
    if app.settings.theme == Theme::Windows7 {
        return seven::statusbar(app, ctx);
    }
    chrome::statusbar(app, ctx);
}

pub(super) fn processes(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    if app.settings.theme == Theme::FrutigerAero {
        return aero::processes(app, ui);
    }
    if app.settings.theme == Theme::Windows98 {
        return classic98::processes(app, ui);
    }
    if app.settings.theme == Theme::Macos {
        return macos::process_table(app, ui, macos::Focus::Cpu);
    }
    if app.settings.theme == Theme::Chromeos {
        return chromeos::processes(app, ui);
    }
    if app.settings.theme == Theme::Windows7 {
        return seven::processes(app, ui);
    }
    process::page(app, ui);
}

pub(super) fn performance(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    if app.settings.theme == Theme::FrutigerAero {
        return aero::performance(app, ui);
    }
    if app.settings.theme == Theme::Chromeos {
        return chromeos::performance(app, ui);
    }
    if app.settings.theme == Theme::Macos {
        return macos::process_table(app, ui, macos::Focus::Memory);
    }
    if app.settings.theme == Theme::Windows7 {
        return seven::performance(app, ui);
    }
    performance::page(app, ui);
}

pub(super) fn networking(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    if app.settings.theme == Theme::FrutigerAero {
        return aero::networking(app, ui);
    }
    if app.settings.theme == Theme::Chromeos {
        return chromeos::networking(app, ui);
    }
    if app.settings.theme == Theme::Macos {
        return macos::network_page(app, ui);
    }
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
    if app.settings.theme == Theme::Windows98 {
        return classic98::close_program(app, ui);
    }
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

pub(super) fn mac_energy(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    macos::process_table(app, ui, macos::Focus::Energy);
}

pub(super) fn mac_disk(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    macos::process_table(app, ui, macos::Focus::Disk);
}

pub(super) fn mac_info_dialog(app: &mut TaskottoApp, ctx: &egui::Context) {
    macos::info_dialog(app, ctx);
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
