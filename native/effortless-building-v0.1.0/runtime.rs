//! Effortless Building native integration scaffold for Starling 0.8.0.
//! The settings page exposes tool selection; world editing remains gated
//! until the Starling native edit hooks are wired and compiled.

pub const ID: &str = "effortless_building";
pub const NAME: &str = "Effortless Building";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Native building tools: Floor, Wall, Pillar and Cube. Preview and confirmation workflow.";

pub const TOOLS: [&str; 4] = ["Floor", "Wall", "Pillar", "Cube"];

pub fn selected_tool(read: impl Fn(&str) -> Option<String>) -> usize {
    read("tool")
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|i| *i < TOOLS.len())
        .unwrap_or(0)
}

pub fn setting_labels(tool: usize) -> Vec<String> {
    let selected = tool.min(TOOLS.len() - 1);
    let mut labels: Vec<String> = TOOLS
        .iter()
        .enumerate()
        .map(|(index, name)| {
            if index == selected {
                format!("[x] {name}")
            } else {
                format!("[ ] {name}")
            }
        })
        .collect();
    labels.push("Preview: not connected yet".to_owned());
    labels.push("Placement: locked until native hook validation".to_owned());
    labels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_setting_is_bounded() {
        assert_eq!(selected_tool(|_| None), 0);
        assert_eq!(selected_tool(|_| Some("3".into())), 3);
        assert_eq!(selected_tool(|_| Some("4".into())), 0);
        assert_eq!(selected_tool(|_| Some("-1".into())), 0);
    }

    #[test]
    fn settings_show_all_four_tools_and_mark_selection() {
        let labels = setting_labels(2);
        assert_eq!(labels.len(), 6);
        assert_eq!(&labels[..4], ["[ ] Floor", "[ ] Wall", "[x] Pillar", "[ ] Cube"]);
    }

    #[test]
    fn settings_clamp_invalid_selection() {
        assert_eq!(setting_labels(99)[3], "[x] Cube");
    }
}
