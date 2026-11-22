# MVP scope: four building tools

The first usable release is intentionally narrowed to four tools:

1. **Floor** — filled horizontal rectangle between two corners, at one y level.
2. **Wall** — filled rectangular wall footprint with a chosen height, rising from the first corner's y.
3. **Pillar** — one-block-wide vertical column from a base position and height.
4. **Cube** — filled inclusive cuboid between two corners.

The geometry core already contains floor, wall and cube. This change adds the dedicated one-block pillar generator. Other geometry routines remain in the pure core for now, but are not part of the MVP UI or acceptance criteria. They can be removed later if keeping the binary/source lean is worthwhile.

## Shared behavior required for all four tools

- Show a bounded preview that never edits the world.
- Require a separate explicit Apply confirmation.
- Read and validate the complete pre-edit snapshot for every target position.
- Reject unloaded columns, incomplete reads, unsupported block states and any uncertain precondition.
- Place in bounded batches through the authoritative Horizon server-side edit path only.
- Store stable block-state identifiers (not transient palette indexes) for Undo/Redo.
- Before Undo/Redo, verify the current world still matches the transaction's expected state.
- Never bypass server authority or claim that an edit succeeded unless the API reports the expected changed count.

## Mobile interaction

The first UI should be deliberately small: tool selection, two-corner/base selection, height input, preview, Apply, Undo and Redo. Avoid implementing a full desktop-style toolbar. Touch event routing must be verified against Starling's Android touch controls before enabling any overlay hit targets.

## Deferred

Circle, cylinder, sphere, cone, pyramid, dome, line drawing, copy/paste, rotation, mirroring, inventory simulation and multiplayer permission bypass are not MVP features.

A sample native mod is useful for validating actual bundle and hook syntax, but it does not change this four-tool scope.
