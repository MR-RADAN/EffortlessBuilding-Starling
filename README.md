# Effortless Building — Starling Edition

- Project version: 1.2.0
- Native bundle version: 0.1.0
- Port maintainer / native bundle creator: MR
- Upstream author: Requios
- Target: Starling 0.9.0 / Horizon ARM64
- Upstream license: LGPL-3.0-or-later

## Current state

This repository contains a Starling 0.9.0 native bundle scaffold, a mobile
builder tool-selection screen, a pure Rust geometry/transaction core, and
version-pinned package metadata. The native screen can open, render, select
Floor/Wall/Pillar/Cube and close while capturing clicks inside its panel.
The screen now explains the two-point workflow. With the mod enabled, a
main-hand right-click while holding a stick records point A, a second records
point B, and a third starts a new pair. Other held items retain normal use.
This uses the game's own block ray pick and refuses remote multiplayer clients. The new hook is awaiting native build
validation; it does not preview or modify blocks.

## Implemented in the pure Rust core

- Filled floor, cuboid, wall, circle, cylinder, sphere, square pyramid, cone,
  upper dome, and line geometry.
- Bounded geometry generation and checked coordinate conversion.
- Chunk-aware run grouping, including negative world coordinates.
- Column snapshot decoding with strict palette, range and unloaded-column
  rejection.
- Two-corner geometry mapping for Floor, Wall, Pillar and Cube, with bounded
  output and inclusive vertical spans.
- Snapshot-gated placement planning; missing or duplicate snapshot entries
  fail closed.
- Guarded undo/redo planning that rejects stale world state rather than
  overwriting later player edits.
- Deterministic capped preview sampling.
- A confirmation session that keeps preview separate from commit intent and
  consumes a confirmation only once.

These are pure planning routines. They do not themselves read or modify the
live game world.

## Starling 0.9.0 integration

The native bundle manifest and legacy `.hnmod` source layout follow the
v0.9.0 importer contract. The native hook scaffold is adapted against the
pinned source and is being validated by GitHub Actions.

Starling 0.9.0 exposes an authoritative server-side bulk block-edit path and
bounded column snapshot reads. The required safety contract is documented in
[docs/WORLD-EDIT-ADAPTER-CONTRACT.md](docs/WORLD-EDIT-ADAPTER-CONTRACT.md).
The adapter that connects those engine APIs to this mod is not implemented
yet. Client-side/remote multiplayer edits, implicit chunk loading, and
block-entity NBT preservation are explicitly out of scope for the first safe
implementation.

## Not yet implemented or verified

- In-world preview markers and live world snapshot/block-edit adapter.
- Live world snapshot and block-edit adapter.
- Placement commit, conflict-aware live undo/redo and persistent history.
- Inventory consumption and authoritative Unlimited Blocks behavior.
- Runtime testing in Starling 0.9.0 on Android/POCO F5.
- A final phone-importable `.hnmod` package.

See [docs/PORTING-STATUS.md](docs/PORTING-STATUS.md),
[docs/STARLING-0.9.0-COMPATIBILITY-AUDIT.md](docs/STARLING-0.9.0-COMPATIBILITY-AUDIT.md),
and [docs/WORLD-EDIT-ADAPTER-CONTRACT.md](docs/WORLD-EDIT-ADAPTER-CONTRACT.md).
