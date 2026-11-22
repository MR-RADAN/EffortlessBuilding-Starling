# CI artifact verification — 2026-11-22

Creator / port maintainer: MR  
Project: Effortless Building 1.2.0  
Native package: 0.1.0  
Target: Starling 0.9.0 / Android arm64-v8a

## Workflow result

GitHub Actions run 120 completed successfully on branch `migration/native-sdk-0.2`.

- Commit: `0d2ee7f2c1684f72aae86c737c1040d9db523cec`
- Workflow: `.github/workflows/native-sdk-check.yml`
- `native-sdk-check`: success
- `android-arm64-build`: success

## Downloaded artifact inspection

### HMOD source package

The workflow artifact `effortless-building-v0.1.0-hmod` contains
`effortless_building-0.1.0.hmod`. The inner HMOD is a valid ZIP archive and
contains the SDK 0.2 manifest, Cargo metadata, README, three module sources,
and nine mixin helper sources. The manifest declares:

- ID: `effortless_building`
- Version: `0.1.0`
- Creator: `MR`
- API: `0.2`
- Game: `26.3`
- Horizon SDK revision: `4dc6455d4a6773224792eb1f121a9b939dff3170`
- horizon-mixins revision: `5a64af938ec86b00d53d16a408aabc0038fb3e57`

This confirms archive and manifest structure. It does not prove that the
Starling launcher imports or builds this exact HMOD through its own build
service.

### Android ARM64 build artifact

The workflow artifact `effortless-building-v0.1.0-arm64-native` contains
`libhorizon_android.so` and `build.json`.

Verified from the downloaded bytes:

- ELF class: ELF64
- ELF machine: AArch64
- Android ABI: `arm64-v8a`
- Starling version: `0.9.0`
- Starling source commit: `ff303502a4f7982551a0e0d3622d9268efe9e829`
- horizon-mixins revision: `5a64af938ec86b00d53d16a408aabc0038fb3e57`
- Included mod: `effortless_building` version `0.1.0`
- Shared-library size: 111,317,328 bytes
- SHA-256: `406fb56fbcbce9fdc5468c28c68f4cbc5353e4bdaa7a19bdc364e96016447a53`

The SHA-256 independently calculated from the downloaded shared library
matches `build.json`.

**Important build-path distinction:** the workflow first validates and packages
the SDK 0.2 HMOD with `cargo horizon-mod check/build`. For its separate
ARM64 compilation job, it runs `scripts/adapt_hmod_for_android.py` to
translate the manifest structure into the legacy `horizon-mixins` bundle
layout, then invokes Starling's Android `native_build.sh` against the pinned
0.9.0 source. The adapter itself explicitly does not rewrite Rust APIs or
hook targets. Thus the ARM64 result is a successful pinned-source native build
using the adapted source bundle; it is not evidence that the launcher-side
SDK 0.2 HMOD import/build-service path has been exercised.

The `.so` is the full Starling Android native shared library built with the
mod included, not a standalone mod-only library. Do not copy it manually into
the phone or distribute it as an installable mod.

## Remaining verification gates

1. Add the repository branch through Starling 0.9.0's **Mods → Add from GitHub**
   flow and confirm that the launcher discovers the flat HMOD package.
2. Confirm the launcher build service accepts the exact HMOD and produces an
   instance-bound build.
3. Enable the mod and verify its entry, builder screen, touch behavior, point
   selection, and read-only world preview in a real world.
4. Only after those gates, implement the authoritative world-edit transaction,
   explicit confirmation, inventory behavior, and guarded undo/redo.

The current implementation remains read-only. No block placement or removal
is enabled, and CI success is not a substitute for device runtime testing.
