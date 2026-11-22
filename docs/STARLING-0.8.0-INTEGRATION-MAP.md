# Starling 0.8.0 integration map — exact release source

Date: 2026-11-21

This note records a second pass over the exact Starling release source, commit
`ac874422ed8f892505ff73d67ae486ecfee7a516`.

## Confirmed helper targets in the release catalog

The release-root `native-hooks.json` includes these helper targets:

- `edit_blocks` → `horizon_client::game::native_world_edit::Game<'_>::native_edit_blocks`
- `read_columns` → `horizon_client::game::native_world_columns::Game<'_>::native_read_columns`
- `view_hit` → `horizon_client::game::native_claimed_items::Game<'_>::native_view_hit`
- `raycast` → `horizon_client::game::native_claimed_items::Game<'_>::native_raycast`
- `mod_screen_open` / `mod_screen_close` / `mod_screen`
- `setting` / `set_setting`

These are catalogued helper methods, not automatically injectable event hooks. The mixin manifest still needs to declare valid injections into actual source functions, and the helper bindings must match the target function's Rust signature and visibility.

## World-edit contract

At this release, `native_edit_blocks` edits runs directly in the authoritative level being ticked. It refuses a different dimension, a client-side server world, invalid palette/filter entries, NBT-bearing states and out-of-range indexes. It skips unloaded chunks and out-of-column run portions; it does not notify neighbours. This makes a preflight snapshot and an exact expected-changed-count check mandatory for this mod.

`native_read_columns` returns a palette plus per-column bases and block indexes, and is bounded by the source API's read limits. An unloaded column is represented by a missing base. It must be rejected for any transaction that would touch it.

## UI decision

The first interaction path will be a native mod screen opened from a release-supported key/button route. The screen should expose shape, first/second anchor capture, preview, confirm, undo and redo. Until a working screen render/input injection is compiled against the exact release source, no custom overlay or touch placement is claimed as implemented.

## Safety correction

The bulk-edit helper does not promise transactional rollback. Therefore the first usable version must:
1. preflight every target position and snapshot its exact before-state;
2. reject if the number of writable positions cannot be fully represented by loaded columns;
3. issue one bounded edit request, not multiple partial batches;
4. verify the returned changed count equals the planned count;
5. if the count differs, report a partial/uncertain result and do not offer an unsafe blind undo;
6. make undo/redo conditional on every current block matching the expected state.

This is stricter than simply batching a large fill operation and avoids presenting a non-atomic world edit as atomic.

## Validation status

Source inspection only. No native bundle has yet passed the release build service, no ARM64 library has been linked, and no in-game UI or world-edit test has been run.
