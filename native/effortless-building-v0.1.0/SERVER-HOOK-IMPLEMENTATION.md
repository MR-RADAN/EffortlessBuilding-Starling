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
must not be claimed as preserved unless separately proven. Survival inventory
consumption and unlimited-block mode remain disabled until a verified item
accounting path is implemented.

## Current implementation state

The current client mixin records two points, exposes four tool selectors, and
draws a bounded read-only wireframe/sample preview. It does not yet perform
bulk world edits, and the UI does not yet contain the required confirmation
and material-selection flow. The ARM64 native build compiles the mixin bundle
into Starling's Android library; a successful compile alone is not proof of
in-game functionality. Device testing on Starling 0.9.0 is still required.
\n\n## Current verification update\n\nThe core test workflow passed on the current `main` revision (54 tests). The native integration remains a separate compile-and-device-test gate: no block writes are enabled in this revision.\n