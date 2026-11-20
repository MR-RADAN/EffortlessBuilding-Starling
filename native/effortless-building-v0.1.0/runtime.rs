//! Effortless Building native integration scaffold for Starling 0.8.0.
//! This first bundle stage only registers the mod and its settings page.
//! World edits are intentionally not exposed until their exact hooks compile.

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
    vec![
        format!("Tool: {}", TOOLS[tool.min(TOOLS.len() - 1)]),
        "Build preview: not connected yet".to_owned(),
        "Placement: disabled until native edit hook is verified".to_owned(),
    ]
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
        assert_eq!(setting_labels(3)[0], "Tool: Cube");
    }
}
