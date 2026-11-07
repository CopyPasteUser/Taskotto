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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Screen {
    pub id: String,
    pub title: String,
    pub widgets: Vec<Widget>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Widget {
    Heading { text: String },
    Metric { label: String, binding: Metric },
    Plot { label: String, binding: Plot },
    ProcessList { columns: Vec<ProcessColumn> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Plot {
    Cpu,
    Memory,
    NetworkReceived,
    NetworkSent,
    DiskRead,
    DiskWrite,
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
            for widget in &screen.widgets {
                match widget {
                    Widget::Heading { text } => valid_caption(text, 120, "heading text")?,
                    Widget::Metric { label, .. } | Widget::Plot { label, .. } => {
                        valid_caption(label, 48, "widget label")?;
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
        }
        Ok(())
    }
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
            }],
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
        let malicious = r#"{"kind":"metric","label":"CPU","binding":"shell-command"}"#;
        assert!(serde_json::from_str::<Widget>(malicious).is_err());
        let unknown = r#"{"kind":"heading","text":"hi","script":"launch"}"#;
        assert!(serde_json::from_str::<Widget>(unknown).is_err());
    }
}
