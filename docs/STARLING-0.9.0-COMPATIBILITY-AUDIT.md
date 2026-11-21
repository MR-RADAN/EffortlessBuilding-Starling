# Starling 0.9.0 compatibility audit — 2026-11-21

## Result: native bundle target is not yet verified

The project owner reports that the installed Android app displays version **0.9.0**. The public Starling sources and build-service map inspected for this audit do not currently establish a matching 0.9.0 native-build target:

- The public Starling releases page currently identifies **v0.8.0** as the latest release.
- The public `starling-client/mod-builds/versions.json` currently maps only `0.8.0` to Horizon commit `ac874422ed8f892505ff73d67ae486ecfee7a516`.
- The current Starling native-mod documentation explains that the phone submits an `.hnmod` source bundle to a build service, which compiles against the Horizon revision registered for that Starling version. The resulting native library is tied to the installed APK build and must be rebuilt after an APK update.
- Native mods are Rust source mixin bundles. The `.hnmod` format is still supported; it is not the old Starling-specific/WASM format.

Sources checked:
- Starling releases: https://github.com/starling-client/starling/releases
- Starling native-mod build flow: https://github.com/starling-client/starling/blob/starling/docs/starling-native-mods.md
- Build-service version map: https://github.com/starling-client/mod-builds/blob/main/versions.json
- Horizon native-mod template: https://github.com/horizon-rust/horizon-native-mod-template

## Consequence for Effortless Building

Do **not** change the project's Horizon or horizon-mixins pins to guessed values, and do not package a bundle claiming Starling 0.9.0 compatibility yet. The existing repository pins are for the 0.8.0 target and cannot be assumed compatible with the user's reported 0.9.0 installation.

Before implementing the live world adapter, identify the exact source/build commit used by the installed 0.9.0 APK and confirm that Starling's build service has a corresponding `versions.json` entry and base build cache. If 0.9.0 is a locally built, preview, or forked APK, its exact source commit and build-service configuration are required.

## Safety and implementation boundary

The existing Rust geometry and transaction planner remains useful as a pure, testable core. It does not itself read or mutate the game world. The live adapter must be implemented only against verified hook declarations and the authoritative Horizon edit API for the exact pinned revision.

Until that work is complete:
- preview and cancel remain read-only;
- no placement is advertised as working;
- no undo/redo is advertised as connected to the live world;
- no inventory or Unlimited Blocks behavior is promised;
- no installable `.hnmod` is claimed to have been compiled or tested.

## Maintainer

MR
