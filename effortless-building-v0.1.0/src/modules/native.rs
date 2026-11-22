// In-game point selection for Effortless Building.
// This module records selected block coordinates and draws a read-only spatial
// preview. It never reads block states or changes the world; the authoritative
// world-edit adapter is a separate implementation gate.
use super::{Game, GuestUse};
use horizon_client::command::args::GameMode;
use horizon_client::first_person::Hand;
use horizon_client::text::Text;
use horizon_core::{geometry::{Direction, Vec3}, position::BlockPos};
use horizon_world::level::{ItemKind, PlaceContext};
use horizon_protocol::packets::play::{self, Clientbound as C};
use std::collections::BTreeSet;

std::thread_local! {
    // Reuse both the bounded coordinate plan and line buffer across frames.
    // The plan is rebuilt only when the selection or tool changes.
    static PREVIEW_LINES: std::cell::RefCell<Vec<crate::mod_api::WorldLine>> =
        std::cell::RefCell::new(Vec::with_capacity(204));
    static PREVIEW_PLAN: std::cell::RefCell<Option<(crate::game::effortless_building::SelectionPoints, usize, Vec<[i32; 3]>)>> =
        std::cell::RefCell::new(None);
}


/// One fully validated, read-only target snapshot from the active local world.
/// Kept in the native mixin layer because BlockStateId is an engine type.
pub(super) struct NativePreflightBlock {
    pub pos: [i32; 3],
    /// Keep the engine's registry-scoped identity typed until the explicit
    /// pure-core handoff; never treat this number as globally meaningful.
    pub before_state: horizon_world::registry::BlockStateId,
}

impl NativePreflightBlock {
    /// Exact representation expected by WorldPreflightInput::before_state.
    /// The caller must keep this record within the same loaded registry/world.
    pub(super) fn core_before_state(&self) -> u32 {
        self.before_state.0
    }
}

/// Immutable result of one complete preflight pass. The selection and tool
/// are retained with the snapshots so a future confirmation/commit phase can
/// reject stale UI state instead of silently rebuilding a different plan.
pub(super) struct NativePreflightPlan {
    pub selection: crate::game::effortless_building::SelectionPoints,
    pub tool: usize,
    /// Registry-scoped target state resolved from the offhand block item.
    /// The selection stick stays in the main hand; the offhand is the material palette.
    pub target_state: horizon_world::registry::BlockStateId,
    pub blocks: Vec<NativePreflightBlock>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NativePreflightError {
    NotAuthoritativePlayer,
    InvalidPlan,
    OutsideHeight { index: usize },
    UnloadedChunk { index: usize },
    OutsideBorder { index: usize },
    NoSelectionCornerInReach,
    MissingOffhandStack,
    UnsupportedOffhandBlockItem,
    UnsupportedTargetBlock,
    TargetOccupied { index: usize },
    UnresolvedState { index: usize },
    VanillaPlacementRejected { index: usize },
    PlacementRedirected { index: usize },
    PlacementHasCompanion { index: usize },
    PlacementStateMismatch { index: usize },
}

/// Drop coordinate-only preview allocations when their owning world session ends.
pub(super) fn reset_effortless_building_preview() {
    PREVIEW_PLAN.with(|cache| *cache.borrow_mut() = None);
    PREVIEW_LINES.with(|lines| lines.borrow_mut().clear());
}

impl Game<'_> {
    /// Capture a main-hand block use as one corner of the build selection.
    /// Guests connected to another server are deliberately not intercepted.
    pub(super) fn effortless_building_use(&mut self) -> Option<Option<Hand>> {
        use crate::game::effortless_building as eb;

        if !self.native_authoritative()
            || self.remote.is_some()
            || self.player.is_none()
            || self.spectator()
            || !matches!(self.mode, GameMode::Survival | GameMode::Creative)
            || !self.native_on(eb::ID)
        {
            return None;
        }
        // SelectionPoints is process-global; do not let delegated LAN actors
        // write into the local player's selection or share another actor's points.
        if self.lan_acting.is_some() || !self.native_window() {
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
        let recorded = self.with_player_actor(|game| {
            eb::record_selection_point(pos);
            // Give immediate feedback so touch/right-click input can be
            // distinguished from a silent no-op on mobile.
            let label = eb::selection_label();
            game.eb_tell(&format!("Effortless Building: {label}"));
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
    /// Build the frozen target list from the current selection/tool and collect
    /// its complete read-only snapshot. Call again after confirmation before
    /// any future write phase. Arbitrary caller-supplied target lists are not accepted.
    pub(super) fn effortless_building_preflight_selection(
        &mut self,
    ) -> Result<NativePreflightPlan, NativePreflightError> {
        use crate::game::effortless_building as eb;

        let selection = eb::selection_points();
        let tool = eb::selected_tool(|key| self.native_setting(eb::ID, key));
        let targets = eb::plan_target_positions(selection, tool)
            .map_err(|_| NativePreflightError::InvalidPlan)?;
        if targets.is_empty() || targets.len() > 8_192 {
            return Err(NativePreflightError::InvalidPlan);
        }
        // The planner is specified to emit unique positions in stable order;
        // fail closed if that contract is ever broken.
        let mut seen = BTreeSet::new();
        if targets.iter().any(|pos| !seen.insert(*pos)) {
            return Err(NativePreflightError::InvalidPlan);
        }
        if !self.native_authoritative()
            || self.remote.is_some()
            || self.player.is_none()
            || self.spectator()
            || !matches!(self.mode, GameMode::Survival | GameMode::Creative)
            || !self.native_on(eb::ID)
            || self.lan_acting.is_some()
            || !self.native_window()
        {
            return Err(NativePreflightError::NotAuthoritativePlayer);
        }
        // The main hand is reserved for the selection stick. Resolve the
        // intended material from the offhand block item, without consuming it.
        // Items without a same-named default block state are rejected rather
        // than guessed (some item/block mappings are intentionally non-1:1).
        let target_item = self.offhand.as_ref()
            .filter(|stack| stack.count > 0)
            .ok_or(NativePreflightError::MissingOffhandStack)?;
        let target_state = self.pack.assets.registry
            .default_state(target_item.item.as_ref())
            .filter(|state| {
                self.pack.assets.registry.state(*state).is_some_and(|block| {
                    // Only accept an exact item-ID/block-ID match. Starling's
                    // Pick Block path intentionally maps several blocks to
                    // different items (seeds, crops, stems, potted plants,
                    // piston heads, etc.); guessing the reverse mapping here
                    // could place an unrelated block.
                    block.name.as_str() == target_item.item.as_ref()
                        && block.name.as_str() != "minecraft:air"
                })
            })
            .ok_or(NativePreflightError::UnsupportedOffhandBlockItem)?;
        let target_item_name = target_item.item.as_ref().to_owned();

        // First write milestone is deliberately limited to blocks whose
        // placement does not create a block entity. The target block's
        // metadata is resolved from the same pinned game registry.
        if self.res.facts().get(target_state).block_entity {
            return Err(NativePreflightError::UnsupportedTargetBlock);
        }

        let layout = self.world.layout();
        let reach = self.reach() + 1.0;
        let eye = self.eye_position();
        // A and B are selected by the player's ray pick. At preflight time the
        // player may have walked from A to B, so require at least one corner
        // to remain in reach rather than incorrectly requiring every target
        // block in a large cuboid to be within arm's reach.
        let corner_reachable = |pos: [i32; 3]| {
            eye.is_some_and(|eye| {
                let distance_squared: f64 = (0..3)
                    .map(|axis| (f64::from(pos[axis]) + 0.5 - eye[axis]).powi(2))
                    .sum();
                distance_squared <= reach * reach
            })
        };
        if ![selection.first, selection.second]
            .into_iter()
            .flatten()
            .any(corner_reachable)
        {
            return Err(NativePreflightError::NoSelectionCornerInReach);
        }
        let mut snapshot = Vec::with_capacity(targets.len());
        for (index, pos) in targets.iter().copied().enumerate() {
            if pos[1] < layout.min_y() || pos[1] >= layout.max_y() {
                return Err(NativePreflightError::OutsideHeight { index });
            }
            if !self.world.loaded(pos[0], pos[2]) {
                return Err(NativePreflightError::UnloadedChunk { index });
            }
            if !self.border.contains(f64::from(pos[0]), f64::from(pos[2])) {
                return Err(NativePreflightError::OutsideBorder { index });
            }
            let state_id = self.world.block(pos[0], pos[1], pos[2]);
            if self.pack.assets.registry.state(state_id).is_none() {
                return Err(NativePreflightError::UnresolvedState { index });
            }
            // Never destroy or replace an existing block, including
            // replaceable plants, fluids, snow layers, and other soft blocks.
            // Only air is an empty target; an already-correct target is a no-op.
            if state_id != target_state
                && state_id != self.res.facts().air
            {
                return Err(NativePreflightError::TargetOccupied { index });
            }
            // Preserve the typed, registry-scoped ID at the native boundary.
            snapshot.push(NativePreflightBlock { pos, before_state: state_id });
        }
        // Validate air targets with Starling's own BlockItem placement simulator.
        // The first adapter accepts only property-free, non-block-entity blocks;
        // companion placements remain unsupported until multi-position rollback exists.
        let target_facts = self.res.facts().get(target_state);
        let target_data = self.pack.assets.registry.state(target_state)
            .ok_or(NativePreflightError::UnsupportedTargetBlock)?;
        if !target_data.properties.is_empty() || target_facts.block_entity {
            return Err(NativePreflightError::UnsupportedTargetBlock);
        }
        let (yaw, pitch) = self.interaction_rotation();
        for (index, block) in snapshot.iter().enumerate() {
            if block.before_state == target_state {
                continue; // Already-correct blocks are no-ops.
            }
            let [x, y, z] = block.pos;
            let at = BlockPos::new(x, y, z);
            let context = PlaceContext {
                clicked_pos: at,
                face: Direction::Up,
                hit: Vec3::new(f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5),
                inside: false,
                yaw,
                pitch,
                sneaking: false,
                item: target_item_name.clone(),
                can_use_game_master_blocks: false,
            };
            let preview = self.with_level(|behaviours, level, ticks| {
                // The initial transaction supports only ordinary BlockItem
                // bindings. Special item subclasses can place fluids, redirect
                // placement, or perform extra writes outside the preview cell.
                if behaviours.item_binding(&context.item)
                    .is_none_or(|binding| binding.kind != ItemKind::Block)
                {
                    return None;
                }
                behaviours.placement_preview(level, ticks, &context)
            }).flatten().ok_or(NativePreflightError::VanillaPlacementRejected { index })?;
            if preview.pos != at {
                return Err(NativePreflightError::PlacementRedirected { index });
            }
            if preview.companion.is_some() {
                return Err(NativePreflightError::PlacementHasCompanion { index });
            }
            if preview.state != target_state {
                return Err(NativePreflightError::PlacementStateMismatch { index });
            }
        }

        Ok(NativePreflightPlan {
            selection,
            tool,
            target_state,
            blocks: snapshot,
        })
    }
}

impl Game<'_> {
    /// Re-check a frozen preflight immediately before a future commit phase.
    /// This is deliberately read-only: it rejects stale selection, authority,
    /// world bounds, reach, unloaded chunks, and any block changed since capture.
    pub(super) fn effortless_building_preflight_is_current(
        &self,
        plan: &NativePreflightPlan,
    ) -> bool {
        use crate::game::effortless_building as eb;

        if !self.native_authoritative()
            || self.remote.is_some()
            || self.player.is_none()
            || self.spectator()
            || !matches!(self.mode, GameMode::Survival | GameMode::Creative)
            || !self.native_on(eb::ID)
            || self.lan_acting.is_some()
            || !self.native_window()
            || eb::selection_points() != plan.selection
            || eb::selected_tool(|key| self.native_setting(eb::ID, key)) != plan.tool
            || plan.blocks.is_empty()
            || plan.blocks.len() > 8_192
        {
            return false;
        }

        let Some(target_item) = self.offhand.as_ref().filter(|stack| stack.count > 0) else {
            return false;
        };
        let Some(target_state) = self.pack.assets.registry
            .default_state(target_item.item.as_ref())
            .filter(|state| {
                self.pack.assets.registry.state(*state).is_some_and(|block| {
                    block.name.as_str() == target_item.item.as_ref()
                        && block.name.as_str() != "minecraft:air"
                })
            })
        else {
            return false;
        };
        if target_state != plan.target_state
            || self.res.facts().get(target_state).block_entity
        {
            return false;
        }

        let layout = self.world.layout();
        let reach = self.reach() + 1.0;
        let eye = self.eye_position();
        let corner_reachable = |pos: [i32; 3]| {
            eye.is_some_and(|eye| {
                let distance_squared: f64 = (0..3)
                    .map(|axis| (f64::from(pos[axis]) + 0.5 - eye[axis]).powi(2))
                    .sum();
                distance_squared <= reach * reach
            })
        };
        if ![plan.selection.first, plan.selection.second]
            .into_iter()
            .flatten()
            .any(corner_reachable)
        {
            return false;
        }
        for block in &plan.blocks {
            let [x, y, z] = block.pos;
            if y < layout.min_y()
                || y >= layout.max_y()
                || !self.world.loaded(x, z)
                || !self.border.contains(f64::from(x), f64::from(z))
            {
                return false;
            }
            let current_state = self.world.block(x, y, z);
            if current_state != block.before_state
                || self.pack.assets.registry.state(current_state).is_none()
                || (current_state != target_state
                    && current_state != self.res.facts().air)
            {
                return false;
            }
        }
        true
    }
}

impl Game<'_> {
    /// Draw the exact bounded planner geometry. The cached plan is rebuilt
    /// only after selection/tool changes, avoiding per-frame coordinate work.
    /// Wall is intentionally omitted until its plane semantics are finalized.
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
        let tool = eb::selected_tool(|key| self.native_setting(eb::ID, key));
        let valid = PREVIEW_PLAN.with(|cache| {
            let mut cache = cache.borrow_mut();
            let rebuild = cache.as_ref().map_or(true, |(s, t, _)| *s != selection || *t != tool);
            if rebuild {
                let plan = eb::plan_target_positions(selection, tool).ok()?;
                *cache = Some((selection, tool, plan));
            }
            Some(cache.as_ref().map(|(_, _, plan)| !plan.is_empty()).unwrap_or(false))
        }).unwrap_or(false);
        if !valid {
            return;
        }

        PREVIEW_PLAN.with(|cache| {
            let cache = cache.borrow();
            let Some((_, _, plan)) = cache.as_ref() else { return; };
            if plan.is_empty() { return; }

            let mut min = plan[0];
            let mut max = plan[0];
            for p in plan.iter().skip(1) {
                for axis in 0..3 {
                    min[axis] = min[axis].min(p[axis]);
                    max[axis] = max[axis].max(p[axis]);
                }
            }

            let corners = [
                [min[0], min[1], min[2]], [max[0], min[1], min[2]],
                [min[0], max[1], min[2]], [max[0], max[1], min[2]],
                [min[0], min[1], max[2]], [max[0], min[1], max[2]],
                [min[0], max[1], max[2]], [max[0], max[1], max[2]],
            ];
            const EDGES: [(usize, usize); 12] = [
                (0,1),(0,2),(0,4),(1,3),(1,5),(2,3),
                (2,6),(3,7),(4,5),(4,6),(5,7),(6,7),
            ];
            let world = |p: [i32; 3]| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])];
            PREVIEW_LINES.with(|buffer| {
                let mut lines = buffer.borrow_mut();
                lines.clear();
                for (from, to) in EDGES {
                    lines.push(WorldLine::new(
                        world(corners[from]), world(corners[to]),
                        0xFF42D69A, 2.0, WorldDepth::Tested,
                    ));
                }

                // At most 64 planned positions, evenly sampled across the
                // complete deterministic plan. Each marker is three short lines.
                for index in eb::sampled_plan_indices(plan.len(), 64) {
                    let p = plan[index];
                    let center = [f64::from(p[0]) + 0.5, f64::from(p[1]) + 0.5, f64::from(p[2]) + 0.5];
                    let d = 0.18;
                    lines.push(WorldLine::new([center[0]-d,center[1],center[2]],[center[0]+d,center[1],center[2]],0xDFFFFFFF,1.5,WorldDepth::Tested));
                    lines.push(WorldLine::new([center[0],center[1]-d,center[2]],[center[0],center[1]+d,center[2]],0xDFFFFFFF,1.5,WorldDepth::Tested));
                    lines.push(WorldLine::new([center[0],center[1],center[2]-d],[center[0],center[1],center[2]+d],0xDFFFFFFF,1.5,WorldDepth::Tested));
                }
                let _ = draw.lines(&lines);
            });
        });
    }
}

impl Game<'_> {
    /// Show selection feedback in the acting player's action bar. A LAN guest
    /// receives the same overlay through the normal server chat packet path.
    fn eb_tell(&mut self, text: &str) {
        let line = Text::literal(text.to_owned());
        if self.lan_acting.is_some() {
            self.guest_packets.push(C::SystemChat(play::SystemChat {
                content: horizon_client::lan::data::component_nbt(&line.component()),
                overlay: true,
            }));
        } else {
            self.titles.set_overlay(line, false);
        }
    }
}
