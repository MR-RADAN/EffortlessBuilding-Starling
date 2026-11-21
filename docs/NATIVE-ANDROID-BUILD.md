# Native Android build validation

## Historical result: successful Starling 0.8.0 compile

The GitHub Actions run [44834111100](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44834111100) completed successfully for commit `48d00a4e2f2ab4030b4cf0ec2412073d9d685695`.

The job checked out Starling tag `v0.8.0`, verified Horizon revision
`ac874422ed8f892505ff73d67ae486ecfee7a516`, installed the Android SDK/NDK
and Rust Android toolchain, and ran Starling's upstream
`packaging/android/native_build.sh` with `NATIVE_LIBRARY_ONLY=1`.

The workflow produced artifact `effortless-building-starling-0.8.0-arm64`
(about 31.5 MB; SHA-256:
`0908140e5441bd2f222d426e2ee556901e3e0b130aa0165a15eabb7333ef6bae`).
It contains `libhorizon_android.so` and `build.json`. The artifact expires
2026-11-27. It is a compiled game library, not an APK and not a standalone
installable mod package.

The source bundle was also packaged successfully in run
[44834111076](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44834111076).
That artifact is `effortless_building-0.1.0-hnmod` (5,742 bytes; SHA-256:
`deaf45e50b957f41960ddf0fc5e1ee8739d4677a186185dd848c820031cf6924`).
The .hnmod is a source bundle intended for the Starling launcher, which
compiles it against the launcher's pinned Horizon revision.

## What success proves — and what it does not

The successful native build proves that the current mixin bundle applies and
compiles against the pinned Starling 0.8.0 source for Android ARM64.

It does **not** prove that the UI renders correctly or that touch input works
on the POCO F5. Those require importing the .hnmod into Starling 0.8.0 and
testing it in-game.

The current UI milestone does not yet select world coordinates, render a
spatial preview, read block snapshots, place blocks, or implement undo/redo.
No block-edit API is called, so this milestone cannot alter the world.


## Starling 0.9.0 validation — hook ID correction and ARM64 retry

The project target pins and native package manifests have been updated to:
- Starling/Horizon source: `ff303502a4f7982551a0e0d3622d9268efe9e829`
- global `horizon-mixins`: `5a64af938ec86b00d53d16a408aabc0038fb3e57`

The GitHub Actions workflow now checks out the exact `v0.9.0` tag and verifies that commit before invoking Starling's own `native_build.sh`.

- 0.9.0 build run [44875302971](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44875302971) failed before Rust compilation because `rust-src` was missing. The mixin transformation completed with seven injections and one module. The workflow now installs `rust-src`.
- The first corrected retry [44875870180](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44875870180) passed source checkout, revision verification, SDK installation and Rust setup, then entered native compilation.
- The latest full build attempt [44876927294](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44876927294) reached Starling's native build adapter and transformed the project hooks, but stopped before Rust compilation with `E_DUPLICATE_ID`: the preview module and injection shared the ID `preview`. The injection has been renamed to `world_preview`.
- The corrected retry [44877289336](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44877289336) is now running against the latest source.
- The package workflow [44877289299](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44877289299) is also running for the corrected manifest.
- The Rust core suite, including bounded BuildHistory and conflict-aware undo/redo tests, passed in run [44877215141](https://github.com/MR-RADAN/EffortlessBuilding-Starling/actions/runs/44877215141).

A successful package ZIP workflow only proves archive generation and its local structural checks. It does not prove that Starling's 0.9.0 importer accepts it or that the native bundle compiles. The retry must finish successfully before claiming source compatibility.

The 0.9.0 source audit also confirmed the supported server-side edit path and column snapshot API. The world-edit adapter contract was rewritten against those exact 0.9.0 sources; it no longer describes the obsolete 0.8.0 helper names or run layout. The actual mod adapter remains unimplemented.

