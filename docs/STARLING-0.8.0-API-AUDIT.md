# Starling 0.8.0 native integration audit

## Confirmed from Starling's native-mod documentation
- Native mods are Rust/horizon-mixins bundles, not Java/NeoForge JARs.
- Starling packages native mods as .hnmod and compiles them through its build service; the resulting library is selected for the next game start.
- Native code executes with the game's full access. It must be treated as trusted code.
- A native screen is an overlay on the running world; it does not pause singleplayer. UI/input behavior therefore needs explicit testing.
- Starling 0.8.0 pins Horizon commit ac874422ed8f892505ff73d67ae486ecfee7a516 and horizon-mixins commit ddd9700b21997c95d95ac6ee5ecafe3830cf622a.

## Important distinction
The standalone geometry core in this repository is not a native mod by itself. It has no game hooks, no native screen and no world mutation capability. It cannot yet be imported into Starling as a working feature.

## API details still requiring source-level confirmation
Before implementing the adapter, inspect the exact pinned Horizon and mixins source for:
- native mod manifest and mixin hook syntax accepted by this release;
- screen registration/open/close and touch event dispatch;
- block-state reads, loaded-chunk checks and batch edit semantics;
- block entity, neighbor update, fluid, lighting and drop behavior;
- inventory reads/consumption and server-side validation;
- undo snapshot lifetime and world/dimension identity.

Do not infer these contracts from a newer Horizon Nightly or from API names mentioned in unrelated releases. In particular, do not assume a client-side bulk edit is authoritative in multiplayer.

## Release acceptance gates
1. Source applies cleanly with the exact pins.
2. Rust crate and game workspace pass the target build checks.
3. Package validator accepts the .hnmod.
4. Starling build service compiles the mod for ARM64.
5. POCO F5 runtime tests cover preview, cancel, confirm, each shape, undo/redo, inventory use, and touch controls.
