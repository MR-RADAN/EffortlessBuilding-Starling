# Starling 0.8.0 native integration audit

## Confirmed against the pinned Horizon source
Target:
- Starling 0.8.0
- Horizon: ac874422ed8f892505ff73d67ae486ecfee7a516
- horizon-mixins: ddd9700b21997c95d95ac6ee5ecafe3830cf622a

The Horizon source at that exact commit documents these native hook points:

- Hook API 111: native mod-owned screen open/close and screen identity. The screen is drawn over the world; it does not pause a single-player world. It opens only on the window's own client and only when no overlay is already open. A gamepad can close it but cannot click it.
- Hook API 126: native_edit_blocks(dimension, edit) for server mods. It applies edits in columns, from a height or heightmap, with command-style filters; writes are made together directly into level chunks, with block-entity handling described as fill strict; changed chunks are sent to players as a whole.
- Hook API 127: native_surface_heights and native_read_columns for server mods, reading column heights and block palettes around those heights.

These API levels are present in the pinned source documentation, so the old assumption that these APIs were merely unverified can be narrowed: their existence and high-level contract are verified. The precise Rust type signatures, edit payload shape, screen event parameter structs and client/server hook wiring still need source-level inspection before implementation.

## Architecture consequence
The safe architecture is likely split into:
1. A client-side native screen for selecting shape, anchors, block and preview state.
2. A server-side edit path using the API 126 bulk-edit helper where its semantics match the operation.
3. Explicit snapshot/readback and bounded transaction bookkeeping for undo/redo.
4. A client preview that is strictly visual and never calls the world mutation path.

Do not assume API 126 alone provides a complete reversible transaction. Its documented behavior does not promise a complete pre-edit snapshot, preservation of every block-entity payload, inventory accounting, or client-side undo. Those must be separately verified or implemented.

## Still not implemented or verified
- No Horizon hooks are connected in this repository.
- No native screen is registered or opened.
- No Android touch-input isolation is implemented.
- No world read/write adapter or edit payload serializer exists.
- No inventory consumption or Unlimited Blocks implementation exists.
- No undo/redo snapshot persistence or conflict checks exist.
- No server-authority behavior has been tested in a multiplayer world.
- No .hnmod package has been produced.
- Rust tests and Cargo checks have not run here because Rust/Cargo are unavailable.

## Adapter safety requirements
1. Preview is read-only.
2. Bulk placement requires explicit user confirmation.
3. Snapshot original states before mutation; abort if the snapshot is incomplete.
4. Bound operation size and undo memory; do not load chunks just to place blocks.
5. Verify current state before undo/redo to avoid overwriting later edits.
6. Inventory changes must use the authoritative game/server path.
7. Unlimited Blocks is only valid where the authoritative server accepts edits; it is not a client-side multiplayer bypass.
8. The native screen runs over the world and does not pause singleplayer. Touch isolation must not modify Starling, MCPE or GoyControl touch layouts.

## Release acceptance gates
1. Read the exact source definitions and hook wiring for APIs 111, 126 and 127.
2. Implement the smallest client/server adapter and compile against the pinned source.
3. Run the target Rust/game workspace checks.
4. Validate the .hnmod package.
5. Compile through Starling's build service for ARM64.
6. Test on the POCO F5: preview, cancel, confirm, all shapes, undo/redo, inventory use, and touch controls.
