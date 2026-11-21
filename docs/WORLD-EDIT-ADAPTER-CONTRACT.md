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
