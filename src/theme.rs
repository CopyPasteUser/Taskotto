//! Data-driven UI tokens. RiceBowl or a generated theme can write these
//! settings as TOML without patching application widgets.
use crate::config::{Density, Settings, Theme};
use eframe::egui::{self, Color32, FontId, TextStyle, Vec2};

#[derive(Clone, Debug)]
pub struct ThemeStyle {
    pub accent: Color32,
    pub surface: Color32,
    pub text: Color32,
    pub dark: bool,
    pub rounding: u8,
    pub spacing: f32,
    pub font_scale: f32,
}

impl ThemeStyle {
    pub fn from_settings(settings: &Settings) -> Self {
        let (dark, surface, text, accent, rounding) = match settings.theme {
            Theme::System => (true, (25, 36, 51), (231, 237, 247), (130, 180, 255), 8),
            Theme::Windows98 => (false, (196, 196, 196), (20, 20, 20), (0, 128, 128), 0),
            Theme::XpLuna => (false, (237, 235, 222), (28, 39, 65), (48, 105, 215), 5),
            Theme::Windows7 => (false, (222, 238, 248), (29, 44, 65), (43, 138, 195), 6),
            Theme::Windows10 => (true, (37, 39, 42), (240, 240, 240), (0, 120, 215), 2),
            Theme::FrutigerAero => (false, (226, 243, 238), (23, 57, 48), (37, 149, 114), 9),
            Theme::Chromeos => (false, (246, 248, 252), (35, 49, 65), (68, 125, 220), 12),
            Theme::Macos => (false, (241, 242, 245), (34, 37, 44), (24, 118, 238), 9),
        };
        let default_surface = rgb(surface);
        let custom_surface = settings.surface_hex.as_deref().and_then(parse_hex);
        let surface = custom_surface.unwrap_or(default_surface);
        // Generated ricepacks may switch light/dark regardless of preset name.
        let dark = if custom_surface.is_some() {
            luminance(surface) < 0.18
        } else {
            dark
        };
        let requested_text = settings
            .text_hex
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or_else(|| rgb(text));
        // User-generated palettes may otherwise create unreadable dialogs.
        let text = if contrast_ratio(requested_text, surface) >= 4.5 {
            requested_text
        } else {
            readable_text(surface)
        };
        let accent = settings
            .accent_hex
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or_else(|| rgb(accent));
        Self {
            accent,
            surface,
            text,
            dark,
            rounding: settings.corner_radius.unwrap_or(rounding).min(16),
            spacing: settings.ui_spacing.clamp(3.0, 16.0),
            font_scale: settings.font_scale.clamp(0.8, 1.5),
        }
    }

    pub fn apply(&self, ctx: &egui::Context, settings: &Settings) {
        let mut visuals = if self.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.panel_fill = self.surface;
        visuals.window_fill = self.surface;
        visuals.override_text_color = Some(self.text);
        visuals.selection.bg_fill = self.accent;
        visuals.selection.stroke.color = readable_text(self.accent);
        visuals.hyperlink_color = if contrast_ratio(self.accent, self.surface) >= 4.5 {
            self.accent
        } else {
            self.text
        };
        visuals.widgets.active.bg_fill = self.accent;
        let rounding = egui::CornerRadius::same(self.rounding);
        visuals.widgets.noninteractive.corner_radius = rounding;
        visuals.widgets.inactive.corner_radius = rounding;
        visuals.widgets.hovered.corner_radius = rounding;
        visuals.widgets.active.corner_radius = rounding;
        visuals.widgets.open.corner_radius = rounding;
        ctx.set_visuals(visuals);
        let mut style = (*ctx.style()).clone();
        let spacing = if settings.density == Density::Compact {
            self.spacing * 0.65
        } else {
            self.spacing
        };
        style.spacing.item_spacing = Vec2::new(spacing, spacing * 0.75);
        style.spacing.button_padding = Vec2::new(spacing, spacing * 0.6);
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::proportional(20.0 * self.font_scale),
        );
        style.text_styles.insert(
            TextStyle::Body,
            FontId::proportional(14.0 * self.font_scale),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::proportional(14.0 * self.font_scale),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::proportional(11.0 * self.font_scale),
        );
        ctx.set_style(style);
    }
}

fn rgb((r, g, b): (u8, u8, u8)) -> Color32 {
    Color32::from_rgb(r, g, b)
}

fn parse_hex(hex: &str) -> Option<Color32> {
    if !crate::config::is_hex_color(hex) {
        return None;
    }
    Some(Color32::from_rgb(
        u8::from_str_radix(&hex[1..3], 16).ok()?,
        u8::from_str_radix(&hex[3..5], 16).ok()?,
        u8::from_str_radix(&hex[5..7], 16).ok()?,
    ))
}

fn luminance(color: Color32) -> f32 {
    let linear = |component: u8| {
        let value = component as f32 / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    linear(color.r()) * 0.2126 + linear(color.g()) * 0.7152 + linear(color.b()) * 0.0722
}

fn contrast_ratio(first: Color32, second: Color32) -> f32 {
    let a = luminance(first);
    let b = luminance(second);
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn readable_text(background: Color32) -> Color32 {
    let black = Color32::BLACK;
    let white = Color32::WHITE;
    if contrast_ratio(black, background) >= contrast_ratio(white, background) {
        black
    } else {
        white
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preset_has_legible_text() {
        for theme in Theme::ALL {
            let settings = Settings {
                theme,
                ..Settings::default()
            };
            let tokens = ThemeStyle::from_settings(&settings);
            assert!(
                contrast_ratio(tokens.text, tokens.surface) >= 4.5,
                "{}",
                theme.label()
            );
        }
    }

    #[test]
    fn generated_colors_have_accessible_fallback() {
        let settings = Settings {
            surface_hex: Some("#FFFFFF".into()),
            text_hex: Some("#EEEEEE".into()),
            accent_hex: Some("#3355EE".into()),
            ..Settings::default()
        };
        let tokens = ThemeStyle::from_settings(&settings);
        assert_eq!(tokens.surface, Color32::WHITE);
        assert_eq!(tokens.text, Color32::BLACK);
        assert_eq!(tokens.accent, Color32::from_rgb(51, 85, 238));
        assert!(!tokens.dark);
    }
}
