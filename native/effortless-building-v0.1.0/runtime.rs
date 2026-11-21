// Effortless Building native integration for Starling 0.9.0.
// The screen is a UI-only milestone: tool choice is persisted, but this
// module still does not read or modify blocks in the world.

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Mutex;

pub const ID: &str = "effortless_building";
pub const NAME: &str = "Effortless Building";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str =
    "Mobile-first building tools: Floor, Wall, Pillar and Cube; select two world points with right-click.";

pub const TOOLS: [&str; 4] = ["Floor", "Wall", "Pillar", "Cube"];
pub const ACTION_CLOSE: i32 = 4;
pub const SELECTION_ITEM: &str = "minecraft:stick";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectionPoints {
    pub first: Option<[i32; 3]>,
    pub second: Option<[i32; 3]>,
}

impl SelectionPoints {
    /// First point sets A, second sets B, and a third starts a fresh pair.
    pub fn record(&mut self, pos: [i32; 3]) {
        match (self.first, self.second) {
            (None, _) => {
                self.first = Some(pos);
                self.second = None;
            }
            (Some(_), None) => self.second = Some(pos),
            (Some(_), Some(_)) => {
                self.first = Some(pos);
                self.second = None;
            }
        }
    }
}

static SELECTION_POINTS: Mutex<SelectionPoints> = Mutex::new(SelectionPoints {
    first: None,
    second: None,
});

/// Record a point from the authoritative in-game block ray pick.
pub fn record_selection_point(pos: [i32; 3]) {
    SELECTION_POINTS.lock().unwrap_or_else(|e| e.into_inner()).record(pos);
}

pub fn selection_points() -> SelectionPoints {
    *SELECTION_POINTS.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn clear_selection_points() {
    *SELECTION_POINTS.lock().unwrap_or_else(|e| e.into_inner()) = SelectionPoints::default();
}

pub fn selection_block_count_for(selection: SelectionPoints, tool: usize) -> Option<u128> {
    let (a, b) = (selection.first?, selection.second?);
    let span = |x: i32, y: i32| -> u128 {
        (i64::from(x.max(y)) - i64::from(x.min(y)) + 1) as u128
    };
    let dx = span(a[0], b[0]);
    let dy = span(a[1], b[1]);
    let dz = span(a[2], b[2]);
    match tool {
        0 => dx.checked_mul(dz),
        1 | 3 => dx.checked_mul(dy)?.checked_mul(dz),
        2 => Some(dy),
        _ => None,
    }
}

pub fn selection_block_count(tool: usize) -> Option<u128> {
    selection_block_count_for(selection_points(), tool)
}

pub fn selection_label_for_tool(tool: usize) -> String {
    let mut label = selection_label();
    if let Some(count) = selection_block_count(tool) {
        if count > 16_384 {
            label.push_str(&format!(" | {count} blocks (limit 16384)"));
        } else {
            label.push_str(&format!(" | {count} blocks"));
        }
    }
    label
}

pub fn selection_label() -> String {
    let state = selection_points();
    match (state.first, state.second) {
        (None, _) => "Point A: not selected".to_owned(),
        (Some(a), None) => format!("A: {}, {}, {}  |  select B", a[0], a[1], a[2]),
        (Some(a), Some(b)) => format!(
            "A: {}, {}, {}  B: {}, {}, {}",
            a[0], a[1], a[2], b[0], b[1], b[2]
        ),
    }
}

const NO_ACTION: i32 = -1;
static PENDING_UI_ACTION: AtomicI32 = AtomicI32::new(NO_ACTION);
static CAPTURED_LEFT_PRESS: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiLayout {
    pub panel: [f32; 4],
    pub tools: [[f32; 4]; 4],
    pub close: [f32; 4],
    pub footer_y: f32,
}

/// Return the persisted tool index, defaulting to Floor for invalid values.
pub fn selected_tool(read: impl Fn(&str) -> Option<String>) -> usize {
    read("tool")
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|i| *i < TOOLS.len())
        .unwrap_or(0)
}

/// The settings page keeps the four quick selectors and opens the full builder.
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
    labels.push("Open Builder".to_owned());
    labels
}

/// Layout is expressed in GUI pixels and adapts to the current screen rectangle.
pub fn ui_layout(rect: [f32; 4]) -> UiLayout {
    let x = if rect[0].is_finite() { rect[0] } else { 0.0 };
    let y = if rect[1].is_finite() { rect[1] } else { 0.0 };
    let width = if rect[2].is_finite() { rect[2].max(1.0) } else { 1.0 };
    let height = if rect[3].is_finite() { rect[3].max(1.0) } else { 1.0 };

    let panel_w = width.min(340.0).max(width.min(180.0));
    let panel_h = height.min(236.0).max(height.min(180.0));
    let scale = (panel_h / 236.0).clamp(0.5, 1.0);
    let panel_x = x + (width - panel_w) * 0.5;
    let panel_y = y + (height - panel_h) * 0.5;
    let button_w = ((panel_w - 44.0) * 0.5).max(0.0);
    let button_h = 34.0 * scale;
    let left = panel_x + 14.0;
    let right = left + button_w + 8.0;

    UiLayout {
        panel: [panel_x, panel_y, panel_w, panel_h],
        tools: [
            [left, panel_y + 68.0 * scale, button_w, button_h],
            [right, panel_y + 68.0 * scale, button_w, button_h],
            [left, panel_y + 110.0 * scale, button_w, button_h],
            [right, panel_y + 110.0 * scale, button_w, button_h],
        ],
        close: [
            panel_x + 14.0,
            panel_y + 156.0 * scale,
            (panel_w - 28.0).max(0.0),
            button_h,
        ],
        footer_y: panel_y + 202.0 * scale,
    }
}

pub fn contains(rect: [f32; 4], point: [f32; 2]) -> bool {
    point[0] >= rect[0]
        && point[1] >= rect[1]
        && point[0] <= rect[0] + rect[2]
        && point[1] <= rect[1] + rect[3]
}

/// Map a screen tap to a tool index or the close action.
pub fn ui_action_at(layout: &UiLayout, point: [f32; 2]) -> Option<i32> {
    for (index, rect) in layout.tools.iter().enumerate() {
        if contains(*rect, point) {
            return Some(index as i32);
        }
    }
    contains(layout.close, point).then_some(ACTION_CLOSE)
}

pub fn queue_ui_action(action: i32) {
    if (0..=ACTION_CLOSE).contains(&action) {
        PENDING_UI_ACTION.store(action, Ordering::Release);
    }
}

pub fn capture_left_press() {
    CAPTURED_LEFT_PRESS.store(true, Ordering::Release);
}

pub fn release_captured_left_press() -> bool {
    CAPTURED_LEFT_PRESS.swap(false, Ordering::AcqRel)
}

/// Apply UI intent only on the window's Game instance, never on the server tick.
pub fn process_pending_ui_action(game: &mut crate::game::Game<'_>) {
    if !game.native_window() {
        return;
    }

    match PENDING_UI_ACTION.swap(NO_ACTION, Ordering::AcqRel) {
        action if (0..TOOLS.len() as i32).contains(&action) => {
            game.native_set_setting(ID, "tool", &action.to_string());
        }
        ACTION_CLOSE => {
            game.native_mod_screen_close(ID);
            CAPTURED_LEFT_PRESS.store(false, Ordering::Release);
        }
        _ => {}
    }
}

/// Draw the dedicated builder screen. This is intentionally only a coordinate
/// and tool-selection screen; no ghost blocks or world edits are claimed here.
pub fn draw_screen(
    game: &crate::game::Game<'_>,
    batch: &mut crate::mod_api::HudBatch,
    screen: &crate::mod_api::NativeScreenView<'_>,
) {
    if screen.id != "mod" || game.native_mod_screen() != Some(ID) {
        return;
    }

    let layout = ui_layout(screen.rect);
    let [px, py, pw, ph] = layout.panel;
    game.native_fill(batch, layout.panel, 0xF0121B27);
    game.native_fill(batch, [px, py, pw, 2.0], 0xFF4B657E);
    game.native_fill(batch, [px, py + ph - 2.0, pw, 2.0], 0xFF4B657E);
    game.native_fill(batch, [px, py, 2.0, ph], 0xFF4B657E);
    game.native_fill(batch, [px + pw - 2.0, py, 2.0, ph], 0xFF4B657E);

    game.native_text(batch, "Effortless Building", [px + 14.0, py + 14.0], 0xFFFFFF, true);
    game.native_text(batch, "Read-only outline and distributed preview markers", [px + 14.0, py + 36.0], 0xC8D4E0, false);

    let selected = selected_tool(|key| game.native_setting(ID, key));
    for (index, rect) in layout.tools.iter().enumerate() {
        let active = index == selected;
        game.native_fill(
            batch,
            *rect,
            if active { 0xFF245B43 } else { 0xFF26394C },
        );
        let label = if active {
            format!("[x] {}", TOOLS[index])
        } else {
            format!("[ ] {}", TOOLS[index])
        };
        let tx = rect[0] + (rect[2] - game.native_text_width(&label)) * 0.5;
        let ty = rect[1] + (rect[3] - 9.0) * 0.5;
        game.native_text(batch, &label, [tx, ty], 0xFFFFFF, true);
    }

    game.native_fill(batch, layout.close, 0xFF613B43);
    let close_label = "Close";
    let close_x = layout.close[0]
        + (layout.close[2] - game.native_text_width(close_label)) * 0.5;
    game.native_text(
        batch,
        close_label,
        [close_x, layout.close[1] + (layout.close[3] - 9.0) * 0.5],
        0xFFFFFF,
        true,
    );

    game.native_text(
        batch,
        &selection_label_for_tool(selected),
        [px + 14.0, layout.footer_y - 18.0],
        0xD6E2EE,
        false,
    );
    game.native_text(
        batch,
        "Hold a stick; close and right-click two blocks to set A and B",
        [px + 14.0, layout.footer_y],
        0xB8C4D0,
        false,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_have_four_tools_and_builder_button() {
        let labels = setting_labels(2);
        assert_eq!(labels.len(), 5);
        assert_eq!(&labels[..4], ["[ ] Floor", "[ ] Wall", "[x] Pillar", "[ ] Cube"]);
        assert_eq!(labels[4], "Open Builder");
    }

    #[test]
    fn settings_clamp_invalid_selection() {
        assert_eq!(setting_labels(99)[3], "[x] Cube");
    }

    #[test]
    fn layout_places_all_buttons_inside_the_panel() {
        let layout = ui_layout([0.0, 0.0, 854.0, 480.0]);
        assert!(layout.panel[0] >= 0.0 && layout.panel[1] >= 0.0);
        for rect in layout.tools.into_iter().chain([layout.close]) {
            assert!(rect[0] >= layout.panel[0]);
            assert!(rect[1] >= layout.panel[1]);
            assert!(rect[0] + rect[2] <= layout.panel[0] + layout.panel[2]);
            assert!(rect[1] + rect[3] <= layout.panel[1] + layout.panel[3]);
        }
    }

    #[test]
    fn ui_hit_testing_returns_tool_and_close_actions() {
        let layout = ui_layout([0.0, 0.0, 854.0, 480.0]);
        let center = |r: [f32; 4]| [r[0] + r[2] * 0.5, r[1] + r[3] * 0.5];
        assert_eq!(ui_action_at(&layout, center(layout.tools[2])), Some(2));
        assert_eq!(ui_action_at(&layout, center(layout.close)), Some(ACTION_CLOSE));
        assert_eq!(ui_action_at(&layout, [0.0, 0.0]), None);
    }
}


#[cfg(test)]
mod selection_point_tests {
    use super::*;

    #[test]
    fn two_points_are_captured_and_third_click_starts_a_new_pair() {
        let mut selection = SelectionPoints::default();
        selection.record([1, 2, 3]);
        assert_eq!(selection, SelectionPoints { first: Some([1, 2, 3]), second: None });
        selection.record([4, 5, 6]);
        assert_eq!(selection, SelectionPoints { first: Some([1, 2, 3]), second: Some([4, 5, 6]) });
        selection.record([-7, 8, 9]);
        assert_eq!(selection, SelectionPoints { first: Some([-7, 8, 9]), second: None });
    }
}


#[cfg(test)]
mod selection_count_tests {
    use super::*;

    #[test]
    fn selected_tool_counts_match_two_corner_geometry() {
        let selection = SelectionPoints {
            first: Some([1, 2, 3]),
            second: Some([2, 4, 6]),
        };
        assert_eq!(selection_block_count_for(selection, 0), Some(8));
        assert_eq!(selection_block_count_for(selection, 1), Some(24));
        assert_eq!(selection_block_count_for(selection, 2), Some(3));
        assert_eq!(selection_block_count_for(selection, 3), Some(24));
    }

    #[test]
    fn selection_count_uses_wide_checked_arithmetic() {
        let selection = SelectionPoints {
            first: Some([i32::MIN, i32::MIN, i32::MIN]),
            second: Some([i32::MAX, i32::MAX, i32::MAX]),
        };
        assert_eq!(
            selection_block_count_for(selection, 3),
            Some(1u128 << 96)
        );
    }
}
