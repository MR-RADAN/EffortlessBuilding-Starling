# Effortless Building — native bundle 0.1.0

Maintainer: MR

This source bundle targets Starling 0.8.0 / Horizon 26.3.

## Package layout

The eventual `.hnmod` must contain:
- `horizon-native.json`
- `mixins/effortless_building/horizon-mixin.toml`
- `mixins/effortless_building/*.rs`

The manifest pins the exact Horizon and horizon-mixins revisions used by Starling 0.8.0.

## Current functionality

The mixin registers the mod in Options > Mods and exposes a persisted tool selector:
Floor, Wall, Pillar, Cube.

This is a registration-only prototype. It does not yet read or edit the world, render a spatial preview, or implement confirmation/undo/redo. Do not treat it as a finished building mod.

## Security

Native mods are compiled into the game and run with the game's full privileges. Review source before compiling. This prototype has no network, filesystem, or world-edit behavior beyond Starling's built-in settings helper.
