# Native mixin module layout findings

Inspected the public `horizon-rust/horizon-native-mod-template` source tree and
its example files to understand the native Mixins source layout. The template's
Horizon revision is newer than this project's Starling 0.8.0 pin; its revision
must not be copied into this project.

## Confirmed template conventions

A bundle manifest uses:
- `[mod]` with `id`, `version`, `priority`.
- `[compatibility]` with `horizon_revision`.
- One or more `[[module]]` entries mapping a Rust source file into a target
  crate file and module name.
- One or more `[[injection]]` entries with an id, injection kind, target,
  helper file and explicit bindings.

An injection helper is a small function that delegates into the injected
same-crate module. The module source is compiled as a standalone Cargo library
and is also injected into the target crate; this is the template's documented
layout pattern. The example's `runtime.rs` is both the Cargo `[lib].path`
and the module source.

The template's packager builds a source-only ZIP with
`horizon-native.json` and `mixins/<id>/`; the archive is not an APK or a
precompiled shared library. Its package script hardcodes game version 26.3
for that template. This project independently verified its own importer
schema and game version pin; do not blindly copy the script or its revision.

## Screen hook requirements refined from the pinned Starling source

At the project's pinned Horizon revision, native screen support is documented
as API 111. The input-related additions relevant to an interactive screen are:
- API 78: screen render-post.
- API 80: mouse button pressed/released pre-hooks.
- API 93: screen scroll pre-hook.
- API 104: key pressed and character typed pre-hooks.
- API 111: native mod screen open/close/render lifecycle.
- API 123: screen mouse-drag pre-hook.

The pinned source also documents API 114 raw mouse button hooks and API 121
complete window keyboard input, but these are not a substitute for proving
Android touch-to-mouse delivery into the screen event path.

## Implementation consequence

Do not add guessed `[[module]]` or `[[injection]]` entries yet. Each target
must be checked against the exact pinned source signature, and each binding's
type and value must match the Mixins parser. The native template's newer pin is
reference material for structure only.

Android touch remains a release blocker: source-level presence of mouse/screen
hooks does not establish that Starling 0.8.0 converts touch gestures to those
events on the user's Android build. The first interactive implementation must
be tested on device before placement can be enabled.
