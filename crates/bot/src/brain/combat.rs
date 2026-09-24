//! Combat prediction: who wins if these two forces fight it out, from metal value and the duel table
//! (`docs/harness/duels.md`). Checked against 273 decisive engagements of recorded games
//! (`run/predict_check.py`): the side given the higher power lost the smaller share of its force in 89 % of them, 94 %
//! when the ratio was beyond 2:1. Plain metal value does nearly as well (88 %); the matchup weighting matters when
//! the two sides field different kinds of unit. What it cannot know is what we have not seen.

use std::collections::HashMap;

use bot_protocol::UnitDefId;

use super::Brain;

/// A turret is worth more in a fight than its metal: it is tough for its cost and does not need to walk anywhere.
/// Over the decisive engagements of v18 the prediction improved steadily as this rose (right 92 % at 1.5, 93 % at
/// 2.5, 95 % at 4; correlation 0.86 to 0.91), and waves that passed the gate at 1.5 were wiped 5:1 by turret lines.
pub const TURRET_WORTH: f32 = 3.0;

/// `margin[(unit, against)]` at equal metal, -1..1, by unit name.
pub struct Matchups(HashMap<(String, String), f32>);

impl Default for Matchups {
    fn default() -> Self {
        let rows = include_str!("../../data/matchups.csv").lines().filter(|l| !l.starts_with('#'));
        Matchups(
            rows.filter_map(|line| {
                let mut fields = line.split(',');
                Some(((fields.next()?.to_string(), fields.next()?.to_string()), fields.next()?.parse().ok()?))
            })
            .collect(),
        )
    }
}

impl Matchups {
    /// What one metal of `unit` is worth against one metal of `against`: the Lanchester square law run backwards
    /// from the duel margin (equal forces, winner keeps a share m of its value: effectiveness ratio 1 / (1 - m^2)).
    fn effectiveness(&self, unit: &str, against: &str) -> f32 {
        let margin = self.0.get(&(unit.to_string(), against.to_string())).copied().unwrap_or(0.0).clamp(-0.95, 0.95);
        if margin >= 0.0 { 1.0 / (1.0 - margin * margin) } else { 1.0 - margin * margin }
    }
}

/// A force: how many of each mobile unit type, and the metal of the turrets standing with it.
#[derive(Default, Clone)]
pub struct Force {
    pub units: HashMap<UnitDefId, usize>,
    /// Metal of the armed buildings standing with it that can hit ground (hover and ship count as ground).
    pub turret_metal: f32,
    /// Metal of those that can hit aircraft (docs/design/2026-09-22-domains.md, decision 6).
    pub turret_metal_air: f32,
    /// Radar contacts: something mobile is there, of a type we cannot see.
    pub unidentified: usize,
}

/// What one unidentified radar contact is taken to be worth (tier-1 soldiers run 43-270, most 110-140).
const UNIDENTIFIED_METAL: f32 = 110.0;

impl Force {
    pub fn add(&mut self, def: UnitDefId) {
        *self.units.entry(def).or_default() += 1;
    }
}

impl Brain {
    /// Whether every mobile unit of a force flies: what its opponent must be able to hit.
    pub(super) fn all_air(&self, force: &Force) -> bool {
        !force.units.is_empty() && force.units.keys().all(|def| self.world.domain_of(*def) == crate::world::Domain::Air)
    }

    /// Whether a unit of this type has a weapon for aircraft, or for the ground: from the simulator's table; a type
    /// the table lacks is taken to hit ground only.
    pub(super) fn can_hit(&self, def: UnitDefId, air: bool) -> bool {
        match self.contacts.sim_defs.get(&def).map(|i| &self.contacts.rules.units.list[*i]) {
            Some(unit) => if air { unit.reach_air() > 0.0 } else { unit.reach() > 0.0 },
            None => !air && self.world.def(def).is_some_and(|d| d.weapon_count > 0),
        }
    }

    /// Whether anything in a force can hit `air` (or ground) at all: soldiers or turrets.
    pub(super) fn force_can_hit(&self, force: &Force, air: bool) -> bool {
        force.units.keys().any(|def| self.can_hit(*def, air)) || (if air { force.turret_metal_air } else { force.turret_metal }) > 0.0 || force.unidentified > 0
    }

    /// The square-law strength per metal of a tier-1 soldier, `sqrt(damage a second x health) / metal`, the median
    /// over the simulator's table: what a builder's fighting worth is measured against. Computed once.
    fn worth_scale(&self) -> f32 {
        let cached = self.worth_scale.get();
        if cached > 0.0 {
            return cached;
        }
        let scale = tier1_scale(&self.contacts.rules.units.list);
        self.worth_scale.set(scale);
        scale
    }

    /// What a unit that builds (the commander, a constructor) is worth in a fight, in the metal of tier-1 soldiers
    /// of the same square-law strength (H-HANDS-COMMANDER-WORTH): sqrt(damage a second x health) over the tier-1
    /// scale. A 2700-metal Armada commander comes to about 400: the fighter it is, not the base it can build. A
    /// constructor without a weapon is worth nothing (wake-4: a commander walked into two Stouts and three Warriors
    /// under "we outweigh it heavily", its metal against theirs).
    pub(super) fn fighting_worth(&self, def: UnitDefId) -> f32 {
        match self.contacts.sim_defs.get(&def).map(|i| &self.contacts.rules.units.list[*i]) {
            Some(unit) => worth_of(unit, self.worth_scale()),
            None => {
                let strength = super::pianist::glossary::entry(self.name(def)).map_or(0.0, |e| e.dps.unwrap_or(0.0) * e.health);
                if strength <= 0.0 { 0.0 } else { strength.sqrt() / self.worth_scale() }
            }
        }
    }

    /// A unit's weight in a force: its metal for a soldier, its fighting worth for a builder.
    fn weight(&self, def: UnitDefId) -> f32 {
        if self.world.def(def).is_some_and(|d| d.build_speed > 0.0) { self.fighting_worth(def) } else { self.world.def(def).map_or(0.0, |d| d.metal_cost) }
    }

    /// The reach of a type's D-gun (its `command_fire` weapon); 0 for anything but a commander.
    pub(super) fn dgun_reach(&self, def: UnitDefId) -> f32 {
        self.contacts
            .sim_defs
            .get(&def)
            .map(|i| &self.contacts.rules.units.list[*i])
            .map_or(0.0, |u| u.weapons.iter().filter(|w| w.command_fire && !w.paralyzer).map(|w| w.range).fold(0.0, f32::max))
    }

    /// Fighting power of `force` against `other`, in metal-equivalents: only what can hit the other's domain counts.
    /// A soldier weighs its metal; a builder its fighting worth (`fighting_worth`).
    fn power(&self, force: &Force, other: &Force) -> f32 {
        let metal = |def: UnitDefId| self.weight(def);
        let other_air = self.all_air(other);
        let other_total: f32 = other.units.iter().map(|(def, n)| metal(*def) * *n as f32).sum();
        let soldiers: f32 = force
            .units
            .iter()
            .filter(|(def, _)| self.can_hit(**def, other_air))
            .map(|(def, n)| {
                let effectiveness = if other_total > 0.0 {
                    other.units.iter().map(|(against, k)| metal(*against) * *k as f32 / other_total * self.matchups.effectiveness(self.name(*def), self.name(*against))).sum()
                } else {
                    1.0
                };
                metal(*def) * *n as f32 * effectiveness.sqrt()
            })
            .sum();
        // A radar contact is counted as an average tier-1 soldier at face value: ignoring it made a column of
        // twenty blips weigh nothing.
        let turrets = if other_air { force.turret_metal_air } else { force.turret_metal };
        soldiers + TURRET_WORTH * turrets + UNIDENTIFIED_METAL * force.unidentified as f32
    }

    /// Our power over theirs if `ours` fights `theirs`: above 1 we should win, and by the square law a ratio r leaves
    /// the winner about sqrt(1 - 1/r^2) of its force.
    pub(super) fn odds(&self, ours: &Force, theirs: &Force) -> f32 {
        self.power(ours, theirs) / self.power(theirs, ours).max(1.0)
    }
}

/// The median square-law strength per metal, `sqrt(damage a second x health) / metal`, over the tier-1 mobile ground
/// fighters of the simulator's table: what a builder's fighting worth is measured against (2.0 for an empty table).
fn tier1_scale(units: &[combatsim::units::Unit]) -> f32 {
    let mut ratios: Vec<f32> = units
        .iter()
        .filter(|u| u.mobile() && !u.builder && !u.air && u.tech == 1 && u.reach() > 0.0 && u.metal > 0.0 && u.dps() > 0.0)
        .map(|u| (u.dps() * u.health).sqrt() / u.metal)
        .collect();
    ratios.sort_by(f32::total_cmp);
    if ratios.is_empty() { 2.0 } else { ratios[ratios.len() / 2] }
}

/// A unit's fighting worth in tier-1 metal: its square-law strength over the scale; 0 for a unit that cannot fight.
fn worth_of(unit: &combatsim::units::Unit, scale: f32) -> f32 {
    let strength = unit.dps() * unit.health;
    if strength <= 0.0 { 0.0 } else { strength.sqrt() / scale }
}

#[cfg(test)]
mod tests {
    use super::*;
    use combatsim::units::Units;

    fn unit<'a>(units: &'a Units, name: &str) -> &'a combatsim::units::Unit {
        let i = units.names.iter().position(|n| n == name).expect(name);
        &units.list[i]
    }

    /// The commander weighs as the fighter it is: about two Stouts, not twelve (wake-4). Five Pawns still read as
    /// prey; two Stouts and three Warriors as death.
    #[test]
    fn commander_worth_is_a_fighters() {
        let units = Units::default();
        let scale = tier1_scale(&units.list);
        assert!(scale > 1.0 && scale < 4.0, "scale {scale}");
        let com = worth_of(unit(&units, "armcom"), scale);
        let stout = unit(&units, "armstump").metal;
        assert!(com > stout && com < 3.0 * stout, "commander {com} against a Stout's {stout}");
        let pawns = 5.0 * unit(&units, "armpw").metal;
        let line = 2.0 * stout + 3.0 * unit(&units, "armwar").metal;
        assert!(com / pawns >= 1.3, "five Pawns: {}", com / pawns);
        assert!(com / line < 0.8, "two Stouts and three Warriors: {}", com / line);
        assert_eq!(worth_of(unit(&units, "armck"), scale), 0.0, "an unarmed constructor fights nothing");
    }
}

