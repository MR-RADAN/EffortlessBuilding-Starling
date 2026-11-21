# Effortless Building — native bundle 0.1.0

Creator / port maintainer: MR  
Upstream author: Requios  
Target: Starling 0.9.0 / Horizon ARM64  
Pinned Starling source: `ff303502a4f7982551a0e0d3622d9268efe9e829`

## Current native source functionality

- Registers in the native Mods list and exposes Floor, Wall, Pillar and Cube.
- Opens a dedicated builder screen with tool selection and a Close action.
- Captures the main-hand block use only while holding a stick and only on the local game window. First use records point A, second records B, and third starts a new pair.
- Uses Starling's own block ray-pick result; other held items retain normal use behavior.
- Draws a read-only selection wireframe and up to 64 spatially distributed sample crosses.
- Reports the geometric target-position count and warns when the selection exceeds the core's 16,384-position limit.

## Safety and implementation boundary

This native milestone is read-only. It does not inspect live block states, place or remove blocks, consume inventory, bypass multiplayer authority, or implement live undo/redo. The pure Rust core contains geometry, snapshot-gated planning and guarded history primitives, but those are not yet connected to Starling's authoritative server-side edit API.

The source hooks and package layout are being validated against the exact Starling 0.9.0 checkout by GitHub Actions. A successful source-package ZIP check is not proof of native compilation or launcher import. Even after ARM64 compilation succeeds, in-game Android testing is still required.

## Package

- Native bundle directory: `native/effortless-building-v0.1.0/`
- Package filename: `effortless_building-0.1.0.hnmod`
- Metadata file: `horizon-native.json`
- All Rust module sources and helper files referenced by `horizon-mixin.toml` must be included in the package.

