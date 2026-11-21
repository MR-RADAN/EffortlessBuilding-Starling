# Porting status — project 1.2.0 / native bundle 0.1.0

## Pinned target

- Starling 0.9.0 source commit: `ff303502a4f7982551a0e0d3622d9268efe9e829`.
- `horizon-mixins`: `5a64af938ec86b00d53d16a408aabc0038fb3e57`.
- Native bundle directory: `native/effortless-building-v0.1.0/`.
- Native package filename: `effortless_building-0.1.0.hnmod`.
- Native metadata author/creator: MR.

Historical 0.8.0 build and pin information is retained in
`docs/NATIVE-ANDROID-BUILD.md`; it is not the current target.

## Implemented source core

- Deterministic geometry: line, wall, floor, cuboid, circle, cylinder, sphere,
  pyramid, cone and dome.
- Bounded block counts and checked coordinate conversion.
- Reverse transaction planning with repeated-position coalescing.
- Chunk-aware batch grouping, including negative coordinates.
- Deterministic capped preview sampling (preview samples are not placement lists).
- Two-corner geometry mapping for Floor, Wall, Pillar and Cube, including
  lower-Y floor placement, inclusive wall/pillar height and full cuboid bounds.
- Snapshot-gated transaction preparation: a build cannot be planned unless
  every target position has a before-state snapshot.
- Undo/redo preflight helper rejects the entire transaction if any current
  block differs from the expected state.
- Regression tests cover missing snapshots, duplicate positions, no-op
  filtering, guarded undo/redo and intervening edits.

## Native UI scaffold

- Native bundle manifest and strict `.hnmod` package layout are present.
- Settings list, dedicated builder panel, tool selection and close action are
  wired through the native hook manifest.
- Touch press/release inside the panel is captured to prevent the same click
  reaching the world.
- The native UI exposes the four initial tools and reports the current point
  selection. A cancellable `Game::use_item` hook mirrors Starling's Wrench
  pattern: with the mod enabled, a main-hand right-click while holding a stick records the block under the
  player's ray pick. Other held items retain normal use behavior. First click
  sets A, second sets B, and a third starts a new pair. The hook deliberately
  refuses remote multiplayer clients.
- A `submit_custom_geometry` hook now draws a read-only world-space wireframe
  and up to 64 evenly distributed sample crosses over the selected volume.
  The per-thread line buffer is reused between frames; the renderer's own
  fixed-capacity batch validates the whole submission atomically.
- The builder screen calculates the selected tool's exact geometric block
  target-position count with wide arithmetic and flags any selection above the
  16,384-position core limit before any commit can be offered.
- The preview represents the selected region, not individual ghost block
  states. It does not read block snapshots or modify the world.
- Point capture and preview remain source-only until the next native Android
  build validates the new `use_item` and `submit_custom_geometry` hooks.

## Starling 0.9.0 source audit

Verified directly against the pinned source:

- Legacy `.hnmod` import remains supported; `horizon-native.json` is required
  at the archive root and source files live under `mixins/<id>/`.
- The importer requires exact matching id/version/Horizon revision in the
  JSON and mixin TOML, plus exact pinned Horizon and mixins commit hashes.
- The legacy native adapter remaps old hook targets and source calls for the
  reorganized Horizon tree.
- The native UI's seven hook injections and one module were transformed
  successfully by `native_mod_adapt.py` in the first 0.9.0 build attempt.
- Starling 0.9.0 provides an authoritative server-side typed bulk edit path
  (`edit_blocks`), bounded column snapshot reads (`read_columns`), and
  loaded-column height queries. These operate only on the level being ticked;
  remote client edits are refused.
- Column reads are limited to 1,048,576 block positions per call and do not
  load missing chunks. Bulk edits can skip unloaded chunks and do not preserve
  arbitrary block-entity NBT.
- The exact API constraints and transaction safeguards are documented in
  `docs/WORLD-EDIT-ADAPTER-CONTRACT.md`.

## Build validation

- Rust core tests passed for the two-corner geometry commit in run
  [44877215141](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44877215141).
  Snapshot decoding, guarded undo/redo, bounded history transitions, two-corner
  selection geometry and overflow regressions are covered by the passing suite.
- Source package validation passed in run
  [44875262029](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44875262029).
  This validates ZIP structure only, not importer acceptance or compilation.
- The first Starling 0.9.0 Android build attempt,
  [44875302971](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44875302971),
  failed before Cargo compilation because `rust-src` was missing. The
  transformation phase itself completed with seven injections and one module.
  The Android workflow now installs `rust-src` before invoking the native
  build. A subsequent run must confirm the correction and reveal any actual
  Rust compile errors.

## Not yet implemented or verified

- No live world read/write adapter is connected to the UI.
- Two-point block selection and a bounded read-only spatial outline/sample
  preview are implemented in source but await native build validation. No
  per-block ghost preview or world-edit adapter exists yet.
- No block placement, wall/floor/cuboid commit, or shape commit is available.
- No live undo/redo is connected to snapshots.
- No inventory consumption or Unlimited Blocks implementation is available.
- No runtime test on Starling 0.9.0 or POCO F5 has been completed.
- No installable APK has been produced. The expected native build output is a
  Starling ARM64 library, not a standalone APK.

## Safety gates

1. Preview remains read-only.
2. Explicit confirmation is required before bulk mutation.
3. Every affected position must be snapshotted; incomplete snapshots abort.
4. No implicit chunk loading.
5. Undo/redo must validate every current state before applying any inverse.
6. No block-entity/NBT preservation promise until proven.
7. Inventory changes and Unlimited Blocks require verified authoritative
   server-side behavior; no multiplayer client bypass.
8. The custom screen must not interfere with Starling, MCPE or GoyControl
   touch layouts.
