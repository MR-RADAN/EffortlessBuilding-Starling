# World-edit adapter — Starling 0.9.0

Creator: MR  
Mod version: 0.1.0  
Target revision: `ff303502a4f7982551a0e0d3622d9268efe9e829`

## Verified integration route

This project is packaged as a legacy `.hnmod` `horizon-mixins` bundle. It
must not pretend that `Hooks::server_tick(&mut dyn ServerHost)` is available
inside a client `Game<'_>` injection. That is a separate Horizon code-mod API.

For this package format, the verified Starling 0.9.0 reference is the built-in
Wrench native mod at `native-mods/wrench/mixins/wrench/native.rs`. It handles
a player's action in `Game::use_item`, rejects remote clients, and accesses
the acting local single-player/LAN-host world through the same `Game`
instance. It reads `self.world`, checks `self.world.loaded(...)`, and invokes
block behaviour through `self.with_level(|behaviours, level, ticks| ...)`.
The Wrench implementation uses `Behaviours::set_block` /
`destroy_block` with `horizon_world::level::flags::UPDATE_ALL`.

This is the implementation route for Effortless Building 0.1.0. Do not add
invented `ServerHost` parameters or a second crate that the current importer
does not load.

## Authority and scope

- Only the local single-player game or the acting LAN host may mutate.
- Reject `self.remote.is_some()`, missing player, spectator, and non-acting
  server/window contexts, following Wrench's guards.
- A LAN guest may request an action only through the normal acting-player
  path; never mutate the guest's local client copy of a remote server world.
- Resolve and validate the entire target set before the first write.
- Enforce the hard cap of 8,192 unique target positions; show a large-build warning above 4,096.
- Reject out-of-height, unloaded, out-of-border, and otherwise invalid targets.
- Require a distinct explicit confirmation action after selection/preview.
- Keep the selection tool and preview read-only; no block changes on point
  selection, menu opening, tool selection, or preview rendering.

## Transaction design

The Wrench API demonstrates a block-behaviour write path, not a bulk atomic
transaction. Starling 0.9.0's public mixin route does not expose the
`ServerHost` snapshot/edit contract described by the separate mod API.

Before enabling bulk placement, the adapter must:

1. Freeze an immutable list of unique target coordinates and the intended
   complete `BlockState`; reject empty/oversized selections.
2. Verify every target is inside the world's Y layout, loaded, and within the
   world border. Do not implicitly load chunks.
3. Snapshot each current `BlockStateId` and resolve it through the block
   registry. Abort if any state cannot be resolved.
4. Validate the replacement state and all target-specific survival/shape
   constraints before mutation where the behaviour API permits.
5. Apply through `with_level` and `Behaviours::set_block` using the engine's
   update flags. Count actual successful writes; never report the requested
   count as the committed count.
6. If a write fails after earlier writes succeeded, attempt a guarded
   best-effort restoration only where the current state still equals the
   state written by this operation. Report partial failure explicitly; do not
   claim atomicity or guaranteed rollback.
7. Record undo data only for positions whose writes actually succeeded.
   Undo/redo must re-check each current state against the transaction's
   expected state before attempting a guarded change.

The native registry path exposes `Game::item_block(ItemKey) -> Option<BlockKey>` and `Game::block_state_default(BlockKey) -> Option<BlockStateId>` in Starling 0.9.0. These establish a registry-backed route from a held block item to its default block state, including special item-to-block bindings such as seeds to crops. They are currently private to Starling's modding implementation and have not yet been proven callable from this mixin module; do not duplicate or guess their internal mapping. Confirm visibility or use a supported registry/behaviour binding before wiring material selection.\n\nThe engine API may trigger neighbour updates and block behaviour side effects.
A block entity's NBT, inventories, scheduled ticks, and arbitrary side effects
must not be claimed as preserved unless separately proven. Inventory accounting is intentionally out of scope for the selected unlimited-block mode: placement must not require or consume an inventory stack. This does not remove the world-edit safety limits or authorize writes before validation and confirmation.

## Held-item resolver investigation

The held inventory entry used by the existing use_item hook is an engine ItemStack; its item field is the canonical namespaced item identifier (string-like), as confirmed by Starling's held-item probe and Wrench hook. The native registry helper Game::item_block does not accept that identifier: it accepts an opaque ItemKey, then resolves its registered identifier via mod_api::__private::item_key_id. The vocabulary crate's exposed token constructor constructs an ItemKey from a numeric registration slot, not from an item identifier. No verified string-to-ItemKey conversion was found in the pinned revision.

Therefore the next safe adapter must resolve the held identifier through the engine behaviour table's item-to-block binding (the same binding item_block consumes) or use a supported key lookup. Do not fabricate a numeric token from a registry index: registration slots and runtime item registry order are not proven equivalent. The resolved block identifier can then be converted with Game::block_key, followed by Game::block_state_default. The current build remains read-only until that path and the per-target validation compile against the pinned source.

## Current implementation state

The current client mixin records two points, exposes four tool selectors, and
draws a bounded read-only wireframe/sample preview. It does not yet perform
bulk world edits, and the UI does not yet contain the required confirmation
and material-selection flow. The ARM64 native build compiles the mixin bundle
into Starling's Android library; a successful compile alone is not proof of
in-game functionality. Device testing on Starling 0.9.0 is still required.
\n\n## Current verification update\n\nThe core test workflow passed on the current `main` revision (54 tests). The native integration remains a separate compile-and-device-test gate: no block writes are enabled in this revision.\n

## Pinned-revision API check — unlimited placement

The pinned Horizon revision contains a bulk API, but it is not callable from
this legacy client mixin route:

- `Game::edit_blocks` is a server-side helper for the authoritative level
  being ticked.
- Its public mod-facing counterpart is `WorldEditServerHost::edit_blocks`,
  declared with `#[access(WorldOnly)]` and reached through `ServerHost`.
- The current `.hnmod` mixin is injected into client `Game<'_>`; it does not
  receive a `ServerHost`. Calling this API here would cross the wrong
  authority boundary and must not be attempted.
- Horizon's bulk helper also has different limits and run semantics; it does
  not replace this project's 8,192-unique-position cap or the required
  explicit confirmation.

Accordingly, the local single-player/LAN-host mixin must use the verified
Wrench-style `with_level` + `Behaviours::set_block` route, after a complete
read-only planning pass. Unlimited mode means no inventory lookup or
consumption; it does not mean unbounded work. Placement remains disabled
until target validation, confirmation, and safe bounded application are
implemented and compile-checked.


## Bounded target planner milestone

The core runtime now has a pure `plan_target_positions` helper. It derives
unique coordinates for Floor, Wall, Pillar, and Cube selections in stable
X/Y/Z order, computes the target count before allocating, and rejects
incomplete selections, invalid tools, and any plan above 8,192 positions.
The helper is read-only: it does not inspect chunks, resolve block states,
or mutate the world. The 4,096 large-build warning remains a UI concern.

This is a planning milestone only. World-boundary/chunk-loaded checks,
replacement-state validation, explicit confirmation, batched writes, and
undo bookkeeping are not implemented. No placement is enabled.


## Integration review — bounded plan is not yet a world validator

A follow-up review of the current `native.rs`, `runtime.rs`, and
`horizon-mixin.toml` confirms that `plan_target_positions` is currently a
pure coordinate helper and is not called by the preview or a placement path.
It must not be treated as proof that targets are valid in the active world.

The native mixin still captures points through `Game::pick()`; the preview
draws a geometric outline and sample crosses only. Neither path checks world
border, loaded-chunk state, target replaceability, or a selected block state.

The pinned Horizon source could not be re-resolved through the connected
repository lookup during this pass, so no world-query API names or signatures
are added by guesswork. Before placement work resumes, resolve the exact
upstream repository for revision `ff303502a4f7982551a0e0d3622d9268efe9e829`
and inspect the Wrench implementation and world/chunk accessors at that
revision.

Also, the current planner's Wall geometry is still volumetric (same target
count as Cube). Do not expose Wall placement until its intended plane/orientation
semantics are defined and the planner, count, and preview all agree.

No world writes, chunk loads, or placement hooks were added in this milestone.


## Next implementation gate — source provenance and Wall semantics

The connected GitHub installation does not currently expose the pinned
Horizon upstream repository, and searches for the exact world/chunk APIs and
Wrench hook return no indexed source matches. The prior Wrench API notes remain
useful design evidence, but this pass cannot independently verify their exact
signatures against the pinned revision.

Do not add guessed calls for world border, loaded chunks, block registry, or
block-state construction. The next safe action is to make the upstream Horizon
repository accessible to source inspection, then verify those APIs at the
pinned revision before implementing a read-only validator.

Wall remains a UI-visible tool label but its current planner/count/preview
semantics are not a true wall plane. It must be treated as unavailable for
placement until the intended orientation is defined and all three layers use
the same plane geometry. The existing planner is not yet wired into UI preview,
and no placement path should consume it yet.


## Pinned Horizon source recovered — exact revision verified

The upstream repository is `starling-client/starling`, and the pinned commit
`ff303502a4f7982551a0e0d3622d9268efe9e829` is readable. The reference
implementation was fetched directly from that commit:

- `native-mods/wrench/mixins/wrench/native.rs`
- `native-mods/wrench/mixins/wrench/use.rs`
- `crates/horizon-client/src/game/modding/native_world_edit.rs`

The Wrench source confirms the exact preflight pattern for this legacy
`.hnmod` integration:

- vertical bounds: `self.world.layout().min_y()` inclusive and
  `max_y()` exclusive;
- loaded-chunk check: `self.world.loaded(x, z)`;
- border check: `self.border.contains(f64::from(x), f64::from(z))`;
- player reach: `self.reach()` plus eye-position distance;
- current block read: `self.world.block(x, y, z)`;
- state resolution: `self.pack.assets.registry.state(state)`;
- mutation reference only: `self.with_level(... b.set_block(...,
  flags::UPDATE_ALL))`, followed by `process_level_events()`.

The separate `Game::edit_blocks` helper is explicitly server-mod-only and
requires `native_level_open(dimension)`; it is not callable from this
legacy Game mixin. It also skips unloaded chunks rather than loading them.

## Read-only preflight design (not yet wired)

The next adapter should first create a frozen target list using the existing
8,192-position planner cap, then walk it without yielding to a world write:

1. Reject remote clients, spectators, and non-acting contexts using the same
   Wrench authority gates.
2. Check each target's Y range, loaded chunk, world border, and reach.
3. Read each current state and require successful registry resolution.
4. Store compact before-state records only after every target passes.
5. Return a typed preflight result; do not mutate, load chunks, or consume
   inventory in this phase.

Only a fully successful preflight may be presented for a separate explicit
confirmation. A future commit phase must revalidate mutable assumptions before
writing, use bounded batches, and report partial failure honestly. The
preflight helper itself remains read-only. Wall placement stays disabled
until the plane orientation is specified consistently in planner, count,
and preview.

No source compilation or device test was run in this update.


## Pure fail-closed preflight core added

The pure Rust core now defines `WorldPreflightInput`, `PreflightBlock`,
`PreflightError`, and `preflight_targets(...)`.

The helper accepts the frozen ordered target list plus one adapter-collected
fact record per target. It rejects count/order mismatches, duplicate targets,
out-of-height positions, unloaded chunks, positions outside the world border,
out-of-reach targets, and unresolved before-states. It returns compact
before-state records only if every target passes. It also enforces an absolute
8,192-target ceiling even if a caller supplies a larger cap.

This is deliberately a pure validation gate, not a Horizon adapter: it does
not call `world.loaded`, `world.block`, `border.contains`, or
`registry.state` itself. The native adapter must collect those facts from the
pinned Wrench-confirmed APIs, in target order, and must not mutate or load
chunks while collecting them. A later commit phase must revalidate live state
after confirmation because preflight snapshots can become stale.

The new helper has unit tests for successful complete snapshots, unloaded
chunks, all individual validation failures, duplicate/mismatched input, and
the hard cap. These tests have been added to source but have not been run in
this update. No native hook or world write was enabled.


## Planner-driven preview and Wall safety gate

The world preview now consumes the same bounded `plan_target_positions`
geometry as the future placement path, rather than drawing a generic cuboid
that could disagree with the selected tool. It caches the coordinate plan by
selection/tool and rebuilds only when either changes. Per-frame rendering
iterates at most 64 evenly distributed target samples plus the fixed outline,
keeping the line buffer within its 204-line capacity and avoiding repeated
8,192-position allocations on mobile.

Wall geometry was deliberately removed from the active planner/count path:
tool index 1 now returns an invalid/unavailable plan until its plane and
orientation semantics are defined. The UI states that Wall geometry is not
finalized. Floor, Pillar, and Cube remain preview-only.

The preview remains read-only. It does not validate world state, choose a
material, ask for confirmation, or write blocks. No compile or device test
was run for these changes.


## Native read-only world preflight adapter

A native adapter has now been added to `native.rs` using the exact Wrench
access pattern at the pinned Horizon revision. It accepts a frozen target
slice and, in one non-yielding read-only pass:

- rejects remote clients, missing players, spectators, and non-acting contexts;
- enforces non-empty input and the 8,192 hard cap;
- checks Y bounds, already-loaded chunks, world border, and eye-to-block reach;
- reads each current `BlockStateId` and verifies that the pinned block registry
  resolves it;
- returns typed `NativePreflightBlock` snapshots only after every target passes.

The adapter does not request chunk loads, yield between checks, consume
inventory, or write blocks. Its snapshots are intentionally native-layer
records containing the engine's `BlockStateId`; they are not yet converted
into the separate pure-core `WorldPreflightInput` representation, and the
adapter is not yet called by a confirmation/placement flow. Duplicate-target
validation remains the responsibility of the frozen planner/core gate.

This is still a source-level implementation only. No compile or device test
was run, so Rust visibility/type compatibility must be confirmed by the
project's native build before this adapter is relied upon. No placement or
world-write hook was added.
