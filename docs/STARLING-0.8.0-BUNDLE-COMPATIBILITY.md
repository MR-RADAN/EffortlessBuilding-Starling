# Native bundle compatibility findings — Starling 0.8.0

Date: 2026-11-21

Target game revision:
`ac874422ed8f892505ff73d67ae486ecfee7a516`

Target mixin revision:
`ddd9700b21997c95d95ac6ee5ecafe3830cf622a`

## Why the bundle skeleton is not copied from the current registry

The current `horizon-rust/horizon-mods` examples are not automatically valid templates for the Starling 0.8.0 pin. For example, the current Coordinates HUD and Recipes manifests declare Horizon revision `4baa571566e39bfd6594988d3bb2bc85440c30e7`, not the 0.8.0 revision above. Their hooks and module declarations therefore must not be copied into this project without a source-level compatibility check.

The current native-mod audit for Starling 0.8.0 also reports that a Rust `cargo check` is not an APK-link or in-game test. It records both injection failures and Rust API mismatches among packages that passed the bundle archive validator. This project will keep those validation stages separate.

## Verified runtime API shape in the 0.8.0 source

The exact Starling 0.8.0 source exposes native helpers through Horizon's native mixin hooks, not as a standalone public Rust SDK that an arbitrary crate can call directly.

Verified capabilities include:

- `native_view_hit` / `native_raycast` for a block hit.
- `native_read_columns` for palette-indexed block snapshots, with explicit unloaded-column markers.
- `native_edit_blocks` for authoritative bulk block edits using palette entries and vertical runs.
- Native mod registration and mod-specific screens/settings through native hook points.
- Input hooks for key, mouse, drag, wheel and screen events.

The implementation must bind these through a manifest and helper functions that match the exact pinned mixin source. A regular Cargo crate calling these methods directly would be an incorrect integration.

## Safety decisions

- Reject a placement if any target column is unloaded or any target block lacks a pre-edit snapshot.
- Keep preview read-only; never call the edit helper during preview.
- Require a separate explicit confirmation action before placement.
- Cap a single confirmed edit to a bounded operation size; do not split an undoable operation into multiple independently committed batches.
- Before undo/redo, compare the current state against the transaction's expected state. Refuse stale transactions rather than overwriting player or server changes.
- Use the server-authoritative edit helper only. No client-side packet spoofing or multiplayer bypass.
- Start with a textual coordinate/count preview. Ghost-block rendering is deferred until the exact render hook and touch behavior are validated on device.

## Next integration gate

Before writing the `.hnmod` bundle, inspect the 0.8.0-pinned mixin repository's manifest schema, helper binding types and native hook definitions. Then implement one minimal registration hook and compile it against the exact release pins. Only after that gate passes should world-edit hooks and the mobile UI be added.

A package archive being accepted by the importer is not proof that the mixin injects, compiles, links or runs in-game.
