//! Which unit fills which role, per faction.

use bot_protocol::UnitDefId;

use crate::world::World;

/// Unit names for one faction.
pub struct Roster {
    pub commander: &'static str,
    extractor: &'static str,
    converter: &'static str,
    lab: &'static str,
    turret: &'static str,
    constructor: &'static str,
    /// The vehicle plant and its constructor: the factory for flat open maps (K-maps-factory-by-terrain).
    plant: &'static str,
    vehicle_constructor: &'static str,
    /// Tier 2 (docs/knowledge/tier2.md): the advanced bot lab, its constructor, the extractor that one builds over a
    /// tier-1 extractor.
    advanced_lab: &'static str,
    advanced_constructor: &'static str,
    advanced_extractor: &'static str,
    raider: &'static str,
    /// The unit the line is made of (K-units-duel-*).
    line: &'static str,
    /// Raises wrecks and takes them apart (`reclaim.rs`).
    resurrector: &'static str,
}

pub const ROSTERS: [Roster; 2] = [
    Roster {
        commander: "armcom", extractor: "armmex",
        converter: "armmakr", lab: "armlab", turret: "armllt", constructor: "armck",
        plant: "armvp", vehicle_constructor: "armcv",
        advanced_lab: "armalab", advanced_constructor: "armack", advanced_extractor: "armmoho",
        raider: "armpw", line: "armham", resurrector: "armrectr",
    },
    Roster {
        commander: "corcom", extractor: "cormex",
        converter: "cormakr", lab: "corlab", turret: "corllt", constructor: "corck",
        plant: "corvp", vehicle_constructor: "corcv",
        advanced_lab: "coralab", advanced_constructor: "corack", advanced_extractor: "cormoho",
        raider: "corak", line: "corthud", resurrector: "cornecro",
    },
];

/// What a builder is offered when the player has whitelisted nothing for it (docs/design/2026-09-22-full-roster.md,
/// decision 5): the Kit's buildings and the usual mid-game ones, cut to what the builder can build. The rest of its
/// build list is reached through `produce`, and always by the policy. Extractors are the `extractor` option.
// No energy converters (armmakr, cormakr, armmmkr, cormmkr): the hands built them unasked whenever energy banked
// (bluegecko-3v1-comet-catcher-8, 5:52), and the OS-48 players say a metal-heavy map has no use for them; the
// player names one in a `queue` or `produce` list when it wants them.
const USUAL_ARMADA: &[&str] = &[
    "armsolar", "armwin", "armtide", "armfrad", "armtl", "armadvsol", "armlab", "armvp", "armap", "armhp", "armsy", "armllt", "armhlt", "armrl", "armrad", "armarad", "armnanotc",
    "armmstor", "armestor", "armalab", "armavp", "armaap", "armmoho", "armfus", "armguard", "armflak", "armdl",
    // Tier 2 (the advanced constructors' lists): advanced fusion and geothermal, the advanced storages, the jammer,
    // the tier-2 defences, the anti-nuke, the targeting facility.
    "armafus", "armageo", "armuwadvms", "armuwadves", "armveil", "armpb", "armanni", "armamb", "armamd", "armtarg",
];
const USUAL_CORTEX: &[&str] = &[
    "corsolar", "corwin", "cortide", "corfrad", "cortl", "coradvsol", "corlab", "corvp", "corap", "corhp", "corsy", "corllt", "corhlt", "corrl", "corrad", "corarad", "cornanotc",
    "cormstor", "corestor", "coralab", "coravp", "coraap", "cormoho", "corfus", "corpun", "corflak", "cordl",
    "corafus", "corageo", "coruwadvms", "coruwadves", "corshroud", "corvipe", "cordoom", "cortoast", "corfmd", "cortarg",
];

/// The usual list for a builder of the faction its name belongs to.
pub fn usual_menu(builder_name: &str) -> &'static [&'static str] {
    match World::faction_of(builder_name) {
        "cortex" => USUAL_CORTEX,
        _ => USUAL_ARMADA,
    }
}

/// A roster resolved against the running game's unit definitions.
#[derive(Clone, Copy)]
pub struct Kit {
    pub commander: UnitDefId,
    pub extractor: UnitDefId,
    pub converter: UnitDefId,
    pub lab: UnitDefId,
    pub turret: UnitDefId,
    pub constructor: UnitDefId,
    /// The vehicle plant and the constructor it makes.
    pub plant: UnitDefId,
    pub vehicle_constructor: UnitDefId,
    pub advanced_lab: UnitDefId,
    pub advanced_constructor: UnitDefId,
    pub advanced_extractor: UnitDefId,
    pub raider: UnitDefId,
    pub line: UnitDefId,
    pub resurrector: UnitDefId,
}

impl Kit {
    /// Either tier: what holds a metal spot for us.
    pub fn is_extractor(&self, def: UnitDefId) -> bool {
        def == self.extractor || def == self.advanced_extractor
    }

    /// Any factory: the bot lab, the vehicle plant, the advanced bot lab.
    pub fn is_factory(&self, def: UnitDefId) -> bool {
        def == self.lab || def == self.plant || def == self.advanced_lab
    }

}

impl Roster {
    /// `Err` names the first unit this game does not define.
    pub fn resolve(&self, world: &World) -> Result<Kit, &'static str> {
        let id = |name: &'static str| world.def_named(name).ok_or(name);
        Ok(Kit {
            commander: id(self.commander)?,
            extractor: id(self.extractor)?,
            converter: id(self.converter)?,
            lab: id(self.lab)?,
            turret: id(self.turret)?,
            constructor: id(self.constructor)?,
            plant: id(self.plant)?,
            vehicle_constructor: id(self.vehicle_constructor)?,
            advanced_lab: id(self.advanced_lab)?,
            advanced_constructor: id(self.advanced_constructor)?,
            advanced_extractor: id(self.advanced_extractor)?,
            raider: id(self.raider)?,
            line: id(self.line)?,
            resurrector: id(self.resurrector)?,
        })
    }
}
