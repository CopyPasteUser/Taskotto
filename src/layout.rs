//! Small, versioned RicePack screen vocabulary. Never executes user code.
//! Everything exposed here is a named, read-only Taskotto data source.
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const MAX_PAGES: usize = 4;
pub const MAX_WIDGETS_PER_PAGE: usize = 16;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutSpec {
    pub pages: Vec<Screen>,
    /// Content area width, excluding client navigation, below which the
    /// optional compact widget set takes over. Defaults to 680 logical px.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compact_below: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Screen {
    pub id: String,
    pub title: String,
    pub widgets: Vec<Widget>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub compact_widgets: Vec<Widget>,
}

impl Screen {
    pub fn widgets_for_width(&self, width: f32, breakpoint: f32) -> &[Widget] {
        if width < breakpoint && !self.compact_widgets.is_empty() {
            &self.compact_widgets
        } else {
            &self.widgets
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Widget {
    Heading { text: String },
    Metric { label: String, binding: Metric },
    Plot { label: String, binding: Plot },
    Gauge { label: String, binding: Gauge },
    StatusStrip { bindings: Vec<Metric> },
    Toolbar { actions: Vec<ToolbarAction> },
    ProcessList { columns: Vec<ProcessColumn> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Metric {
    CpuPercent,
    MemoryUsed,
    MemoryTotal,
    MemoryPercent,
    Processes,
    Uptime,
    NetworkReceived,
    NetworkSent,
    DiskRead,
    DiskWrite,
}

impl Metric {
    pub fn label(self) -> &'static str {
        match self {
            Self::CpuPercent => "CPU",
            Self::MemoryUsed => "Memory used",
            Self::MemoryTotal => "Memory total",
            Self::MemoryPercent => "RAM",
            Self::Processes => "Processes",
            Self::Uptime => "Uptime",
            Self::NetworkReceived => "Receive",
            Self::NetworkSent => "Send",
            Self::DiskRead => "Disk read",
            Self::DiskWrite => "Disk write",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Gauge {
    CpuPercent,
    MemoryPercent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolbarAction {
    Refresh,
    FocusSearch,
    EndProcess,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Plot {
    Cpu,
    Memory,
    NetworkReceived,
    NetworkSent,
    DiskRead,
    DiskWrite,
    /// Two observations on a single percentage axis (0–100%).
    CpuMemory,
    /// Both directions share one dynamically scaled bytes/second axis.
    NetworkTraffic,
    /// Both directions share one dynamically scaled bytes/second axis.
    DiskTraffic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProcessColumn {
    Name,
    Pid,
    Cpu,
    Memory,
    Threads,
}

impl ProcessColumn {
    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "Process",
            Self::Pid => "PID",
            Self::Cpu => "CPU",
            Self::Memory => "Memory",
            Self::Threads => "Threads",
        }
    }
}

impl LayoutSpec {
    pub fn validate(&self) -> Result<(), String> {
        if self
            .compact_below
            .is_some_and(|width| !(480..=1024).contains(&width))
        {
            return Err("layout.compact_below must be 480–1024 logical pixels".into());
        }
        if self.pages.is_empty() || self.pages.len() > MAX_PAGES {
            return Err(format!("layout.pages must contain 1–{MAX_PAGES} screens"));
        }
        let mut ids = HashSet::new();
        for screen in &self.pages {
            if screen.id.is_empty()
                || screen.id.len() > 24
                || !screen
                    .id
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
                || !ids.insert(&screen.id)
            {
                return Err(
                    "layout screen ids must be unique 1–24 character lowercase slugs".into(),
                );
            }
            valid_caption(&screen.title, 40, "layout screen title")?;
            if screen.widgets.is_empty() || screen.widgets.len() > MAX_WIDGETS_PER_PAGE {
                return Err(format!(
                    "layout screen widgets must contain 1–{MAX_WIDGETS_PER_PAGE} items"
                ));
            }
            validate_widgets(&screen.widgets)?;
            if screen.compact_widgets.len() > MAX_WIDGETS_PER_PAGE {
                return Err(format!(
                    "layout compact_widgets must contain at most {MAX_WIDGETS_PER_PAGE} items"
                ));
            }
            validate_widgets(&screen.compact_widgets)?;
        }
        Ok(())
    }
}

fn validate_widgets(widgets: &[Widget]) -> Result<(), String> {
    for widget in widgets {
        match widget {
            Widget::Heading { text } => valid_caption(text, 120, "heading text")?,
            Widget::Metric { label, .. }
            | Widget::Plot { label, .. }
            | Widget::Gauge { label, .. } => {
                valid_caption(label, 48, "widget label")?;
            }
            Widget::StatusStrip { bindings } => {
                if bindings.is_empty() || bindings.len() > 5 {
                    return Err("status-strip requires 1–5 named metrics".into());
                }
                let mut seen = HashSet::new();
                if bindings.iter().any(|binding| !seen.insert(*binding)) {
                    return Err("status-strip contains duplicate metrics".into());
                }
            }
            Widget::Toolbar { actions } => {
                if actions.is_empty() || actions.len() > 3 {
                    return Err("toolbar requires 1–3 predefined actions".into());
                }
                let mut seen = HashSet::new();
                if actions.iter().any(|action| !seen.insert(*action)) {
                    return Err("toolbar contains duplicate actions".into());
                }
            }
            Widget::ProcessList { columns } => {
                if columns.is_empty() || columns.len() > 5 {
                    return Err("process-list requires 1–5 named columns".into());
                }
                if !columns.contains(&ProcessColumn::Name) {
                    return Err("process-list requires a selectable name column".into());
                }
                let mut seen = HashSet::new();
                if columns.iter().any(|column| !seen.insert(*column)) {
                    return Err("process-list contains duplicate columns".into());
                }
            }
        }
    }
    Ok(())
}

fn valid_caption(value: &str, max: usize, field: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.chars().count() > max || value.chars().any(char::is_control)
    {
        return Err(format!("{field} must be 1–{max} printable characters"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_plot_bindings_are_data_only_and_roundtrip() {
        let pack = crate::ricepack::RicePack::from_toml(include_str!(
            "../examples/ricepacks/comparison-studio.toml"
        ))
        .expect("comparison example is a valid v1 RicePack");
        let layout = pack.layout.as_ref().unwrap();
        assert_eq!(layout.pages.len(), 2);
        assert_eq!(layout.pages[0].compact_widgets.len(), 2);
        for (idx, expected) in [Plot::CpuMemory, Plot::NetworkTraffic, Plot::DiskTraffic]
            .into_iter()
            .enumerate()
        {
            assert!(
                matches!(&layout.pages[0].widgets[idx + 1], Widget::Plot { binding, .. } if *binding == expected)
            );
        }
        let encoded = toml::to_string(&pack).unwrap();
        assert_eq!(
            crate::ricepack::RicePack::from_toml(&encoded).unwrap(),
            pack
        );
        assert!(serde_json::from_str::<Plot>("\"arbitrary-process-network\"").is_err());
    }

    #[test]
    fn strict_bounds_and_named_metrics_block_unsafe_layouts() {
        let good = LayoutSpec {
            pages: vec![Screen {
                id: "overview".into(),
                title: "Overview".into(),
                widgets: vec![
                    Widget::Metric {
                        label: "CPU".into(),
                        binding: Metric::CpuPercent,
                    },
                    Widget::ProcessList {
                        columns: vec![ProcessColumn::Name, ProcessColumn::Pid],
                    },
                ],
                compact_widgets: vec![],
            }],
            compact_below: None,
        };
        assert!(good.validate().is_ok());
        let mut bad = good.clone();
        bad.pages[0].widgets.push(Widget::Heading {
            text: "bad\nheader".into(),
        });
        assert!(bad.validate().is_err());
        bad = good.clone();
        bad.pages[0].widgets = vec![good.pages[0].widgets[0].clone(); MAX_WIDGETS_PER_PAGE + 1];
        assert!(bad.validate().is_err());
        bad = good.clone();
        bad.pages.push(good.pages[0].clone());
        assert!(bad.validate().is_err());
        bad = good.clone();
        bad.pages[0].widgets[1] = Widget::ProcessList {
            columns: vec![ProcessColumn::Pid],
        };
        assert!(bad.validate().is_err());
        bad = good.clone();
        bad.pages[0].widgets.push(Widget::Toolbar {
            actions: vec![ToolbarAction::Refresh, ToolbarAction::Refresh],
        });
        assert!(bad.validate().is_err());
        bad = good.clone();
        bad.pages[0].widgets.push(Widget::StatusStrip {
            bindings: vec![Metric::CpuPercent, Metric::CpuPercent],
        });
        assert!(bad.validate().is_err());
        let malicious = r#"{"kind":"metric","label":"CPU","binding":"shell-command"}"#;
        assert!(serde_json::from_str::<Widget>(malicious).is_err());
        let unknown = r#"{"kind":"heading","text":"hi","script":"launch"}"#;
        assert!(serde_json::from_str::<Widget>(unknown).is_err());
        let shell = r#"{"kind":"toolbar","actions":["shell"]}"#;
        assert!(serde_json::from_str::<Widget>(shell).is_err());
        bad = good.clone();
        bad.compact_below = Some(200);
        assert!(bad.validate().is_err());
        bad = good.clone();
        bad.pages[0].compact_widgets = vec![Widget::Heading {
            text: "bad\ncompact heading".into(),
        }];
        assert!(bad.validate().is_err());
        bad = good.clone();
        bad.pages[0].compact_widgets =
            vec![good.pages[0].widgets[0].clone(); MAX_WIDGETS_PER_PAGE + 1];
        assert!(bad.validate().is_err());
        let mut responsive = good.clone();
        responsive.pages[0].compact_widgets = vec![Widget::Heading {
            text: "Compact".into(),
        }];
        assert_eq!(
            responsive.pages[0].widgets_for_width(500.0, 680.0),
            &responsive.pages[0].compact_widgets
        );
        assert_eq!(
            responsive.pages[0].widgets_for_width(900.0, 680.0),
            &responsive.pages[0].widgets
        );
        responsive.pages[0].compact_widgets.clear();
        assert_eq!(
            responsive.pages[0].widgets_for_width(500.0, 680.0),
            &responsive.pages[0].widgets
        );
        let encoded = toml::to_string(&good).unwrap();
        assert!(!encoded.contains("compact_widgets"));
        assert!(!encoded.contains("compact_below"));
    }
}
