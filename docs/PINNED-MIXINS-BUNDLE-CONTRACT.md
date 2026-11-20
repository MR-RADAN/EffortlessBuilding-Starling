# Pinned Mixins bundle contract

This note records the manifest contract verified directly from
`horizon-mixins` commit `ddd9700b21997c95d95ac6ee5ecafe3830cf622a`.
It deliberately does not copy a newer native-mod template into the Starling
0.8.0 target.

## Bundle discovery and manifest

At this Mixins revision, the loader accepts either:
- a TOML file directly under the mods root; or
- a one-level bundle directory containing `horizon-mixin.toml`.

The manifest is TOML and rejects unknown fields. Its top-level sections include
`[mod]`, optional `[compatibility]`, `[[injection]]`, `[[module]]`,
`[[bridge]]`, `[[state]]`, `[[crate]]` and `[[post_build]]`.

A support module declaration has these required fields:
- `id`: unique module identifier;
- `file`: destination Rust source file in the target Horizon crate;
- `name`: Rust module identifier to add;
- `source`: path to the bundle's self-contained Rust source.

The payload is parsed as Rust and must not contain file-level inner attributes,
a shebang, external `mod foo;` declarations, `#[path]`, or
`include! / include_str! / include_bytes!`. This means the geometry core can
be injected as one self-contained module, but its source must be kept inline
and must not depend on separate source files.

The module is not automatically given extra Cargo dependencies. Its code is
compiled in the destination Horizon crate, so only imports and dependencies
already available there can be used. The standalone crate used for local tests
is not itself automatically injected into Horizon.

## Injection and helper constraints

An injection declares a unique `id`, `kind`, `helper`, and either a raw
`target` or a named `hook` with `hook_version`. Typed bindings are explicit.
The helper is compiled locally inside the target function, so visibility,
borrowing, and exact parameter types are checked by Rust during the staged
workspace check—not merely by parsing the TOML.

The pinned manifest parser supports the `compatibility.horizon_revision`
field. The project must pin the Horizon commit and must not infer compatibility
from a newer template's current defaults.

## What this enables—and what it does not

Verified:
- a same-crate support module can host the clean-room geometry/planning core;
- small typed helpers can bridge the native hook into that module;
- bundle crate declarations exist in this Mixins revision, but they require
  their own Cargo package and are not a substitute for checking the injected
  target crate's dependencies.

Not yet verified for this project:
- the exact `.hnmod` metadata and packaging contract for Starling 0.8.0;
- whether the pinned Starling build service accepts a bundle using this exact
  Mixins revision today;
- the exact module destination paths and helper bindings for every required
  hook in the combined client/server feature set;
- Android touch-to-mouse delivery on the POCO F5.

## Release rule

Do not package a guessed manifest as a usable mod. First resolve the exact
native hook catalog entries and module destinations at the pinned Horizon
revision, then run the pinned Mixins planner/check against a pristine export,
then run Starling's own package/build validation. A package that merely has
the right filename or parses as TOML is not a validated `.hnmod`.
