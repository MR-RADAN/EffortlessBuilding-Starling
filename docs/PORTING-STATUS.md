# Porting status — 1.2.0

## Target pins
- Starling: 0.8.0
- Horizon: ac874422ed8f892505ff73d67ae486ecfee7a516
- horizon-mixins: ddd9700b21997c95d95ac6ee5ecafe3830cf622a

These pins must not be replaced with newer template pins without checking the user's installed Starling build.

## Implemented source core
- Deterministic geometry: line, wall, floor, cuboid, circle, cylinder, sphere, pyramid, cone and dome.
- Bounded block counts and coordinate conversion.
- Reverse transaction planning with repeated-position coalescing.
- Chunk-aware batch grouping, including negative coordinates.
- Deterministic capped preview sampling.

## Not implemented or verified
- No Horizon hooks are connected.
- No native screen or Android touch-input isolation exists yet.
- No world read/write adapter exists.
- No inventory consumption or Unlimited Blocks implementation exists.
- No server-authority or multiplayer behavior has been tested.
- No .hnmod package has been produced.
- Rust tests and Cargo checks have not run here because Rust/Cargo are unavailable.

## Adapter safety requirements
1. Preview must be read-only.
2. Bulk placement requires explicit confirmation.
3. Snapshot original states before mutation; abort if the snapshot is incomplete.
4. Bound operation size and undo memory; do not load chunks just to place blocks.
5. Verify current state before undo/redo to avoid overwriting later edits.
6. Inventory changes must use the authoritative game/server path.
7. Unlimited Blocks is only valid where the authoritative server accepts edits; it is not a client-side multiplayer bypass.
8. The native screen runs over the world and does not pause singleplayer. Touch isolation must not modify Starling, MCPE or GoyControl touch layouts.

## Starling 0.9.0 compatibility gate

The owner reports an installed app version of 0.9.0, but the public Starling release/build-service sources inspected on 2026-11-21 establish only a 0.8.0 release mapping in `starling-client/mod-builds/versions.json`. The existing pins below therefore remain historical 0.8.0 pins and must not be presented as compatible with the reported 0.9.0 app. See [the compatibility audit](STARLING-0.9.0-COMPATIBILITY-AUDIT.md). Do not package or claim a 0.9.0 native bundle until the installed APK's exact source commit and matching build-service version/cache are identified.

## Next gate
Inspect exact 0.8.0 Horizon and horizon-mixins source for hook names, signatures and block-edit semantics. Only then add hook configuration and the native adapter. Do not guess hook names.
