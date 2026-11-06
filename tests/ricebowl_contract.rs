//! Cross-repository compatibility fixture produced by RiceBowl's
//! `ricebowl de taskotto-export --recipe classic` v1.
//!
//! The source fixture lives in RiceBowl tests/fixtures/taskotto-classic-v1.toml;
//! retaining a copy lets Taskotto verify offline without pulling RiceBowl.
use taskotto::{
    config::{LayoutMode, Settings, Theme},
    ricepack::RicePack,
};

const EXPORTED_BY_RICEBOWL: &str = include_str!("fixtures/ricebowl-classic-v1.toml");

#[test]
fn ricebowl_v1_export_loads_without_ricebowl_installed() {
    let pack = RicePack::from_toml(EXPORTED_BY_RICEBOWL).unwrap();
    assert_eq!(pack.format_version, 1);
    assert_eq!(pack.name, "RiceBowl: Classic bottom taskbar");
    assert_eq!(pack.base_theme, Theme::System);
    assert_eq!(pack.appearance.accent.as_deref(), Some("#4179aa"));
    assert_eq!(pack.appearance.surface.as_deref(), Some("#203c59"));
    assert_eq!(pack.appearance.text.as_deref(), Some("#f3f7fa"));

    let mut settings = Settings::default();
    settings.theme = Theme::Windows7;
    settings.refresh_ms = 5_000;
    settings.layout_mode = LayoutMode::Sidebar;
    pack.apply(&mut settings);
    assert_eq!(settings.theme, Theme::System);
    assert_eq!(settings.layout_mode, LayoutMode::Auto);
    assert_eq!(settings.refresh_ms, 5_000); // No monitoring permissions/options in a pack.
}
