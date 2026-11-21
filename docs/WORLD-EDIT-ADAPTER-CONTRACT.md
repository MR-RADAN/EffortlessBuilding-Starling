# World-edit adapter contract — Starling 0.9.0

This contract is based on the pinned Starling source revision
`ff303502a4f7982551a0e0d3622d9268efe9e829`, specifically
`crates/horizon-client/src/game/modding/native_world_edit.rs`,
`native_world_columns.rs`, `native_world_read.rs`, and
`crates/horizon-client/src/game/blocks/block_edits.rs` (plus the typed edit
preparation in `game/modding/api_values/block_edit.rs`).

## Verified 0.9.0 engine path

Starling 0.9.0 has a server-side bulk edit path on `Game`:

- `edit_blocks(dimension, &mod_api::BlockEdit)` verifies that the named
  dimension is the authoritative level currently being ticked and rejects
  clients of a server and other dimensions.
- It prepares every palette state and every filter before mutation.
- It resolves the runs before applying the edit.
- It calls the engine's `World::edit_runs` and returns the number of changed
  blocks.
- It preserves strict block-entity behavior: replacing a block can remove its
  block entity; the API cannot represent NBT. It sends changed chunks to
  players whole and does not notify neighbours.
- Unloaded chunks and portions outside a column are skipped, so a returned
  changed count smaller than the intended count is a failed/incomplete build
  for this mod.

The game also has `read_columns(dimension, columns, heightmap, (below, above))`
for server-side column snapshots. It is restricted to the authoritative
currently-ticked level. The call reads bottom-first, returns complete block
states through a palette and per-column indices, and never implies that
unloaded chunks have been loaded. Its maximum is 1,048,576 block positions per
call; split large reads into bounded batches. `surface_heights` is capped at
65,536 columns.

The lower-level `native_server_block` read helper requires a valid
`LifecycleContext`; it is not an unrestricted arbitrary-time read API.

## Preflight and snapshot

Before showing confirmation:

1. Restrict the first adapter to the active authoritative single-player
   server level. Do not attempt client-side or remote multiplayer edits.
2. Generate the full intended position set from the geometry core and enforce
   the operation cap before allocating large snapshots.
3. Group positions by chunk/column. First call `surface_heights` for every
   target `(x, z)` column (at most 65,536 columns per call) and abort if any
   result is `None`. This is mandatory: `read_columns` represents unloaded
   columns as air in its returned palette/index stream, so a read alone cannot
   distinguish an unloaded target column from real air. After the loaded-column
   preflight succeeds, call `read_columns` in batches no larger than 1,048,576
   block positions.
4. Reconstruct the exact before-state for every intended position from the
   returned bases, palette and block indices. If any preflight height is
   missing, any index is invalid, or any position is outside the returned
   range, abort the entire operation.
5. Reject block entities and states whose restoration semantics are not
   proven. Bulk edits cannot preserve arbitrary block-entity NBT.
6. Store exact expected before/after states and bound both transaction count
   and undo memory. The pure-core history stores compact state indices; the
   adapter must also retain the canonical palette-state strings needed to
   recreate BlockEdit palettes and count that string table against the same
   memory budget.

## Commit, undo and redo

Only after explicit confirmation may the adapter construct a typed
`mod_api::BlockEdit` and call the authoritative server edit path. The
palette, filters and runs must be fully validated before that call.

Require the engine's changed-block count to equal the number of intended
positions that actually differ from their before-state. If it does not,
treat the operation as incomplete, do not add it to the undo stack, and do
not claim success.

Before undo or redo, read back every affected position and compare it with
the transaction's expected current state. If any position differs, reject the
whole transaction rather than overwriting intervening player or server edits.
Undo and redo must use the same authoritative server edit path and must also
verify the returned changed count.

## Inventory and Unlimited Blocks

A successful bulk edit is not evidence of inventory consumption. Inventory
accounting requires a separately verified authoritative API and must be
implemented before survival placement is advertised. Unlimited Blocks stays
disabled until server-side permission and game-mode semantics are verified.
It must never bypass a multiplayer server's authority.

## First-adapter exclusions

- No remote multiplayer edits.
- No unloaded-chunk edits or implicit chunk loading.
- No block-entity/NBT preservation promise.
- No partial operation represented as a successful transaction.
- No undo/redo that overwrites intervening world changes.
- No survival inventory or unlimited-block claim until separately verified.

This document describes verified Starling 0.9.0 engine capabilities and the
required adapter safeguards. It does not claim the adapter itself has been
implemented, compiled, or runtime-tested.

## Source-path clarification

The exact Starling 0.9.0 files are under `crates/horizon-client/src/game/`, not a repository-root `game/` directory. In particular, `surface_heights` is the required loaded-column preflight because `read_columns` fills unloaded columns with air. The adapter must never infer "loaded and air" from the column read alone.


## Mod-facing API boundary

The engine helpers in `game/modding/native_world_edit.rs` and
`native_world_columns.rs` are `pub(super)` methods on the internal `Game`
implementation. A mixin module placed beside `game.rs` must not assume it can
call those private helpers directly. The supported mod-facing route is the
public `ServerHost` API declared in
`crates/horizon-mod-vocab/src/host/world_edit.rs`: `surface_heights`,
`read_columns`, and `edit_blocks`. The adapter therefore needs to execute
inside a server-side native-mod callback that receives `&mut dyn ServerHost`
(or an equivalent supported host handle), and it must keep preview/UI intent
separate from the authoritative server callback. Do not work around this
visibility boundary by reaching into private `Game` fields or by injecting a
client-side write.

The existing mixin integration currently captures selection and draws a
read-only preview; it is not yet a `ServerHost` callback implementation. The
next implementation milestone is to add a proper server native-mod hook and
route snapshot, commit, undo and redo through its `ServerHost` methods.


## Confirmed server-hook route (Starling 0.9.0)

The supported mod entry point is the root `impl mod_api::Hooks` marked with
`#[mod_api::horizon_mod]`. Server-side listeners receive
`&mut dyn ServerHost` plus the hook event. The `server_tick` hook is a
server-side listen hook; its contract explicitly permits block reads through
`native_server_block` and is the appropriate authoritative tick boundary for
bounded adapter work. A listener must not attempt to use a client-side
`Game` mixin as a substitute for this host.

The public server host exposes these world-edit methods (declared in
`crates/horizon-mod-vocab/src/host/world_edit.rs`):

- `surface_heights(dimension, columns, heightmap)`: returns `None` for an
  unloaded column; at most 65,536 columns per call.
- `read_columns(dimension, columns, heightmap, range)`: at most 1,048,576
  block positions per call. Unloaded columns are represented as air, so this
  read is only a valid snapshot after every target column has passed the
  `surface_heights` preflight.
- `edit_blocks(dimension, edit)`: validates the complete palette, filters,
  and run references before mutation. Its engine contract also says runs in
  unloaded chunks and run portions outside the column are left out. Therefore
  the adapter must preflight every target column and vertical bound before
  issuing the edit; a successful return alone is not proof that every intended
  position changed.

### Conflict-safe commit design

1. Build a deterministic list of unique target columns and reject a target
   count above the core's 16,384-position cap before allocating large buffers.
2. Query `surface_heights` in bounded batches; if any target column returns
   `None`, abort before reading or writing.
3. Read the exact target Y span in batches no larger than 1,048,576 positions.
   Preserve full `BlockState` values, not only block identifiers.
4. Build the edit runs and expected-state filters from that snapshot. A run
   must only match the exact pre-commit state captured for its position; group
   identical predicates to stay below the 65,536-filter limit.
5. Recheck that the operation is still the current confirmed preview, the
   world epoch/dimension has not changed, and the request is within the safe
   vertical bounds immediately before `edit_blocks`.
6. Treat any refusal, count mismatch, or stale confirmation as a failed
   operation. Do not record it as committed history. Only create an undo entry
   after the host reports the complete expected change count.

The API does not expose a general multi-call world transaction or a rollback
primitive. In particular, the read and edit happen in separate host calls, so
this is optimistic conflict checking, not a database-style lock. The adapter
must be conservative about external edits between snapshot and commit, and it
must never promise strict atomic rollback for a failure after engine mutation.
Undo/redo must use the same snapshot/filter/preflight discipline and must
refuse to overwrite positions whose current states no longer match the
expected post-state.
