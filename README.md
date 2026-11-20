# Effortless Building — Starling Edition

- Project version: 1.2.0
- Port maintainer: MR
- Upstream author: Requios
- Intended target: Starling 0.8.0 (ARM64)
- Upstream license: LGPL-3.0-or-later

## Current state

This repository now contains the versioned project scaffold, a standalone Rust geometry/transaction core, and porting safety requirements. It is still a source prototype: it has **not** been compiled, packaged as an .hnmod, or tested in Starling.

## Implemented in the pure Rust core

- Filled floor, cuboid, wall, circle, cylinder, sphere, square pyramid, cone and upper dome.
- Bounded geometry generation and checked coordinate conversion.
- Undo transaction ordering with repeated-position coalescing.
- Chunk-aware batch grouping, including negative world coordinates.
- Deterministic preview sampling with a hard point cap.

These routines do not access the game world. The preview sampler is not a placement list, and the undo planner is not connected to live world snapshots yet.

## Target compatibility pins

- Horizon: ac874422ed8f892505ff73d67ae486ecfee7a516
- horizon-mixins: ddd9700b21997c95d95ac6ee5ecafe3830cf622a

Do not substitute newer template pins without checking the installed Starling build.

## Not yet implemented

- Horizon native hooks and world read/write adapter.
- Native screen, touch-input isolation and mobile controls integration.
- Inventory accounting and authoritative Unlimited Blocks behavior.
- Persistent undo/redo snapshots and conflict checks.
- Final .hnmod package, build-service compilation and in-game tests.

See [the porting status](docs/PORTING-STATUS.md) and [license notice](LICENSE-NOTICE.md).
