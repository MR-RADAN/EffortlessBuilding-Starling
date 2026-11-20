#!/usr/bin/env python3
"""Build a deterministic Starling native .hnmod package (stdlib only)."""
from __future__ import annotations
import json
import pathlib
import sys
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
VERSION = "0.1.0"
MOD_ID = "effortless_building"
SOURCE = ROOT / "native" / f"{MOD_ID}-v{VERSION}"
OUTPUT = ROOT / "dist" / f"{MOD_ID}-{VERSION}.hnmod"

REQUIRED = (
    "horizon-native.json",
    "mixins/effortless_building/horizon-mixin.toml",
    "mixins/effortless_building/runtime.rs",
    "mixins/effortless_building/list.rs",
    "mixins/effortless_building/settings.rs",
    "mixins/effortless_building/click.rs",
)

def main() -> int:
    missing = [p for p in REQUIRED if not (SOURCE / p).is_file()]
    if missing:
        print("Missing package inputs:", *missing, sep="\n  ", file=sys.stderr)
        return 2

    metadata = json.loads((SOURCE / "horizon-native.json").read_text(encoding="utf-8"))
    if metadata.get("id") != MOD_ID or metadata.get("version") != VERSION:
        raise SystemExit("horizon-native.json id/version does not match package constants")

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(OUTPUT, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as zf:
        for relative in REQUIRED:
            info = zipfile.ZipInfo(relative, date_time=(2020, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            zf.writestr(info, (SOURCE / relative).read_bytes())

    with zipfile.ZipFile(OUTPUT, "r") as zf:
        names = set(zf.namelist())
        if names != set(REQUIRED) or zf.testzip() is not None:
            raise SystemExit("Package validation failed")
    print(f"Created {OUTPUT}")
    print(f"Entries: {len(REQUIRED)}")
    print(f"Size: {OUTPUT.stat().st_size} bytes")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
