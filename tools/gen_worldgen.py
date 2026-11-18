#!/usr/bin/env python3
"""Generate the shipped swamp-arch worldgen from the carried originals.

The jar's swamp arches are eleven `yungsextras:swamp_arch` configured features,
each naming one template, placed by a chain that ends in the mod's own
`yungsextras:rng_initializer` placement modifier. Neither type can be named at
this pin (see `notes.md`, "The placement question"): a content mod's features are
decoded from a fixed set of types, and a mod registers a new one only through an
SDK `FeatureType`, which the SDK does not have. So this tool rewrites *the shipped
form* of the family onto the one template-placement surface the pin does decode —
vanilla's `minecraft:template` configured feature
(`crates/horizon-world/src/worldgen/structure/features.rs`) — and leaves the
jar's files untouched in `tools/carry/`.

    python3 tools/gen_worldgen.py            # write data/yungsextras/worldgen/...
    python3 tools/gen_worldgen.py --check    # prove the shipped files are this

What each rewrite is, field by field, is in `notes.md` under
"## The swamp arch family, rewritten". The three edits:

1. `type`: `yungsextras:swamp_arch` -> `minecraft:template`. 26.3's configured
   feature is **flat** — `config` is unwrapped, as the campaign's own carry writes
   it (`crates/horizon-port-pack/src/upgrade/features.rs`, "A configured feature in
   26.3's form: `config` unwrapped"), and the decoder reads the config's keys at the
   root: `TemplateFeature::decode` reads `templates` and `processors`. The mod's
   `config.location` becomes a root-level `templates` entry `{"id": <location>,
   "rotations": ["none"]}` — one template, one rotation, as the mod places with
   `new StructurePlaceSettings()` (`Rotation.NONE`, `Mirror.NONE`). A file that keeps
   the 1.21.1 `config` wrapper loads and then never decodes: the game measures it as
   "11 placed features cannot run and will be skipped … invalid type: null".
2. root-level `processors`: the mod's `SwampFeatureProcessor`, expressed as data —
   `horizon:yung_pillar` for the gray-glass leg (the block plus a stone-brick
   column down to the ground) and two `horizon:pick`s for the weighted stone
   bricks and stairs (YUNG's `BlockStateRandomizer` draws one `nextFloat` against
   cumulative bounds; `horizon:pick` draws the same and takes `default`
   otherwise). The arch family carries no candle and no light-gray marker, so
   those two parts of the processor have nothing to act on here.
3. the placed feature's `yungsextras:rng_initializer` is replaced by
   `minecraft:offset` y -1: the heightmap placement puts the origin on the first
   free height, the mod centres its template on the last *solid* one.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CARRIED = ROOT / "tools/carry"
DATA = ROOT / "data/yungsextras"
FAMILY = "swamp/arches"
PREFIX = "swamp_arch_"

# SwampFeatureProcessor's stone-brick choice: default stone bricks, mossy 0.45,
# cracked 0.45 of YUNG's cumulative BlockStateRandomizer.draw.
STONE_BRICK_PICK = {
    "entries": [
        {"blockState": {"id": "minecraft:mossy_stone_bricks"}, "probability": 0.45},
        {"blockState": {"id": "minecraft:cracked_stone_bricks"}, "probability": 0.45},
    ],
    "defaultBlockState": {"id": "minecraft:stone_bricks"},
}
# ... and its stairs choice: 0.6 mossy stairs, else plain, properties kept.
STAIR_PICK = {
    "entries": [
        {"blockState": {"id": "minecraft:mossy_stone_brick_stairs"}, "probability": 0.6},
    ],
    "defaultBlockState": {"id": "minecraft:stone_brick_stairs"},
}
PROCESSORS = [
    {
        # The arch's leg: one gray marker block, replaced and run down to ground.
        "processor_type": "horizon:yung_pillar",
        "target_block": {"id": "minecraft:gray_stained_glass"},
        "target_block_output": STONE_BRICK_PICK,
        "pillar_states": STONE_BRICK_PICK,
    },
    {
        "processor_type": "horizon:pick",
        "block": "minecraft:stone_bricks",
        "picks": [
            {"below": 0.45, "state": {"id": "minecraft:mossy_stone_bricks"}},
            {"below": 0.90, "state": {"id": "minecraft:cracked_stone_bricks"}},
        ],
        "default": {"state": {"id": "minecraft:stone_bricks"}},
    },
    {
        "processor_type": "horizon:pick",
        "block": "minecraft:stone_brick_stairs",
        "picks": [{"below": 0.6, "state": {"id": "minecraft:mossy_stone_brick_stairs"}}],
        "default": {"state": {"id": "minecraft:stone_brick_stairs"}},
        "keep_properties": True,
    },
]


def arches() -> list[str]:
    """The family's template stems in numeric order: swamp_arch_0 .. _10."""
    stems = sorted(
        (p.stem for p in (CARRIED / "worldgen/configured_feature" / FAMILY).glob("*.json")),
        key=lambda s: int(s.rsplit("_", 1)[1]),
    )
    if not stems or not all(s.startswith(PREFIX) for s in stems):
        raise SystemExit(f"gen_worldgen.py: no {PREFIX}* configured features carried")
    return stems


def feature(stem: str) -> dict:
    original = json.loads(
        (CARRIED / "worldgen/configured_feature" / FAMILY / f"{stem}.json").read_text())
    if original["type"] != "yungsextras:swamp_arch":
        raise SystemExit(f"gen_worldgen.py: {stem} is {original['type']}, not an arch")
    return {
        "type": "minecraft:template",
        # 26.3 is flat: the config's keys sit at the root (no `config` wrapper).
        # A weighted list entry is `{"data": <payload>, "weight": n}`.
        "templates": [
            {
                "data": {"id": original["config"]["location"], "rotations": ["none"]},
                "weight": 1,
            },
        ],
        "processors": PROCESSORS,
    }


def placed(stem: str) -> dict:
    original = json.loads(
        (CARRIED / "worldgen/placed_feature" / FAMILY / f"{stem}.json").read_text())
    placement = []
    removed = False
    for step in original["placement"]:
        if step["type"] == "yungsextras:rng_initializer":
            removed = True
            continue
        placement.append(step)
    if not removed:
        raise SystemExit(f"gen_worldgen.py: {stem} has no rng_initializer step")
    # The heightmap modifier leaves the origin on the first free height; the mod
    # centres on the topmost solid block, one below.
    placement.append({"type": "minecraft:offset", "x": 0, "y": -1, "z": 0})
    return {"feature": f"yungsextras:{FAMILY}/{stem}", "placement": placement}


def modifier(stems: list[str]) -> dict:
    return {
        "type": "neoforge:add_features",
        "biomes": "#yungsextras:has_structure/swamp_structures",
        "features": [f"yungsextras:{FAMILY}/{stem}" for stem in stems],
        "step": "surface_structures",
    }


def outputs() -> dict[Path, str]:
    stems = arches()
    out: dict[Path, str] = {}
    for stem in stems:
        out[DATA / "worldgen/feature" / FAMILY / f"{stem}.json"] = feature(stem)
        out[DATA / "worldgen/placed_feature" / FAMILY / f"{stem}.json"] = placed(stem)
    out[DATA / "neoforge/biome_modifier/swamp_arches.json"] = modifier(stems)
    return out


def render(value: dict) -> str:
    return json.dumps(value, indent=2) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    files = {path: render(value) for path, value in outputs().items()}
    bad = 0
    for path, text in files.items():
        rel = path.relative_to(ROOT)
        if args.check:
            if not path.is_file():
                print(f"  MISSING {rel}")
                bad += 1
            elif path.read_text() != text:
                print(f"  DIFFERS {rel}")
                bad += 1
            continue
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
    stems = arches()
    if args.check:
        print(f"gen_worldgen --check: {len(files) - bad}/{len(files)} generated files "
              f"match ({len(stems)} arches)")
        return 1 if bad else 0
    print(f"gen_worldgen.py: wrote {len(files)} files for {len(stems)} arches")
    return 0


if __name__ == "__main__":
    sys.exit(main())
