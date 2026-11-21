# Effortless Building — native mod 0.1.0

Creator / port maintainer: MR  
Upstream author: Requios  
Target: Starling 0.9.0 / Horizon Native SDK 0.2  
Pinned Starling source: `ff303502a4f7982551a0e0d3622d9268efe9e829`  
Pinned Horizon SDK build: `4dc6455d4a6773224792eb1f121a9b939dff3170`  
Pinned horizon-mixins revision: `5a64af938ec86b00d53d16a408aabc0038fb3e57`

## Current native source functionality

- Registers in the native Mods list and exposes Floor, Wall, Pillar and Cube.
- Opens a dedicated builder screen with tool selection and a Close action.
- Captures the main-hand block use only while holding a stick and only for the local window player. Delegated LAN actors are excluded because the current selection store is process-global; this prevents different players from sharing or overwriting selection points. First use records point A, second records B, and third starts a new pair.
- Retires points A/B, preflight status, queued UI actions, and the cached preview plan at Starling's world-close lifecycle hook. Closing/reopening only the builder panel within the same world preserves the selection.
- Uses Starling's own block ray-pick result; other held items retain normal use behavior.
- Draws a read-only selection wireframe and up to 64 spatially distributed sample crosses.
- Reports the geometric target-position count and warns when the selection exceeds the core's 8,192-position limit.

## Safety and implementation boundary

This native milestone is read-only. It does not place or remove blocks, consume inventory, bypass multiplayer authority, or implement live undo/redo. The pure Rust core contains geometry, snapshot-gated planning and guarded history primitives. The native preflight path is not yet a complete transactional world-edit/commit implementation.

No world-write path is enabled.

## Package and build model

- Source directory: `native/effortless-building-v0.1.0/`
- Package filename: `effortless_building-0.1.0.hmod`
- Package manifest: `horizon-mod.toml`
- Creator attribution: `authors = ["MR"]`

This is a Horizon Native SDK 0.2 flat native package. It is **not** the older Starling 0.8 `horizon-native.json` / `horizon-mixin.toml` bundle consumed by `packaging/android/native_build.sh`. That legacy Android build script cannot compile this package and must not be used as its build path.

For SDK 0.2, the launcher compiles the native source package against the exact pinned Horizon build and links it with the matching prebuilt game. The game itself is not rebuilt for each mod. The resulting `.hmod` is source input, not an Android shared library and not directly installable as a finished APK.

## What CI currently proves

The `native-sdk-check` workflow successfully:
- verifies the pinned SDK metadata/signature,
- validates the mod crate with `cargo horizon-mod check`,
- checks declared helper-file presence and Rust syntax,
- packages and re-checks the `.hmod` artifact.

It does **not** yet prove that every injected helper type-checks inside the complete game crate, that all mixin target/binding combinations compile, that the launcher imports this exact package successfully, or that the mod runs in Minecraft on Android. Those remain separate integration and runtime gates.
