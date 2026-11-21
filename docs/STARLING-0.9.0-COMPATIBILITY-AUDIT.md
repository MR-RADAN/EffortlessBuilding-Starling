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
