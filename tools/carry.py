#!/usr/bin/env python3
"""Carry YUNG's Extras' data out of the jar, and prove the carry byte for byte.

The jar is evidence, not a source: every file this port ships is either the
jar's own bytes (templates, loot, tags, licence) or a rewrite whose shipped form
is generated from the jar's file by `tools/gen_worldgen.py`. This tool owns the
"jar's own bytes" half.

    python3 tools/carry.py            # write data/ and tools/carry/ from the jar
    python3 tools/carry.py --check    # compare every carried file to the jar

What goes where, and why:

  data/yungsextras/structure/**            the 62 templates: inert data, read on
                                           demand by `/place template` (the
                                           content root moves them to
                                           `worldgen/structure/<ns>/`), and the
                                           templates the shipped arches name.
  data/yungsextras/loot_table/**           the three desert chest/archaeology
                                           tables; the templates' chests name two
                                           of them.
  data/yungsextras/tags/worldgen/biome/**  the mod's own biome tags; the shipped
                                           biome modifier names one of them.
  tools/carry/<rest>                       everything that cannot be shipped:
                                           the 62 configured features and 62
                                           placed features (they name feature
                                           and placement-modifier *types* the pin
                                           cannot register - see notes.md) and
                                           the six biome-modifier files that
                                           name them. Not packed (the packer
                                           packs `horizon-mod.toml`, `assets/`,
                                           `data/` and the root licence files),
                                           kept as the byte-identical record the
                                           next slice starts from.
  LICENSE                                  the jar's `LICENSE_YungsExtras`
                                           (LGPL-3.0), verbatim.

`--check` exits 0 only when every one of those files is byte-identical to the
jar's, and prints the count per destination.
"""
from __future__ import annotations

import argparse
import hashlib
import shutil
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
JAR = (ROOT.parents[1] / "ports/_shared/licence-cache"
       / "YungsExtras-1.21.1-NeoForge-5.1.1.jar")
PREFIX = "data/yungsextras/"

# Jar-relative prefixes that are inert enough to ship, with their destination.
SHIPPED = (
    "structure/",
    "loot_table/",
    "tags/worldgen/biome/has_structure/",
)
# Everything else under data/yungsextras/ is carried but not packed.
HELD = (
    "worldgen/configured_feature/",
    "worldgen/placed_feature/",
    "neoforge/biome_modifier/",
    "forge/biome_modifier/",
)


def destination(name: str) -> Path | None:
    """Where a jar entry lives in this port, or None when it is not carried."""
    if name == "LICENSE_YungsExtras":
        return ROOT / "LICENSE"
    if not name.startswith(PREFIX):
        return None
    rest = name[len(PREFIX):]
    if any(rest.startswith(p) for p in SHIPPED):
        return ROOT / "data/yungsextras" / rest
    if any(rest.startswith(p) for p in HELD):
        return ROOT / "tools/carry" / rest
    raise SystemExit(f"carry.py: no destination for {name}")


def carried(jar: zipfile.ZipFile) -> list[tuple[str, Path, bytes]]:
    out = []
    for name in sorted(jar.namelist()):
        if name.endswith("/"):
            continue
        dest = destination(name)
        if dest is not None:
            out.append((name, dest, jar.read(name)))
    return out


def sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    if not JAR.is_file():
        print(f"carry.py: jar missing: {JAR}")
        return 2
    with zipfile.ZipFile(JAR) as jar:
        files = carried(jar)
    if not args.check:
        for _, dest, raw in files:
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes(raw)
        print(f"carry.py: wrote {len(files)} files from {JAR.name} "
              f"({JAR.stat().st_size} B, sha256 {sha256(JAR.read_bytes())[:16]})")
        return 0

    bad = 0
    counts: dict[str, int] = {}
    for name, dest, raw in files:
        rel = str(dest.relative_to(ROOT))
        where = "carry" if rel.startswith("tools/carry/") else (
            "license" if rel == "LICENSE" else "data")
        counts[where] = counts.get(where, 0) + 1
        if not dest.is_file():
            print(f"  MISSING {rel}")
            bad += 1
        elif dest.read_bytes() != raw:
            print(f"  DIFFERS {rel}")
            bad += 1
    total = sum(counts.values())
    print(f"carry --check: {total - bad}/{total} byte-identical to the jar "
          f"({', '.join(f'{k}={v}' for k, v in sorted(counts.items()))})")
    if bad:
        print(f"carry --check: {bad} file(s) differ")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
