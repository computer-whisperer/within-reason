//! The air (`docs/design/2026-10-25-distilled-air.md`; `README.md` D-AIR-SEEN to D-BOMB): his aircraft as shown, an
//! aircraft plant from 30 metal a second of income, fighters to the air shown with build power and energy for them,
//! Nettles as the bridge until the fighters are up, and a squadron of bombers at his energy fields. Rebuilt
//! 2026-10-25 from D-ANTI-AIR (four Nettles by the plants once any aircraft had been seen) after the user's eight-seat
//! game, where 23 Wasps and 5 Dragons came at 27:00, on the user's game knowledge (`knowledge/army.md`
//! K-army-air-plant-by-30-income, K-army-fighters-cheapest-anti-air, K-army-bombers-chain-energy-fields).

use std::collections::{BTreeMap, BTreeSet};

use bot_protocol::{Command, OwnUnit, Resource, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};

use crate::army::SEEN_FOR;
use crate::economy::Job;
use crate::{Distilled, MINUTE, SECOND};

/// D-AIR-PLANT: the aircraft plant from this metal income (the user, 2026-10-25: "by 30 metal/s an air lab is
/// considered obligatory"), once this many vehicle plants stand: the bot reaches 30 at 6-7 min on one plant, and
/// there the aircraft plant took the second's place (begun 6.1-7.7 min, the second vehicle plant at a median 11
/// against 7.6, 41 soldiers made by 10:00 against 50, 1,750 metal banked, four losses of eight to BARb
/// hard_aggressive where there had been none: dist28-airfit-barb against dist19).
const AIR_PLANT_FROM: f32 = 30.0;
const AIR_PLANT_AFTER: usize = 2;
/// D-FIGHTERS: fighters at the least; twice his fighters, by strength (√(damage to aircraft × health) against ours:
/// by Lanchester's square law twice his number keeps three quarters of ours, where an even number lost five Falcons of
/// eight to four Valiants escorting seven Whirlwinds, dist20-airsmoke-glitters game 01); and enough besides to kill
/// the rest of the air shown within `KILL_WITHIN` seconds; never more than `FIGHTERS_AT_MOST`.
const FIGHTERS_LEAST: usize = 4;
const FIGHTERS_TO_HIS: f32 = 2.0;
const KILL_WITHIN: f32 = 10.0;
const FIGHTERS_AT_MOST: usize = 100;
/// His aircraft in sight on our half within this of a building of ours are fought at; the fighters are sent again
/// when the place moves this far or this long has passed.
const AIR_DEFEND: f32 = 1500.0;
const REFIGHT: f32 = 200.0;
const REFIGHT_AFTER: i32 = 3 * SECOND;
/// An aircraft of ours this far from the air station goes back to it when idle; bombers circle, so a squadron within
/// `STATION_WAIT` of it is there for a sortie.
const STATION_NEAR: f32 = 400.0;
const STATION_WAIT: f32 = 700.0;
/// The fighters keep this much more than his anti-air's reach clear.
const AA_CLEAR: f32 = 150.0;
/// D-ANTI-AIR: Nettles at the most, and builders on them at once; one stands within this of a plant to count as its.
const AA_AT_MOST: usize = 8;
const AA_BUILDERS: usize = 2;
const AA_NEAR: f32 = 400.0;
/// The weight of a turret against the air shown, as T-AIR and D-WEIGHT count it: three times its metal.
const AA_WEIGHT: f32 = 3.0;
/// D-BOMB: the squadron, and a sortie no sooner than this after the last.
const BOMBERS: usize = 4;
const SORTIE_GAP: i32 = MINUTE;
/// His economy buildings within this of a point are one field; a field of fewer than `FIELD_LEAST` is not bombed. A
/// building's worth is its metal and its energy at the converter's rate (70 energy a metal, K-mech-converter-rates).
const FIELD: f32 = 200.0;
const FIELD_LEAST: usize = 3;
pub(crate) const ENERGY_A_METAL: f32 = 70.0;
/// A bomber that has been within `PASS` of its target for `PASS_FOR` has dropped and is called back; a sortie is
/// called back whole once it has lasted its flight time and `SORTIE_SLACK`; its kills are counted for `SETTLE` after
/// the last bomber is called back (the chain takes seconds).
const PASS: f32 = 500.0;
const PASS_FOR: i32 = 4 * SECOND;
const SORTIE_SLACK: i32 = 20 * SECOND;
const SETTLE: i32 = 10 * SECOND;
/// A sortie that loses half its bombers stops the bombing, and the making of bombers, this long. After `PROVING`
/// sorties the squadron is made up again only while what its sorties were seen to destroy is worth what they lost
/// (dist28-airfit-barb: 58 bombers made in eight games, 33 lost for 43 buildings, four games losing more than they
/// took).
const LOSS_PAUSE: i32 = 5 * MINUTE;
const PROVING: usize = 2;
/// Aircraft that answer nothing yet (the least fighters before he flies, the bombers) are made only with the energy
/// store over this share: a fighter is 40 energy a metal, and the aircraft plant on them drew 124 a second, 40 % of the
/// energy income at 8:00 (dist28-airfit-barb: the store at 856 against 1,748 at 8:00).
const SPARE_ENERGY: f32 = 0.5;
/// A sortie goes only where his known anti-air would do the squadron no more than this many bombers' health, on the
/// straight line there and back and over the target for `PASS_FOR`: one Thistle beside the field (about 690) is taken,
/// two are not (dist21-airsmoke2-glitters: with every field within reach of one of his turrets refused, game 00 flew
/// no sortie in 30 minutes and game 01 one).
const SORTIE_RISK: f32 = 1.5;
/// The engine's move states: hold position never attacks on its own; manoeuvre attacks within 1,000.
const HOLD_POSITION: i32 = 0;
const MANOEUVRE: i32 = 1;

/// What the combat tables say a type does to aircraft.
#[derive(Clone, Copy, Default)]
pub(crate) struct AirNumbers {
    /// Damage to aircraft a second, and the reach of it.
    dps: f32,
    reach: f32,
    /// Every weapon hits aircraft only: a fighter.
    fighter: bool,
    health: f32,
}

struct Sortie {
    target: Vec3,
    began: i32,
    /// The flight there, in frames.
    flight: i32,
    bombers: BTreeSet<UnitId>,
    /// Per bomber, since when it has been within `PASS` of the target.
    near_since: BTreeMap<UnitId, i32>,
    recalled: BTreeSet<UnitId>,
    /// When the last bomber was called back.
    over: Option<i32>,
    /// His economy buildings seen destroyed within the field, and their worth.
    killed: usize,
    worth: f32,
}

#[derive(Default)]
pub(crate) struct Air {
    pub numbers: BTreeMap<UnitDefId, AirNumbers>,
    /// D-AIR-SEEN: his armed aircraft as last seen: the type and the frame.
    pub shown: BTreeMap<UnitId, (UnitDefId, i32)>,
    /// The frame until which no aircraft plant is asked for, after one found no site (D-ORDERS).
    pub plant_refused: i32,
    /// Our aircraft told their move state.
    told: BTreeSet<UnitId>,
    /// Where the fighters were last sent to fight, and when.
    fight_order: Option<(Vec3, i32)>,
    sortie: Option<Sortie>,
    last_sortie: i32,
    paused_until: i32,
    /// Sorties flown, bombers lost on them, his economy buildings destroyed and their worth.
    flown: usize,
    lost: usize,
    killed: usize,
    worth: f32,
}

impl Distilled {
    /// At the start: what every type does to aircraft, from the combat tables. A bomb's category admits aircraft but
    /// it only falls (`CBombDropper::TestRange`), so a bomber's figure here is not real; nothing below reads one.
    pub(crate) fn init_air(&mut self, table: &combatsim::units::Units) {
        for def in self.defs.values() {
            let Some(i) = table.index(&def.name) else { continue };
            let unit = &table.list[i];
            let fighter = !unit.weapons.is_empty() && unit.weapons.iter().all(|w| w.only_targets == "VTOL");
            self.air.numbers.insert(def.id, AirNumbers { dps: unit.dps_air(), reach: unit.reach_air(), fighter, health: unit.health });
        }
    }

    fn numbers(&self, def: Option<UnitDefId>) -> AirNumbers {
        def.and_then(|d| self.air.numbers.get(&d)).copied().unwrap_or_default()
    }

    /// D-AIR-SEEN: his armed aircraft in sight join the air shown; one unseen for `SEEN_FOR` leaves it, as one seen
    /// destroyed does (`lib.rs`).
    fn observe_air(&mut self, tick: &Tick) {
        for e in &tick.snapshot.enemies {
            let Some(def) = e.def else { continue };
            if self.def(def).is_some_and(|d| flies(d) && d.weapon_count > 0) {
                self.air.shown.insert(e.id, (def, self.frame));
            }
        }
        let frame = self.frame;
        self.air.shown.retain(|_, (_, seen)| frame - *seen < SEEN_FOR);
    }

    /// The air shown: its metal and its health.
    pub(crate) fn air_shown(&self) -> (f32, f32) {
        self.air.shown.values().fold((0.0, 0.0), |(metal, health), (def, _)| (metal + self.def(*def).map_or(0.0, |d| d.metal_cost), health + self.numbers(Some(*def)).health))
    }

    /// What his fighters shown do to aircraft a second.
    fn his_fighters(&self) -> f32 {
        self.air.shown.values().map(|(def, _)| self.numbers(Some(*def))).filter(|n| n.fighter).fold(0.0, |sum, n| sum + n.dps)
    }

    /// D-FIGHTERS: the fighters wanted: `FIGHTERS_LEAST`, `FIGHTERS_TO_HIS` times his fighters in ours by strength,
    /// and enough to kill the rest of the air shown within `KILL_WITHIN`.
    pub(crate) fn fighters_wanted(&self) -> usize {
        let ours = self.numbers(self.kit.fighter);
        if !self.options.air || ours.dps <= 0.0 {
            return 0;
        }
        let strength = |n: AirNumbers| (n.dps * n.health).sqrt();
        let (mut fighters, mut rest) = (0.0, 0.0);
        for (def, _) in self.air.shown.values() {
            let n = self.numbers(Some(*def));
            if n.fighter {
                fighters += strength(n) / strength(ours);
            } else {
                rest += n.health;
            }
        }
        (FIGHTERS_LEAST + (FIGHTERS_TO_HIS * fighters).ceil() as usize + (rest / (ours.dps * KILL_WITHIN)).ceil() as usize).min(FIGHTERS_AT_MOST)
    }

    /// Fighters fewer than wanted, those being made counted.
    fn fighters_short(&self) -> bool {
        self.count(self.kit.fighter) < self.fighters_wanted()
    }

    /// D-AIR-PLANT: the builder can build the aircraft plant, the income has reached `AIR_PLANT_FROM`, and none
    /// stands, is going up or has been ordered.
    pub(crate) fn air_plant_wanted(&self, builder: &UnitDefInfo) -> bool {
        let Some(plant) = self.kit.air_plant.filter(|d| builder.build_options.contains(d)) else { return false };
        self.options.air
            && self.metal_income >= AIR_PLANT_FROM
            && self.finished(self.kit.plant).count() >= AIR_PLANT_AFTER
            && self.count(Some(plant)) == 0
            && self.economy.working_on(|job| job == Job::AirPlant) == 0
            && self.frame >= self.air.plant_refused
    }

    /// The aircraft plant's site: home (300 behind it, in the open, BARb's raiders killed it five times in eight
    /// games, dist28-airfit-barb).
    pub(crate) fn air_plant_site(&self) -> Vec3 {
        self.home
    }

    /// D-AIR-POWER: while the fighters are short of the count his air shown asks for, a builder that would help a
    /// plant helps the aircraft plant, and construction turrets may go beside it.
    pub(crate) fn air_power_wanted(&self) -> bool {
        self.finished(self.kit.air_plant).next().is_some() && !self.air.shown.is_empty() && self.fighters_short()
    }

    /// The energy a second the aircraft plant draws while it has fighters to make against the air shown: its build
    /// power and its helpers' times the fighter's energy over its build time (D-ENERGY adds it to the soldiers' draw; a
    /// plant on fighters draws 124 a second alone). What answers nothing yet is made from spare energy and is not counted.
    pub(crate) fn air_draw(&self) -> f32 {
        if self.air.shown.is_empty() || !self.fighters_short() {
            return 0.0;
        }
        let Some(unit) = self.kit.fighter.and_then(|d| self.def(d)) else { return 0.0 };
        let plants: f32 = self.finished(self.kit.air_plant).filter_map(|p| self.def(p.def)).map(|d| d.build_speed).sum();
        if plants == 0.0 {
            return 0.0;
        }
        let helpers: f32 = self.economy.orders.iter().filter(|(_, o)| o.job == Job::AirAssist).filter_map(|(id, _)| self.own.get(id)).filter_map(|u| self.def(u.def)).map(|d| d.build_speed).sum();
        (plants + helpers) * unit.energy_cost / unit.build_time.max(1.0)
    }

    /// D-FIGHTERS, D-BOMB: what the aircraft plant makes next: fighters to the count wanted, then the squadron's
    /// bombers, else nothing; with no air of his shown, only from spare energy (`SPARE_ENERGY`).
    pub(crate) fn next_aircraft(&self, energy: Resource) -> Option<UnitDefId> {
        // D-NUKE-LOOK: a scout plane to look for the anti-nuke behind an interception comes first.
        if self.air_scout_wanted() {
            return self.kit.air_scout;
        }
        let spare = energy.current > SPARE_ENERGY * energy.storage;
        if self.fighters_short() && (spare || !self.air.shown.is_empty()) {
            return self.kit.fighter;
        }
        let bombing = self.frame >= self.air.paused_until && (self.air.flown < PROVING || self.air.worth >= self.air.lost as f32 * self.worth(self.kit.bomber));
        (spare && bombing && self.options.air && self.kit.bomber.is_some() && self.count(self.kit.bomber) < BOMBERS && !self.fighters_short()).then_some(self.kit.bomber).flatten()
    }

    /// A unit's worth: its metal and its energy at the converter's rate.
    fn worth(&self, def: Option<UnitDefId>) -> f32 {
        def.and_then(|d| self.def(d)).map_or(0.0, |d| d.metal_cost + d.energy_cost / ENERGY_A_METAL)
    }

    /// D-ANTI-AIR (the bridge): while the fighters are short of the count wanted, Nettles enough that three times
    /// their metal covers the air shown, at least one and at most `AA_AT_MOST`, `AA_BUILDERS` builders at a time.
    pub(crate) fn anti_air_wanted(&self, builder: &UnitDefInfo) -> bool {
        let Some(aa) = self.kit.aa_turret.filter(|d| builder.build_options.contains(d)) else { return false };
        let on_it = self.economy.working_on(|job| job == Job::AntiAir);
        if self.air.shown.is_empty() || !self.fighters_short() || on_it >= AA_BUILDERS {
            return false;
        }
        let metal = self.def(aa).map_or(1.0, |d| d.metal_cost.max(1.0));
        let wanted = ((self.air_shown().0 / (AA_WEIGHT * metal)).ceil() as usize).clamp(1, AA_AT_MOST);
        self.count(Some(aa)) + on_it < wanted
    }

    /// A Nettle's site: by the plant with the fewest within `AA_NEAR`, on its enemy side; the base point when no plant
    /// stands.
    pub(crate) fn anti_air_site(&self) -> Vec3 {
        let plants: Vec<&OwnUnit> = self.finished(self.kit.plant).chain(self.finished(self.kit.advanced_plant)).collect();
        let aa_near = |p: &OwnUnit| self.own.values().filter(|u| Some(u.def) == self.kit.aa_turret && u.pos.dist2d(p.pos) < AA_NEAR).count();
        plants.iter().min_by_key(|p| aa_near(p)).map_or(self.toward_enemy(self.home, 300.0), |p| self.toward_enemy(p.pos, 200.0))
    }

    /// The air station: halfway from home to the hold point.
    fn air_station(&self) -> Vec3 {
        let hold = self.hold_point();
        self.toward_point(self.home, hold, 0.5 * self.home.dist2d(hold))
    }

    /// His anti-air known: his buildings that hit aircraft, with their reach and their damage a second.
    fn his_anti_air(&self) -> Vec<AntiAir> {
        self.army
            .his_buildings
            .values()
            .filter_map(|b| {
                let n = self.numbers(b.def);
                (n.dps > 0.0 && n.reach > 0.0).then_some(AntiAir { at: b.pos, reach: n.reach, dps: n.dps })
            })
            .collect()
    }

    /// Every second: his air shown, our aircraft's move states, the fighters and the bombers.
    pub(crate) fn run_air(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        self.observe_air(tick);
        // Fighters on manoeuvre take aircraft within 1,000 on their own; bombers on hold position wait at the station
        // (an idle bomber otherwise flies off at the nearest target within 1,000 times its move state).
        let alive: BTreeSet<UnitId> = self.own.keys().copied().collect();
        self.air.told.retain(|id| alive.contains(id));
        for unit in self.own.values().filter(|u| !u.being_built && (Some(u.def) == self.kit.fighter || Some(u.def) == self.kit.bomber)) {
            if self.air.told.insert(unit.id) {
                commands.push(Command::MoveState { unit: unit.id, state: if Some(unit.def) == self.kit.fighter { MANOEUVRE } else { HOLD_POSITION } });
            }
        }
        let station = self.air_station();
        let aa = self.his_anti_air();
        self.run_fighters(tick, station, &aa, commands);
        self.run_bombers(station, &aa, commands);
    }

    /// D-FIGHTERS: at his aircraft in sight on our half, near something of ours and clear of his anti-air, the one
    /// nearest the fighters; else idle fighters back to the station.
    fn run_fighters(&mut self, tick: &Tick, station: Vec3, aa: &[AntiAir], commands: &mut Vec<Command>) {
        let fighters: Vec<OwnUnit> = self.finished(self.kit.fighter).cloned().collect();
        if fighters.is_empty() {
            return;
        }
        let n = fighters.len() as f32;
        let middle = Vec3 { x: fighters.iter().map(|u| u.pos.x).sum::<f32>() / n, y: 0.0, z: fighters.iter().map(|u| u.pos.z).sum::<f32>() / n };
        let ours: Vec<Vec3> = self.own.values().filter(|u| self.def(u.def).is_some_and(|d| d.speed == 0.0)).map(|u| u.pos).collect();
        let threat = tick
            .snapshot
            .enemies
            .iter()
            .filter(|e| e.def.and_then(|d| self.def(d)).is_some_and(flies))
            .map(|e| e.pos)
            .filter(|p| !self.his_half(*p) && !covered(aa, *p) && ours.iter().any(|b| b.dist2d(*p) < AIR_DEFEND))
            .min_by(|a, b| a.dist2d(middle).total_cmp(&b.dist2d(middle)));
        if let Some(at) = threat {
            if self.air.fight_order.is_none_or(|(p, f)| p.dist2d(at) >= REFIGHT || self.frame - f >= REFIGHT_AFTER) {
                self.air.fight_order = Some((at, self.frame));
                commands.extend(fighters.iter().map(|u| Command::Fight { unit: u.id, to: at, queue: false }));
            }
            return;
        }
        self.air.fight_order = None;
        commands.extend(fighters.iter().filter(|u| u.idle && u.pos.dist2d(station) > STATION_NEAR).map(|u| Command::Move { unit: u.id, to: station, queue: false }));
    }

    /// D-BOMB: a sortie under way is followed and called back; otherwise, with the squadron whole at the station, a
    /// minute since the last, the bombing not paused and his fighters shown unable to kill the squadron within
    /// `KILL_WITHIN`, the squadron goes at the densest field of his economy buildings clear of his anti-air.
    fn run_bombers(&mut self, station: Vec3, aa: &[AntiAir], commands: &mut Vec<Command>) {
        if self.air.sortie.is_some() {
            return self.follow_sortie(station, commands);
        }
        let bombers: Vec<OwnUnit> = self.finished(self.kit.bomber).cloned().collect();
        commands.extend(bombers.iter().filter(|u| u.idle && u.pos.dist2d(station) > STATION_NEAR).map(|u| Command::Move { unit: u.id, to: station, queue: false }));
        if bombers.len() < BOMBERS || self.frame < self.air.paused_until || self.frame - self.air.last_sortie < SORTIE_GAP || bombers.iter().any(|u| u.pos.dist2d(station) > STATION_WAIT) {
            return;
        }
        let health: f32 = bombers.iter().map(|u| u.health).sum();
        if self.his_fighters() * KILL_WITHIN >= health {
            return;
        }
        let Some(target) = self.bomb_target(station, aa) else { return };
        let flight = (station.dist2d(target.at) / self.bomber_speed() * SECOND as f32) as i32;
        eprintln!(
            "[distilled ai={}] f={} sortie: {} bombers from ({:.0},{:.0}) at ({:.0},{:.0}), {} economy buildings worth {:.0} within {FIELD:.0}, his anti-air {:.0} damage on the way, flight {:.0} s",
            self.hello.ai_id,
            self.frame,
            bombers.len(),
            station.x,
            station.z,
            target.at.x,
            target.at.z,
            target.buildings,
            target.worth,
            target.exposure,
            flight as f32 / SECOND as f32
        );
        let target = target.at;
        commands.extend(bombers.iter().map(|u| Command::FireAt { unit: u.id, at: target }));
        self.air.sortie = Some(Sortie { target, began: self.frame, flight, bombers: bombers.iter().map(|u| u.id).collect(), near_since: BTreeMap::new(), recalled: BTreeSet::new(), over: None, killed: 0, worth: 0.0 });
        self.air.flown += 1;
    }

    /// A bomber that has been over the target for `PASS_FOR` has dropped and goes back to the station (an attack at a
    /// place never ends on its own); all go back once the sortie has lasted its flight time and `SORTIE_SLACK`. The
    /// sortie closes `SETTLE` after the last is called back, and one that lost half its bombers pauses the bombing.
    fn follow_sortie(&mut self, station: Vec3, commands: &mut Vec<Command>) {
        let frame = self.frame;
        let Some(sortie) = self.air.sortie.as_mut() else { return };
        let alive: Vec<(UnitId, Vec3)> = sortie.bombers.iter().filter_map(|id| self.own.get(id)).map(|u| (u.id, u.pos)).collect();
        let late = frame - sortie.began > sortie.flight + SORTIE_SLACK;
        for (id, pos) in &alive {
            if sortie.recalled.contains(id) {
                continue;
            }
            if pos.dist2d(sortie.target) <= PASS {
                sortie.near_since.entry(*id).or_insert(frame);
            }
            if late || sortie.near_since.get(id).is_some_and(|since| frame - since >= PASS_FOR) {
                sortie.recalled.insert(*id);
                commands.push(Command::Move { unit: *id, to: station, queue: false });
            }
        }
        if sortie.over.is_none() && alive.iter().all(|(id, _)| sortie.recalled.contains(id)) {
            sortie.over = Some(frame);
        }
        if sortie.over.is_none_or(|over| frame - over < SETTLE) {
            return;
        }
        let lost = sortie.bombers.len() - alive.len();
        eprintln!(
            "[distilled ai={}] f={} sortie over: at ({:.0},{:.0}), {} of {} bombers lost, {} economy buildings of his destroyed worth {:.0}, {:.0} s",
            self.hello.ai_id,
            frame,
            sortie.target.x,
            sortie.target.z,
            lost,
            sortie.bombers.len(),
            sortie.killed,
            sortie.worth,
            (frame - sortie.began) as f32 / SECOND as f32
        );
        if 2 * lost >= sortie.bombers.len() {
            self.air.paused_until = frame + LOSS_PAUSE;
        }
        self.air.lost += lost;
        self.air.killed += sortie.killed;
        self.air.worth += sortie.worth;
        self.air.last_sortie = frame;
        self.air.sortie = None;
    }

    /// A building of his seen destroyed within the field of a sortie under way counts to it (before `forget`).
    pub(crate) fn bombed(&mut self, enemy: UnitId) {
        let Some(seen) = self.army.his_buildings.get(&enemy) else { return };
        if !seen.def.and_then(|d| self.def(d)).is_some_and(economy_building) {
            return;
        }
        let worth = self.worth(seen.def);
        let at = seen.pos;
        if let Some(sortie) = self.air.sortie.as_mut().filter(|s| s.target.dist2d(at) <= 2.0 * FIELD) {
            sortie.killed += 1;
            sortie.worth += worth;
        }
    }

    /// The bombers' speed (the slowest type's), elmos a second.
    fn bomber_speed(&self) -> f32 {
        self.kit.bomber.and_then(|d| self.def(d)).map_or(1.0, |d| d.speed.max(1.0))
    }

    /// The densest field of his known economy buildings (by worth within `FIELD`, at least `FIELD_LEAST` of them) where
    /// his known anti-air would do the squadron no more than `SORTIE_RISK` bombers' health.
    fn bomb_target(&self, from: Vec3, aa: &[AntiAir]) -> Option<Target> {
        self.bomb_fields(from, aa).1
    }

    /// His economy buildings known, the fields of `FIELD_LEAST` or more among them, those within `SORTIE_RISK`, and the
    /// densest of these.
    fn bomb_fields(&self, from: Vec3, aa: &[AntiAir]) -> ([usize; 3], Option<Target>) {
        let speed = self.bomber_speed();
        let risk = SORTIE_RISK * self.numbers(self.kit.bomber).health;
        let economy: Vec<(Vec3, f32)> = self
            .army
            .his_buildings
            .values()
            .filter_map(|b| {
                let d = self.def(b.def?)?;
                economy_building(d).then(|| (b.pos, d.metal_cost + d.energy_cost / ENERGY_A_METAL))
            })
            .collect();
        let mut best: Option<Target> = None;
        let mut counts = [economy.len(), 0, 0];
        for (at, _) in &economy {
            let field: Vec<f32> = economy.iter().filter(|(p, _)| p.dist2d(*at) <= FIELD).map(|(_, w)| *w).collect();
            if field.len() < FIELD_LEAST {
                continue;
            }
            counts[1] += 1;
            let exposure = exposure(aa, from, *at, speed);
            if exposure > risk {
                continue;
            }
            counts[2] += 1;
            let worth: f32 = field.iter().sum();
            if best.as_ref().is_none_or(|b| worth > b.worth) {
                best = Some(Target { at: *at, worth, buildings: field.len(), exposure });
            }
        }
        (counts, best)
    }

    /// `air:` every 30 s: the air shown, the plant, the fighters and bombers against what is wanted, the Nettles, the
    /// sorties.
    pub(crate) fn log_air(&self) {
        let (metal, health) = self.air_shown();
        let mut types: BTreeMap<&str, usize> = BTreeMap::new();
        for (def, _) in self.air.shown.values() {
            *types.entry(self.def(*def).map_or("?", |d| d.name.as_str())).or_default() += 1;
        }
        let seen: Vec<String> = types.iter().map(|(name, n)| format!("{n} {name}")).collect();
        let plant = if self.finished(self.kit.air_plant).next().is_some() {
            "standing"
        } else if self.count(self.kit.air_plant) > 0 {
            "frame"
        } else if self.economy.working_on(|job| job == Job::AirPlant) > 0 {
            "ordered"
        } else {
            "none"
        };
        let helpers = self.economy.working_on(|job| job == Job::AirAssist);
        let station = self.air_station();
        let aa = self.his_anti_air();
        let ([known, fields, within], _) = self.bomb_fields(station, &aa);
        let waiting = self.finished(self.kit.bomber).filter(|u| u.pos.dist2d(station) <= STATION_WAIT).count();
        eprintln!(
            "[distilled ai={}] f={} air: shown {metal:.0} metal, {health:.0} health ({}){} | his fighters {:.0}/s | plant {plant}, {helpers} helping, draw {:.0} | fighters {}/{} | bombers {}/{} | nettles {} | sorties {} ({} bombers lost, {} buildings worth {:.0}){} | his economy known {known}, fields {fields}, within the risk {within}, his anti-air known {}, bombers at the station {waiting}",
            self.hello.ai_id,
            self.frame,
            if seen.is_empty() { "none".to_string() } else { seen.join(", ") },
            if self.air.sortie.is_some() { " | sortie out" } else { "" },
            self.his_fighters(),
            self.air_draw(),
            self.count(self.kit.fighter),
            self.fighters_wanted(),
            self.count(self.kit.bomber),
            if self.kit.bomber.is_some() { BOMBERS } else { 0 },
            self.count(self.kit.aa_turret),
            self.air.flown,
            self.air.lost,
            self.air.killed,
            self.air.worth,
            if self.frame < self.air.paused_until { " | paused" } else { "" },
            aa.len(),
        );
    }
}

/// An aircraft: it moves and has no ground movement class.
pub(crate) fn flies(def: &UnitDefInfo) -> bool {
    def.speed > 0.0 && def.move_class.is_none()
}

/// A building that makes, converts or stores energy, unarmed and no factory: what D-BOMB bombs.
fn economy_building(def: &UnitDefInfo) -> bool {
    def.speed == 0.0
        && def.weapon_count == 0
        && def.build_options.is_empty()
        && (def.energy_make > 0.0 || def.energy_upkeep < 0.0 || def.wind_cap > 0.0 || def.tidal_make > 0.0 || def.converter.is_some() || def.energy_storage > 0.0)
}

/// A turret of his that hits aircraft.
pub(crate) struct AntiAir {
    at: Vec3,
    reach: f32,
    dps: f32,
}

/// A field to bomb: its middle, its worth and buildings, and the damage his anti-air would do on the way.
struct Target {
    at: Vec3,
    worth: f32,
    buildings: usize,
    exposure: f32,
}

/// Within reach of his known anti-air, by `AA_CLEAR` more.
fn covered(aa: &[AntiAir], at: Vec3) -> bool {
    aa.iter().any(|a| a.at.dist2d(at) <= a.reach + AA_CLEAR)
}

/// The damage his anti-air does to aircraft flying at `speed` from `from` to `to` and back on the straight line, and
/// over `to` for `PASS_FOR`: each turret's damage a second for the time the line runs within its reach.
fn exposure(aa: &[AntiAir], from: Vec3, to: Vec3, speed: f32) -> f32 {
    let over = PASS_FOR as f32 / SECOND as f32;
    aa.iter().map(|a| a.dps * (2.0 * inside(a.at, a.reach, from, to) / speed.max(1.0) + if a.at.dist2d(to) <= a.reach { over } else { 0.0 })).sum()
}

/// The length of the segment from `a` to `b` that lies within `reach` of `centre`.
fn inside(centre: Vec3, reach: f32, a: Vec3, b: Vec3) -> f32 {
    let (dx, dz) = (b.x - a.x, b.z - a.z);
    let (fx, fz) = (a.x - centre.x, a.z - centre.z);
    let length2 = dx * dx + dz * dz;
    if length2 <= 0.0 {
        return 0.0;
    }
    let half_b = fx * dx + fz * dz;
    let c = fx * fx + fz * fz - reach * reach;
    let disc = half_b * half_b - length2 * c;
    if disc <= 0.0 {
        return 0.0;
    }
    let root = disc.sqrt();
    let t1 = ((-half_b - root) / length2).clamp(0.0, 1.0);
    let t2 = ((-half_b + root) / length2).clamp(0.0, 1.0);
    (t2 - t1) * length2.sqrt()
}

#[cfg(test)]
mod tests {
    use super::{AntiAir, covered, exposure, inside};
    use bot_protocol::Vec3;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    #[test]
    fn a_line_through_a_turrets_reach_is_exposed_for_its_chord() {
        let turret = at(1000.0, 0.0);
        // Straight across the middle: the whole diameter.
        assert!((inside(turret, 765.0, at(0.0, 0.0), at(2000.0, 0.0)) - 1530.0).abs() < 1.0);
        // Ending at the turret: the radius.
        assert!((inside(turret, 765.0, at(0.0, 0.0), turret) - 765.0).abs() < 1.0);
        // Wide of it.
        assert_eq!(inside(turret, 765.0, at(0.0, 1200.0), at(2000.0, 1200.0)), 0.0);
        let aa = [AntiAir { at: turret, reach: 765.0, dps: 68.0 }];
        assert!(covered(&aa, at(1000.0, 900.0)));
        assert!(!covered(&aa, at(1000.0, 950.0)));
        // A Thistle beside the target: 765 in and out at 250 a second, and 4 s over it: 68 * (6.1 + 4) = 688, about a
        // Stormbringer's 670.
        let damage = exposure(&aa, at(0.0, 0.0), turret, 250.0);
        assert!((damage - 68.0 * (2.0 * 765.0 / 250.0 + 4.0)).abs() < 1.0, "{damage}");
    }
}
