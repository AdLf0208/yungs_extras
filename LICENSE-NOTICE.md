# Licence notice — YUNG's Extras for Horizon

## Upstream

- **Mod:** YUNG's Extras (`yungsextras`), version **1.21.1-NeoForge-5.1.1**
- **Authors (from the jar's `META-INF/neoforge.mods.toml`):** YUNGNICKYOUNG, Acarii
- **Source:** <https://github.com/yungnickyoung/YUNGs-Extras>, branch `1.21.1`
  (read at commit `ebc3bc669e8e23ddde99fa7c663ffc92078ffbd8`, `gradle.properties`
  `version=5.1.1`), as an analysis aid only
- **Jar read:** `YungsExtras-1.21.1-NeoForge-5.1.1.jar`, **198,183 B**, SHA-256
  `0cd26474e514f5dc` … (the campaign's
  `ports/_shared/licence-cache/YungsExtras-1.21.1-NeoForge-5.1.1.jar`)
- **Licence:** **LGPL-3.0-only**. The jar's `neoforge.mods.toml` declares
  `license="LGPLv3"`, and the jar ships the GNU Lesser General Public License
  version 3 as `LICENSE_YungsExtras` (7,817 B). That file is this package's
  `LICENSE`, byte for byte. Rank 1 (copyleft): redistributable and modifiable
  under the same licence with the complete corresponding source.

## What each file in this package is

| Files | Origin |
|---|---|
| `data/yungsextras/structure/**` (62 `.nbt`) | the jar's own templates, byte-identical |
| `data/yungsextras/loot_table/desert/*.json` (3) | the jar's own loot tables, byte-identical |
| `data/yungsextras/tags/worldgen/biome/has_structure/*.json` (3) | the jar's own biome tags, byte-identical |
| `data/yungsextras/worldgen/feature/swamp/arches/*.json` (11) | **rewritten** by this port's `tools/gen_worldgen.py` from the jar's own configured features: the same template, placed as vanilla's `minecraft:template` feature because a content mod cannot register `yungsextras:swamp_arch` |
| `data/yungsextras/worldgen/placed_feature/swamp/arches/*.json` (11) | **rewritten** the same way from the jar's own placed features: the mod's chain minus `yungsextras:rng_initializer` plus the one-block ground offset |
| `data/yungsextras/neoforge/biome_modifier/swamp_arches.json` | **rewritten** from the jar's `swamp_structures_addition.json`, listing only the 11 pieces this package ships |
| `src/`, `Cargo.toml`, `horizon-mod.toml`, `README.md`, `tools/` | this port's own work (a clean-room Rust rewrite; no Java was translated) |
| `LICENSE` | the jar's `LICENSE_YungsExtras`, verbatim |

The jar's unshipped worldgen (the other 51 configured and 62 placed features,
and the six biome-modifier files) is kept byte-identical in the port tree under
`tools/carry/`, and `python3 tools/carry.py --check` proves every carried file
against the jar.

This is an alteration of the original work, as the LGPL requires it to be
identified: the feature types are not registered, the swamp arch family is
placed by vanilla's template feature instead, and the worldgen JSON was
rewritten accordingly.
