// In-game point selection for Effortless Building.
// This module records selected block coordinates and draws a read-only spatial
// preview. It never reads block states or changes the world; the authoritative
// world-edit adapter is a separate implementation gate.
use super::{Game, GuestUse};
use horizon_client::first_person::Hand;

std::thread_local! {
    // Reuse the preview batch between frames; the geometry renderer also owns
    // fixed-capacity per-view storage, so this avoids a per-frame heap alloc.
    static PREVIEW_LINES: std::cell::RefCell<Vec<crate::mod_api::WorldLine>> =
        std::cell::RefCell::new(Vec::with_capacity(204));
}

impl Game<'_> {
    /// Capture a main-hand block use as one corner of the build selection.
    /// Guests connected to another server are deliberately not intercepted.
    pub(super) fn effortless_building_use(&mut self) -> Option<Option<Hand>> {
        use crate::game::effortless_building as eb;

        if self.remote.is_some()
            || self.player.is_none()
            || self.spectator()
            || !self.native_on(eb::ID)
        {
            return None;
        }
        if self.lan_acting.is_none() && !self.native_window() {
            return None;
        }
        match self.guest_use {
            Some(GuestUse::OnBlock(Hand::Main)) => {}
            None if !self.entity_targeted() => {}
            _ => return None,
        }

        let held = self.inventory.get(self.selected).and_then(Option::as_ref)?;
        if *held.item != *eb::SELECTION_ITEM {
            return None;
        }

        let hit = self.pick()?;
        let pos = hit.pos;
        let recorded = self.with_player_actor(|_game| {
            eb::record_selection_point(pos);
            true
        });
        if !recorded {
            return None;
        }
        self.swing_arm();
        Some(Some(Hand::Main))
    }
}


impl Game<'_> {
    /// Draw a bounded wireframe and evenly distributed sample crosses for the
    /// current two-corner selection. This hook is strictly read-only.
    pub(super) fn effortless_building_preview(
        &self,
        _view: &crate::mod_api::WorldRenderView,
        draw: &mut horizon_client::world_geometry::WorldGeometry,
    ) {
        use crate::game::effortless_building as eb;
        use crate::mod_api::{WorldDepth, WorldLine};

        if !self.native_on(eb::ID) {
            return;
        }
        let selection = eb::selection_points();
        let (Some(a), Some(b)) = (selection.first, selection.second) else {
            return;
        };

        let low_y = a[1].min(b[1]);
        let high_y = a[1].max(b[1]);
        let tool = eb::selected_tool(|key| self.native_setting(eb::ID, key));
        let (min, max) = match tool {
            0 => ([a[0].min(b[0]), low_y, a[2].min(b[2])],
                  [a[0].max(b[0]), low_y, a[2].max(b[2])]),
            1 => ([a[0].min(b[0]), low_y, a[2].min(b[2])],
                  [a[0].max(b[0]), high_y, a[2].max(b[2])]),
            2 => ([a[0], low_y, a[2]], [a[0], high_y, a[2]]),
            _ => ([a[0].min(b[0]), low_y, a[2].min(b[2])],
                  [a[0].max(b[0]), high_y, a[2].max(b[2])]),
        };

        let corners = [
            [min[0], min[1], min[2]],
            [max[0], min[1], min[2]],
            [min[0], max[1], min[2]],
            [max[0], max[1], min[2]],
            [min[0], min[1], max[2]],
            [max[0], min[1], max[2]],
            [min[0], max[1], max[2]],
            [max[0], max[1], max[2]],
        ];
        const EDGES: [(usize, usize); 12] = [
            (0, 1), (0, 2), (0, 4), (1, 3), (1, 5), (2, 3),
            (2, 6), (3, 7), (4, 5), (4, 6), (5, 7), (6, 7),
        ];
        let world = |p: [i32; 3]| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])];
        PREVIEW_LINES.with(|buffer| {
            let mut lines = buffer.borrow_mut();
            lines.clear();
            for (from, to) in EDGES {
                lines.push(WorldLine::new(
                    world(corners[from]),
                    world(corners[to]),
                    0xFF42D69A,
                    2.0,
                    WorldDepth::Tested,
                ));
            }

            // Four evenly spaced samples per non-degenerate axis (at most 64
            // crosses), including both ends so the whole selected volume is shown.
            let samples = |lo: i32, hi: i32| -> ([i32; 4], usize) {
                if lo == hi {
                    return ([lo; 4], 1);
                }
                let mut out = [lo; 4];
                for (i, value) in out.iter_mut().enumerate() {
                    *value = (i64::from(lo)
                        + (i64::from(hi) - i64::from(lo)) * i as i64 / 3) as i32;
                }
                (out, 4)
            };
            let (xs, nx) = samples(min[0], max[0]);
            let (ys, ny) = samples(min[1], max[1]);
            let (zs, nz) = samples(min[2], max[2]);
            for x in &xs[..nx] {
                for y in &ys[..ny] {
                    for z in &zs[..nz] {
                        let center = [f64::from(*x) + 0.5, f64::from(*y) + 0.5, f64::from(*z) + 0.5];
                        let d = 0.18;
                        lines.push(WorldLine::new(
                            [center[0] - d, center[1], center[2]],
                            [center[0] + d, center[1], center[2]],
                            0xDFFFFFFF, 1.5, WorldDepth::Tested,
                        ));
                        lines.push(WorldLine::new(
                            [center[0], center[1] - d, center[2]],
                            [center[0], center[1] + d, center[2]],
                            0xDFFFFFFF, 1.5, WorldDepth::Tested,
                        ));
                        lines.push(WorldLine::new(
                            [center[0], center[1], center[2] - d],
                            [center[0], center[1], center[2] + d],
                            0xDFFFFFFF, 1.5, WorldDepth::Tested,
                        ));
                    }
                }
            }
            // WorldGeometry validates the whole batch atomically and enforces its
            // fixed per-view line capacity. A refusal draws nothing from this batch.
            let _ = draw.lines(&lines);
        });
    }
}
