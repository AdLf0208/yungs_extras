//! YUNG's Extras — the one `impl Hooks` at the crate root.
//!
//! The mod has no hook: its pieces are placed by the world generation, from data
//! (`data/yungsextras/worldgen/`, `data/yungsextras/neoforge/biome_modifier/`),
//! and its 62 templates are read by the game's own template reader. Nothing in
//! this slice registers a block, an item, an entity or a structure type, so the
//! `impl Hooks` below is empty on purpose.
//!
//! `src/runtime.rs` is the port's catalogue: what the jar registers, what the
//! pin can place, and the tests that prove the carried files and the catalogue
//! agree.
pub mod runtime;

use mod_api::{self, Hooks};

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {}
