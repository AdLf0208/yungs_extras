//! What YUNG's Extras is, in one table, and what this pin can place of it.
//!
//! The mod is not a structure mod: it registers eleven Java **feature types**
//! that place one structure template at a feature's origin, and one Java
//! **placement-modifier type** (`yungsextras:rng_initializer`) that reseeds the
//! placement random before the feature runs. This pin decodes a content mod's
//! configured features from a fixed set of types and has no feature-type or
//! placement-modifier registry a mod can add to, so the catalogue below is the
//! port's record of that: which type, which templates, which config shape, and
//! whether the port ships it.
//!
//! The tests read the port tree (`data/`, `tools/carry/`) and prove that the
//! catalogue and the carried files agree, and that nothing shipped names a type
//! the pin cannot decode. They are the only code in the crate: the mod has no
//! hook to run, because a feature is placed by the worldgen, not by a hook.
//!
//! Nothing here is `mod_api`: this module is plain Rust so `cargo test` runs on
//! the host, without a staged game.

/// What a family's feature config holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Config {
    /// `{"location": "<template id>"}`: the template is the whole config.
    Location,
    /// `{"location": ..., "radius": n}`: the desert well's dig radius.
    LocationAndRadius,
    /// `{}`: the feature computes its own template id in code.
    Computed,
}

/// One family of pieces: the Java feature type the jar registers, the template
/// directory it draws from, and what the port does with it.
#[derive(Clone, Copy, Debug)]
pub struct Family {
    /// The registered type id, as a configured feature's `type` names it.
    pub feature_type: &'static str,
    /// The template directory under `data/yungsextras/structure/`.
    pub directory: &'static str,
    /// How many templates the jar's configured features of this type name.
    pub templates: usize,
    /// What the family's config holds.
    pub config: Config,
    /// Whether the port places it (`true`) or carries it only (`false`).
    pub shipped: bool,
}

/// Every feature type the jar registers, in the order the modules declare them.
pub const FAMILIES: [Family; 11] = [
    Family {
        feature_type: "yungsextras:desert_well",
        directory: "desert/wells",
        templates: 9,
        config: Config::LocationAndRadius,
        shipped: false,
    },
    Family {
        feature_type: "yungsextras:desert_obelisk",
        directory: "desert/obelisk",
        templates: 4,
        config: Config::Location,
        shipped: false,
    },
    Family {
        feature_type: "yungsextras:desert_giant_torch",
        directory: "desert/misc",
        templates: 1,
        config: Config::Computed,
        shipped: false,
    },
    Family {
        feature_type: "yungsextras:desert_ruins_0",
        directory: "desert/misc",
        templates: 1,
        config: Config::Computed,
        shipped: false,
    },
    Family {
        feature_type: "yungsextras:desert_chillzone",
        directory: "desert/misc",
        templates: 1,
        config: Config::Computed,
        shipped: false,
    },
    Family {
        feature_type: "yungsextras:swamp_pillar",
        directory: "swamp/pillar",
        templates: 3,
        config: Config::Location,
        shipped: false,
    },
    Family {
        feature_type: "yungsextras:swamp_ogre",
        directory: "swamp/misc",
        templates: 1,
        config: Config::Location,
        shipped: false,
    },
    Family {
        feature_type: "yungsextras:swamp_cubby",
        directory: "swamp/cubby",
        templates: 8,
        config: Config::Location,
        shipped: false,
    },
    Family {
        feature_type: "yungsextras:swamp_arch",
        directory: "swamp/arch",
        templates: 11,
        config: Config::Location,
        shipped: true,
    },
    Family {
        feature_type: "yungsextras:swamp_double_arch",
        directory: "swamp/double_arch",
        templates: 22,
        config: Config::Location,
        shipped: false,
    },
    Family {
        feature_type: "yungsextras:swamp_church",
        directory: "swamp/misc",
        templates: 1,
        config: Config::Location,
        shipped: false,
    },
];

/// The placement-modifier type the jar registers and this pin cannot name: every
/// one of the jar's 62 placed features ends its chain with it.
pub const MISSING_PLACEMENT_MODIFIER: &str = "yungsextras:rng_initializer";

/// The family a configured feature's `type` names, if the mod registers it.
#[must_use]
pub fn family(feature_type: &str) -> Option<&'static Family> {
    FAMILIES.iter().find(|f| f.feature_type == feature_type)
}

/// The families the port carries but does not place, with the type that blocks
/// each one: its own feature type cannot be registered at this pin.
#[must_use]
pub fn blocked() -> Vec<(&'static str, String)> {
    FAMILIES
        .iter()
        .filter(|f| !f.shipped)
        .map(|f| {
            (
                f.feature_type,
                format!(
                    "blocked(no FeatureType registration for {})",
                    f.feature_type
                ),
            )
        })
        .collect()
}

/// The mod's feature types are not in the pin's `minecraft:`-only set: a shipped
/// file may name `minecraft:*` (vanilla's features and placement modifiers) or
/// `horizon:*` (the pin's own processors), never a `yungsextras:` type.
#[must_use]
pub fn type_is_decodable(kind: &str) -> bool {
    kind.starts_with("minecraft:")
        || kind.starts_with("horizon:")
        || kind == "neoforge:add_features"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    /// The port root, where `data/` and `tools/carry/` live.
    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn files_under(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let Ok(entries) = fs::read_dir(dir) else {
            return out;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(files_under(&path));
            } else {
                out.push(path);
            }
        }
        out.sort();
        out
    }

    /// The values of `"key"` in `json`, in file order: a string value, a number
    /// value, or the strings of an array value.
    fn values_of<'a>(json: &'a str, key: &str) -> Vec<&'a str> {
        let needle = format!("\"{key}\"");
        let mut out = Vec::new();
        let mut rest = json;
        while let Some(at) = rest.find(&needle) {
            rest = &rest[at + needle.len()..];
            let Some(after) = rest.trim_start().strip_prefix(':') else {
                continue;
            };
            let after = after.trim_start();
            if let Some(after) = after.strip_prefix('"') {
                if let Some(end) = after.find('"') {
                    out.push(&after[..end]);
                }
            } else if let Some(after) = after.strip_prefix('[') {
                // An array of strings (`rotations`, `features`): every quoted
                // string up to the matching closing bracket.
                let mut depth = 1usize;
                let mut end = after.len();
                for (at, c) in after.char_indices() {
                    match c {
                        '[' => depth += 1,
                        ']' => {
                            depth -= 1;
                            if depth == 0 {
                                end = at;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                for part in after[..end].split('"').skip(1).step_by(2) {
                    out.push(part);
                }
            } else {
                let end = after
                    .find(|c: char| !(c.is_ascii_digit() || c == '-'))
                    .unwrap_or(after.len());
                if end > 0 {
                    out.push(&after[..end]);
                }
            }
        }
        out
    }

    fn carried(kind: &str) -> Vec<(PathBuf, String)> {
        let dir = root().join("tools/carry/worldgen").join(kind);
        files_under(&dir)
            .into_iter()
            .map(|p| {
                let text = fs::read_to_string(&p).expect("carried json reads");
                (p, text)
            })
            .collect()
    }

    fn shipped_arches() -> Vec<(PathBuf, String)> {
        let dir = root().join("data/yungsextras/worldgen/feature/swamp/arches");
        files_under(&dir)
            .into_iter()
            .map(|p| {
                let text = fs::read_to_string(&p).expect("shipped feature reads");
                (p, text)
            })
            .collect()
    }

    #[test]
    fn every_carried_feature_names_a_registered_family() {
        let features = carried("configured_feature");
        assert_eq!(features.len(), 62, "the jar's configured features");
        let mut per_family = std::collections::BTreeMap::new();
        for (path, text) in &features {
            let kind = values_of(text, "type");
            let kind = kind.first().unwrap_or_else(|| panic!("{path:?}: no type"));
            let family = family(kind)
                .unwrap_or_else(|| panic!("{path:?}: {kind} is not a registered family"));
            *per_family.entry(family.feature_type).or_insert(0) += 1;
        }
        for family in &FAMILIES {
            assert_eq!(
                per_family.get(family.feature_type).copied().unwrap_or(0),
                family.templates,
                "{} configured features",
                family.feature_type
            );
        }
    }

    #[test]
    fn every_family_has_exactly_its_templates() {
        for family in &FAMILIES {
            let dir = root()
                .join("data/yungsextras/structure")
                .join(family.directory);
            let templates = files_under(&dir)
                .into_iter()
                .filter(|p| p.extension().is_some_and(|e| e == "nbt"))
                .count();
            // Two families share `swamp/misc` and three share `desert/misc`, so a
            // directory can hold more templates than one family names; the
            // per-family count is proved by the locations in the test below.
            assert!(
                templates >= family.templates,
                "{}: {} templates under {}, expected {}",
                family.feature_type,
                templates,
                family.directory,
                family.templates
            );
        }
        let all = files_under(&root().join("data/yungsextras/structure"))
            .into_iter()
            .filter(|p| p.extension().is_some_and(|e| e == "nbt"))
            .count();
        assert_eq!(all, 62, "templates on disk");
        assert_eq!(
            FAMILIES.iter().map(|f| f.templates).sum::<usize>(),
            62,
            "templates the catalogue accounts for"
        );
    }

    #[test]
    fn every_carried_feature_names_a_carried_template() {
        let mut per_family: std::collections::BTreeMap<&str, std::collections::BTreeSet<String>> =
            std::collections::BTreeMap::new();
        for (path, text) in carried("configured_feature") {
            let kind = values_of(&text, "type");
            let family = family(kind[0]).expect("registered type");
            match family.config {
                Config::Location | Config::LocationAndRadius => {
                    let location = values_of(&text, "location");
                    let location = location
                        .first()
                        .unwrap_or_else(|| panic!("{path:?}: no location"));
                    let relative = location
                        .strip_prefix("yungsextras:")
                        .unwrap_or_else(|| panic!("{path:?}: {location} is not ours"));
                    let template = root()
                        .join("data/yungsextras/structure")
                        .join(format!("{relative}.nbt"));
                    assert!(template.is_file(), "{path:?}: missing {template:?}");
                    per_family
                        .entry(family.feature_type)
                        .or_default()
                        .insert(relative.to_owned());
                }
                Config::Computed => {
                    assert!(
                        values_of(&text, "location").is_empty(),
                        "{path:?}: a computed family names no location"
                    );
                }
            }
        }
        for family in &FAMILIES {
            let named = per_family
                .get(family.feature_type)
                .map_or(0, std::collections::BTreeSet::len);
            match family.config {
                Config::Location | Config::LocationAndRadius => assert_eq!(
                    named, family.templates,
                    "{}: templates named by its configured features",
                    family.feature_type
                ),
                Config::Computed => assert_eq!(
                    family.templates, 1,
                    "{}: a computed family places one template",
                    family.feature_type
                ),
            }
        }
    }

    #[test]
    fn every_carried_chain_ends_in_the_missing_modifier() {
        let placed = carried("placed_feature");
        assert_eq!(placed.len(), 62, "the jar's placed features");
        let carried_features: Vec<String> = carried("configured_feature")
            .into_iter()
            .map(|(path, _)| {
                path.file_stem()
                    .expect("stem")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        for (path, text) in &placed {
            let feature = values_of(text, "feature");
            let feature = feature
                .first()
                .unwrap_or_else(|| panic!("{path:?}: no feature"));
            let stem = feature.rsplit('/').next().expect("stem");
            assert!(
                carried_features.iter().any(|name| name == stem),
                "{path:?}: {feature} has no configured feature"
            );
            let kinds = values_of(text, "type");
            assert!(
                kinds.contains(&MISSING_PLACEMENT_MODIFIER),
                "{path:?}: the chain does not end in {MISSING_PLACEMENT_MODIFIER}"
            );
        }
    }

    #[test]
    fn shipped_arches_use_the_pins_template_feature() {
        let arches = shipped_arches();
        assert_eq!(arches.len(), 11, "shipped arch features");
        for (path, text) in &arches {
            let kind = values_of(text, "type");
            assert_eq!(
                kind.first().copied(),
                Some("minecraft:template"),
                "{path:?}"
            );
            let ours: Vec<&str> = values_of(text, "id")
                .into_iter()
                .filter(|id| id.starts_with("yungsextras:"))
                .collect();
            assert_eq!(ours.len(), 1, "{path:?}: one template id, got {ours:?}");
            let id = ours[0];
            assert!(id.starts_with("yungsextras:swamp/arch/swamp_arch_"), "{id}");
            let relative = id.strip_prefix("yungsextras:").expect("our namespace");
            assert!(
                root()
                    .join("data/yungsextras/structure")
                    .join(format!("{relative}.nbt"))
                    .is_file(),
                "{path:?}: {id} has no template"
            );
            assert_eq!(values_of(text, "rotations"), vec!["none"], "{path:?}");
            assert_eq!(
                values_of(text, "processor_type"),
                vec!["horizon:yung_pillar", "horizon:pick", "horizon:pick"],
                "{path:?}"
            );
        }
    }

    #[test]
    fn shipped_arches_keep_the_mods_placement_chain() {
        let arches = carried("placed_feature")
            .into_iter()
            .filter(|(p, _)| p.to_string_lossy().contains("swamp_arch_"))
            .filter(|(p, _)| !p.to_string_lossy().contains("double_arch"))
            .count();
        assert_eq!(arches, 11, "carried arch chains");
        let dir = root().join("data/yungsextras/worldgen/placed_feature/swamp/arches");
        let shipped = files_under(&dir);
        assert_eq!(shipped.len(), 11, "shipped arch chains");
        for path in shipped {
            let text = fs::read_to_string(&path).expect("placed feature reads");
            let kinds = values_of(&text, "type");
            assert_eq!(
                kinds,
                vec![
                    "minecraft:rarity_filter",
                    "minecraft:in_square",
                    "minecraft:heightmap",
                    "minecraft:biome",
                    "minecraft:offset"
                ],
                "{path:?}"
            );
            assert_eq!(values_of(&text, "chance"), vec!["2400"], "{path:?}");
            assert_eq!(values_of(&text, "y"), vec!["-1"], "{path:?}");
        }
    }

    #[test]
    fn shipped_modifier_lists_exactly_the_shipped_chains() {
        let path = root().join("data/yungsextras/neoforge/biome_modifier/swamp_arches.json");
        let text = fs::read_to_string(&path).expect("biome modifier reads");
        let ids = values_of(&text, "features");
        assert_eq!(ids.len(), 11, "eleven arch ids: {ids:?}");
        for id in ids {
            assert!(id.starts_with("yungsextras:swamp/arches/"), "{id}");
            let stem = id.rsplit('/').next().expect("stem");
            assert!(
                root()
                    .join("data/yungsextras/worldgen/placed_feature/swamp/arches")
                    .join(format!("{stem}.json"))
                    .is_file(),
                "{id} is not shipped"
            );
        }
        assert_eq!(
            values_of(&text, "step"),
            vec!["surface_structures"],
            "upstream's step"
        );
        assert_eq!(
            values_of(&text, "biomes"),
            vec!["#yungsextras:has_structure/swamp_structures"],
            "upstream's biome tag"
        );
    }

    #[test]
    fn no_shipped_file_names_a_type_the_pin_cannot_decode() {
        let mut checked = 0;
        let mut files = 0;
        for path in files_under(&root().join("data")) {
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }
            files += 1;
            let text = fs::read_to_string(&path).expect("json reads");
            for kind in values_of(&text, "type") {
                assert!(
                    type_is_decodable(kind),
                    "{path:?} names {kind}, which this pin cannot decode"
                );
                checked += 1;
            }
        }
        assert_eq!(files, 29, "json files under data/ (11 + 11 + 1 + 3 + 3)");
        assert!(checked >= 60, "only {checked} types checked");
    }

    #[test]
    fn shipped_features_use_26_3_flat_config() {
        // 26.3's configured feature is **flat**: `config` is unwrapped
        // (`horizon-port-pack/src/upgrade/features.rs`: "A configured feature in
        // 26.3's form: `config` unwrapped"), and the decoder reads the config's
        // keys at the root — `TemplateFeature::decode` reads `templates` and
        // `processors`, `FossilFeature::decode` `fossil_structures`. A file that
        // keeps the 1.21.1 wrapper loads but never decodes, and the game says so:
        // "11 placed features cannot run and will be skipped … invalid type:
        // null, expected a sequence".
        for (path, text) in shipped_arches() {
            assert!(
                !text.contains("\"config\""),
                "{path:?} keeps the pre-26.3 `config` wrapper"
            );
            assert!(
                text.contains("\"templates\""),
                "{path:?} has no `templates` at the root"
            );
        }
        let path = root().join("data/yungsextras/worldgen/placed_feature/swamp/arches");
        for (path, text) in files_under(&path).into_iter().map(|p| {
            let text = fs::read_to_string(&p).expect("placed feature reads");
            (p, text)
        }) {
            assert!(
                !text.contains("\"config\""),
                "{path:?} keeps a `config` wrapper"
            );
        }
    }

    #[test]
    fn shipped_features_wrap_the_template_in_a_weighted_entry() {
        // The pin's `WeightedList::decode` requires each entry as
        // `{"data": <payload>, "weight": n>` (`worldgen/weighted.rs:71`), and
        // `TemplateFeature::decode` reads `id`/`rotations` from that payload. A
        // bare `{"id": ...}` entry decodes as "invalid world generation field:
        // data" and the placed feature is skipped.
        for (path, text) in shipped_arches() {
            assert!(
                text.contains("\"data\": {"),
                "{path:?}: a weighted list entry needs a `data` object"
            );
            assert_eq!(values_of(&text, "weight"), vec!["1"], "{path:?}");
        }
    }

    #[test]
    fn blocked_families_name_their_missing_symbol() {
        let blocked = blocked();
        assert_eq!(blocked.len(), 10, "ten families carried only");
        for (feature_type, reason) in blocked {
            assert!(reason.contains(feature_type), "{reason}");
            assert!(reason.starts_with("blocked(no FeatureType"), "{reason}");
        }
        assert!(family("yungsextras:swamp_arch").is_some_and(|f| f.shipped));
        assert!(family("minecraft:template").is_none(), "not ours");
    }
}
