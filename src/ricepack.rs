//! Declarative, untrusted, *data-only* Taskotto appearance packs.
//!
//! RiceBowl and generative tools can select an existing client layout and
//! override its supported tokens without loading executable code or assets.
use crate::config::{Density, LayoutMode, Settings, Theme, is_hex_color};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

const MAX_RICEPACK_BYTES: u64 = 64 * 1024;
pub const FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RicePack {
    pub format_version: u32,
    pub name: String,
    pub base_theme: Theme,
    #[serde(default)]
    pub appearance: Appearance,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Appearance {
    pub accent: Option<String>,
    pub surface: Option<String>,
    pub text: Option<String>,
    pub density: Option<Density>,
    pub navigation: Option<LayoutMode>,
    pub font_scale: Option<f32>,
    pub spacing: Option<f32>,
    pub corner_radius: Option<u8>,
    pub row_height: Option<f32>,
}

impl RicePack {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            name: format!("{} Custom", settings.theme.label()),
            base_theme: settings.theme,
            appearance: Appearance {
                accent: settings.accent_hex.clone(),
                surface: settings.surface_hex.clone(),
                text: settings.text_hex.clone(),
                density: Some(settings.density),
                navigation: Some(settings.layout_mode),
                font_scale: Some(settings.font_scale),
                spacing: Some(settings.ui_spacing),
                corner_radius: settings.corner_radius,
                row_height: settings.table_row_height,
            },
        }
    }

    /// Export only to a new path; never truncate an existing user's design.
    pub fn write_new(&self, path: &Path) -> io::Result<()> {
        let contents = toml::to_string_pretty(self).map_err(io::Error::other)?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(path)?;
        let result = file
            .write_all(contents.as_bytes())
            .and_then(|()| file.sync_all());
        if result.is_err() {
            let _ = std::fs::remove_file(path);
        }
        result
    }

    pub fn from_toml(source: &str) -> Result<Self, String> {
        let pack: Self = toml::from_str(source).map_err(|err| err.to_string())?;
        if pack.format_version != FORMAT_VERSION {
            return Err(format!(
                "unsupported RicePack format_version {}; expected {FORMAT_VERSION}",
                pack.format_version
            ));
        }
        if pack.name.trim().is_empty()
            || pack.name.chars().count() > 64
            || pack.name.chars().any(char::is_control)
        {
            return Err("RicePack name must be 1–64 printable characters".to_owned());
        }
        for (label, value) in [
            ("accent", &pack.appearance.accent),
            ("surface", &pack.appearance.surface),
            ("text", &pack.appearance.text),
        ] {
            if value.as_deref().is_some_and(|value| !is_hex_color(value)) {
                return Err(format!("appearance.{label} must be #RRGGBB"));
            }
        }
        for (label, value, min, max) in [
            ("font_scale", pack.appearance.font_scale, 0.8, 1.5),
            ("spacing", pack.appearance.spacing, 3.0, 16.0),
            ("row_height", pack.appearance.row_height, 22.0, 48.0),
        ] {
            if value.is_some_and(|value| !value.is_finite() || !(min..=max).contains(&value)) {
                return Err(format!(
                    "appearance.{label} must be between {min} and {max}"
                ));
            }
        }
        if pack
            .appearance
            .corner_radius
            .is_some_and(|value| value > 16)
        {
            return Err("appearance.corner_radius must be between 0 and 16".to_owned());
        }
        Ok(pack)
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = file.take(MAX_RICEPACK_BYTES + 1);
        let mut source = String::new();
        reader.read_to_string(&mut source)?;
        if source.len() as u64 > MAX_RICEPACK_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "RicePack exceeds 64 KiB",
            ));
        }
        Self::from_toml(&source).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    /// Apply a temporary design. Other user preferences keep their persistence.
    pub fn apply(&self, settings: &mut Settings) {
        settings.record_ricepack_session(&self.name);
        settings.theme = self.base_theme;
        let appearance = &self.appearance;
        if let Some(value) = &appearance.accent {
            settings.accent_hex = Some(value.clone());
        }
        if let Some(value) = &appearance.surface {
            settings.surface_hex = Some(value.clone());
        }
        if let Some(value) = &appearance.text {
            settings.text_hex = Some(value.clone());
        }
        if let Some(value) = appearance.density {
            settings.density = value;
        }
        if let Some(value) = appearance.navigation {
            settings.layout_mode = value;
        }
        if let Some(value) = appearance.font_scale {
            settings.font_scale = value;
        }
        if let Some(value) = appearance.spacing {
            settings.ui_spacing = value;
        }
        if let Some(value) = appearance.corner_radius {
            settings.corner_radius = Some(value);
        }
        if let Some(value) = appearance.row_height {
            settings.table_row_height = Some(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!("../examples/ricepacks/sea-glass.toml");

    #[test]
    fn example_pack_parses_and_applies_without_changing_monitoring_options() {
        let pack = RicePack::from_toml(SAMPLE).unwrap();
        let mut settings = Settings {
            refresh_ms: 5_000,
            theme: Theme::XpLuna,
            ..Settings::default()
        };
        pack.apply(&mut settings);
        assert_eq!(settings.theme, Theme::FrutigerAero);
        assert_eq!(settings.layout_mode, LayoutMode::TopTabs);
        assert_eq!(settings.accent_hex.as_deref(), Some("#159DA4"));
        assert_eq!(settings.refresh_ms, 5_000);
        assert_eq!(settings.launch_ricepack_name.as_deref(), Some("Sea Glass"));
        let style = crate::theme::ThemeStyle::from_settings(&settings);
        assert_eq!(style.accent, eframe::egui::Color32::from_rgb(21, 157, 164));
    }

    #[test]
    fn invalid_pack_is_rejected_not_silently_clamped() {
        for source in [
            SAMPLE.replace("format_version = 1", "format_version = 2"),
            SAMPLE.replace("#159DA4", "#not-hex"),
            SAMPLE.replace("font_scale = 1.05", "font_scale = 500"),
            SAMPLE.replace("Sea Glass", ""),
            format!("{SAMPLE}\nunknown = true"),
        ] {
            assert!(RicePack::from_toml(&source).is_err(), "{source}");
        }
    }

    #[test]
    fn oversized_ricepack_is_rejected_before_parsing() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "taskotto-oversized-ricepack-{}-{nonce}.toml",
            std::process::id()
        ));
        std::fs::write(&path, vec![b' '; MAX_RICEPACK_BYTES as usize + 1]).unwrap();
        let error = RicePack::load(&path).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("64 KiB"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn exported_settings_roundtrip_through_the_versioned_pack_schema() {
        let settings = Settings {
            theme: Theme::Windows7,
            accent_hex: Some("#123ABC".into()),
            layout_mode: LayoutMode::Sidebar,
            ui_spacing: 11.0,
            ..Settings::default()
        };
        let pack = RicePack::from_settings(&settings);
        let text = toml::to_string_pretty(&pack).unwrap();
        let loaded = RicePack::from_toml(&text).unwrap();
        assert_eq!(loaded.base_theme, Theme::Windows7);
        assert_eq!(loaded.appearance.accent.as_deref(), Some("#123ABC"));
        assert_eq!(loaded.appearance.navigation, Some(LayoutMode::Sidebar));
        assert_eq!(loaded.appearance.spacing, Some(11.0));
        let mut restored = Settings::default();
        loaded.apply(&mut restored);
        assert_eq!(restored.theme, settings.theme);
        assert_eq!(restored.accent_hex, settings.accent_hex);
    }
}
