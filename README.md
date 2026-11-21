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
version-pinned package metadata. The native screen renderer and mouse-press Post callback are wired against
Starling 0.9.0. The Post callback has no cancellation parameter, so click
suppression/consumption is not established; in-game interaction still needs
verification. Tool selection and close actions are queued from that callback.
The screen now explains the two-point workflow. With the mod enabled, a
main-hand right-click while holding a stick records point A, a second records
point B, and a third starts a new pair. Other held items retain normal use.
This uses the game's own block ray pick and refuses remote multiplayer clients.
A separate read-only world-render hook draws a selection wireframe plus up to
64 evenly distributed sample crosses. The screen reports the geometric
target-position count and flags selections above the 8,192-position native planner
limit. These previews do not read block states or modify the world. The latest SDK 0.2 workflow validates declared helper syntax, the signed SDK
pin, manifest, and `.hmod` packaging. It does not type-check injected helpers
inside the full Starling game crate or validate launcher import; those remain
separate gates.

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
- Bounded in-memory build history with separate undo/redo stacks; history
  moves only after the caller confirms the authoritative edit succeeded.
- Deterministic capped preview sampling.
- A confirmation session that keeps preview separate from commit intent and
  consumes a confirmation only once.

These are pure planning routines. They do not themselves read or modify the
live game world.

## Starling 0.9.0 integration

The active migration branch uses Starling's Native SDK 0.2 manifest and
versioned `.hmod` source package. The older `.hnmod` bundle is retained only
as a reference; it is not the SDK 0.2 install artifact. GitHub Actions checks
the pinned SDK, declared source files, Rust syntax, manifest, and `.hmod`
packaging. This does not type-check every injected helper inside the game
crate or prove launcher-side import.

The current native mixin follows Starling's in-process Wrench pattern: read-only
world checks through `Game`, then (when implemented) block behaviour calls
through `Game::with_level`. The read-only preflight adapter now snapshots and
revalidates selected target states, but no write path is connected. The required
safety contract is documented in [docs/WORLD-EDIT-ADAPTER-CONTRACT.md](docs/WORLD-EDIT-ADAPTER-CONTRACT.md). Client-side/remote multiplayer edits, implicit chunk loading, and
block-entity NBT preservation are explicitly out of scope for the first safe
implementation.

## Not yet implemented or verified

- Per-block ghost-state preview and live world snapshot/block-edit adapter.
- Placement commit, conflict-aware live undo/redo and persistent history.
- Inventory consumption and authoritative Unlimited Blocks behavior.
- Runtime testing in Starling 0.9.0 on Android/POCO F5.
- Type-checking all injected native helper code against the pinned game crate.
- Launcher build-service import validation and a tested release package.

See [docs/PORTING-STATUS.md](docs/PORTING-STATUS.md),
[docs/STARLING-0.9.0-COMPATIBILITY-AUDIT.md](docs/STARLING-0.9.0-COMPATIBILITY-AUDIT.md),
and [docs/WORLD-EDIT-ADAPTER-CONTRACT.md](docs/WORLD-EDIT-ADAPTER-CONTRACT.md).
