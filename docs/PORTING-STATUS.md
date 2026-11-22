# Porting status — project 1.2.0 / native package 0.1.0

## Active target and package

- Active branch: `migration/native-sdk-0.2`.
- Project version: `1.2.0`.
- Native package version: `0.1.0`.
- Package directory: `effortless-building-v0.1.0/`.
- Launcher source package: `effortless_building-0.1.0.hmod`.
- Mod ID: `effortless_building`.
- Creator / port maintainer: `MR`.
- Native SDK API: `0.2`.
- Target game: Starling `0.9.0` / game `26.3`.
- Starling source commit: `ff303502a4f7982551a0e0d3622d9268efe9e829`.
- Horizon SDK revision: `4dc6455d4a6773224792eb1f121a9b939dff3170`.
- `horizon-mixins` revision: `5a64af938ec86b00d53d16a408aabc0038fb3e57`.

The older `.hnmod` bundle and its legacy implementation notes are retained
as reference material. They are not the active launcher package.

## Implemented pure Rust core

- Geometry routines for line, wall, floor, cuboid, circle, cylinder, sphere,
  square pyramid, cone and upper dome.
- Checked coordinate arithmetic and bounded geometry generation.
- Chunk-aware grouping and deterministic capped preview sampling.
- Two-point geometry mapping and target planning for Floor, Pillar and Cube.
- Wall remains unavailable in the native UI because its plane/orientation
  semantics are not finalized.
- Snapshot-gated placement planning, guarded undo/redo planning, and bounded
  in-memory history exist as pure Rust routines. They do not themselves read
  or modify the live game world.

## Implemented native integration

- Three injected game support modules and nine native mixin helpers are
  declared by the SDK 0.2 manifest.
- Main-hand stick right-click records point A, then point B; a third click
  starts a new pair. Other held items are not claimed.
- Material selection is read from the offhand block item. The native resolver
  accepts only an exact item-ID/default-block-state match and rejects
  unsupported or non-1:1 mappings rather than guessing. This identifies a
  candidate only: it does not yet reproduce vanilla placement context,
  orientation/half/waterlogging, multiblock placement or block-entity setup.
  Those candidates must remain blocked from writes until separately handled.
- The native UI exposes Floor, Pillar and Cube as available tools; Wall is
  explicitly unavailable.
- A bounded read-only world-space wireframe and up to 64 spatially distributed
  sample crosses visualize the planned area. This is not a per-block ghost
  state preview and does not write blocks.
- The **Check area** action performs a read-only preflight: authority/mode,
  selected-corner reach, world height, loaded chunks, world border, resolved
  before-states, target material and target count are checked. The frozen
  preflight can be revalidated against the same selection/tool/material and
  current world states.
- The `finish_close_world` mixin clears A/B selection, queued UI actions,
  preflight status and preview cache when the world closes.
- The screen mouse callback is a Post hook without a cancellation parameter.
  The mod can detect taps and queue UI actions, but suppression of the
  underlying screen/world click is not established.

## CI verification

- Standalone Rust core regression workflow [44978773735](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44978773735) passed on the migration branch: **60 passed, 0 failed**. This includes the new regression proving reach is checked against the selected anchor rather than every target block.
Workflow run [44978959476](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44978959476)
completed successfully on `migration/native-sdk-0.2` (commit `7ba93247ece885e15ba8aa31787d8fb5fe626d27`):

- `native-sdk-check`: passed SDK pin/signature, manifest, declared helper
  files, syntax checks and `.hmod` package validation.
- `android-arm64-build`: passed the pinned Starling 0.9.0 integration build.
- The run uploaded HMOD artifact `14737340947` and ARM64 artifact `14737965965`.
- Core regression workflow [44978773735](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44978773735): **60 passed, 0 failed**.

The ARM64 job uses `scripts/adapt_hmod_for_android.py` to adapt the SDK 0.2
source bundle to Starling's legacy mixin build layout, then runs the pinned
Starling Android `native_build.sh`. This compiles the injected helpers inside
the full Starling game crate. It is **not** proof that the launcher-side SDK
0.2 build service accepts this HMOD.

Downloaded artifacts were independently checked:
- The `.hmod` is a valid source ZIP with the expected manifest, Cargo
  metadata, three modules and nine helper files.
- The Android output is an ELF64 AArch64 `libhorizon_android.so`; its SHA-256
  matches `build.json`.
- The `.so` is the full Starling Android library with this mod included,
  not a standalone mod library. Do not copy it manually or treat it as the
  launcher install artifact.

See [CI-ARTIFACT-VERIFICATION.md](CI-ARTIFACT-VERIFICATION.md) for exact
artifact contents, hashes and build-path limitations.

## Not implemented or not verified

- No live block placement/removal or bulk commit path.
- No explicit commit-confirmation action wired to the native UI.
- No live undo/redo, partial-failure recovery, or persistent transaction
  history connected to the world.
- No inventory consumption; the offhand item currently selects material only.
- No per-block ghost-state preview.
- No verified suppression of underlying clicks from the Post screen hook.
- No launcher-side SDK 0.2 import/build-service result for this exact HMOD.
- No on-device runtime test on Starling 0.9.0 / Android / POCO F5.
- No standalone APK is produced or required; the expected deliverable is the
  HMOD source package built by Starling's native build flow.

## Next gates

1. In Starling 0.9.0, use **Mods → Add from GitHub** with the active branch,
   confirm the flat package is discovered, then run the launcher's **Build**.
2. Enable the mod in the intended instance and test its entry, builder screen,
   touch behavior, A/B point capture, offhand material resolver and read-only
   preview in a real world.
3. Only after launcher/runtime verification, implement the bounded
   authoritative world-edit transaction with explicit confirmation,
   before-state validation, guarded partial-failure handling and conflict-safe
   undo/redo.

## Safety invariants

1. Selection, settings, preview and preflight remain read-only.
2. No mutation before an explicit confirmation action.
3. Freeze a unique target list and snapshot every before-state.
4. Reject unloaded chunks; never implicitly load them.
5. Validate all targets for world height and border; require at least one
   selected corner to remain within player reach.
6. Count actual successful writes; never claim atomicity or guaranteed
   rollback.
7. Undo/redo must reject stale state rather than overwrite later player edits.
8. Do not claim block-entity/NBT, inventory, scheduled-tick or arbitrary
   side-effect preservation without separate proof.
9. No remote multiplayer client or delegated LAN actor may mutate the world.
