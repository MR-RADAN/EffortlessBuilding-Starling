# Native Android build validation

The `.hnmod` package workflow checks the source archive. This separate workflow
performs the real Starling 0.8.0 Android ARM64 native build using the exact
upstream release script:

`starling-source/packaging/android/native_build.sh`

It checks out Starling tag `v0.8.0`, verifies Horizon revision
`ac874422ed8f892505ff73d67ae486ecfee7a516`, installs the same pinned Android
SDK/NDK and cargo-ndk toolchain as Starling, applies the bundle with the pinned
`horizon-mixins` revision, and compiles `libhorizon_android.so`.

The workflow intentionally builds only the library (`NATIVE_LIBRARY_ONLY=1`)
rather than an APK. Its artifact contains:
- `libhorizon_android.so`: Starling's ARM64 game library with Effortless
  Building compiled in.
- `build.json`: the source commit, profile, mod id/version, Mixins revision,
  ABI and library checksum.

This is a compiler/integration check, not an in-game test. A successful build
proves that the bundle applies and links against the pinned source; it does not
prove that the screen renders correctly or that taps work on the POCO F5.
The workflow artifact is short-lived (7 days) and is not automatically
published to Starling's official native-build feed.
