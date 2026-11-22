#!/usr/bin/env python3
"""Convert one Starling HMOD 0.2 native source folder to a legacy horizon-mixins bundle.

This is a structural adapter only. It does not rewrite Rust APIs, hook targets,
bindings, or module bodies. Unsupported manifest fields fail closed.
"""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import shutil
import sys
import tomllib

EXPECTED_MIXINS = "5a64af938ec86b00d53d16a408aabc0038fb3e57"
IDENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_-]*$")

def toml_value(value: object) -> str:
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, list):
        return "[" + ", ".join(toml_value(item) for item in value) + "]"
    if isinstance(value, dict):
        return "{ " + ", ".join(
            f"{key} = {toml_value(item)}" for key, item in value.items()
        ) + " }"
    raise ValueError(f"unsupported TOML value: {type(value).__name__}")

def source_file(root: pathlib.Path, relative: str) -> pathlib.Path:
    rel = pathlib.PurePosixPath(relative)
    if rel.is_absolute() or not rel.parts or any(part in ("", ".", "..") for part in rel.parts):
        raise ValueError(f"unsafe source path: {relative!r}")
    path = root.joinpath(*rel.parts)
    current = root
    for part in rel.parts:
        current = current / part
        if current.is_symlink():
            raise ValueError(f"symlinked source path is not allowed: {relative}")
    if not path.is_file():
        raise ValueError(f"missing source file: {relative}")
    if path.suffix != ".rs":
        raise ValueError(f"native source must be Rust: {relative}")
    return path

def render_manifest(mod: dict, native: dict) -> tuple[str, list[str]]:
    allowed_native = {"game", "horizon", "mixins", "module", "mixin"}
    unknown_native = sorted(set(native) - allowed_native)
    if unknown_native:
        raise ValueError("unsupported [native] fields: " + ", ".join(unknown_native))
    if native.get("mixins") != EXPECTED_MIXINS:
        raise ValueError(
            "native.mixins does not match the Android builder's pinned horizon-mixins revision"
        )
    if not all(isinstance(native.get(key), str) and native[key] for key in ("game", "horizon")):
        raise ValueError("[native] must pin game and horizon separately")

    mod_id, version = mod.get("id"), mod.get("version")
    if not isinstance(mod_id, str) or not IDENT.fullmatch(mod_id):
        raise ValueError("invalid mod id")
    if not isinstance(version, str) or not version:
        raise ValueError("invalid mod version")

    lines = ["[mod]", f"id = {toml_value(mod_id)}", f"version = {toml_value(version)}", "priority = 100", ""]
    referenced: list[str] = []
    seen_ids: set[str] = set()

    for row in native.get("module", []):
        allowed = {"id", "file", "name", "source", "visibility"}
        unknown = sorted(set(row) - allowed)
        if unknown:
            raise ValueError(f"unsupported native.module fields: {', '.join(unknown)}")
        ident = row.get("id")
        if not isinstance(ident, str) or not IDENT.fullmatch(ident) or ident in seen_ids:
            raise ValueError(f"invalid or duplicate module id: {ident!r}")
        seen_ids.add(ident)
        for key in ("file", "name", "source"):
            if not isinstance(row.get(key), str) or not row[key]:
                raise ValueError(f"module {ident} is missing {key}")
        if "visibility" in row and row["visibility"] not in ("pub", "pub(crate)"):
            raise ValueError(f"unsupported visibility on module {ident}")
        lines.append("[[module]]")
        for key in ("id", "file", "name", "source", "visibility"):
            if key in row:
                lines.append(f"{key} = {toml_value(row[key])}")
        lines.append("")
        referenced.append(row["source"])

    for row in native.get("mixin", []):
        allowed = {"id", "kind", "target", "helper", "bindings", "cancellable"}
        unknown = sorted(set(row) - allowed)
        if unknown:
            raise ValueError(f"unsupported native.mixin fields: {', '.join(unknown)}")
        ident = row.get("id")
        if not isinstance(ident, str) or not IDENT.fullmatch(ident) or ident in seen_ids:
            raise ValueError(f"invalid or duplicate mixin id: {ident!r}")
        seen_ids.add(ident)
        for key in ("kind", "target", "helper"):
            if not isinstance(row.get(key), str) or not row[key]:
                raise ValueError(f"mixin {ident} is missing {key}")
        if "bindings" in row and not isinstance(row["bindings"], list):
            raise ValueError(f"mixin {ident}: bindings must be an array")
        lines.append("[[injection]]")
        for key in ("id", "kind", "target", "helper", "bindings", "cancellable"):
            if key in row:
                lines.append(f"{key} = {toml_value(row[key])}")
        lines.append("")
        referenced.append(row["helper"])

    if not native.get("module") or not native.get("mixin"):
        raise ValueError("expected at least one native.module and one native.mixin")
    if len(native["mixin"]) != 9 or len(native["module"]) != 3:
        raise ValueError(
            f"unexpected Starling port shape: {len(native['mixin'])} mixins, "
            f"{len(native['module'])} modules (expected 9 and 3)"
        )
    return "\n".join(lines).rstrip() + "\n", referenced

def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=pathlib.Path, help="HMOD 0.2 source folder")
    parser.add_argument("output", type=pathlib.Path, help="horizon-mixins mods directory")
    args = parser.parse_args()

    root = args.source.resolve()
    manifest_path = root / "horizon-mod.toml"
    if not manifest_path.is_file():
        raise SystemExit(f"missing {manifest_path}")
    manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("api") != "0.2":
        raise SystemExit("expected HMOD API 0.2")
    if "mod" in manifest or "native" not in manifest:
        raise SystemExit("expected a native HMOD 0.2 manifest")

    try:
        text, referenced = render_manifest(manifest, manifest["native"])
        checked = [(name, source_file(root, name)) for name in referenced]
        if len(set(referenced)) != len(referenced):
            raise ValueError("a Rust source is referenced more than once")
    except (KeyError, TypeError, ValueError) as error:
        raise SystemExit(f"HMOD adapter: {error}") from error

    bundle = args.output / manifest["id"]
    if bundle.exists():
        raise SystemExit(f"refusing to overwrite existing bundle: {bundle}")
    bundle.mkdir(parents=True, exist_ok=False)
    try:
        for relative, path in checked:
            destination = bundle.joinpath(*pathlib.PurePosixPath(relative).parts)
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, destination)
        (bundle / "horizon-mixin.toml").write_text(text, encoding="utf-8")
    except Exception:
        shutil.rmtree(bundle, ignore_errors=True)
        raise

    print(f"Converted {len(manifest['native']['mixin'])} mixins and "
          f"{len(manifest['native']['module'])} support modules.")
    print(f"Bundle: {bundle}")
    print("No Rust API adaptation or compilation was performed.")
    return 0

if __name__ == "__main__":
    sys.exit(main())
