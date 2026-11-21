# Native Android build validation

## Latest result: successful compile

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
