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
    /// What `raider` and `line` name at the vehicle plant, which builds neither of the lab's.
    vehicle_raider: &'static str,
    vehicle_line: &'static str,
    /// Raises wrecks and takes them apart (`reclaim.rs`).
    resurrector: &'static str,
    /// The rest of what a role word names (`ROLE_WORDS`): the generators, the air plant, the radar, the
    /// construction turret, the storages.
    solar: &'static str,
    wind: &'static str,
    air_plant: &'static str,
    radar: &'static str,
    nano: &'static str,
    metal_storage: &'static str,
    energy_storage: &'static str,
}

/// The role words a list step or a `produce` entry may use in place of a unit's internal name, resolved against
/// the seat's faction when the step or entry is used (human-10, 2026-10-02: on a lobby whose factions are set at
/// the start, the player could write no pre-game lists because `queue` and `produce` took internal names only,
/// and the hands played prose for the first twenty seconds).
/// `extractor` is not among them: it is a list step of its own (`lists.rs`: a spot, or the nearest free one), and
/// player-43 lost in its first minute when it was turned into `armmex spot_67`, which the list reader takes for a
/// tier-2 upgrade over a standing extractor.
pub const ROLE_WORDS: [&str; 19] = [
    "solar", "wind", "lab", "plant", "air_plant", "turret", "radar", "nano", "constructor", "vehicle_constructor", "raider", "line", "rez", "converter", "advanced_lab", "advanced_constructor", "advanced_extractor", "metal_storage", "energy_storage",
];
/// The role words that name a factory (the `queue` tool's note on a factory step without an id).
pub const FACTORY_ROLES: [&str; 4] = ["lab", "plant", "air_plant", "advanced_lab"];

impl Roster {
    /// The unit a role word names in this faction; None for anything else, a unit's internal name included.
    pub fn role(&self, word: &str) -> Option<&'static str> {
        Some(match word {
            "solar" => self.solar,
            "wind" => self.wind,
            "lab" => self.lab,
            "plant" => self.plant,
            "air_plant" => self.air_plant,
            "turret" => self.turret,
            "radar" => self.radar,
            "nano" => self.nano,
            "constructor" => self.constructor,
            "vehicle_constructor" => self.vehicle_constructor,
            "raider" => self.raider,
            "line" => self.line,
            "rez" => self.resurrector,
            "converter" => self.converter,
            "advanced_lab" => self.advanced_lab,
            "advanced_constructor" => self.advanced_constructor,
            "advanced_extractor" => self.advanced_extractor,
            "metal_storage" => self.metal_storage,
            "energy_storage" => self.energy_storage,
            _ => return None,
        })
    }

    /// A `produce` entry for the vehicle plant: `raider`, `line` and `constructor` are the plant's own there (the
    /// lab's cannot be built at it: sonnet-low-1, `raider:5` in the list before the game was `armpw:5` at an Armada
    /// vehicle plant, taken without a word, and the plant stood idle from 1:58 to 2:11).
    pub fn resolve_words_at_the_plant(&self, entry: &str) -> String {
        let (head, rest) = entry.split_at(entry.find([' ', ':']).unwrap_or(entry.len()));
        match head {
            "raider" => format!("{}{rest}", self.vehicle_raider),
            "line" => format!("{}{rest}", self.vehicle_line),
            "constructor" => format!("{}{rest}", self.vehicle_constructor),
            _ => self.resolve_words(entry),
        }
    }

    /// A list step or a `produce` entry with its role word, if it has one, replaced by the unit's internal name:
    /// "solar" to "armsolar", "turret spot_3" to "armllt spot_3", "constructor:1" to "armck:1".
    pub fn resolve_words(&self, entry: &str) -> String {
        let (head, rest) = entry.split_at(entry.find([' ', ':']).unwrap_or(entry.len()));
        match self.role(head) {
            Some(name) => format!("{name}{rest}"),
            None => entry.to_string(),
        }
    }
}

pub static ROSTERS: [Roster; 2] = [
    Roster {
        commander: "armcom", extractor: "armmex",
        converter: "armmakr", lab: "armlab", turret: "armllt", constructor: "armck",
        plant: "armvp", vehicle_constructor: "armcv",
        advanced_lab: "armalab", advanced_constructor: "armack", advanced_extractor: "armmoho",
        raider: "armpw", line: "armham", vehicle_raider: "armflash", vehicle_line: "armstump", resurrector: "armrectr",
        solar: "armsolar", wind: "armwin", air_plant: "armap", radar: "armrad", nano: "armnanotc", metal_storage: "armmstor", energy_storage: "armestor",
    },
    Roster {
        commander: "corcom", extractor: "cormex",
        converter: "cormakr", lab: "corlab", turret: "corllt", constructor: "corck",
        plant: "corvp", vehicle_constructor: "corcv",
        advanced_lab: "coralab", advanced_constructor: "corack", advanced_extractor: "cormoho",
        raider: "corak", line: "corthud", vehicle_raider: "corgator", vehicle_line: "corraid", resurrector: "cornecro",
        solar: "corsolar", wind: "corwin", air_plant: "corap", radar: "corrad", nano: "cornanotc", metal_storage: "cormstor", energy_storage: "corestor",
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
    /// The roster this kit was resolved from, for the role words (`Roster::role`).
    pub roster: &'static Roster,
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
    pub fn resolve(&'static self, world: &World) -> Result<Kit, &'static str> {
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
            roster: self,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A role word names the faction's unit; a step or an entry keeps what follows the word; an internal name or
    /// anything else is left as it is.
    #[test]
    fn a_role_word_is_the_factions_unit() {
        let (arm, cor) = (&ROSTERS[0], &ROSTERS[1]);
        for word in ROLE_WORDS {
            assert!(arm.role(word).is_some_and(|n| n.starts_with("arm")) && cor.role(word).is_some_and(|n| n.starts_with("cor")), "{word}");
        }
        assert_eq!(arm.resolve_words("solar"), "armsolar");
        assert_eq!(cor.resolve_words("turret spot_3"), "corllt spot_3");
        assert_eq!(cor.resolve_words("constructor:1"), "corck:1");
        assert_eq!(arm.resolve_words("lab #l1"), "armlab #l1");
        assert_eq!(arm.resolve_words("corsolar"), "corsolar");
        assert_eq!(arm.resolve_words_at_the_plant("raider:5"), "armflash:5");
        assert_eq!(cor.resolve_words_at_the_plant("constructor:1"), "corcv:1");
        assert_eq!(arm.resolve_words_at_the_plant("line"), "armstump");
        assert_eq!(arm.resolve_words_at_the_plant("rez"), "armrectr");
        // The list's own words are not touched: `extractor` is a step of its own, not a unit.
        for step in ["extractor spot_4", "extractor", "extractor nearest", "assist 40", "assist", "stop", "reclaim constructor_3"] {
            assert_eq!(arm.resolve_words(step), step, "{step}");
        }
        assert!(FACTORY_ROLES.iter().all(|w| ROLE_WORDS.contains(w)));
    }
}
