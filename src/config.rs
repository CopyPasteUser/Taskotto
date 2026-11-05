use serde::{Deserialize, Serialize};
use std::{env, fs, io, path::PathBuf};

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
            Self::FrutigerAero => "Aero",
            Self::Chromeos => "Chrome",
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
pub enum SortBy {
    #[default]
    Cpu,
    Memory,
    Name,
    Pid,
}

impl SortBy {
    pub const ALL: [Self; 4] = [Self::Cpu, Self::Memory, Self::Name, Self::Pid];

    pub fn label(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Memory => "Memory",
            Self::Name => "Name",
            Self::Pid => "PID",
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
    pub tree_view: bool,
    pub only_my_processes: bool,
    pub show_kernel_threads: bool,
    pub accent_hex: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            density: Density::Comfortable,
            sort_by: SortBy::Cpu,
            descending: true,
            refresh_ms: 1_500,
            tree_view: false,
            only_my_processes: false,
            show_kernel_threads: false,
            accent_hex: None,
        }
    }
}

impl Settings {
    pub fn normalize(&mut self) {
        self.refresh_ms = self.refresh_ms.clamp(500, 10_000);
        if self
            .accent_hex
            .as_deref()
            .is_some_and(|hex| !is_hex_color(hex))
        {
            self.accent_hex = None;
        }
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
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("invalid config path"))?;
        fs::create_dir_all(parent)?;
        let contents = toml::to_string_pretty(self).map_err(io::Error::other)?;
        let temporary = parent.join(format!(".config-{}.tmp", std::process::id()));
        let result = (|| {
            fs::write(&temporary, contents)?;
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
            ..Settings::default()
        };
        config.normalize();
        assert_eq!(config.refresh_ms, 500);
        assert!(config.accent_hex.is_none());
    }

    #[test]
    fn toml_settings_roundtrip() {
        let config = Settings::default();
        let encoded = toml::to_string(&config).unwrap();
        let decoded: Settings = toml::from_str(&encoded).unwrap();
        assert_eq!(decoded.refresh_ms, config.refresh_ms);
        assert_eq!(decoded.theme, config.theme);
    }
}
