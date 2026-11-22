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

Starling exposes a lower-level world-write route to injected child modules:
`self.with_level(...)`, `Behaviours::set_block(...)`, and
`self.process_level_events()`. The Wrench source proves that these calls are
reachable from this injection model. This is only a mutation primitive; it is
**not yet a safe placement adapter** and does not reproduce vanilla
`BlockItem.place` semantics.

Before this project enables any writes, it must still establish a supported
way to derive a complete placement state and reproduce required placement
side effects. A conservative first implementation may whitelist only states
whose property and block-entity behavior has been source-verified; all other
items must fail closed. Even a property-free state still needs replacement,
survival, neighbour-shape, entity-collision, and block-specific side-effect
checks. The current resolver only identifies a candidate default state.

The future mutation primitive, if and only if those checks are implemented,
will use `self.with_level(...)`, `Behaviours::set_block(...,
level_flags::UPDATE_ALL)`, and `self.process_level_events()`. Work must remain
bounded, revalidate the frozen plan immediately before mutation, record only
actual successful writes, and use guarded best-effort restoration after
partial failure. Do not claim atomicity or guaranteed rollback.

This finding removes the previous API-access blocker, but it does not mean
bulk placement is already safe, implemented, or validated. The current native
bundle remains read-only. Placement semantics, block entities, inventory
accounting, partial failure handling, and runtime behavior remain explicit
gates.

Reference implementation at the pinned Starling revision:
- `native-mods/wrench/mixins/wrench/native.rs` — in-crate `with_level`
  access, survival/shape validation, `set_block`, and event processing.
- `native-mods/wrench/mixins/wrench/use.rs` — cancellable item-use hook.
- `crates/horizon-client/src/game.rs` — definitions of `with_level`,
  `level_set_with_flags`, and `process_level_events`.

## Exact placement API discovery — pinned 0.9.0

A further source audit found a higher-level API than the previously documented set_block primitive:

- Behaviours::placement_preview(level, ticks, &PlaceContext) computes the actual BlockItem.place result without writing. Its PlacementPreview includes the final position/state and an optional companion block (doors, beds, tall plants).
- Behaviours::place_block(level, ticks, &PlaceContext) executes the real placement path: replacement-position resolution, getStateForPlacement, survival checks, entity obstruction, the write, setPlacedBy, and placement sound/event handling. It returns PlaceOutcome.

Both methods are public in the pinned horizon_world::level::Behaviours API and can be called from the injected child module through self.with_level. This is materially safer and more faithful than synthesizing a BlockState and calling set_block directly.

### Consequences for the first commit adapter

1. Construct a deterministic PlaceContext for each intended target and compute placement_preview before any write.
2. Require the preview's resolved pos to equal the intended target.
3. Reject any preview with a companion block until the transaction snapshots, validates, accounts for, and safely rolls back every companion position.
4. Restrict initial supported items/states to a verified whitelist. The existence of vanilla placement logic does not mean every block's placement side effects are safe for bulk use.
5. Revalidate the complete frozen snapshot immediately before commit, then call place_block on the authoritative acting game and count only PlaceOutcome::Placed results.
6. Keep the first mutation milestone Creative-only until item consumption, inventory mutation, and survival permission semantics are explicitly implemented. Do not silently grant Survival-mode free blocks.
7. Keep per-write ledger updates and guarded rollback; a series of individual vanilla placements is not an atomic multi-block transaction.

The adapter still needs to prove that the synthetic click context gives appropriate orientation for each supported block, that neighbour-dependent placement remains deterministic across the planned order, and that no untracked side effect is introduced. Runtime testing remains mandatory.

This discovery supersedes the older note that a generic public placement API had not been found. The current project code is still read-only: no placement calls have been wired into the mod yet.

## Placement-preview limitations — audited against pinned source

The current preflight uses `placement_preview`, but this API is deliberately
read-only and does **not** execute `set_placed_by`. The pinned
`crates/horizon-world/src/level/place.rs` shows that the real placement path
writes the primary block, then invokes `set_placed_by`, which may create a
companion block or schedule ticks / trigger additional behavior (for example,
pistons and tripwire hooks). The preview reports the known companion for beds,
doors, and tall plants, but it is not a complete simulation of every post-write
effect.

The current guard requiring `ItemKind::Block` and an empty
`BlockData::chain` is a conservative filter: `chain` is the recognised Java
block-class override chain used by Horizon's behaviour dispatch. An empty chain
rules out those recognised class overrides; it is **not** a universal proof
that neighbour updates, scheduled ticks, external mod hooks, or all other
world side effects are absent.

Consequently:

- Keep the current preflight read-only.
- Do not interpret a successful preview as proof that a multi-block edit is
  side-effect-free or rollback-safe.
- Freeze the exact item ID and all `PlaceContext` inputs used by preview
  (including face, hit position, yaw, pitch, and sneaking) in the preflight
  record. Revalidation must use those frozen inputs and confirm the offhand
  item identity still matches.
- Before the first write milestone, explicitly audit the event queue and
  neighbour-update behavior for the whitelist. Reject unsupported states
  rather than broadening the whitelist based only on property-free states.
- Preserve the per-write ledger and guarded rollback. Even a successful
  `place_block` result does not make a sequence of placements atomic.

Source: pinned Starling/Horizon revision
`ff303502a4f7982551a0e0d3622d9268efe9e829`, especially
`crates/horizon-world/src/level/place.rs` (`placement`,
`set_placed_by`, `place`, and `placement_preview`) and
`crates/horizon-world/src/level/data.rs` (`BlockData::chain`).
