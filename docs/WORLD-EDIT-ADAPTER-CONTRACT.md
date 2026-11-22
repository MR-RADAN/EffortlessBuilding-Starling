# World-edit adapter contract — Starling 0.9.0

Creator: MR  
Mod version: 0.1.0  
Target revision: `ff303502a4f7982551a0e0d3622d9268efe9e829`

## Verified route for this package

The active migration branch is a Native SDK 0.2 `.hmod` source package whose
modules and mixins are injected into Starling's `Game<'_>` implementation. It
does not receive `&mut dyn ServerHost`, and it cannot call a server API merely
by adding a similarly shaped function to an injected `Game<'_>` module. The
older `.hnmod` bundle is retained only as historical reference.

The supported reference for this exact package type is Starling 0.9.0's own
Wrench mod at `native-mods/wrench/mixins/wrench/native.rs`. It handles
`Game::use_item` on the acting local single-player/LAN-host game, rejects
`self.remote.is_some()`, checks that the world is loaded and in bounds, then
uses `self.with_level(|behaviours, level, ticks| ...)` to call
`Behaviours::set_block` or `destroy_block` with
`horizon_world::level::flags::UPDATE_ALL`.

The correct 0.1.0 implementation must follow this in-process Game/behaviour
route. Do not use the separate `#[mod_api::horizon_mod] impl Hooks` /
`ServerHost` design. That API belongs to a different native code-mod integration.

## Authority and preflight

- Only the local single-player game or the acting LAN host may mutate.
- Reject remote server clients, missing players, spectators, and non-acting
  server/window contexts.
- Resolve a frozen, unique target list before the first write.
- Enforce the 8,192-position operation cap before allocating snapshots.
- Validate every target against world Y layout, loaded chunk state, and world
  border. For player-initiated work, require at least one selected corner to
  remain within reach; do not require every block in a large selected volume
  to be within arm's reach.
- Never implicitly load chunks.
- Require an explicit confirmation action after selection and preview.
- Selection, tool selection, menu navigation, and preview rendering remain
  strictly read-only.

## Snapshot and commit

Before enabling bulk placement, the adapter must:

1. Freeze the target coordinates and intended complete block state.
2. Validate all target positions and reject any unloaded/out-of-bounds target.
3. Snapshot each current `BlockStateId` through `self.world.block` and
   resolve it through the registry. Abort if any state is unresolved.
4. Validate replacement state and target-specific survival/shape constraints
   through the behaviour API where possible.
5. Apply with `self.with_level` and `Behaviours::set_block`, using
   `UPDATE_ALL`. Count actual successful writes; do not equate requested
   positions with committed positions.
6. If a write fails after earlier writes succeeded, attempt only guarded
   best-effort restoration: restore a position only if its current state still
   equals the state written by this operation. Report partial failure; never
   claim atomicity or guaranteed rollback.
7. Store undo data only for positions actually changed. Before undo/redo,
   re-read every current state and refuse to overwrite any position that no
   longer matches the transaction's expected state.

The behaviour API may trigger neighbour updates and side effects. The adapter
must not claim block-entity NBT, inventories, scheduled ticks, or arbitrary
side effects are preserved unless separately proven. Survival inventory
consumption and Unlimited Blocks remain disabled until authoritative item
accounting and permission semantics are verified.

## Current implementation state

The current source records two selection points, exposes Floor/Wall/Pillar/
Cube selectors, and draws a bounded read-only wireframe/sample preview. It
does not yet implement bulk world edits, material selection, the explicit
commit confirmation, or live undo/redo. A successful ARM64 compilation would
validate integration and compilation only; device testing on Starling 0.9.0
is still required.

## Source of truth

- Starling 0.9.0 reference: `native-mods/wrench/mixins/wrench/native.rs`
- Wrench's `Game::use_item` interception: `native-mods/wrench/mixins/wrench/use.rs`
- Wrench mixin manifest: `native-mods/wrench/mixins/wrench/horizon-mixin.toml`
- Project implementation notes: `native/effortless-building-v0.1.0/SERVER-HOOK-IMPLEMENTATION.md`

## Starling 0.9.0 native write route — verified

The previous conclusion that the private `Game::with_level` method was
inaccessible from this native bundle was incorrect. Native SDK 0.2 injects
this module as a child module of `crate::game`; Rust child modules can access
private items of their parent module. Starling's own Wrench native mod proves
the exact supported pattern: its injected `crate::game::wrench_native`
implementation calls `self.with_level(...)`, invokes
`Behaviours::can_survive`, `update_from_neighbour_shapes`, and
`set_block`, then calls `self.process_level_events()`.

The pinned Starling source confirms that `with_level`,
`level_set`, and `level_set_with_flags` are private, but private visibility
does not prevent this injected child module from calling them. This is not a
general public Native Mod API: it is an in-crate capability provided by the
SDK's source-injection model. Follow the Wrench pattern rather than inventing
a separate `ServerHost` interface or using unsafe/private-field workarounds.

The safe placement path for this project is therefore available in principle:

1. Revalidate the frozen selection and every before-state immediately before
   mutation, on the authoritative local logical-server Game instance.
2. Resolve the intended complete `BlockStateId` through the active registry;
   reject unresolved or registry-incompatible states.
3. Use `self.with_level(|behaviours, level, ticks| ...)` for survival checks
   and `Behaviours::set_block(..., level_flags::UPDATE_ALL)`; never mutate
   chunk storage directly.
4. Process level events through `self.process_level_events()` after writes.
5. Keep work bounded, yield/cancel between batches, record only actual
   successful writes, and use guarded best-effort restoration if a later
   write fails. Do not claim atomicity or guaranteed rollback.

This finding removes the previous API-access blocker, but it does not mean
bulk placement is already implemented or validated. The current native
bundle remains read-only until the transaction/confirmation path is
implemented and tested. In particular, placement semantics, block entities,
inventory accounting, partial failure handling, and runtime behaviour still
need explicit verification.

Reference implementation at the pinned Starling revision:
- `native-mods/wrench/mixins/wrench/native.rs` — in-crate `with_level`
  access, survival/shape validation, `set_block`, and event processing.
- `native-mods/wrench/mixins/wrench/use.rs` — cancellable item-use hook.
- `crates/horizon-client/src/game.rs` — definitions of `with_level`,
  `level_set_with_flags`, and `process_level_events`.
