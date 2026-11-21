# Porting status — 1.2.0

## Historical 0.8.0 pins
- Starling: 0.8.0
- Horizon: ac874422ed8f892505ff73d67ae486ecfee7a516
- horizon-mixins: ddd9700b21997c95d95ac6ee5ecafe3830cf622a

These are historical 0.8.0 pins only. The current target pins are recorded in VERSION.json and the native bundle manifests.

## Implemented source core
- Deterministic geometry: line, wall, floor, cuboid, circle, cylinder, sphere, pyramid, cone and dome.
- Bounded block counts and coordinate conversion.
- Reverse transaction planning with repeated-position coalescing.
- Chunk-aware batch grouping, including negative coordinates.
- Deterministic capped preview sampling.
- Checked dimension multiplication for cuboids, floors and walls; wall Y-coordinate overflow is rejected before generation.
- Regression tests added for extreme coordinate ranges and wall-height overflow (not executed in this environment).

## Current Starling 0.9.0 target pins
- Horizon/Starling source: ff303502a4f7982551a0e0d3622d9268efe9e829.
- Global horizon-mixins: 5a64af938ec86b00d53d16a408aabc0038fb3e57.
- VERSION.json, horizon-native.json and horizon-mixin.toml have been updated consistently.
- Legacy .hnmod import requires strict horizon-native.json fields including both exact horizon and mixins revisions; identity and horizon revision must match horizon-mixin.toml.

## Confirmed Starling 0.9.0 source facts
- Official tag: v0.9.0.
- Source commit: ff303502a4f7982551a0e0d3622d9268efe9e829.
- The .hnmod native bundle route remains supported; it was not replaced by the old Starling-specific/WASM format.
- Top-level native bundle metadata is horizon-native.json; the official Wrench example uses game, id, mixins, schema, and version.
- The v0.9.0 native build script pins horizon-mixins to 5a64af938ec86b00d53d16a408aabc0038fb3e57.
- Starling includes native_mod_adapt.py to adapt legacy hook targets and the old Starling mod API to the reorganized Horizon source tree.
- The native build path sends native bundle source to a build service, checks the returned library's SHA-256, and only loads it for the APK install it was built against.
- The v0.9.0 in-repository native-mods.yml workflow builds reviewed bundles already in Starling's own native-mods/ tree; it is not a generic project upload/build route.
- The public starling-client/mod-builds/versions.json checked on 2026-11-21 still maps only 0.8.0. This does not establish the active configuration of Starling's separately published HTTPS gateway.

See the corrected compatibility audit in STARLING-0.9.0-COMPATIBILITY-AUDIT.md.

## Not implemented or verified
- No Horizon hooks are connected.
- No native screen or Android touch-input isolation exists yet.
- No world read/write adapter exists.
- No inventory consumption or Unlimited Blocks implementation exists.
- No server-authority or multiplayer behavior has been tested.
- A source bundle scaffold and deterministic .hnmod packager exist, but the updated v0.9.0 package has not been produced or imported.
- Rust tests and Cargo checks have not run here because Rust/Cargo are unavailable. The latest overflow-regression tests are committed but remain unexecuted.
- The v0.9.0 native bundle importer/build-service route for this project's own bundle has not yet been exercised. The default gateway URL is published in Starling source, but its server-side version registry is not public; the public GitHub-runner map still lists only 0.8.0.

## Adapter safety requirements
1. Preview must be read-only.
2. Bulk placement requires explicit confirmation.
3. Snapshot original states before mutation; abort if the snapshot is incomplete.
4. Bound operation size and undo memory; do not load chunks just to place blocks.
5. Verify current state before undo/redo to avoid overwriting later edits.
6. Inventory changes must use the authoritative game/server path.
7. Unlimited Blocks is only valid where the authoritative server accepts edits; it is not a client-side multiplayer bypass.
8. The native screen runs over the world and does not pause singleplayer. Touch isolation must not modify Starling, MCPE or GoyControl touch layouts.

## Next gate
Inspect the v0.9.0 Wrench example, the adapted hook catalog, and the game's authoritative block placement/destruction/inventory implementation. Then build the smallest safe native bundle skeleton against the exact v0.9.0 pins. Do not invent API names or claim runtime behavior before compilation and in-game tests.
