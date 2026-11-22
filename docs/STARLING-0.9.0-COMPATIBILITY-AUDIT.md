# Starling 0.9.0 compatibility audit — 2026-11-21

## Result: the native .hnmod route still exists; the previous audit was wrong

The official Starling v0.9.0 release and source tag are real. The release was published on 2026-11-20. Its source commit is:

ff303502a4f7982551a0e0d3622d9268efe9e829

This audit supersedes the earlier claim that v0.9.0 was not publicly available or that .hnmod had been replaced by an unrelated new format.

### Confirmed from the v0.9.0 source

- Starling still accepts native .hnmod files. docs/starling-native-mods.md describes importing them into horizon-native-mods/.
- crates/horizon-client/src/game/starling/native_builds.rs recognizes both .hnmod and .hmod as native-mod inputs, parses them through horizon_mod_package::index::native::Any, and serializes the selected bundle with for_build().
- The v0.9.0 importer/build flow is not a simple local Rust compile on Android. The phone sends selected native bundle source to a build service; the returned library is SHA-256 checked and tied to the exact APK install before it can be loaded.
- The native bundle's top-level metadata is horizon-native.json. The official Wrench example declares game, id, mixins, schema, and version. Its source files live under mixins/<id>/, including horizon-mixin.toml and Rust helpers.
- The native build script in this tag pins horizon-mixins to 5a64af938ec86b00d53d16a408aabc0038fb3e57.
- Starling includes native_mod_adapt.py. Its documented purpose is to adapt older Horizon hook targets and the previous Starling mod API to the reorganized Horizon source tree. It preserves a compatibility layer such as horizon_client::starling_mod_api; older hook-based bundles may still be buildable after adaptation.
- The official Wrench example is a real native bundle. It uses a cancellable HEAD injection into horizon_client::game::Game<'_>::use_item, plus the legacy Starling mod-list/settings hooks. Its world changes are performed on the authoritative game-side simulation path, not by a generic client-side block setter.

### What the build-service evidence does and does not prove

The public starling-client/mod-builds/versions.json was checked and currently contains only:

    {
      "0.8.0": "ac874422ed8f892505ff73d67ae486ecfee7a516"
    }

That is evidence that this public GitHub-runner build map does not yet list 0.9.0. It is not, by itself, proof that Starling's separately published HTTPS build gateway has no 0.9.0 configuration.

The v0.9.0 Starling source also contains .github/workflows/native-mods.yml, which can build only reviewed native-mod folders already present in Starling's own native-mods/ tree. Its current prebuilt list names Wrench. That workflow is not a general upload endpoint for this project.

Therefore, the remaining service question is whether the public HTTPS build gateway currently has the v0.9.0 source commit and a clean cache registered, or whether a separately operated build server is needed. Do not infer the answer from the old mod-builds map alone.

## Consequence for Effortless Building

The pure Rust geometry/transaction core remains reusable. The old 0.8.0 pins in this repository are historical and must not be silently reused for v0.9.0.

The next implementation stage must:
1. Port bundle metadata and archive layout to the exact v0.9.0 .hnmod importer contract.
2. Use the v0.9.0 global Mixins revision and verify the adapter's handling of our exact injection targets.
3. Inspect the exact v0.9.0 hook declarations and authoritative world-edit/inventory paths before writing game-facing code.
4. Establish a working 0.9.0 build route before claiming that a bundle can be compiled by the public service.
5. Keep preview read-only, require explicit confirmation, snapshot before mutation, and make undo/redo conflict-aware.

No installable artifact or successful native build is claimed by this audit.

## Sources

- Official release: https://github.com/starling-client/starling/releases/tag/v0.9.0
- Exact source tag: https://github.com/starling-client/starling/tree/v0.9.0
- Starling native build flow: https://github.com/starling-client/starling/blob/v0.9.0/docs/starling-native-mods.md
- Starling build script: https://github.com/starling-client/starling/blob/v0.9.0/packaging/android/native_build.sh
- Starling native bundle example: https://github.com/starling-client/starling/tree/v0.9.0/native-mods/wrench
- Public GitHub-runner version map: https://github.com/starling-client/mod-builds/blob/main/versions.json

## Maintainer

MR


## Exact legacy .hnmod importer contract (confirmed in v0.9.0)

The v0.9.0 source's `crates/horizon-mod-package/src/index/native/legacy.rs` is decisive: the phone still imports the Starling legacy source bundle, not the flat `.hmod` package layout.

The ZIP must contain:
- `horizon-native.json` at the archive root.
- Source files only under `mixins/<id>/`.
- `mixins/<id>/horizon-mixin.toml` and its referenced Rust helpers/modules.

The JSON manifest is strict (`deny_unknown_fields`) and requires:
- `schema: 1`
- valid lowercase `id`
- non-empty `version` and `game`
- exact 40-character lowercase-hex `horizon` commit
- exact 40-character lowercase-hex `mixins` commit
- optional positive `hooks` integer.

The importer checks that JSON `id`, `version`, and `horizon` exactly match `[mod].id`, `[mod].version`, and `[compatibility].horizon_revision` in the mixin TOML. It rejects unknown fields, unsafe paths, duplicate entries, unexpected files and packages above 64 MiB expanded source / 512 entries.

**Important distinction:** the in-tree Wrench source bundle's `horizon-native.json` is used as a build input inside Starling's own repository. Its shortened metadata is not a valid example of the phone-imported legacy `.hnmod` manifest. The strict legacy importer requires `horizon` as well as `mixins`.

## This project's pins updated

The repository now records:
- Starling/Horizon source: `ff303502a4f7982551a0e0d3622d9268efe9e829`
- global horizon-mixins: `5a64af938ec86b00d53d16a408aabc0038fb3e57`

The project's `VERSION.json`, native JSON manifest and `horizon-mixin.toml` compatibility pin have been updated to these exact revisions. The old screen/settings injections remain legacy hook targets and rely on Starling's adapter; their compatibility is not proven until the v0.9.0 build actually runs.

## Build-service availability

The v0.9.0 source publishes Starling's default gateway address in `packaging/android/build-server.txt`:

`https://starling-compile.starling-client.workers.dev`

The gateway's documentation says each release needs its own source commit in the server-side `versions.json`; GitHub-runner builds additionally need the version in `starling-client/mod-builds/versions.json` and a clean base workflow run. The public GitHub-runner map checked on 2026-11-21 still lists only 0.8.0. The server-side registry is not public in the source repository, and the gateway was not exercised from this audit. Thus v0.9.0 compilation remains unverified; the GitHub-runner path is not currently evidenced as configured for 0.9.0.

## Immediate next engineering gate

Before changing screen or world-edit code, run the source-only bundle through the v0.9.0 importer and compile pipeline. If the default gateway rejects the version, the release owner must publish the 0.9.0 source commit and clean cache to the build service (and the GitHub-runner map if that route is intended). Only after a clean build should the legacy UI hooks be debugged and the block-edit adapter implemented.

