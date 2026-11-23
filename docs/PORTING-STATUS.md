# Porting status — project 1.2.1 / native bundle 0.1.0

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
- The 0.9.0 adapter has rewritten all nine hook targets, including `use_item`
  and `submit_custom_geometry`; the first compile attempt then caught a duplicate
  module/injection ID before Rust compilation. The injection ID is now
  `world_preview`. The latest ARM64 workflow [44879334414](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44879334414)
  is building commit `ec35bd7`, which includes the action-bar selection feedback
  and corrected integration notes.

## Starling 0.9.0 source audit

Verified directly against the pinned source:

- Legacy `.hnmod` import remains supported; `horizon-native.json` is required
  at the archive root and source files live under `mixins/<id>/`.
- The importer requires exact matching id/version/Horizon revision in the
  JSON and mixin TOML, plus exact pinned Horizon and mixins commit hashes.
- The legacy native adapter remaps old hook targets and source calls for the
  reorganized Horizon tree.
- The first 0.9.0 attempt transformed seven UI injections and one module.
- The later preview-enabled attempt transformed all nine hooks (the new
  `use_item` and `submit_custom_geometry` targets were mapped to their exact
  0.9.0 source locations) and brought all three project modules up to date.
  It then stopped at manifest validation because module and injection IDs
  collided; the injection has since been renamed.
- The built-in Wrench mod is the verified reference for this legacy mixin
  package: it mutates the acting local single-player/LAN-host world through
  `Game::with_level` and block behaviour methods, while rejecting remote
  clients. The project must follow this route rather than inventing a
  `ServerHost` callback inside a `Game<'_>` mixin.
- Wrench's single-block `set_block` / `destroy_block` examples prove the
  callable integration pattern, not bulk atomicity, rollback, or block-entity
  preservation. Those safeguards remain implementation requirements.
- The corrected adapter boundary and transaction requirements are documented
  in `docs/WORLD-EDIT-ADAPTER-CONTRACT.md`.

## Build validation

- Rust core tests passed for the two-corner geometry commit in run
  [44877215141](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44877215141).
  All 46 tests passed: snapshot decoding, guarded undo/redo, bounded history
  transitions, two-corner selection geometry and overflow regressions.
- Latest source package validation passed in run
  [44878280192](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44878280192). Its artifact (9,648-byte outer ZIP containing the 10,783-byte `.hnmod`) was downloaded and inspected: the inner archive contains `horizon-native.json`, `horizon-mixin.toml`, all three Rust modules and all nine helper files; both ZIP integrity checks pass. This validates package structure, not importer acceptance or native compilation.
- Earlier source package validation also passed in run
  [44875262029](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44875262029).
  This validates ZIP structure only, not importer acceptance or compilation.
- The first Starling 0.9.0 build,
  [44875302971](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44875302971),
  exposed the missing `rust-src` toolchain component; the workflow now installs it.
- Build [44876927294](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44876927294)
  reached manifest validation after adapting all nine hook targets, but failed
  with `E_DUPLICATE_ID` because the module and injection both used `preview`.
- The injection ID was changed to `world_preview`. Corrected build
  [44877289336](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44877289336)
  was cancelled before a result was produced. Retry [44878280143](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44878280143)
  was cancelled before compilation completed. Run [44879000402](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44879000402)
  was also superseded before compilation. Run [44879229267](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44879229267)
  was cancelled by the subsequent bundle documentation update.
  The latest retry is [44879334414](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44879334414);
  no ARM64 library artifact has been produced yet.

## Not yet implemented or verified

- No live world read/write adapter is connected to the UI.
- Two-point block selection and a bounded read-only spatial outline/sample
  preview are implemented in source but await native build validation. No
  per-block ghost preview or world-edit adapter exists yet.
- No block placement, wall/floor/cuboid commit, or shape commit is available.
- No live undo/redo is connected to snapshots.
- No inventory consumption or Unlimited Blocks implementation is available.
- The user confirmed that the mod imports/activates on Starling 0.9.0 and its
  Options > Mods page shows the four tool selectors. However, the tools do not
  perform their expected in-world actions yet. The new action-bar point-capture
  feedback has not yet been tested on-device.
- No end-to-end world-edit test on Starling 0.9.0 or POCO F5 has been completed.
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


## Selection-input diagnostic update

The current `Game::use_item` mixin now sends an action-bar message immediately
after it captures a point, using the same local/LAN overlay path as Starling's
Wrench mod. This makes the two-click selection observable on a phone instead
of relying only on the preview. It does not implement block placement.

Important integration boundary: this legacy `.hnmod` is a `horizon-mixins`
bundle, not a `#[horizon_mod]` crate. The verified world-write pattern for
this package is Starling's built-in Wrench implementation: its cancellable
`Game::use_item` path checks the acting local/LAN-host context and uses
`Game::with_level` plus the behaviour API. The project's bulk adapter must
extend that exact pattern with complete preflight, snapshots, explicit user
confirmation, guarded partial-failure recovery, and conflict-safe history.
Do not add fake `ServerHost` parameters or claim atomic transactions.


## Local diagnostic logging added (source-level)

- Runtime writes JSONL events through `Game::native_config_dir()` to `config/effortless_building/diagnostics.jsonl`.
- Events: selection-point capture (including coordinates), tool selection, read-only preflight result and builder close.
- Active file rotates at 256 KiB; one previous file is retained. Detail is capped at 2,048 characters per event.
- No automatic upload or sharing. Collection instructions are in `docs/DIAGNOSTICS.md`.
- This is source-level only until the Starling 0.9.0 ARM64 build succeeds and the file is verified on-device.
