use serde::{Deserialize, Serialize};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    #[default]
    System,
    Windows98,
    XpLuna,
    Windows7,
    Windows10,
    FrutigerAero,
    Chromeos,
    Macos,
}

impl Theme {
    pub const ALL: [Self; 8] = [
        Self::System,
        Self::Windows98,
        Self::XpLuna,
        Self::Windows7,
        Self::Windows10,
        Self::FrutigerAero,
        Self::Chromeos,
        Self::Macos,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::System => "Taskotto dark",
            Self::Windows98 => "Classic 98",
            Self::XpLuna => "Luna",
            Self::Windows7 => "Seven",
            Self::Windows10 => "Modern 10",
            Self::FrutigerAero => "Frutiger Aero",
            Self::Chromeos => "ChromeOS",
            Self::Macos => "Mac",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name
            .trim()
            .to_ascii_lowercase()
            .replace(['_', ' '], "-")
            .as_str()
        {
            "system" | "default" | "taskotto" => Some(Self::System),
            "windows98" | "windows-98" | "win98" | "98" | "classic98" => Some(Self::Windows98),
            "xp" | "xp-luna" | "windows-xp" | "windowsxp" | "luna" => Some(Self::XpLuna),
            "windows7" | "windows-7" | "win7" | "7" => Some(Self::Windows7),
            "windows10" | "windows-10" | "win10" | "10" => Some(Self::Windows10),
            "frutiger-aero" | "frutigeraero" | "aero" => Some(Self::FrutigerAero),
            "chromeos" | "chrome-os" | "chrome" => Some(Self::Chromeos),
            "macos" | "mac-os" | "mac" | "macrice" => Some(Self::Macos),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Density {
    Compact,
    #[default]
    Comfortable,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LayoutMode {
    #[default]
    Auto,
    Sidebar,
    TopTabs,
}

impl LayoutMode {
    pub fn resolve(self, theme: Theme) -> Self {
        match self {
            Self::Auto => match theme {
                Theme::Windows98
                | Theme::XpLuna
                | Theme::Windows7
                | Theme::Windows10
                | Theme::FrutigerAero
                | Theme::Macos => Self::TopTabs,
                _ => Self::Sidebar,
            },
            mode => mode,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SortBy {
    #[default]
    Cpu,
    Memory,
    Io,
    Name,
    Pid,
}

impl SortBy {
    pub const ALL: [Self; 5] = [Self::Cpu, Self::Memory, Self::Io, Self::Name, Self::Pid];

    pub fn label(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Memory => "Memory",
            Self::Io => "I/O rate",
            Self::Name => "Name",
            Self::Pid => "PID",
        }
    }
}

/// Optional semantic widget colors. Absent values keep each rice's own look.
/// These belong to appearance, not permissions or sampling configuration.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ComponentColors {
    pub control: Option<String>,
    pub hover: Option<String>,
    pub border: Option<String>,
    pub table: Option<String>,
    pub selection: Option<String>,
}

impl ComponentColors {
    pub fn is_empty(&self) -> bool {
        self.control.is_none()
            && self.hover.is_none()
            && self.border.is_none()
            && self.table.is_none()
            && self.selection.is_none()
    }

    pub fn validate(&self) -> Result<(), String> {
        for (name, color) in [
            ("control", &self.control),
            ("hover", &self.hover),
            ("border", &self.border),
            ("table", &self.table),
            ("selection", &self.selection),
        ] {
            if color.as_deref().is_some_and(|color| !is_hex_color(color)) {
                return Err(format!("components.{name} must be #RRGGBB"));
            }
        }
        Ok(())
    }

    pub fn normalize(&mut self) {
        for color in [
            &mut self.control,
            &mut self.hover,
            &mut self.border,
            &mut self.table,
            &mut self.selection,
        ] {
            if color.as_deref().is_some_and(|value| !is_hex_color(value)) {
                *color = None;
            }
        }
    }

    /// A missing pack token does not erase a user's custom base token.
    pub fn overlay(&mut self, pack: &Self) {
        for (target, source) in [
            (&mut self.control, &pack.control),
            (&mut self.hover, &pack.hover),
            (&mut self.border, &pack.border),
            (&mut self.table, &pack.table),
            (&mut self.selection, &pack.selection),
        ] {
            if let Some(source) = source {
                *target = Some(source.clone());
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    pub theme: Theme,
    pub density: Density,
    pub sort_by: SortBy,
    pub descending: bool,
    pub refresh_ms: u64,
    /// Optional sampling/animation tradeoff for battery-sensitive desktops.
    pub low_overhead: bool,
    pub tree_view: bool,
    pub only_my_processes: bool,
    pub show_kernel_threads: bool,
    pub show_io_column: bool,
    pub show_user_column: bool,
    pub table_row_height: Option<f32>,
    pub accent_hex: Option<String>,
    pub surface_hex: Option<String>,
    pub text_hex: Option<String>,
    pub layout_mode: LayoutMode,
    pub font_scale: f32,
    pub ui_spacing: f32,
    /// Scales drawn performance charts while preserving each rice's proportions.
    pub chart_scale: f32,
    pub corner_radius: Option<u8>,
    #[serde(skip_serializing_if = "ComponentColors::is_empty")]
    pub components: ComponentColors,
    /// A generated screen layout belongs to the active RicePack session only.
    #[serde(skip)]
    pub(crate) custom_layout: Option<crate::layout::LayoutSpec>,
    /// Session-only RiceBowl/CLI override; never replace the user's saved theme.
    #[serde(skip)]
    pub launch_theme_original: Option<Theme>,
    /// RicePack design overrides are transient and must not overwrite the user's config.
    #[serde(skip)]
    pub(crate) launch_ricepack_original: Option<Box<AppearanceBackup>>,
    #[serde(skip)]
    pub launch_ricepack_name: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct AppearanceBackup {
    theme: Theme,
    density: Density,
    layout_mode: LayoutMode,
    accent_hex: Option<String>,
    surface_hex: Option<String>,
    text_hex: Option<String>,
    font_scale: f32,
    ui_spacing: f32,
    chart_scale: f32,
    corner_radius: Option<u8>,
    table_row_height: Option<f32>,
    components: ComponentColors,
    custom_layout: Option<crate::layout::LayoutSpec>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            density: Density::Comfortable,
            sort_by: SortBy::Cpu,
            descending: true,
            refresh_ms: 1_500,
            low_overhead: false,
            tree_view: false,
            only_my_processes: false,
            show_kernel_threads: false,
            show_io_column: true,
            show_user_column: true,
            table_row_height: None,
            accent_hex: None,
            surface_hex: None,
            text_hex: None,
            layout_mode: LayoutMode::Auto,
            font_scale: 1.0,
            ui_spacing: 8.0,
            chart_scale: 1.0,
            corner_radius: None,
            components: ComponentColors::default(),
            custom_layout: None,
            launch_theme_original: None,
            launch_ricepack_original: None,
            launch_ricepack_name: None,
        }
    }
}

impl Settings {
    /// Effective worker period without modifying the user's saved interval.
    /// The profile changes sampling cadence, not which monitors are enabled.
    pub(crate) fn effective_refresh_ms(&self) -> u64 {
        let interval = self.refresh_ms.clamp(500, 10_000);
        if self.low_overhead {
            interval.max(3_000)
        } else {
            interval
        }
    }

    pub(crate) fn record_ricepack_session(&mut self, name: &str) {
        if self.launch_ricepack_original.is_none() {
            self.launch_ricepack_original = Some(Box::new(AppearanceBackup {
                theme: self.theme,
                density: self.density,
                layout_mode: self.layout_mode,
                accent_hex: self.accent_hex.clone(),
                surface_hex: self.surface_hex.clone(),
                text_hex: self.text_hex.clone(),
                font_scale: self.font_scale,
                ui_spacing: self.ui_spacing,
                chart_scale: self.chart_scale,
                corner_radius: self.corner_radius,
                table_row_height: self.table_row_height,
                components: self.components.clone(),
                custom_layout: self.custom_layout.clone(),
            }));
        }
        self.launch_ricepack_name = Some(name.to_owned());
    }

    /// Restore the user's pre-pack appearance before applying a revised pack.
    /// Keep the backup so subsequent saves still persist the original design.
    pub(crate) fn reset_ricepack_appearance(&mut self) {
        if let Some(original) = &self.launch_ricepack_original {
            self.theme = original.theme;
            self.density = original.density;
            self.layout_mode = original.layout_mode;
            self.accent_hex.clone_from(&original.accent_hex);
            self.surface_hex.clone_from(&original.surface_hex);
            self.text_hex.clone_from(&original.text_hex);
            self.font_scale = original.font_scale;
            self.ui_spacing = original.ui_spacing;
            self.chart_scale = original.chart_scale;
            self.corner_radius = original.corner_radius;
            self.table_row_height = original.table_row_height;
            self.components.clone_from(&original.components);
            self.custom_layout.clone_from(&original.custom_layout);
        }
    }

    pub fn normalize(&mut self) {
        self.refresh_ms = self.refresh_ms.clamp(500, 10_000);
        if self
            .accent_hex
            .as_deref()
            .is_some_and(|hex| !is_hex_color(hex))
        {
            self.accent_hex = None;
        }
        for color in [&mut self.surface_hex, &mut self.text_hex] {
            if color.as_deref().is_some_and(|hex| !is_hex_color(hex)) {
                *color = None;
            }
        }
        self.font_scale = if self.font_scale.is_finite() {
            self.font_scale.clamp(0.8, 1.5)
        } else {
            1.0
        };
        self.ui_spacing = if self.ui_spacing.is_finite() {
            self.ui_spacing.clamp(3.0, 16.0)
        } else {
            8.0
        };
        self.chart_scale = if self.chart_scale.is_finite() {
            self.chart_scale.clamp(0.75, 1.5)
        } else {
            1.0
        };
        self.corner_radius = self.corner_radius.map(|radius| radius.min(16));
        self.table_row_height = self
            .table_row_height
            .filter(|value| value.is_finite())
            .map(|value| value.clamp(22.0, 48.0));
        self.components.normalize();
    }

    pub fn load() -> io::Result<Self> {
        let path = config_file()?;
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(err) => return Err(err),
        };
        let mut config: Self =
            toml::from_str(&text).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        config.normalize();
        Ok(config)
    }

    pub fn save(&self) -> io::Result<()> {
        let path = config_file()?;
        self.save_to(&path)
    }

    fn save_to(&self, path: &Path) -> io::Result<()> {
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("invalid config path"))?;
        fs::create_dir_all(parent)?;
        let mut persistent = self.clone();
        if let Some(original) = &self.launch_ricepack_original {
            persistent.theme = original.theme;
            persistent.density = original.density;
            persistent.layout_mode = original.layout_mode;
            persistent.accent_hex.clone_from(&original.accent_hex);
            persistent.surface_hex.clone_from(&original.surface_hex);
            persistent.text_hex.clone_from(&original.text_hex);
            persistent.font_scale = original.font_scale;
            persistent.ui_spacing = original.ui_spacing;
            persistent.chart_scale = original.chart_scale;
            persistent.corner_radius = original.corner_radius;
            persistent.table_row_height = original.table_row_height;
            persistent.components.clone_from(&original.components);
        }
        if let Some(original) = self.launch_theme_original {
            persistent.theme = original;
        }
        let contents = toml::to_string_pretty(&persistent).map_err(io::Error::other)?;
        let identifier = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temporary = parent.join(format!(".taskotto-{}-{identifier}.tmp", std::process::id()));
        let result = (|| {
            // A broken config must never be silently lost on the first change.
            match fs::read_to_string(path) {
                Ok(existing) if toml::from_str::<Self>(&existing).is_err() => {
                    let backup = parent.join(format!(
                        "config.invalid-{}-{identifier}.toml",
                        std::process::id()
                    ));
                    fs::copy(path, backup)?;
                }
                Ok(_) => {}
                Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                Err(err) => return Err(err),
            }
            use std::io::Write;
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&temporary)?;
            file.write_all(contents.as_bytes())?;
            file.sync_all()?;
            fs::rename(&temporary, path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        result
    }
}

pub fn configured_theme() -> Option<Theme> {
    // RiceBowl may opt in by exporting either of these when starting Taskotto.
    ["RICEBOWL_RICE", "RICEBOWL_THEME"]
        .iter()
        .find_map(|key| env::var(key).ok().and_then(|value| Theme::parse(&value)))
}

pub fn is_hex_color(text: &str) -> bool {
    text.len() == 7
        && text.starts_with('#')
        && text[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn config_file() -> io::Result<PathBuf> {
    if let Some(base) = env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
        let base = PathBuf::from(base);
        if base.is_absolute() {
            return Ok(base.join("taskotto/config.toml"));
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "XDG_CONFIG_HOME must be absolute",
        ));
    }
    let home = env::var_os("HOME")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
    Ok(PathBuf::from(home).join(".config/taskotto/config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ricebowl_theme_aliases() {
        assert_eq!(Theme::parse("xp-luna"), Some(Theme::XpLuna));
        assert_eq!(Theme::parse("Frutiger_Aero"), Some(Theme::FrutigerAero));
        assert_eq!(Theme::parse("unknown"), None);
    }

    #[test]
    fn invalid_settings_are_clamped() {
        let mut config = Settings {
            refresh_ms: 1,
            accent_hex: Some("#nope".into()),
            surface_hex: Some("wrong".into()),
            font_scale: f32::INFINITY,
            ui_spacing: -100.0,
            chart_scale: f32::NAN,
            corner_radius: Some(255),
            table_row_height: Some(f32::NEG_INFINITY),
            components: ComponentColors {
                control: Some("invalid".into()),
                ..ComponentColors::default()
            },
            ..Settings::default()
        };
        config.normalize();
        assert_eq!(config.refresh_ms, 500);
        assert!(config.accent_hex.is_none());
        assert!(config.surface_hex.is_none());
        assert_eq!(config.font_scale, 1.0);
        assert_eq!(config.ui_spacing, 3.0);
        assert_eq!(config.chart_scale, 1.0);
        assert_eq!(config.corner_radius, Some(16));
        assert!(config.table_row_height.is_none());
        assert!(config.components.control.is_none());
    }

    #[test]
    fn toml_settings_roundtrip() {
        let config = Settings::default();
        let encoded = toml::to_string(&config).unwrap();
        let decoded: Settings = toml::from_str(&encoded).unwrap();
        assert_eq!(decoded.refresh_ms, config.refresh_ms);
        assert_eq!(decoded.theme, config.theme);
    }

    #[test]
    fn low_overhead_profile_clamps_effective_period_without_changing_preferences() {
        let mut settings = Settings::default();
        assert_eq!(settings.effective_refresh_ms(), 1_500);
        settings.low_overhead = true;
        assert_eq!(settings.effective_refresh_ms(), 3_000);
        settings.refresh_ms = 5_000;
        assert_eq!(settings.effective_refresh_ms(), 5_000);
        settings.refresh_ms = 500;
        assert_eq!(settings.effective_refresh_ms(), 3_000);
        let saved = toml::to_string(&settings).unwrap();
        let mut loaded: Settings = toml::from_str(&saved).unwrap();
        assert!(loaded.low_overhead);
        assert_eq!(loaded.refresh_ms, 500);
        loaded.low_overhead = false;
        assert_eq!(loaded.effective_refresh_ms(), 500);
        let legacy: Settings = toml::from_str("refresh_ms = 1200").unwrap();
        assert!(!legacy.low_overhead);
        assert_eq!(legacy.effective_refresh_ms(), 1200);
    }

    #[test]
    fn old_settings_are_backward_compatible() {
        let old = "theme = 'windows7'\nrefresh_ms = 3000\n";
        let settings: Settings = toml::from_str(old).unwrap();
        assert_eq!(
            settings.layout_mode.resolve(settings.theme),
            LayoutMode::TopTabs
        );
        assert_eq!(settings.font_scale, 1.0);
        assert_eq!(settings.chart_scale, 1.0);
    }

    #[test]
    fn chart_scale_is_bounded_for_saved_preferences() {
        let mut settings = Settings {
            chart_scale: 8.0,
            ..Settings::default()
        };
        settings.normalize();
        assert_eq!(settings.chart_scale, 1.5);
        settings.chart_scale = -1.0;
        settings.normalize();
        assert_eq!(settings.chart_scale, 0.75);
    }

    #[test]
    fn session_theme_does_not_overwrite_persistent_preference() {
        let test_dir = std::env::temp_dir().join(format!(
            "taskotto-theme-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&test_dir).unwrap();
        let path = test_dir.join("config.toml");
        let config = Settings {
            theme: Theme::XpLuna,
            launch_theme_original: Some(Theme::Windows10),
            ..Settings::default()
        };
        config.save_to(&path).unwrap();
        let loaded: Settings = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(loaded.theme, Theme::Windows10);
        assert!(loaded.launch_theme_original.is_none());
        fs::remove_file(&path).unwrap();
        fs::remove_dir(&test_dir).unwrap();
    }

    #[test]
    fn ricepack_appearance_is_session_only_but_monitor_settings_are_saved() {
        let test_dir = std::env::temp_dir().join(format!(
            "taskotto-ricepack-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&test_dir).unwrap();
        let path = test_dir.join("config.toml");
        let mut config = Settings {
            theme: Theme::Windows10,
            accent_hex: Some("#2277CC".into()),
            components: ComponentColors {
                border: Some("#123456".into()),
                ..ComponentColors::default()
            },
            layout_mode: LayoutMode::Sidebar,
            chart_scale: 1.2,
            refresh_ms: 3_000,
            ..Settings::default()
        };
        crate::ricepack::RicePack::from_toml(include_str!("../examples/ricepacks/sea-glass.toml"))
            .unwrap()
            .apply(&mut config);
        config.refresh_ms = 5_000;
        config.save_to(&path).unwrap();
        let loaded: Settings = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(loaded.theme, Theme::Windows10);
        assert_eq!(loaded.layout_mode, LayoutMode::Sidebar);
        assert_eq!(loaded.chart_scale, 1.2);
        assert_eq!(loaded.accent_hex.as_deref(), Some("#2277CC"));
        assert_eq!(loaded.components.border.as_deref(), Some("#123456"));
        assert!(loaded.components.selection.is_none());
        assert_eq!(loaded.refresh_ms, 5_000);
        assert!(loaded.launch_ricepack_name.is_none());
        fs::remove_file(path).unwrap();
        fs::remove_dir(test_dir).unwrap();
    }

    #[test]
    fn invalid_existing_config_is_backed_up_before_save() {
        let test_dir = std::env::temp_dir().join(format!(
            "taskotto-config-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&test_dir).unwrap();
        let path = test_dir.join("config.toml");
        fs::write(&path, "[broken\n").unwrap();
        Settings::default().save_to(&path).unwrap();
        let backups = fs::read_dir(&test_dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|file| {
                file.file_name()
                    .to_string_lossy()
                    .starts_with("config.invalid-")
            })
            .collect::<Vec<_>>();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read_to_string(backups[0].path()).unwrap(), "[broken\n");
        assert!(toml::from_str::<Settings>(&fs::read_to_string(&path).unwrap()).is_ok());
        fs::remove_file(&path).unwrap();
        fs::remove_file(backups[0].path()).unwrap();
        fs::remove_dir(&test_dir).unwrap();
    }
}
