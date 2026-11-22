# Native SDK 0.2 migration plan

Base: `29308e65d4af8e96488561a9eb465f5b05501742`

Target source: Starling `ff303502a4f7982551a0e0d3622d9268efe9e829` (0.9.0).  
Horizon SDK build: `4dc6455d4a6773224792eb1f121a9b939dff3170` (published signed SDK release).

## Confirmed project format

The pinned Starling `cargo-horizon-mod new --template native` generator creates:
- `horizon-mod.toml` with `api = "0.2"`, `side = "both"`, and a `[native]` table pinning game `26.3`, Horizon revision, and the exact mixins tool revision.
- `Cargo.toml` with package edition 2024 and `horizon-mod-api = { path = ".horizon/sdk/api/crates/horizon-mod-api" }`.
- `src/lib.rs`, `assets/`, and `data/`.

The official package output is `.hmod`, not `.hnmod`. Native build is performed by the launcher against the pinned SDK; packaging source alone does not build the ARM64 game or the mod library.

## Migration map

Preserve the nine native mixins declared by the SDK 0.2 manifest:
- `use` -> legacy compatibility `Game::native_use_item_old`, HEAD, claims the held-item action when a selection point is recorded
- `world_preview` -> `Game::native_submit_custom_geometry`, HEAD
- `list` -> legacy compatibility `Game::native_mods`, HEAD
- `settings` -> legacy compatibility `Game::native_settings`, HEAD
- `click` -> legacy compatibility `Game::native_settings_click`, HEAD
- `world_close` -> `Game::finish_close_world`, HEAD; clears selection, queued UI action, preflight status and preview cache
- `tick` -> legacy compatibility `Game::native_tick`, HEAD
- `screen_render` -> `Game::native_screen_render_background`, HEAD
- `screen_click` -> `Game::native_screen_mouse_button_pressed_post`, HEAD

**Input limitation:** Starling 0.9.0 exposes the screen mouse-button callback as a Post event with no cancellation parameter. The SDK 0.2 mixin can detect taps inside the builder panel and queue UI actions, but it cannot prove or enforce suppression of the underlying screen/world click. There is no `screen_release` mixin in this package. Do not describe the current panel as consuming clicks.

Preserve the three game-module support modules by moving their source paths under `src/` and declaring corresponding `[[native.module]]` rows. Their destination remains `crates/horizon-client/src/game.rs`.

## Safety and validation gates

- Do not enable world writes in this migration.
- Keep current preflight read-only and retain the 8192 unique-target cap.
- Treat reach as a selection-anchor check: at least one selected corner must remain within reach; do not require every target block in a large area to be within reach. Continue validating every target for height, loaded chunk and world border.
- Keep screen-click handling explicitly non-cancellable until a supported pre-input hook is verified.
- Keep the Starling source pin, Horizon SDK build pin, game version, and mixins-tool pin as separate identities. The `[native] horizon` field must name a published Horizon SDK build, not the Starling source commit.
- Run source/layout validation before any native build.
- Do not rebuild the Starling ARM64 client. Only build the standalone native mod if and when a verified migration is ready.
- Confirm the produced package is `.hmod` and validate its manifest/layout; do not label the prior ARM64 ZIP as an importable mod package.

## Migration implementation status

The SDK 0.2 project skeleton has now been added. The first CI attempt exposed that `[native] horizon` incorrectly contained the Starling source SHA; it now points to the published Horizon SDK build above. The Starling checkout remains independently pinned in the workflow. The project includes:
- `horizon-mod.toml` pins the original Starling/Horizon and mixins revisions and declares all 9 mixins plus all 3 support modules.
- `Cargo.toml` follows the pinned generator's edition-2024 and local SDK dependency layout.
- `src/modules/` and `src/mixins/` contain source-preserving copies of all 12 legacy Rust files.
- `src/lib.rs` and the generated `.gitignore` are present.

The legacy `horizon-mixin.toml` and root-level Rust files remain as reference; they have not been deleted. GitHub Actions run #59 completed successfully on commit `585151f7cca5354f5c8a62efaa349a2c876a69a3`: it verified the signed SDK pin, validated declared helper-file presence and Rust syntax, ran `cargo horizon-mod check`, built and checked the versioned `.hmod`, and uploaded the package artifact. This workflow does not type-check injected helper code inside the Starling game crate, run the launcher's full native ARM64 build service, or prove in-game behavior. No world-write code has been enabled.


## World-session state retirement

The pinned Starling 0.9.0 source confirms that `Game::finish_close_world(&mut self) -> Result<()>` is the final client-world teardown path. The SDK migration adds a HEAD mixin at that exact method. It calls `reset_world_session()` before the engine releases the active world, clearing the process-global A/B selection, any preflight status, any queued builder UI action, and the coordinate-only preview cache. This hook is deliberately not tied to `native_window()` or the builder screen lifecycle: closing/reopening the panel in the same world keeps the selection, while closing the world retires it. The reset is conservative and also runs on the remote-client leave path before the engine's early return. The source helper and manifest are covered by the native SDK workflow's declared-file and Rust-syntax checks; full injected-helper type-checking and device verification remain separate gates.
