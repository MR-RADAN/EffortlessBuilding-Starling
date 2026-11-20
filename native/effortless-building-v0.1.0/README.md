# Effortless Building — native scaffold 0.1.0

This directory is the first native integration scaffold for Starling 0.8.0.

- Bundle ID: `effortless_building`
- Version: `0.1.0`
- Author/maintainer: MR
- Target Horizon revision: `ac874422ed8f892505ff73d67ae486ecfee7a516`

## What this stage implements

- A native mod entry in Options > Mods.
- A settings page with a selectable tool label: Floor, Wall, Pillar or Cube.
- The selected tool index is persisted with Starling's native per-mod setting helper.

## What it does not implement yet

- No world read or block placement is called.
- No preview, confirmation, undo or redo is wired.
- The settings page explicitly says placement is disabled.
- This scaffold has not yet been passed through Starling's v0.8.0 build service or tested in-game.

It is deliberately non-destructive: importing or enabling this stage cannot modify a world.
