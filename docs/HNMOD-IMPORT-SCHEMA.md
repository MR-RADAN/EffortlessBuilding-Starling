# Native bundle schema audit (Starling 0.8.0 target)

## Exact `.hnmod` manifest fields

The importer reads `horizon-native.json` as a strict Serde struct with
`deny_unknown_fields`. Required fields:

- `schema`: integer, exactly `1`
- `id`: valid lowercase mod id
- `version`: non-empty string
- `game`: non-empty game-version string
- `horizon`: exact 40-character lowercase hexadecimal Horizon commit
- `mixins`: exact 40-character lowercase hexadecimal horizon-mixins commit

Optional: `hooks`: positive integer native-hook API version; zero is rejected.

The manifest's `id`, `version`, and `horizon` must exactly match
`mixins/<id>/horizon-mixin.toml`: `[mod].id`, `[mod].version`, and
`[compatibility].horizon_revision`.

## Archive layout and validation

The archive is a ZIP named `*.hnmod`. Required native entries:

```
horizon-native.json
mixins/effortless_building_mr/horizon-mixin.toml
mixins/effortless_building_mr/<Rust helper files>.rs
```

Allowed native entries are Rust/TOML files, data files in permitted `data/`
locations, and limited crate text files (README/LICENSE/COPYING/NOTICE and
`.gitignore`). Paths must be relative, unique, and free of traversal,
backslashes, colons, and symlink-like entries. The importer caps the archive
at 512 entries and 64 MiB expanded source. The build-service preflight
independently checks the source-only boundary.

## Build service behavior

The build service accepts one or more `.hnmod` ZIP archives in a
length-prefixed request. It extracts the manifest and source tree, rejects
duplicate mod IDs, then invokes the pinned build pipeline. It does not accept
a plain Cargo project, APK, JAR, or a ZIP containing arbitrary binaries.

## Project-specific consequence

The earlier example manifest containing only `game/id/mixins/schema/version`
was incomplete for the current importer: exact `horizon` is also required.
The target Horizon revision is recorded in `VERSION.json`; before emitting
a bundle, that value must be duplicated identically in the native manifest
and `[compatibility].horizon_revision`.

This audit establishes packaging structure, not a successful compile. It
does not invent hook names or claim Android touch input has been verified.