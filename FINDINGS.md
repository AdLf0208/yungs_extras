# YUNG's Extras — behaviour notes from the upstream source

Read at `yungnickyoung/YUNGs-Extras`, branch `1.21.1`, commit
`ebc3bc669e8e23ddde99fa7c663ffc92078ffbd8` (`gradle.properties` `version=5.1.1`), the
version the pack ships. `Common/src/main/java/com/yungnickyoung/minecraft/yungsextras/`
unless stated. **Analysis only: the port is data plus a clean-room Rust catalogue; no
Java was translated.**

## The shape

A *feature* places *one structure template* at the world-position the feature was given:

```
configured_feature/<theme>/<family>/<name>.json
  {"type": "yungsextras:<family>", "config": {"location": "yungsextras:<theme>/<dir>/<name>"[,"radius": n]}}
placed_feature/…      {"feature": …, "placement": [rarity_filter, in_square, heightmap, biome, yungsextras:rng_initializer]}
neoforge/biome_modifier/*.json   add_features → the swamp / desert biome tags
data/yungsextras/structure/<theme>/<dir>/<name>.nbt   the template
```

`FeatureModule.java` registers **11** types; `PlacementModifierTypeModule.java`
registers **1** modifier. There is no structure, no structure set and no template pool
in the jar: the mod is features all the way down.

## The placement machinery (`world/feature/AbstractNbtFeature.java`)

- `createTemplateFromCenter(id, level, rand, centerPos, placement)`: reads the template
  from the level's `StructureTemplateManager`, offsets the corner by
  `-sizeX/2, 0, -sizeZ/2`, places with `placeInWorld(level, corner, centerPos,
  placement, rand, 2)` (flags 2 = no neighbour updates), then runs the feature's
  processors over it.
- `createTemplateFromCorner` is the mirror of it (offset `+sizeX/2`), used by no shipped
  family.
- A missing template id only logs `Failed to create invalid feature {id}` and returns
  null; the placement then reports `false`.

## The swamp arch (`world/feature/swamp/SwampArchFeature.java`)

1. walk down from the feature origin while the block `isEmptyBlock` (air only) and
   `y > 2` → `surfacePos` = the topmost non-air block;
2. `cornerPos = surfacePos.offset(-2, 0, 0)` — for the 5-wide templates, the west foot;
3. refuse if the block 4 below `cornerPos` is air, or 4 below `cornerPos + 4 east`
   (the east foot) is air — "can only extend max 3 down in air";
4. `createTemplateFromCenter(location, …, surfacePos)` with default settings
   (`Rotation.NONE`, `Mirror.NONE`);
5. `AbstractSwampFeature` attaches `SwampFeatureProcessor` to every swamp family.

The arch data: 11 templates, 5×6×1, 8..15 blocks, one `gray_stained_glass` each (the
leg), `stone_bricks`, `chiseled_stone_bricks`, `stone_brick_slab`, `stone_brick_stairs`.

## `world/processor/SwampFeatureProcessor.java`

- `gray_stained_glass` → `STONE_BRICK_SELECTOR` (stone bricks default; mossy 0.45,
  cracked 0.45 of a cumulative draw), and every block below it that is air, liquid or
  replaceable → the same selector until a solid block.
- `light_gray_stained_glass` → `STONE_BRICK_STAIRS_SELECTOR` with a *random* horizontal
  facing.
- `stone_bricks` → `STONE_BRICK_SELECTOR`.
- `stone_brick_stairs` → `STONE_BRICK_STAIRS_SELECTOR` (plain default, mossy 0.6),
  keeping facing/half/shape/waterlogged.
- `green_candle` → a random candle colour, `candles` = `nextInt(4)+1`, `lit` with 10 %.

Only the first, third and fourth can appear in the arch templates (the arch has no
light-gray marker and no candle); the port expresses those three with
`horizon:yung_pillar`, `horizon:pick` and `horizon:pick` — see `tools/gen_worldgen.py`.

## `world/placement/RngInitializerPlacement.java`

Draws two `nextLong() | 1` and reseeds the placement random from
`(x·a·341873128712 + 12412146)·(z·b·132897987541 + 5813717) ^ 423487234`, "to maximize
variability in feature placement". Identity of position, so it affects only which
variants a chain picks. Not expressible at this pin; dropped (deviation 1 in `notes.md`).

## The other families (why they are carried, not placed)

| Feature | What the code does beyond "place the template" |
|---|---|
| `DesertWellFeature` | 7-deep terrain test on 4 corners at `radius`, requires `#minecraft:sand` at the surface, places at **6 blocks below** the surface |
| `DesertObeliskFeature` | template choice + a surface test (`desert/obelisk`) — 4 templates |
| `DesertGiantTorchFeature`, `DesertSmallRuinsFeature`, `ChillzoneDesertFeature` | empty config, the template id is a `static final ResourceLocation` in the class; each still does its own surface/terrain test |
| `SwampPillarFeature`, `SwampCubbyFeature`, `SwampDoubleArchFeature` | their own surface search and air-depth limits; different processors (`light_gray` stairs, candles) |
| `SwampChurchFeature`, `SwampOgreFeature` | a much larger template (13×13×5, 273 blocks) with its own terrain allowance |

None of them is "place this template at the origin", so vanilla's `minecraft:template`
would be a different behaviour rather than a smaller one.

## Data cross-check (what the port's tests assert)

- 62 configured features: 11 + 22 + 8 + 3 + 1 + 1 (swamp) and 9 + 4 + 1 + 1 + 1 (desert).
- 59 of them name a `location`; the 3 `Computed` ones do not. Every location resolves to
  a template in the jar.
- 62 placed features, each naming its configured feature and ending in
  `yungsextras:rng_initializer`.
- 3 chests in the templates name `yungsextras:desert/chillzone` (1) and
  `yungsextras:desert/ruins` (2); `desert/extra_archeology` is named by
  `DesertWellProcessor` for the wishing wells' yellow markers.
