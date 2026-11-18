# YUNG's Extras for Horizon

The swamp and desert decoration of YUNG's Extras (upstream
`yungnickyoung/YUNGs-Extras`, LGPL-3.0-only, version 1.21.1-NeoForge-5.1.1),
carried to Minecraft **26.3** on Horizon's mod API **0.2**.

The mod is eleven Java *feature types* that place one structure template at a
feature's origin, plus one Java placement-modifier type that reseeds the
placement random. Horizon has no feature-type registration for a mod, so this
package places **one family** and carries the rest.

## What places

**The swamp arch** — 11 templates. They generate in swamp biomes
(`#yungsextras:has_structure/swamp_structures`) in the `surface_structures` step
at the mod's own rarity (1 in 2400 per chunk per template), on the surface, in
one of 11 ruined stone-brick forms, with the mod's own marker processing: the
arch's gray-glass leg becomes randomly mossy/cracked stone bricks and is run
down to the ground, and its stone bricks and stairs take the mod's weights
(45 % mossy, 45 % cracked, else plain; 60 % mossy stairs).

## What does not place (carried, `blocked`)

The other ten families — desert wells, obelisks, ruins, the giant torch and the
chillzone; swamp double arches, pillars, cubbies, the church and the ogre. Their
configured features name `yungsextras:desert_well`, `yungsextras:swamp_cubby` and
their siblings, and their placement chains end in `yungsextras:rng_initializer`;
a content mod cannot register either type at this pin. Their templates still
ship, so every piece can be placed by hand:

```
/place template yungsextras:desert/wells/well_lg
/place template yungsextras:swamp/misc/swamp_church
```

## Licence

LGPL-3.0-only. `LICENSE` is the GNU Lesser General Public License version 3 as
the upstream jar ships it; `LICENSE-NOTICE.md` names the upstream authors, the
jar and where each file came from. The complete corresponding source of this
port is this package's own `Cargo.toml`, `src/` and `tools/`.
