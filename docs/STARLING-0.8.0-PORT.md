# Starling 0.8.0 native port notes

Status: compatibility research only. This document does not claim that an installable mod has been built or tested.

## Exact target

- Starling release: `v0.8.0`
- Horizon game source: `ac874422ed8f892505ff73d67ae486ecfee7a516`
- horizon-mixins: `ddd9700b21997c95d95ac6ee5ecafe3830cf622a`
- Android target expected by Starling's build service: ARM64 (`aarch64-linux-android`)

Do not copy the newer generic native template's compatibility pins into this release target. Native mods are compiled against the exact Horizon revision selected by the Starling release/build service.

## Confirmed game-side facilities at the target commit

The pinned source contains these native helper implementations and hook-catalog entries:

- Bulk world edits: `native_edit_blocks`, represented by `NativeBlockEdit`, `NativeBlockRun` and `NativeHeightmap`.
- World reads: `native_surface_heights` and `native_read_columns`, represented by `NativeColumnBlocks`.
- View targeting: `native_raycast` and `native_view_hit`, returning a `NativeHit` with block coordinates, hit point and block-state name.
- A mod-owned screen lifecycle: `native_mod_screen_open`, `native_mod_screen_close` and `native_mod_screen`.
- The native hook catalog also names screen, mouse, scroll, key/character, drag and raw-mouse hook points.

The bulk editor is server-authoritative: it only edits the authoritative level currently being ticked. It skips unloaded chunks and out-of-column portions, rejects invalid palette/filter entries and NBT-bearing block states, and does not notify neighbors. Changed chunks are synchronized to players. This behavior must be accounted for in the transaction design; it is not equivalent to a general-purpose vanilla placement operation.

Column reads are capped at 1,048,576 blocks per call. Height queries are capped at 65,536 columns. Unloaded columns are represented explicitly by missing bases; they must not be silently treated as a complete snapshot.

The mod screen is unavailable over the title screen, another screen, dialogs/prompts, loading, or a dedicated server. Opening it releases normal input capture; Escape closes it. The target source documents that gamepad clicks do not reach mouse hooks. Android touch routing still needs a separate test before promising touch UI behavior.

## MVP implementation order

1. Keep geometry generation pure and unit-testable: Floor, Wall, Pillar and Cube.
2. Add a native adapter pinned to the exact target above.
3. Use raycast results for the first anchor; use an explicit second anchor input for rectangular tools.
4. Build a bounded preview model. Until a verified render hook draws it, show a coordinate/count summary only; do not describe that as an in-world preview.
5. Before Apply, read and validate every target position. Reject incomplete/unloaded snapshots and cap one transaction to a safe bounded size.
6. Apply only after an explicit confirmation.
7. Store before/after states for Undo/Redo, and verify the current state still matches the expected side of the transaction before changing it.
8. Add the native screen and touch controls only after the target's exact screen-hook signatures and Android touch event path have been compiled and tested.

## Build and test

Starling's native-mod workflow accepts `.hnmod` source bundles and offers Compile on ARM64 devices, including a GitHub-runner option. Compilation is not runtime verification. Test first in a disposable single-player world and retain a clean world backup.

No generated binary, successful compile, or in-game test is claimed by this note.
