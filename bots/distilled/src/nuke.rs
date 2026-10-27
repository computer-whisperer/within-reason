//! The nuclear silos (`README.md` D-NUKE, D-NUKE-FIRE; the design `docs/design/2026-10-27-distilled-nukes.md`): a
//! later-game way to spend a bank nothing else spends. A silo is started when the bank holds its metal and the
//! energy covers its draw, as many as the economy keeps paying for; each fires at a place of his chosen at random by
//! worth, outside his known anti-nuke cover. Nothing tells a bot that a missile was intercepted, so a strike's
//! outcome is known only where the place is in our sight or radar; the rules run without it.

use std::collections::BTreeMap;

use bot_protocol::{Command, OwnUnit, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};

use crate::air::ENERGY_A_METAL;
use crate::economy::Job;
use crate::tier2::BANK_FULL;
use crate::{Distilled, MINUTE, SECOND};

/// A silo is told to fire at most once in this long: about 9 s from the order to the launch, then its 30 s reload.
const FIRE_GAP: i32 = 40 * SECOND;
/// A place struck is not struck again within the blast for this long: what stood there is unknown until seen again.
const STRIKE_REST: i32 = 5 * MINUTE;
/// The least worth (metal plus energy over `ENERGY_A_METAL`) a strike must be expected to kill.
const STRIKE_LEAST: f32 = 500.0;
/// Kept beyond his known anti-nuke's coverage, and beyond the blast from any unit of ours or our allies' (a missile
/// lands about 21 s after the order, and a tank walks a thousand in that time).
const COVER_MARGIN: f32 = 200.0;
const FRIENDLY_MARGIN: f32 = 400.0;
/// A strike's outcome is read this long after its launch: 12 s of flight, and the deaths after.
const OUTCOME_AFTER: i32 = 45 * SECOND;
/// With no launch seen this long after the order, the strike is reported as not launched.
const LAUNCH_WAIT: i32 = 30 * SECOND;

/// The silo's missile: the radius of its blast and the damage at the centre, falling to nothing at the edge (BAR's
/// warheads have an edge effectiveness of 0).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Warhead {
    pub radius: f32,
    pub damage: f32,
}

struct Strike {
    silo: UnitId,
    at: Vec3,
    ordered: i32,
    launched: Option<i32>,
    /// What the order expected: the worth and the number of his known buildings killed.
    worth: f32,
    buildings: usize,
    /// Kills and damage the events named the silo for since the launch, and the worth of the kills.
    seen_killed: usize,
    seen_worth: f32,
    seen_damaged: usize,
    reported: bool,
}

#[derive(Default)]
pub(crate) struct Nuke {
    pub warhead: Option<Warhead>,
    /// Each type's health, from the combat tables: what the warhead kills.
    health: BTreeMap<UnitDefId, f32>,
    /// Per silo, the frame of our last order to fire.
    ordered: BTreeMap<UnitId, i32>,
    strikes: Vec<Strike>,
    /// The frame until which no silo is started, after its site was refused (D-ORDERS).
    pub site_refused: i32,
    rng: u64,
}

/// A place worth striking: the centre, the worth killed, the buildings killed.
struct Place {
    at: Vec3,
    worth: f32,
    buildings: usize,
}

impl Distilled {
    /// The warhead of our faction's silo and every type's health, from the combat tables.
    pub(crate) fn init_nuke(&mut self, table: &combatsim::units::Units) {
        self.nuke.health = self.defs.values().filter_map(|d| table.index(&d.name).map(|i| (d.id, table.list[i].health))).collect();
        let silo = self.kit.silo.and_then(|d| self.def(d)).and_then(|d| table.index(&d.name));
        self.nuke.warhead = silo.and_then(|i| table.list[i].weapons.iter().find(|w| w.stockpile && w.command_fire)).map(|w| Warhead { radius: w.aoe / 2.0, damage: w.damage.get("default").copied().unwrap_or(0.0) });
        self.nuke.rng = 0x9e37_79b9_7f4a_7c15 ^ (self.hello.ai_id as u64).wrapping_mul(0x2545_f491_4f6c_dd1d);
    }

    /// D-NUKE's money: the bank holds a silo's metal, or `BANK_FULL` of a store smaller than that.
    pub(crate) fn silo_money(&self, tick: &Tick) -> bool {
        let Some(bill) = self.kit.silo.and_then(|d| self.def(d)).map(|d| d.metal_cost) else { return false };
        let metal = tick.snapshot.metal;
        self.options.nuke && self.frame >= self.nuke.site_refused && metal.current >= bill.min(BANK_FULL * metal.storage)
    }

    /// D-ENERGY's share for the silos: each silo's stockpile draw, frames counted, and one more while D-NUKE's money
    /// is there (the energy comes before the silo it is for).
    pub(crate) fn silo_draw(&self, tick: &Tick) -> f32 {
        let Some(def) = self.kit.silo.and_then(|d| self.def(d)) else { return 0.0 };
        let silos = self.count(self.kit.silo) + usize::from(self.silo_money(tick));
        def.stockpile_energy / def.stockpile_seconds.max(1.0) * silos as f32
    }

    /// D-NUKE: a builder that can build the silo starts one when the money is there, no project is under way, and the
    /// energy, with this silo's draw counted, is not short.
    pub(crate) fn silo_job(&self, builder: &UnitDefInfo, tick: &Tick) -> Option<Job> {
        let silo = self.kit.silo?;
        let wanted = builder.build_options.contains(&silo) && self.silo_money(tick) && !self.project_under_way() && !self.energy_short(tick);
        wanted.then_some(Job::Silo)
    }

    /// D-NUKE-FIRE, once a second: each silo with a missile ready and no order in the last `FIRE_GAP` fires at a place
    /// drawn at random by worth among those worth striking; the strikes' outcomes go to the log.
    pub(crate) fn run_nukes(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        self.report_strikes(tick);
        let Some(warhead) = self.nuke.warhead else { return };
        let ready: Vec<UnitId> = self.finished(self.kit.silo).filter(|s| s.stockpile > 0 && self.nuke.ordered.get(&s.id).is_none_or(|at| self.frame - at >= FIRE_GAP)).map(|s| s.id).collect();
        for silo in ready {
            let places = self.places(tick, warhead);
            let Some(place) = self.draw(&places) else { return };
            let (at, worth, buildings) = (places[place].at, places[place].worth, places[place].buildings);
            let cover = self.his_anti_nukes(tick).iter().map(|(p, _)| p.dist2d(at)).min_by(f32::total_cmp);
            let sensed = match &tick.snapshot.sensors {
                Some(s) if s.los.at(at.x, at.z) => "sight",
                Some(s) if s.radar.at(at.x, at.z) => "radar",
                Some(_) => "none",
                None => "unknown",
            };
            eprintln!(
                "[distilled ai={}] f={} nuke: silo {} fires at ({:.0},{:.0}) blast {:.0}: {buildings} of his buildings worth {worth:.0} expected, nearest known anti-nuke {}, the place in our {sensed}; {} places worth striking",
                self.hello.ai_id,
                self.frame,
                silo.0,
                at.x,
                at.z,
                warhead.radius,
                cover.map_or("none".to_string(), |d| format!("{d:.0} away")),
                places.len()
            );
            commands.push(Command::FireAt { unit: silo, at });
            self.nuke.ordered.insert(silo, self.frame);
            self.nuke.strikes.push(Strike { silo, at, ordered: self.frame, launched: None, worth, buildings, seen_killed: 0, seen_worth: 0.0, seen_damaged: 0, reported: false });
        }
    }

    /// His anti-nukes known: his buildings with an interceptor's coverage, and his mobile ones in sight.
    fn his_anti_nukes(&self, tick: &Tick) -> Vec<(Vec3, f32)> {
        let coverage = |def: Option<UnitDefId>| def.and_then(|d| self.def(d)).map_or(0.0, |d| d.coverage);
        let buildings = self.army.his_buildings.values().map(|b| (b.pos, coverage(b.def)));
        let mobile = tick.snapshot.enemies.iter().map(|e| (e.pos, coverage(e.def)));
        buildings.chain(mobile).filter(|(_, c)| *c > 0.0).collect()
    }

    /// The places worth striking: each of his known buildings as the blast's centre, with the worth the warhead would
    /// kill there, and none under his known anti-nuke cover, near ours or our allies', at our strike body's order, or
    /// struck lately.
    fn places(&self, tick: &Tick, warhead: Warhead) -> Vec<Place> {
        let buildings: Vec<(Vec3, f32, f32, Option<bot_protocol::Blast>)> = self
            .army
            .his_buildings
            .values()
            .filter_map(|b| {
                let def = self.def(b.def?)?;
                let health = self.nuke.health.get(&def.id).copied()?;
                Some((b.pos, health, def.metal_cost + def.energy_cost / ENERGY_A_METAL, def.death_blast))
            })
            .collect();
        let covers = self.his_anti_nukes(tick);
        // Ours and our allies' on the ground: a scout of ours is not worth a strike, and aircraft pass.
        let grounded = |def: UnitDefId| self.def(def).is_some_and(|d| !crate::air::flies(d));
        let ours = self.own.values().filter(|u| grounded(u.def) && !self.army.is_scout(u.id)).map(|u| u.pos);
        let allies = tick.snapshot.allies.iter().filter(|u| grounded(u.def)).map(|u| u.pos);
        let friends: Vec<Vec3> = ours.chain(allies).chain(self.army.strike_target().map(|(p, _)| p)).collect();
        let rest = |at: Vec3| self.nuke.strikes.iter().any(|s| self.frame - s.ordered < STRIKE_REST && s.at.dist2d(at) < warhead.radius);
        buildings
            .iter()
            .map(|(at, ..)| *at)
            .filter(|at| !covers.iter().any(|(p, c)| p.dist2d(*at) < c + COVER_MARGIN))
            .filter(|at| !friends.iter().any(|p| p.dist2d(*at) < warhead.radius + FRIENDLY_MARGIN))
            .filter(|at| !rest(*at))
            .map(|at| {
                let (worth, killed) = blast_kills(at, warhead, &buildings);
                Place { at, worth, buildings: killed }
            })
            .filter(|p| p.worth >= STRIKE_LEAST)
            .collect()
    }

    /// A place drawn at random, each as likely as its worth.
    fn draw(&mut self, places: &[Place]) -> Option<usize> {
        let total: f32 = places.iter().map(|p| p.worth).sum();
        if places.is_empty() || total <= 0.0 {
            return None;
        }
        // xorshift64*: a fixed sequence a seat, so a seed's games repeat.
        let rng = &mut self.nuke.rng;
        *rng ^= *rng >> 12;
        *rng ^= *rng << 25;
        *rng ^= *rng >> 27;
        let roll = (rng.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 11) as f32 / (1u64 << 53) as f32 * total;
        let mut sum = 0.0;
        places.iter().position(|p| {
            sum += p.worth;
            roll < sum
        }).or(Some(places.len() - 1))
    }

    /// One of our silos fired: its strike is launched.
    pub(crate) fn nuke_launched(&mut self, unit: UnitId) {
        if !self.own.get(&unit).is_some_and(|u| Some(u.def) == self.kit.silo) {
            return;
        }
        let Some(strike) = self.nuke.strikes.iter_mut().rev().find(|s| s.silo == unit && s.launched.is_none()) else { return };
        strike.launched = Some(self.frame);
        eprintln!("[distilled ai={}] f={} nuke: silo {} launched at ({:.0},{:.0}), {:.1} s after the order", self.hello.ai_id, self.frame, unit.0, strike.at.x, strike.at.z, (self.frame - strike.ordered) as f32 / SECOND as f32);
    }

    /// An enemy damaged or destroyed by one of our silos (the events name the attacker only where the enemy is in our
    /// sight or radar): it counts to that silo's last launched strike. Before `army.forget`.
    pub(crate) fn nuke_hit(&mut self, enemy: UnitId, attacker: Option<UnitId>, destroyed: bool) {
        let Some(silo) = attacker.filter(|a| self.own.get(a).is_some_and(|u| Some(u.def) == self.kit.silo)) else { return };
        let worth = self.army.his_buildings.get(&enemy).and_then(|b| b.def).and_then(|d| self.def(d)).map_or(0.0, |d| d.metal_cost + d.energy_cost / ENERGY_A_METAL);
        let Some(strike) = self.nuke.strikes.iter_mut().rev().find(|s| s.silo == silo && s.launched.is_some()) else { return };
        if destroyed {
            strike.seen_killed += 1;
            strike.seen_worth += worth;
        } else {
            strike.seen_damaged += 1;
        }
    }

    /// Each strike's outcome as the events showed it, once, `OUTCOME_AFTER` its launch (or `LAUNCH_WAIT` after an
    /// order that never launched); strikes older than `STRIKE_REST` are forgotten.
    fn report_strikes(&mut self, tick: &Tick) {
        let (ai, frame) = (self.hello.ai_id, self.frame);
        let silos: Vec<UnitId> = self.finished(self.kit.silo).map(|s| s.id).collect();
        for s in self.nuke.strikes.iter_mut().filter(|s| !s.reported) {
            let due = match s.launched {
                Some(at) => frame - at >= OUTCOME_AFTER,
                None => frame - s.ordered >= LAUNCH_WAIT,
            };
            if !due {
                continue;
            }
            s.reported = true;
            let sensed = match &tick.snapshot.sensors {
                Some(x) if x.los.at(s.at.x, s.at.z) => "sight",
                Some(x) if x.radar.at(s.at.x, s.at.z) => "radar",
                Some(_) => "none",
                None => "unknown",
            };
            if s.launched.is_none() {
                eprintln!("[distilled ai={ai}] f={frame} nuke: strike at ({:.0},{:.0}) by silo {} not launched (silo {})", s.at.x, s.at.z, s.silo.0, if silos.contains(&s.silo) { "standing" } else { "gone" });
                continue;
            }
            eprintln!(
                "[distilled ai={ai}] f={frame} nuke: strike at ({:.0},{:.0}) by silo {}: expected {} worth {:.0}, seen killed {} worth {:.0}, damaged {}; the place in our {sensed}",
                s.at.x, s.at.z, s.silo.0, s.buildings, s.worth, s.seen_killed, s.seen_worth, s.seen_damaged
            );
        }
        self.nuke.strikes.retain(|s| frame - s.ordered < STRIKE_REST || !s.reported);
    }

    /// The minute's `nuke:` line, once a silo has been begun.
    pub(crate) fn log_nukes(&self, tick: &Tick) {
        let Some(warhead) = self.nuke.warhead.filter(|_| self.count(self.kit.silo) > 0) else { return };
        let silos: Vec<&OwnUnit> = self.finished(self.kit.silo).collect();
        let (ready, queued): (i32, i32) = silos.iter().fold((0, 0), |(r, q), s| (r + s.stockpile, q + s.stockpile_queued));
        let places = self.places(tick, warhead).len();
        let known = self.army.his_buildings.len();
        eprintln!(
            "[distilled ai={}] f={} nuke: silos {} standing, {} frames; missiles {ready} ready, {queued} stockpiling; draw {:.0}/s; strikes {} in the last {} min; places worth striking {places} of {known} buildings known, his anti-nukes known {}",
            self.hello.ai_id,
            self.frame,
            silos.len(),
            self.count(self.kit.silo) - silos.len(),
            self.silo_draw(tick),
            self.nuke.strikes.len(),
            STRIKE_REST / MINUTE,
            self.his_anti_nukes(tick).len()
        );
    }
}

/// The worth and number of the buildings a warhead at `at` kills: each whose health is under the damage at its
/// distance, and then those a killed building's own death blast kills in turn (one step of the chain).
fn blast_kills(at: Vec3, warhead: Warhead, buildings: &[(Vec3, f32, f32, Option<bot_protocol::Blast>)]) -> (f32, usize) {
    let falloff = |damage: f32, radius: f32, d: f32| if d >= radius { 0.0 } else { damage * (1.0 - d / radius) };
    let mut killed: Vec<bool> = buildings.iter().map(|(p, health, ..)| falloff(warhead.damage, warhead.radius, p.dist2d(at)) > *health).collect();
    let first: Vec<usize> = (0..buildings.len()).filter(|i| killed[*i]).collect();
    for i in first {
        let Some(blast) = buildings[i].3 else { continue };
        for (j, (p, health, ..)) in buildings.iter().enumerate() {
            if !killed[j] && falloff(blast.damage, blast.radius, p.dist2d(buildings[i].0)) > *health {
                killed[j] = true;
            }
        }
    }
    buildings.iter().zip(&killed).filter(|(_, k)| **k).fold((0.0, 0), |(w, n), (b, _)| (w + b.2, n + 1))
}

#[cfg(test)]
mod tests {
    use super::{Warhead, blast_kills};
    use bot_protocol::{Blast, Vec3};

    fn at(x: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z: 0.0 }
    }

    /// Armada's warhead (640, 12,500 at the centre): a solar at 300 dies (6,600 against its 340), a fusion at 400 dies
    /// (4,700 against 3,800) and one at 500 stands (2,700), and a fusion killed takes what its own blast reaches.
    #[test]
    fn the_warhead_kills_by_health_at_distance_and_the_chain_one_step() {
        let warhead = Warhead { radius: 640.0, damage: 12500.0 };
        let fusion_blast = Some(Blast { radius: 400.0, damage: 5000.0 });
        let buildings = vec![(at(300.0), 340.0, 155.0, None), (at(400.0), 3800.0, 3600.0, fusion_blast), (at(500.0), 3800.0, 3600.0, None), (at(900.0), 340.0, 155.0, None)];
        // The solar at 300 and the fusion at 400 die; the fusion's blast reaches the fusion at 500 (100 away: 3,750
        // against 3,800, it stands) and the solar at 900 (500 away, beyond its 400).
        assert_eq!(blast_kills(at(0.0), warhead, &buildings), (155.0 + 3600.0, 2));
        let stronger = vec![(at(300.0), 340.0, 155.0, None), (at(400.0), 3800.0, 3600.0, Some(Blast { radius: 600.0, damage: 9000.0 })), (at(900.0), 340.0, 155.0, None)];
        // A bigger blast takes the solar at 900 (500 from the fusion: 1,500 against 340).
        assert_eq!(blast_kills(at(0.0), warhead, &stronger), (155.0 + 3600.0 + 155.0, 3));
    }
}
