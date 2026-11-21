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
