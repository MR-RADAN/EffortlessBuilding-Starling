#!/usr/bin/env python3
"""Build a deterministic Starling native .hnmod package (Python standard library only)."""
from __future__ import annotations
import json
import pathlib
import sys
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
VERSION = "0.1.0"
MOD_ID = "effortless_building"
SOURCE = ROOT / "native" / f"effortless-building-v{VERSION}"
OUTPUT = ROOT / "dist" / f"{MOD_ID}-{VERSION}.hnmod"

SOURCE_FILES = (
    "horizon-mixin.toml",
    "runtime.rs",
    "list.rs",
    "settings.rs",
    "click.rs",
)
PACKAGE_PATHS = {
    "horizon-native.json": "horizon-native.json",
    **{f"mixins/{MOD_ID}/{name}": name for name in SOURCE_FILES},
}

def main() -> int:
    missing = [name for name in ("horizon-native.json", *SOURCE_FILES) if not (SOURCE / name).is_file()]
    if missing:
        print("Missing package inputs:", *missing, sep="\n  ", file=sys.stderr)
        return 2

    metadata = json.loads((SOURCE / "horizon-native.json").read_text(encoding="utf-8"))
    if metadata.get("id") != MOD_ID or metadata.get("version") != VERSION:
        raise SystemExit("horizon-native.json id/version does not match package constants")

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(OUTPUT, "w") as archive:
        for package_name, source_name in PACKAGE_PATHS.items():
            info = zipfile.ZipInfo(package_name, date_time=(2020, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, (SOURCE / source_name).read_bytes(), compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)

    with zipfile.ZipFile(OUTPUT, "r") as archive:
        if set(archive.namelist()) != set(PACKAGE_PATHS) or archive.testzip() is not None:
            raise SystemExit("Package ZIP validation failed")
        packaged_metadata = json.loads(archive.read("horizon-native.json"))
        if packaged_metadata != metadata:
            raise SystemExit("Packaged metadata differs from source metadata")

    print(f"Created {OUTPUT}")
    print(f"Validated {len(PACKAGE_PATHS)} entries; {OUTPUT.stat().st_size} bytes")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
