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

## Bundle format and hook requirements verified

The pinned Horizon documentation confirms that the native bundle is not a plain Cargo crate and is not just a horizon-mixin.toml archive. The .hnmod package includes a horizon-native.json metadata file alongside the mixin manifest and Rust helper files. Native hook bundles declare their required hook API in that metadata as hooks: N.

For this target, the minimum hook API requirements are:
- hooks: 111 for a client-owned native screen. It opens over the world, does not pause single-player, and must be drawn and operated through the normal screen hooks. The API explicitly says a gamepad can close it but cannot click it.
- hooks: 126 for the server-side native_edit_blocks(dimension, edit) bulk-edit hook.
- hooks: 127 for server-side native_surface_heights and native_read_columns.

Therefore the first package must declare at least hook API 127 if it uses all three capabilities. This is a minimum compatibility declaration, not proof that the mod compiles: exact helper signatures, module destinations, event fields, and hook bindings still need to be checked against the pinned source before a manifest is committed.

## Important version-drift finding

The current horizon-rust/horizon-mods examples are not safe templates for this release unless individually audited: their example manifest pins a newer Horizon revision. In particular, the current Coordinates HUD example demonstrates the general horizon-mixin.toml structure, but it is pinned to a different commit and cannot be copied as a Starling 0.8.0 implementation. This project will use only the pinned 0.8.0 source and compatible hook definitions.

## Next implementation gate

Before writing the first executable mixin, inspect the pinned source definitions for:
- the exact native screen render/click/scroll/key/typing hook signatures and their bindings;
- the exact native_edit_blocks edit structure and block-state representation;
- the exact column-read return type and palette indexing;
- the server hook's module path and the client screen's module path.

No placeholder .hnmod should be presented as installable until those definitions are resolved and the bundle passes Starling's package validator and build service.
## Exact helper signatures inspected at the pinned commit

Source files inspected at Horizon commit ac874422ed8f892505ff73d67ae486ecfee7a516:
- crates/horizon-client/src/game/native_world_edit.rs
- crates/horizon-client/src/game/native_world_columns.rs
- crates/horizon-client/src/game/native_hooks.rs
- native-hooks.json

### Server-side world edit

The exact edit payload is:
- NativeHeightmap: WorldSurface, MotionBlocking, MotionBlockingNoLeaves, OceanFloor.
- NativeBlockRun: x, z, y, height, optional surface heightmap, block palette index, optional filter index.
- NativeBlockEdit: palette: Vec<String>, filters: Vec<String>, runs: Vec<NativeBlockRun>.
- Helper: Game::native_edit_blocks(&mut self, dimension: &str, edit: &NativeBlockEdit) -> Result<u64, String>.

Important constraints from the source comments: this only edits the authoritative level currently being ticked; unloaded chunks and out-of-column portions are skipped; palette/filter entries with NBT or unreadable expressions are refused; block-entity behavior matches fill strict; neighbours are not notified; changed chunks are sent to players whole. This is not an arbitrary per-block transaction API and cannot by itself preserve original block-entity NBT for undo.

### Column reads

- NativeColumnBlocks contains palette: Vec<String>, bases: Vec<Option<i32>>, blocks: Vec<u16>, and tints: Vec<u32>.
- native_surface_heights(&self, dimension: &str, columns: &[(i32, i32)], heightmap: NativeHeightmap) -> Result<Vec<Option<i32>>, String>.
- native_read_columns(&self, dimension: &str, columns: &[(i32, i32)], heightmap: NativeHeightmap, range: (u32, u32)) -> Result<NativeColumnBlocks, String>.
- The source caps surface-height reads at 65,536 columns and column reads at 1,048,576 blocks; large reads must be split. Unloaded columns return no base / air in the read representation.
- Tint values are sampled at the column top and may differ from the biome-blended colours rendered near biome borders.

### Screen event wiring

The pinned native-hooks.json exposes these helper targets: mod_screen_open, mod_screen_close, mod_screen; and event hooks screen_render_post, screen_mouse_button_pressed_pre, screen_mouse_scrolled_pre, screen_key_pressed_pre, screen_char_typed_pre. The source signatures use NativeScreenView; render receives &mut HudBatch plus &NativeScreenView; mouse press receives button: i32, modifiers: u32 and &mut canceled; scroll receives [f64; 2], modifiers and &mut canceled; key receives key: &str, repeat: bool, modifiers and &mut canceled; character input receives char, modifiers and &mut canceled.

Screen mouse buttons are GLFW mouse events. The source explicitly says gamepad clicks do not reach mouse hooks. Android touch-to-mouse translation and touch gesture delivery on Starling 0.8.0 have not yet been verified, so this project must not assume the native screen is usable on touch until tested on-device. This is a release blocker for the POCO F5 target.

### Consequence for implementation

The API contract is now concrete enough to design the adapter types, but not enough to safely commit executable hook code: the native hook helper targets are not ordinary public Rust APIs, and the bundle's mixin declaration syntax and module loading still need to be verified against the exact pinned horizon-mixins revision. Undo must also be conservative: snapshot only readable block states, refuse unsupported block entities/NBT, and abort if any requested snapshot column is unloaded or incomplete. No bulk write should occur before the full preflight succeeds.