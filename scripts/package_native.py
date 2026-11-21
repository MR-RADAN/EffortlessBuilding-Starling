#!/usr/bin/env python3
"""Build a deterministic Starling native .hnmod package (Python standard library only)."""
from __future__ import annotations

import json
import pathlib
import re
import sys
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
VERSION = "0.1.0"
MOD_ID = "effortless_building"
SOURCE = ROOT / "native" / f"effortless-building-v{VERSION}"
OUTPUT = ROOT / "dist" / f"{MOD_ID}-{VERSION}.hnmod"


def main() -> int:
    metadata_path = SOURCE / "horizon-native.json"
    manifest_path = SOURCE / "horizon-mixin.toml"
    if not metadata_path.is_file() or not manifest_path.is_file():
        raise SystemExit("Missing native metadata or mixin manifest")

    metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
    if metadata.get("id") != MOD_ID or metadata.get("version") != VERSION:
        raise SystemExit("horizon-native.json id/version does not match package constants")
    if metadata.get("hooks") != 111:
        raise SystemExit("UI bundle requires native hooks API 111 (custom mod screen)")

    manifest = manifest_path.read_text(encoding="utf-8")
    # Every module source and injection helper named by the manifest must ship.
    referenced = set(re.findall(r'(?m)^\s*(?:source|helper)\s*=\s*"([^"]+)"', manifest))
    source_files = sorted(
        p.name for p in SOURCE.iterdir()
        if p.is_file() and (p.suffix == ".rs" or p.name == "horizon-mixin.toml")
    )
    missing = sorted(name for name in referenced if not (SOURCE / name).is_file())
    if missing:
        raise SystemExit("Manifest references missing source files: " + ", ".join(missing))

    package_paths = {
        "horizon-native.json": metadata_path,
        **{f"mixins/{MOD_ID}/{name}": SOURCE / name for name in source_files},
    }
    expected = {"horizon-native.json", *(f"mixins/{MOD_ID}/{name}" for name in source_files)}
    packaged_sources = {pathlib.PurePosixPath(key).name for key in package_paths if key.endswith(".rs") or key.endswith(".toml")}
    if not referenced.issubset(packaged_sources):
        raise SystemExit("Package would omit one or more manifest-referenced sources")

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(OUTPUT, "w") as archive:
        for package_name, source_path in sorted(package_paths.items()):
            info = zipfile.ZipInfo(package_name, date_time=(2020, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(
                info,
                source_path.read_bytes(),
                compress_type=zipfile.ZIP_DEFLATED,
                compresslevel=9,
            )

    with zipfile.ZipFile(OUTPUT, "r") as archive:
        if set(archive.namelist()) != expected or archive.testzip() is not None:
            raise SystemExit("Package ZIP validation failed")
        packaged_metadata = json.loads(archive.read("horizon-native.json"))
        if packaged_metadata != metadata:
            raise SystemExit("Packaged metadata differs from source metadata")
        packaged_names = {pathlib.PurePosixPath(name).name for name in archive.namelist()}
        if not referenced.issubset(packaged_names):
            raise SystemExit("Package validation failed: a manifest source is missing")

    print(f"Created {OUTPUT}")
    print(f"Validated {len(package_paths)} entries, including all {len(referenced)} manifest references; {OUTPUT.stat().st_size} bytes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
