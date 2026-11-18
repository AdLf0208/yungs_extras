# YUNG's Extras — a Horizon port

**YUNG's Extras** is a Minecraft mod. Extra structures and vanilla+ content added throughout Minecraft. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [YUNG-GANG/YUNGs-Extras](https://github.com/YUNG-GANG/YUNGs-Extras) — licence **LGPL-3.0-only**.

| | |
|---|---|
| Upstream mod | YUNG's Extras ([source](https://github.com/YUNG-GANG/YUNGs-Extras)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `yungsextras.hmod` (install it with the Horizon launcher) |
| Licence | **LGPL-3.0-only** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

| Family (`type`) | templates | dir | config | placement |
| `yungsextras:swamp_arch` | 11 | `swamp/arch` | `location` | **shipped** |
| `yungsextras:swamp_double_arch` | 22 | `swamp/double_arch` | `location` | blocked |
| `yungsextras:swamp_cubby` | 8 | `swamp/cubby` | `location` | blocked |
| `yungsextras:swamp_pillar` | 3 | `swamp/pillar` | `location` | blocked |
| `yungsextras:swamp_church` | 1 | `swamp/misc` | `location` | blocked |
| `yungsextras:swamp_ogre` | 1 | `swamp/misc` | `location` | blocked |
| `yungsextras:desert_well` | 9 | `desert/wells` | `location`, `radius` | blocked |
| `yungsextras:desert_obelisk` | 4 | `desert/obelisk` | `location` | blocked |

## Not yet ported

- **The generated arch is not asserted** (above): the descriptor and processors are
- **Ten families place nowhere.** Their templates are `/place template`-able and a
- **No `rng_initializer`, no offset-block terrain checks** (deviations 1 and 2).
- **The wells' markers stay glass.** Their `brown`/`yellow_stained_glass` markers are
- **No diagnostics.** The SDK exports no logger and a native mod may not add a crate;
- **`side = both`** because the content and the worldgen it feeds belong to the server,

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `yungsextras.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

YUNG's Extras was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **LGPL-3.0-only**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/YUNG-GANG/YUNGs-Extras

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run yungsextras` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/yungsextras/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
