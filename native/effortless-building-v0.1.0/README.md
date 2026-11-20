# Effortless Building — native bundle 0.1.0

Maintainer: MR

Target: Starling 0.8.0 / Horizon 26.3, pinned to the release revisions.

## Current functionality

- Registers in Options > Mods.
- Persists selection of Floor, Wall, Pillar or Cube.
- Opens a dedicated native mod screen from the settings page.
- Renders a compact tool-selection panel using Starling's native screen render hook.
- Routes left clicks through the native screen hooks and applies tool selection on the window tick.
- Escape closes the dedicated screen; the panel also has a Close button.

The UI is a first native integration milestone, not the finished building mod. It does not yet select world coordinates, render a spatial/ghost preview, read block snapshots, place blocks, or implement undo/redo. The screen's footer explicitly says that no blocks are changed.

## Touch and input

Starling 0.8.0's source converts touch-pointer actions into cursor and mouse-button window events, and the native screen hooks consume left clicks on the panel. This supports the intended touch path at source level; Android in-game behavior still needs device testing.

## Package

The source-only .hnmod requests native hooks API 111 for the custom mod screen. The launcher compiles the bundled Rust source against its pinned Horizon revision. Successful ZIP packaging is not proof that the native mixin compiles or that the UI works in-game.

## Security

No block-edit API is called by this milestone. World mutation remains disabled until the preview, snapshot validation, authoritative server adapter and explicit confirmation flow are connected and tested.


## Native compilation validationA separate GitHub Actions workflow builds the ARM64 native library against
the exact Starling 0.8.0 release source. See
[Native Android build validation](../../docs/NATIVE-ANDROID-BUILD.md).
The source-bundle ZIP check alone is not a native compilation check.
