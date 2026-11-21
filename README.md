# Effortless Building — Starling Edition

- Project version: 1.2.0
- Port maintainer: MR
- Upstream author: Requios
- Intended target: Starling 0.9.0 (ARM64; source/build route under verification)
- Upstream license: LGPL-3.0-or-later

## Current state

This repository contains the versioned project scaffold, a standalone Rust geometry/transaction core, a historical 0.8.0 Horizon/Mixins audit, and a corrected source-level audit of Starling 0.9.0's native bundle path. It remains a source prototype: it has not been compiled, packaged as an .hnmod, or tested in Starling.

## Implemented in the pure Rust core

- Filled floor, cuboid, wall, circle, cylinder, sphere, square pyramid, cone and upper dome.
- Bounded geometry generation and checked coordinate conversion.
- Undo transaction ordering with repeated-position coalescing.
- Chunk-aware batch grouping, including negative world coordinates.
- Deterministic preview sampling with a hard point cap.

These routines do not access the game world. The preview sampler is not a placement list, and the undo planner is not connected to live world snapshots yet.

## Compatibility status

Starling v0.9.0 still accepts native .hnmod bundles. Its build pipeline uses a newer global horizon-mixins pin and includes an adapter for older hook targets. The old 0.8.0 pins in this repository are historical and are not presented as 0.9.0-compatible. The exact public build-service route for this project's bundle is still being verified.

See the 0.9.0 compatibility audit in docs/STARLING-0.9.0-COMPATIBILITY-AUDIT.md and porting status in docs/PORTING-STATUS.md.

## Not yet implemented

- Executable v0.9.0 native hook declarations and world read/write adapter.
- Native screen, touch-input isolation and mobile controls integration.
- Inventory accounting and authoritative Unlimited Blocks behavior.
- Live world snapshots, undo/redo persistence and conflict checks.
- Final .hnmod package, build-service compilation and in-game tests.

See the world-edit adapter contract in docs/WORLD-EDIT-ADAPTER-CONTRACT.md, porting status in docs/PORTING-STATUS.md and license notice in LICENSE-NOTICE.md.
