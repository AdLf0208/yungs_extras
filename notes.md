# yungsextras — port notes

Ported from **YUNG's Extras `1.21.1-NeoForge-5.1.1`** (`yungnickyoung/YUNGs-Extras`,
branch `1.21.1`, commit `ebc3bc669e8e23ddde99fa7c663ffc92078ffbd8`, `gradle.properties`
`version=5.1.1`) for Minecraft **26.3** on Horizon's mod API **0.2** — SDK
**`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (`package_format` 6, `content_api`
"0.2"), horizon-mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**.

**Class + tier:** `content` / `data` (`ports/ledger.csv` row `yungsextras`, mirroring
`horizon/docs/modpacks/atm10-mods.csv`). The tier's approach per `modpacks.md` is
**carry-over** ("Worldgen JSON: features, structures, template pools, processors,
structure NBT, biome additions … carried into the `worldgen/` bundle"), with the one
piece of code the tier allows: the family whose placement a *vanilla feature type* can
express, expressed as data.

**Licence: LGPL-3.0-only → rank 1 (copyleft, publishable under the same licence with
the complete corresponding source).** Read from the jar, twice:
`META-INF/neoforge.mods.toml` says `license="LGPLv3"`, and the jar ships the GNU Lesser
General Public License version 3 as `LICENSE_YungsExtras` (7,817 B) — reproduced as
`LICENSE` byte for byte. Rank 1 is **not** a publishing bar: the entry would publish
under LGPL-3.0-only with this directory as its source. **Local-only until the
maintainer approves publishing** (ARCHITECTURE.md §8). No separate asset licence:
the repository has one `LICENSE` (the LGPL-3.0 text, 7,652 B) and no
`ASSETS_LICENSE`-style file, and the jar ships only `LICENSE_YungsExtras` — so the
templates and loot this package carries are the mod's own LGPL data.

Provenance of the jar (evidence): `ports/_shared/licence-cache/YungsExtras-1.21.1-NeoForge-5.1.1.jar`,
**198,183 B**, SHA-256 `0cd26474e514f5dc3114aaf5ec7e049bcd285f0c5db191bb45223193f35df70d`.

## The placement question (the headline, answered first)

> **Does the pin expose a way for a mod to register its own feature type — or to place
> a template structure from a feature?**

**A. A mod-defined feature type: NO.** There is no feature-type registration anywhere in
the SDK. Searched, all of them read:

| File | What it exposes |
|---|---|
| `api-source/crates/horizon-mod-vocab/src/world/mod.rs` | the world module list: no `feature` module at all; `structure_type`, `structure_template`, `biome_modifier`, `datapack_registry*` only |
| `.../world/structure_type.rs:58` | `pub trait StructureType` — a **structure** type (vanilla `StructureType`): `find_generation_point`, `load_piece`, `after_place` |
| `.../parts/structures.rs:19` | `Registrar::register_structure_type` — the only worldgen registration the API has; `StructuresRegistrar` has `add_structure_type` and nothing else |
| `.../world/structure_template.rs:36` | `StructureTemplateSnapshot` — plain data (size + blocks), **no loader** |
| `.../host/world_edit.rs:95` | `ServerHost::structure_template(id, at, rotation, mirror) -> Option<StructureTemplateSnapshot>` — reads a template on the **server**; a worldgen worker context has no host |
| `.../world/biome_modifier.rs:105` | `AddFeatures { features: Vec<String>, step }` — names placed-feature **ids**; the modification is data (`neoforge:add_features`-shaped), not a registration |
| `surface.json` | 749 hooks / 215 traits; no `FeatureType`, no `PlacementModifierType`, no `register_feature` |

`grep -rn "FeatureType\|register_feature\|configured_feature\|placed_feature\|ConfiguredFeature\|PlacedFeature"`
over the whole of `api-source/crates/**/*.rs` returns **zero** matches, and
`grep -rn "PlacementModifier\|placement_modifier"` likewise. The pin's worldgen decodes
a content mod's features from a **fixed** set of types —
`crates/horizon-world/src/worldgen/feature_graph.rs:315`
(`FeatureDefinition::decode`: `minecraft:no_op`, `minecraft:template`, `minecraft:fossil`,
`minecraft:freeze_top_layer`, …) — so a mod's own type (`yungsextras:swamp_arch`,
`yungsextras:desert_well`, …) can be named by a file but never decoded, and
`yungsextras:rng_initializer` can never be one of the placement modifiers
(`crates/horizon-world/src/worldgen/placement.rs:136-263` lists every one it reads).

**B. Placing a template from a feature: YES** — but only through **vanilla's**
`minecraft:template` configured feature, which the pin decodes:

- `crates/horizon-world/src/worldgen/feature_graph.rs:317`
  `"minecraft:template" => super::structure::features::TemplateFeature::decode(...)`
- `crates/horizon-world/src/worldgen/structure/features.rs:36-108`
  `TemplateFeature`: a **weighted list** of `{"id": "<ns:path>", "rotations": [...]}`
  plus an optional `processors` holder (a registry name, `{"processors": [...]}`, or a
  bare list), samples one template and one rotation, and **centres** it on the origin
  (`origin - rotated(size/2)` on X then Z, exactly the mod's
  `createTemplateFromCenter`); it places with flags 3 where the mod uses 2.
- Templates are reachable by id without any structure or pool: a content mod's
  `data/<ns>/structure/**.nbt` is overlaid as `worldgen/structure/<ns>/<path>.nbt`
  (`crates/horizon-client/src/mod_content.rs:2533-2543`), and `/place template <ns>:<path>`
  reads it (`crates/horizon-client/src/game/commands/place.rs:38-60`).

So the honest answer is **"no feature type, yes vanilla template placement"**, and the
port takes the matching branch: **one family, implemented** — the swamp arch — through
`minecraft:template` + a data processor list, with the jar's own files carried
byte-identically beside it; the other ten families carry their templates and record
`blocked(...)` for their placement. The brief's stop condition is honoured literally:
**no shipped file names a type the port did not register**, and the runtime test
`no_shipped_file_names_a_type_the_pin_cannot_decode` machine-checks that over every
JSON under `data/`.

### What the pin would have tolerated (and why that is not the gate)

At this pin a *dangling* feature type is **not** fatal: an undecodable placed feature is
skipped and listed — `generator_decoration.rs:380` (`unsupported()`), `:524-533`
(`stats.unsupported`, `stats.skipped.insert(..)`), surfaced by
`Generator::unsupported_features()` ("Placed features of this dimension's biomes that
decoration skips because their algorithm or bindings are not implemented") — and a biome
modifier naming an unknown placed feature is skipped with a warning
(`mod_content_biomes.rs`, test `biome_modifiers_refuse_unknown_names`: *"unknown placed
feature … another mod's ids … are common in real mods, and the mods still load"*).
That is stricter than NeoForge, where an unknown configured-feature type fails the data
pack. The port does **not** rely on the leniency: shipping 62 dead chains would mean 62
permanent "unsupported" lines and a set that looks loaded but places nothing.

## Inventory (from the jar; the scope's evidence)

297 entries, **62 templates**, 62 configured features, 62 placed features, 0 template
pools, 0 structure definitions.

| Family (`type`) | templates | dir | config | placement |
|---|---:|---|---|---|
| `yungsextras:swamp_arch` | 11 | `swamp/arch` | `location` | **shipped** |
| `yungsextras:swamp_double_arch` | 22 | `swamp/double_arch` | `location` | blocked |
| `yungsextras:swamp_cubby` | 8 | `swamp/cubby` | `location` | blocked |
| `yungsextras:swamp_pillar` | 3 | `swamp/pillar` | `location` | blocked |
| `yungsextras:swamp_church` | 1 | `swamp/misc` | `location` | blocked |
| `yungsextras:swamp_ogre` | 1 | `swamp/misc` | `location` | blocked |
| `yungsextras:desert_well` | 9 | `desert/wells` | `location`, `radius` | blocked |
| `yungsextras:desert_obelisk` | 4 | `desert/obelisk` | `location` | blocked |
| `yungsextras:desert_ruins_0` | 1 | `desert/misc` | none (id in code) | blocked |
| `yungsextras:desert_giant_torch` | 1 | `desert/misc` | none (id in code) | blocked |
| `yungsextras:desert_chillzone` | 1 | `desert/misc` | none (id in code) | blocked |

Templates are 5×6×1 (arch) up to 13×13×5 (church), 8..273 blocks, and use **47 distinct
vanilla blocks only** — no mod dependency, nothing 1.21-only. The marker blocks the
mod's processors read ship inside them: `gray_stained_glass` (arch's leg, double arch,
cubby, pillar, church/ogre), `light_gray_stained_glass` (stairs), `green_candle`,
`brown_stained_glass`/`yellow_stained_glass` (the wells' suspicious sand), plus 3 chests
naming `yungsextras:desert/chillzone` (×1) and `yungsextras:desert/ruins` (×2), and 8
campfire block entities.

Other data: 3 loot tables (`desert/chillzone`, `desert/ruins` used by the templates;
`desert/extra_archeology` used by `DesertWellProcessor` for the wishing wells' yellow
markers), 3 biome tags (`has_structure/{swamp_structures,desert_decorations,vanilla_desert_well}`),
6 biome-modifier files (the same 3 in `neoforge/` and `forge/`, the second a datagen
leftover), and one placement-modifier type, `yungsextras:rng_initializer`, which every
placed chain ends with.

## Upstream reading (analysis aid only; no Java was translated)

Read at `ebc3bc66…` (`Common/`): `module/FeatureModule.java`,
`module/PlacementModifierTypeModule.java`, `world/feature/AbstractNbtFeature.java`,
`world/feature/swamp/{AbstractSwampFeature,SwampArchFeature}.java`,
`world/feature/desert/DesertWellFeature.java`, `world/placement/RngInitializerPlacement.java`,
`world/processor/{SwampFeatureProcessor,DesertWellProcessor}.java`. Findings
([`FINDINGS.md`](FINDINGS.md)): the arch's placement is surface search → two 4-down
terrain checks → centre-place with `Rotation.NONE` → `SwampFeatureProcessor` (leg
replace + fill-down, weighted stone bricks, weighted stairs, candle randomisation); the
desert well adds a 7-deep × 4-corner terrain test, a `SAND` requirement, a `radius` used
only by those checks, and places **6 blocks below** the surface. That last one is why the
well is not expressible as a template feature: no vanilla placement modifier reads a
block and moves the origin.

## What ships

```
horizon-mod.toml                               the only manifest: id, version 0.2.0, api 0.2,
                                               side both, registries, [native] SDK pins,
                                               license = "LGPL-3.0-only"
Cargo.toml                                     one crate, horizon-mod-api from .horizon/sdk/
yungsextras.hmod                               delivered 99-entry SDK package
data/yungsextras/structure/**                  62 templates, byte-identical
data/yungsextras/loot_table/desert/*.json      3 loot tables, byte-identical
data/yungsextras/tags/worldgen/biome/has_structure/*.json  3 tags, byte-identical
data/yungsextras/worldgen/feature/swamp/arches/*.json       11 REWRITTEN (minecraft:template)
data/yungsextras/worldgen/placed_feature/swamp/arches/*.json 11 REWRITTEN (offset for the ground)
data/yungsextras/neoforge/biome_modifier/swamp_arches.json  1 REWRITTEN (the 11 shipped ids)
src/lib.rs                                     the one impl mod_api::Hooks (empty by design)
src/runtime.rs                                 the catalogue + 9 tests (the only code)
tools/carry.py                                 199 files from the jar; `--check`
tools/gen_worldgen.py                          the 23 rewritten files; `--check`
tools/e2e.sh                                   the in-game leg (claimed 2026-11-18)
tools/carry/**                                 130 carried files the package does not ship
LICENSE, LICENSE-NOTICE.md, README.md          inside the package
FINDINGS.md, notes.md                          the record
.gitignore                                     /target, /.horizon, /Cargo.lock
```

The jar's worldgen for the ten carried families — 51 configured features, the 62 placed
chains, and the 6 biome-modifier files — sits in `tools/carry/worldgen/` and
`tools/carry/{neoforge,forge}/biome_modifier/`, byte-identical, **not packed**: their
`type` is a `yungsextras:` feature type or `yungsextras:rng_initializer`, exactly what
the stop condition forbids shipping. `tools/carry.py --check` proves all 199.

## The swamp arch family, rewritten

`tools/gen_worldgen.py` derives the 23 shipped files from the carried originals, so the
shipped form is reproducible and auditable, and `--check` fails on a hand edit.

1. **`type`**: `yungsextras:swamp_arch` → `minecraft:template` (the surface from **B**
   above). **26.3's configured feature is flat**: `config` is unwrapped — the campaign's
   carry writes it that way (`crates/horizon-port-pack/src/upgrade/features.rs`, *"A
   configured feature in 26.3's form: `config` unwrapped"*) and the decoder reads the
   config's keys at the root (`TemplateFeature::decode` takes `templates` and
   `processors`). The mod's `config.location` becomes a root `templates` entry, and each
   entry is a **weighted-list** entry
   `{"data": {"id": <location>, "rotations": ["none"]}, "weight": 1}`
   (`worldgen/weighted.rs:71` requires `data` and `weight`; `TemplateFeature` reads `id`
   and `rotations` from that payload). One template, one rotation, as the mod places with
   `new StructurePlaceSettings()` (`Rotation.NONE`, `Mirror.NONE`). Both shapes were
   **measured wrong first** in game — see "Defects the in-game leg caught" below.
2. **root-level `processors`**: the mod's `SwampFeatureProcessor`, as data —
   - `horizon:yung_pillar` on `minecraft:gray_stained_glass`: the leg block becomes
     the weighted stone brick and the column under it is filled to the ground through
     air and fluids (`pillars.rs:685-747`, YUNG's own pillar processor);
   - `horizon:pick` on `minecraft:stone_bricks`: mossy `below 0.45`, cracked
     `below 0.90`, `default` plain — YUNG's `BlockStateRandomizer` draws one
     `nextFloat` against cumulative bounds, which is what `horizon:pick` does;
   - `horizon:pick` on `minecraft:stone_brick_stairs`: mossy `below 0.6`, `default`
     plain, `keep_properties: true` (facing/half/shape/waterlogged survive, as
     `Block.withPropertiesOf` does upstream).
   The arch templates carry no `light_gray_stained_glass` and no `green_candle`, so
   those two parts of the processor (random stair facing, candle colour/count/lit) have
   nothing to act on in this family.
3. **the chain**: `rarity_filter 2400`, `in_square`, `heightmap WORLD_SURFACE_WG`,
   `biome` are carried unchanged; `yungsextras:rng_initializer` is dropped (it cannot
   be named) and replaced by `minecraft:offset {x:0, y:-1, z:0}`. The heightmap
   modifier leaves the origin on the **first free** height; the mod centres its template
   on the **topmost non-air** block. The offset is what reproduces the mod's origin —
   without it every arch would sit one block high.

## Deviations (not defects)

1. **No `yungsextras:rng_initializer`.** The mod reseeds the placement random from the
   placed position (`RngInitializerPlacement`) "to maximize variability". The pin has no
   placement-modifier registration, so the chain uses the game's per-feature random.
   Effect: two arches placed in the same chunk/step index draw the same block variants
   instead of position-mixed ones. Placement positions and rarity are unaffected.
2. **The mod's two 4-down terrain checks are not expressible.** They read the blocks at
   two offsets from the origin and refuse the placement; no placement modifier reads a
   block at an offset. Effect: where the mod would refuse an arch standing over a drop,
   the port places it and `horizon:yung_pillar` runs the leg down to the ground — a long
   leg over a chasm instead of no arch. Rare (1 in 2400 per chunk, swamp only) and the
   leg is the mod's own block choice.
3. **flags 3 instead of 2** for the template's block writes: the pin's `TemplateFeature`
   places with neighbour updates where `StructureTemplate.placeInWorld(…, 2)` does not.
   During chunk generation both write the proto-chunk; no observable difference is known,
   and none was tested in game (gap 1 below).
4. **Ten families carry, do not place** — their feature types cannot be registered. Each
   family's own placement logic (a dig radius, a 6-block-deep origin, a coastline test,
   a multi-template chillzone) is also beyond `minecraft:template` even if the type
   existed.
5. **`forge/` biome-modifier copies are not shipped.** They are the Forge sibling of the
   `neoforge/` files (identical content, `forge:add_features`), dead on NeoForge and
   read by this pin as well; the port ships one rewritten modifier instead.

## Upstream defects fixed

**None found in the shipped family.** Checked, because the artefact invites it:
`SwampArchFeature`'s corner offset (`-2,0,0` for a 5-wide template) matches its data; the
three id-in-code desert features
(`ChillzoneDesertFeature.java:20`, `DesertGiantTorchFeature.java:20`,
`DesertSmallRuinsFeature.java:20`) name templates that all exist in the jar; the swamp
processor's marker blocks (`gray`/`light_gray`/`green_candle`) all occur in the templates
that carry them; every `location` in the 59 configured features that have one resolves to
a shipped `.nbt`. The `forge/`+`neoforge/` duplicate pair is datagen, not a bug (and at
this pin the double add is deduplicated: *"a feature already in the step is not added
twice"*, `docs/mods.md:463`).

## Tests

`src/runtime.rs` — the catalogue (`FAMILIES`, `MISSING_PLACEMENT_MODIFIER`,
`family()`, `blocked()`, `type_is_decodable()`) and **11 tests** that read the port tree
and prove the deliverable, not the code:

| Test | Proves |
|---|---|
| `every_carried_feature_names_a_registered_family` | 62 configured features, every `type` is one of the 11, and each family's count |
| `every_family_has_exactly_its_templates` | every family's directory holds its templates; 62 on disk and 62 accounted for |
| `every_carried_feature_names_a_carried_template` | all 59 `location`s resolve to a shipped `.nbt`; the 3 computed families name none; per-family template counts |
| `every_carried_chain_ends_in_the_missing_modifier` | 62 placed features, each naming a carried configured feature and each ending in `yungsextras:rng_initializer` |
| `shipped_arches_use_the_pins_template_feature` | 11 shipped features are `minecraft:template`, one `yungsextras:` template each, `rotations ["none"]`, the three processors in order |
| `shipped_arches_keep_the_mods_placement_chain` | the 5 shipped modifiers in order, `chance 2400`, `y -1` |
| `shipped_modifier_lists_exactly_the_shipped_chains` | 11 ids, each a shipped placed feature, `step surface_structures`, the mod's biome tag |
| `no_shipped_file_names_a_type_the_pin_cannot_decode` | every `"type"` in all 29 JSON files under `data/` is `minecraft:`/`horizon:`/`neoforge:add_features` — the stop condition, machine-checked |
| `shipped_features_use_26_3_flat_config` | 26.3's flat shape: no `config` wrapper, `templates` at the root (the first in-game defect, pinned) |
| `shipped_features_wrap_the_template_in_a_weighted_entry` | each `templates` entry is `{"data": {…}, "weight": 1}` (the second in-game defect, pinned) |
| `blocked_families_name_their_missing_symbol` | 10 families report `blocked(no FeatureType registration for …)` |

The suite was watched red first, twice over: the compile-time-invisible defects showed up
as assertion failures on the first run (`one template` 13 ids because the processor states
are ids too; `chance` empty because it is a number; `features` empty because it is an
array) until the JSON scanner was fixed; and the two shape defects the in-game leg found
were each pinned by a test written **before** the fix, watched red, then made green. `cargo test` runs on the host:
`src/runtime.rs` is plain Rust with no `mod_api` types, and `src/lib.rs` is the empty
`impl Hooks` the format needs (this port has no hook to implement — worldgen places the
features, not a callback).

## Verification

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

Every compile ran inside a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` (the
27 MB `.horizon/sdk` in the port makes the automatic forward pointless), `CARGO_INCREMENTAL=0`,
`TMPDIR=$PWD/target/tmp`, no `--target-dir`. Logs:
`ports/_shared/evidence/yungsextras/`.

| Step | Command | Result | Evidence |
|---|---|---|---|
| carry | `python3 tools/carry.py` | 199 files from the jar (198,183 B, sha256 `0cd26474…`) | — |
| carry check | `python3 tools/carry.py --check` | **199/199 byte-identical** (`carry=130`, `data=68`, `license=1`) | this run |
| worldgen regenerate | `python3 tools/gen_worldgen.py` → `--check` | **23/23 match** (11 arches) | this run |
| format | `cargo fmt --check` (**not** `--all`) | exit 0 | `sdk-fmt.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `sdk-clippy.log` |
| tests | `cargo test --offline` | **11 passed; 0 failed** (0 doc-tests) | `sdk-test.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `SDK 0b14f8a (build 0b14f8ac…ca4) in ./.horizon/sdk` / `Built ./target/horizon-mod/yungsextras.hmod` | `sdk-build.log` |
| second build | the same, again | `cmp` identical, same sha256 | `sdk-build-second.log`, `sdk-sha256.log` |
| registry check | `cargo horizon-mod check yungsextras.hmod` | **EXIT 0** (verbatim below) | `sdk-check.log` |
| fits | `cargo horizon-mod fits yungsextras.hmod --build nightly` | `0b14f8a  made for`, **EXIT 0** | `sdk-fits.log` |
| runtime untouched | `unzip -p yungsextras.hmod src/runtime.rs \| sha256sum` | `1a3c7359…c3fb44` = `src/runtime.rs` | `sdk-sha256.log` |
| clean-target re-run | `rm -rf target` then `cargo test --offline` | **11 passed**, 0 failed, from scratch (`wall 0:37.28`) | `sdk-test-final-clean-target.log` |
| in game, modded | `launcher-game.sh run yungsextras` | `PASS: …` + `E2E EXIT:0` | `yungsextras-e2e/e2e-launcher.log` |
| in game, control | `GAME=$CONTROL_GAME … bash tools/e2e.sh` | `FAIL: … placed nothing` + `E2E EXIT:1` | `yungsextras-e2e/e2e-launcher-control.log` |
| in game, selftest | `SELFTEST=1 …` | `FAIL: SELFTEST: the false claim failed as required` + `E2E EXIT:1` | `yungsextras-e2e/e2e-launcher-selftest.log` |
| e2e gate | `check-e2e-logs.py yungsextras-e2e --strict` | `logs=3 ok=3 missing-exit=0 bad-exit=0`, exit 0 | this run |

```
$ cargo horizon-mod check yungsextras.hmod
yungsextras 0.2.0 (native-mod, format native-1/0b14f8acfd97fa2c750e42795249dcc981178ca4/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  70404 bytes, SHA-256 28da290a7bff9efe572e46dc89394b8f178f027842f29a0f29ebdc9160f83b5f, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.

$ cargo horizon-mod fits yungsextras.hmod --build nightly
YUNG's Extras:
0b14f8a  made for
```

### Package

`yungsextras.hmod` — **70,404 B**, SHA-256
`28da290a7bff9efe572e46dc89394b8f178f027842f29a0f29ebdc9160f83b5f`, **99 entries**:
`horizon-mod.toml`, `Cargo.toml`, `LICENSE` (7,817 B, the LGPL-3.0 text the jar ships),
`LICENSE-NOTICE.md`, `README.md`, `src/lib.rs`, `src/runtime.rs`, `uses.json`, **91
`data/` files** (62 templates + 3 loot + 3 tags + 11 features + 11 placed features + 1
biome modifier). Built **twice with identical bytes** — and the same bytes are what the
launcher compiled into the game the e2e ran against (`native-0b14f8a-28c4599a63dd`).
No `Cargo.lock`, no `tools/`, no `.horizon/`.

### In game — measured 2026-11-18 (launcher `0.2.78`, Nightly `0b14f8a`)

`ports/tools/launcher-game.sh run yungsextras` compiled this package into a game at the
SDK pin and ran `tools/e2e.sh` in a build slot. Three legs, all through
`ports/tools/build-slot.sh` (logs in `ports/_shared/evidence/yungsextras-e2e/`):

| Leg | Command | Result |
|---|---|---|
| modded | `launcher-game.sh run yungsextras` | `PASS: 62/62 templates read; the swamp arch feature places with its marker replaced and its leg filled to the ground; a carried desert well places by hand and its feature id places nothing` + `E2E EXIT:0` |
| control | `GAME=$CONTROL_GAME build-slot.sh run … bash ports/yungsextras/tools/e2e.sh` | `FAIL: yungsextras:desert/misc/chillzone placed nothing: the template did not read` + `E2E EXIT:1` |
| selftest | `SELFTEST=1 …` (the false claim) | `FAIL: SELFTEST: the false claim failed as required (a vanilla template left '')` + `E2E EXIT:1` |

`python3 ports/tools/check-e2e-logs.py yungsextras-e2e --strict` → `logs=3 ok=3
missing-exit=0 bad-exit=0`, exit 0.

What the run **measured**, verbatim from the log:

```
62/62 templates placed at least one block
arch body: minecraft:chiseled_stone_bricks,minecraft:cracked_stone_bricks,minecraft:mossy_stone_brick_stairs,minecraft:mossy_stone_bricks,minecraft:stone_brick_slab,minecraft:stone_brick_stairs,minecraft:stone_bricks
leg column (y 77..79): minecraft:cracked_stone_bricks,minecraft:mossy_stone_bricks
well_lg by hand: 251 blocks; its feature: 0 blocks
feature reply: {"chat":["Can't find element 'yungsextras:desert/wells/desert_well_sm' in registry 'minecraft:worldgen/feature'","...rt_well_sm 0 95 0<--[HERE]"],"ok":true}
no yungsextras worldgen error in the log
```

- All **62 carried templates** are readable by the game's own template reader and place
  (`/place template yungsextras:<path>`), so the whole content set is delivered, not just
  the shipped family.
- The **swamp arch places through its shipped descriptor**: `/place feature
  yungsextras:swamp/arches/swamp_arch_0` runs the same configured-feature path the biome
  modifier feeds at generation, the arch body carries the weighted brick/stairs picks,
  **no `gray_stained_glass` marker survives**, and the leg is stone bricks down through
  the air gap to the shelf — the pillar processor ran.
- The **ten carried families are carried but not placed**: `well_lg` places 251 blocks by
  hand and the matching feature id answers *"Can't find element … in registry
  'minecraft:worldgen/feature'"* and places nothing. That is the `blocked(no FeatureType
  registration …)` record, measured rather than argued.

What it **cannot** prove, stated exactly:

- **A generated (biome-placed) arch.** Its chain is `rarity_filter 2400` in swamp biomes,
  so a short run does not meet one; `/place feature` runs the same descriptor and the
  same processors, but the biome modifier → feature-sorter → decoration path is not
  exercised by this run. The biome modifier's own shape is checked statically
  (`shipped_modifier_lists_exactly_the_shipped_chains`, and the 26.3 biome-modifier
  reader's rules in `mod_content_biomes.rs`).
- **That a mod-defined feature type exists.** It does not: the arches are data naming
  *vanilla's* `minecraft:template`. Nothing here tests a `yungsextras:` feature type.
- **The `y`-offset and centring against upstream's own output.** The arch lands on the
  floor by construction (`heightmap` → `offset y:-1` → the template's base row replaces
  the top solid block); there is no NeoForge 1.21.1 oracle on this box to diff against.

## Defects the in-game leg caught (both were mine, both fixed)

The first two runs of this e2e were red, and both failures were in the shipped data, not
in the harness — preserved as evidence
(`ports/_shared/evidence/yungsextras-e2e/game-log-run1-feature-config-nesting.log`):

1. **`config` was not unwrapped.** The first generator wrote the 1.21.1 shape
   `{"type":"minecraft:template","config":{"templates":[…]}}`. The game said, for all 11
   arches: `Horizon: 11 placed features cannot run and will be skipped` /
   `yungsextras:swamp/arches/swamp_arch_0: invalid type: null, expected a sequence at line
   1 column 4` (the decoder read `templates` at the root, got null). Fixed by emitting
   26.3's flat form; pinned by the test `shipped_features_use_26_3_flat_config`.
2. **The template list was not a weighted list.** With the flat form the game said
   `invalid world generation field: data`: `WeightedList::decode` requires every entry as
   `{"data": <payload>, "weight": n}`. Fixed by wrapping the template payload; pinned by
   `shipped_features_wrap_the_template_in_a_weighted_entry`.

A third red run was the **harness**: the counterfactual leg cleared from `y=78` while the
leg fills down to `y=77`, so it read the arch's own leg and the `SELFTEST` "false claim"
held. Cleared from `y=74`; the selftest now fails at the intended assertion. It is
recorded here because a selftest that passes for the wrong reason is the exact failure
this gate exists to prevent.

## Gaps

1. **The generated arch is not asserted** (above): the descriptor and processors are
   measured through `/place feature`, the decoration path is not.
2. **Ten families place nowhere.** Their templates are `/place template`-able and a
   hand-placed `well_lg` is measured at 251 blocks, but nothing generates them. Recorded
   as `blocked(no FeatureType registration for yungsextras:<family>)` in `blocked()` and
   in these notes; the resume condition is an SDK `FeatureType` (or an equivalent "place
   this template at the origin" registration) plus `PlacementModifierType` for
   `rng_initializer`.
3. **No `rng_initializer`, no offset-block terrain checks** (deviations 1 and 2).
4. **The wells' markers stay glass.** Their `brown`/`yellow_stained_glass` markers are
   replaced by `DesertWellProcessor` (suspicious sand + the mod's archaeology table); a
   `/place template` of a well leaves them as glass. The loot tables ship so a future
   slice can express the processor.
5. **No diagnostics.** The SDK exports no logger and a native mod may not add a crate;
   this port has nothing to log.
6. **`side = both`** because the content and the worldgen it feeds belong to the server,
   and an integrated server belongs to the client installation.

## Traps new to this port

1. **A 26.3 configured feature is flat, and a template list is a weighted list.** Both
   bit in game before they bit anywhere else. `{"type":…,"config":{…}}` loads and never
   decodes (*"invalid type: null, expected a sequence"* — `TemplateFeature` reads
   `templates` at the root), and a bare `{"id":…}` entry decodes as *"invalid world
   generation field: data"* (`WeightedList::decode` wants `{"data":…,"weight":n}`). The
   campaign's own carry documents the first rule (`horizon-port-pack/src/upgrade/
   features.rs`), which is where to look before writing a configured feature by hand.
2. **26.3 renamed the folder.** `worldgen/configured_feature/` is the 26.2 name;
   `WORLDGEN_BUNDLES` (`mod_content.rs:1899-1909`) reads `worldgen/feature/` first and
   keeps `configured_feature` as an alias. The port writes `feature/` so the tree says
   what 26.3 calls it; a carry that copies the jar's path still loads through the alias.
3. **A dangling feature type is not fatal here — measured.** The first red run printed
   `Horizon: 11 placed features cannot run and will be skipped` and one line per arch, and
   the world still loaded. So a green game proves nothing about a dangling type; the gate
   has to be static (`no_shipped_file_names_a_type_the_pin_cannot_decode`) **and** the
   in-game leg has to assert the feature *places*, not merely that the game starts.
4. **`/place template` and `/place feature` write on the next server tick.** A `ctl
   blocks` read in the same tick sees the empty box (measured: "Loaded template … at 0,
   95, 0" and zero blocks). The e2e waits one tick after each placement; the first version
   did not and reported 62 failures that were all the harness.
5. **`heightmap` is one block above the mod's origin.** `getFirstAvailable` is the first
   **free** height; the mod centres on the **topmost non-air** block. A naive
   `minecraft:template` carry floats every piece by one; `minecraft:offset y:-1` fixes it.
6. **A template is not a structure.** `data/<ns>/structure/**.nbt` needs no structure
   definition, no set and no pool — the content root moves it into the worldgen overlay
   and `/place template <ns>:<path>` reads it (measured: all 62 place in a launcher build).
   That is the only player-facing surface left when the feature type is missing, and it is
   worth carrying the whole set for.
7. **YUNG's markers ship inside the templates.** A "just place the template" family
   leaves `gray_stained_glass` legs and `green_candle` placeholders in the world.
   `horizon:yung_pillar` + `horizon:pick` are the exact data processors for them, and
   `horizon:pick`'s `below` bounds are cumulative in list order — the same draw as
   YUNG's `BlockStateRandomizer`, so the weights transfer verbatim (0.45/0.45, 0.6).
8. **A counterfactual must clear the whole column it reads.** The leg fills down to
   `y=77`; a clear that starts at `y=78` leaves the previous arch's leg in place, and the
   `SELFTEST` "false claim" held for the wrong reason. Clear below the deepest block the
   previous placement wrote (here `y=74`).
9. **`BUILD_LOCAL=1` on every step.** The SDK tree inside the port is 27 MB; the slot
   runner's automatic forward would sync it to the far box for a build that takes
   seconds locally (the same finding as `bellsandwhistles` and `chisel`). The SDK was
   copied from `ports/bellsandwhistles/.horizon` (same pin, gitignored), so no step
   needed the network.
10. **The packer packs `horizon-mod.toml`, `assets/`, `data/`, `Cargo.toml`, `src/`, the
    root `LICENSE`/`LICENSE-NOTICE.md`/`README.md` and `uses.json`, and nothing else.**
    `tools/` is tolerated and ignored, so the unshippable worldgen lives under
    `tools/carry/`; a new top-level `carry/` would be refused as an unexpected package
    path (the `bellsandwhistles` trap 1, same shape).
11. **The launcher leg is not a build-slot e2e.** `build-slot.sh e2e` runs the retired
    `tools/horizon.py build`; use `launcher-game.sh run <mod>`, or the two-step
    `eval "$(launcher-game.sh env <mod>)"` + `build-slot.sh run --step e2e -- bash
    ports/<mod>/tools/e2e.sh` for the control leg (the script's `GAME` must be overridden
    *after* the `eval`, because `env` prints the modded path).

## Changelog (0.2.0)

> **0.2.0 — first publication, as an SDK `.hmod`.** There is no earlier submission of
> this port to supersede. Horizon's mod-API merge (launcher ≥ 0.2.70) retired `.hnmod`,
> so the port ships as one flat package (mod API `0.2`) built against SDK `0b14f8a` /
> mixins `e59da90` for 26.3, under **LGPL-3.0-only** with the complete corresponding
> source (this directory) and the licence text inside the package.
>
> It carries YUNG's Extras' **62 structure templates** and their desert loot tables and
> biome tags, and it **places the swamp arch family** — 11 ruined stone-brick arches in
> swamp biomes at the mod's rarity, with the mod's own marker processing (the gray-glass
> leg becomes weighted mossy/cracked stone bricks and runs down to the ground; stone
> bricks and stairs take the mod's 45/45/10 and 60/40 weights). It places them through
> vanilla's `minecraft:template` feature, because this pin has **no way for a mod to
> register a feature type**: `yungsextras:swamp_arch` and the mod's
> `yungsextras:rng_initializer` placement modifier cannot be named at 26.3.
>
> **The other ten families are carried, not placed**: the desert wells, obelisks, ruins,
> giant torch and chillzone, and the swamp double arches, pillars, cubbies, church and
> ogre. Their templates are in the package and every one can be placed by hand with
> `/place template yungsextras:<path>`; nothing generates them. Their placement is
> recorded as `blocked(no FeatureType registration for yungsextras:<family>)`, and the
> mod's own placement chains are kept byte-identical in the port's `tools/carry/` for the
> slice that resumes them.
>
> Known deviations from upstream: the arches' placement chain drops
> `yungsextras:rng_initializer` (adds `minecraft:offset y:-1` to keep the arch on the
> ground), the mod's two block-offset terrain checks cannot be expressed, and the ten
> carried families generate nowhere.
>
> **Verified in game** (launcher `0.2.78`, Nightly `0b14f8a`, 2026-11-18): all **62**
> templates read and place by hand; `/place feature yungsextras:swamp/arches/swamp_arch_0`
> places the arch with **no `gray_stained_glass` marker left** and its leg filled down to
> the ground through a four-block air gap; the carried families' feature ids are refused by
> the registry (`Can't find element … in registry 'minecraft:worldgen/feature'`) while
> their templates place 251 blocks by hand. `PASS` + `E2E EXIT:0`; the same command on the
> unmodified game fails at the first template (`E2E EXIT:1`), and `SELFTEST=1` fails at the
> false claim (`E2E EXIT:1`). The biome-placed arch is not asserted (rarity 2400 — see
> Gaps). Two packaging-shape defects in this port were found by that run and fixed before
> publication (see "Defects the in-game leg caught").
