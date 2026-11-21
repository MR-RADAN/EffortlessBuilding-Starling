# Effortless Building — native bundle 0.1.0

Creator / port maintainer: MR  
Upstream author: Requios  
Target: Starling 0.9.0 / Horizon ARM64  
Pinned Starling source: `ff303502a4f7982551a0e0d3622d9268efe9e829`  
Pinned Horizon SDK build: `4dc6455d4a6773224792eb1f121a9b939dff3170`

## Current native source functionality

- Registers in the native Mods list and exposes Floor, Wall, Pillar and Cube.
- Opens a dedicated builder screen with tool selection and a Close action.
- Captures the main-hand block use only while holding a stick and only for the local window player. Delegated LAN actors are excluded because the current selection store is process-global; this prevents different players from sharing or overwriting selection points. First use records point A, second records B, and third starts a new pair.
- Uses Starling's own block ray-pick result; other held items retain normal use behavior.
- Draws a read-only selection wireframe and up to 64 spatially distributed sample crosses.
- Reports the geometric target-position count and warns when the selection exceeds the core's 8,192-position limit.

## Safety and implementation boundary

This native milestone is read-only. It does not inspect live block states, place or remove blocks, consume inventory, bypass multiplayer authority, or implement live undo/redo. The pure Rust core contains geometry, snapshot-gated planning and guarded history primitives, but those are not yet connected to Starling's authoritative server-side edit API.

GitHub Actions verifies the signed Horizon SDK pin, checks that every declared helper file exists, parses each helper with rustfmt, and checks the standalone mod crate. This does not type-check injected helpers inside the game crate, validate every Mixin target/binding, or prove launcher import. Native compilation and in-game Android testing remain separate gates. Even after ARM64 compilation succeeds, in-game Android testing is still required.

## Package

- Native bundle directory: `native/effortless-building-v0.1.0/`
- Package filename: `effortless_building-0.1.0.hmod`
- Metadata file: `horizon-native.json`
- All Rust module sources and helper files referenced by `horizon-mixin.toml` must be included in the package.


## Import package build

The `.hmod` archive is a source bundle for Starling's native build service; it intentionally does not embed `libhorizon_android.so`. The ARM64 shared library is produced by the pinned Starling native build pipeline and is validated separately. The package workflow checks the archive layout, metadata, and every Rust/TOML source referenced by `horizon-mixin.toml`.
