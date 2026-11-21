# Native SDK 0.2 migration plan

Base: `29308e65d4af8e96488561a9eb465f5b05501742`

Target: Starling `ff303502a4f7982551a0e0d3622d9268efe9e829` (0.9.0).

## Confirmed project format

The pinned Starling `cargo-horizon-mod new --template native` generator creates:
- `horizon-mod.toml` with `api = "0.2"`, `side = "both"`, and a `[native]` table pinning game `26.3`, Horizon revision, and the exact mixins tool revision.
- `Cargo.toml` with package edition 2024 and `horizon-mod-api = { path = ".horizon/sdk/api/crates/horizon-mod-api" }`.
- `src/lib.rs`, `assets/`, and `data/`.

The official package output is `.hmod`, not `.hnmod`. Native build is performed by the launcher against the pinned SDK; packaging source alone does not build the ARM64 game or the mod library.

## Migration map

Preserve the existing nine native injection helpers as native mixins:
- `use` -> `Game::use_item`, HEAD, cancellable
- `world_preview` -> `Game::native_submit_custom_geometry`, HEAD
- `list` -> `Game::native_mods`, HEAD
- `settings` -> `Game::native_settings`, HEAD
- `click` -> `Game::native_settings_click`, HEAD
- `screen_render` -> `Game::native_screen_render_post`, HEAD
- `screen_click` -> `Game::native_screen_mouse_button_pressed_pre`, HEAD
- `screen_release` -> `Game::native_screen_mouse_button_released_pre`, HEAD
- `tick` -> `Game::native_tick`, HEAD

Preserve the three game-module support modules by moving their source paths under `src/` and declaring corresponding `[[native.module]]` rows. Their destination remains `crates/horizon-client/src/game.rs`.

## Safety and validation gates

- Do not enable world writes in this migration.
- Keep current preflight read-only and retain the 8192 unique-target cap.
- Preserve the existing exact game and mixins pins; do not regenerate pins from a newer Starling build.
- Run source/layout validation before any native build.
- Do not rebuild the Starling ARM64 client. Only build the standalone native mod if and when a verified migration is ready.
- Confirm the produced package is `.hmod` and validate its manifest/layout; do not label the prior ARM64 ZIP as an importable mod package.

## Outstanding implementation

This branch currently records the verified migration plan only. No source files have been moved or rewritten yet. Next implementation change should add the generated-format manifests and migrate helpers/modules with exact source preservation, then run package/layout checks before any native compilation.
