# Starling 0.8.0 build-service compatibility resolution

This document corrects an earlier uncertainty in
`NATIVE-BUNDLE-INTEGRATION-FINDINGS.md`.

## Confirmed Mixins revision

The Starling branch's `packaging/android/native_build.sh` explicitly sets:

```
MIXINS_REV=ddd9700b21997c95d95ac6ee5ecafe3830cf622a
```

The same value is used by Starling's launcher in
`launcher/src/native_mods.rs`. This is exactly the Mixins revision already
pinned in this project. Therefore there is no unresolved global Mixins pin
mismatch for this target.

The in-tree Wrench bundle's `horizon-native.json` records a different
`mixins` value because that is the revision its author tested the bundle
against. It is not the global build-service revision. Starling's build script
copies bundles and rewrites an explicit `compatibility.horizon_revision`
to the Starling source commit; it invokes the global pinned Mixins CLI.

## Build-service contract confirmed from Starling documentation and script

- The input is a bundle directory containing `horizon-mixin.toml`.
- Starling copies the bundle into a temporary build workspace.
- It pins explicit Horizon revision compatibility to the source commit being
  built.
- It runs `horizon-mixins apply --workspace ... --mods ... --out ...`.
- It builds the Android game library for the target ABI and emits
  `build.json` containing the build key, Starling/Horizon commit, profile,
  ABI, Mixins revision, mod metadata and SHA-256.
- The player imports an `.hnmod` into `horizon-native-mods/`; Starling's
  build service compiles it and swaps the resulting native library on the
  next start. The phone does not compile Rust locally.

## Important correction to previous status

The earlier statement that the Wrench example's different metadata hash
implied a Starling 0.8.0 build-service mismatch was incorrect. The build
script's global Mixins revision has now been verified and matches this
project's pin exactly.

## Remaining release gates

- Confirm the exact `.hnmod` registry/import manifest schema and required
  metadata fields against the Starling 0.8.0 importer.
- Add a minimal real bundle using only verified hook targets and exact
  bindings.
- Run the actual pinned `horizon-mixins apply` and Android build service.
- Validate Android touch delivery and interaction on a real POCO F5.
- Validate world edits, confirmation, batching, and undo behavior in game.

No installable artifact or successful native build is claimed by this note.
