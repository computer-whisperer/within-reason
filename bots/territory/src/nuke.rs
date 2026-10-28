//! T-SEEN and T-NUKE (`docs/design/2026-10-27-territory-nukes.md`): his buildings remembered where they were seen, and
//! nuclear silos from a full bank that fire at them. The strikes, their verdicts off the ground, the memory of his
//! cover and the hold are `crates/nukes`' (shared with the distilled bot); here are what the bot knows of his
//! buildings, when and where a silo is started, and what the engine is shown once a second. The program plans no silo
//! and no stockpile's draw: a standing silo is to it one more building that makes nothing.

use std::collections::{BTreeMap, BTreeSet};

use bot_protocol::{Command, OwnUnit, Tick, UnitDefId, UnitId, Vec3};

use crate::defence::flies;
use crate::hands::{Order, What};
use crate::{Territory, SECOND};

/// A store smaller than the silo's metal holds the silo's bill when this full (the distilled bot's `BANK_FULL`).
const BANK_FULL: f32 = 0.9;
/// The converters run on the energy above their switch: below half a store they are taken to burn nothing spare.
const SPARE_STORE: f32 = 0.5;
/// A builder takes the silo within this of the farm's origin, where it goes (T-FARM's big frames).
const SILO_REACH: f32 = 2000.0;
/// With no farm slot for the silo, none is looked for again this long: every advanced constructor asked for one each
/// half second.
const NO_SLOT_WAIT: i32 = 10 * SECOND;

#[derive(Default)]
pub(crate) struct Silos {
    pub engine: nukes::Nukes,
    /// Our side's silo (the simulator's index): the building with the largest warhead our commander's tree builds.
    pub kind: Option<usize>,
    /// T-SEEN: his buildings seen with their types, where they stood.
    pub his_buildings: BTreeMap<UnitId, (Vec3, UnitDefId)>,
    /// Our silos seen begun and seen standing, for the log.
    begun: BTreeSet<UnitId>,
    standing: BTreeSet<UnitId>,
    /// The frame until which no farm slot is looked for (`NO_SLOT_WAIT`).
    no_slot_until: i32,
}

impl Territory {
    /// At the start: our silo type by its data (a building's stockpiled command-fire weapon that paralyzes nothing, the
    /// largest blast: the tactical launchers' are a fraction of it), and the strike engine for this seat.
    pub(crate) fn init_nukes(&mut self) {
        let table = self.defence.table.take().unwrap_or_default();
        let game = self.game.clone().expect("started");
        let units = &game.units;
        let silo = units
            .reachable(game.commander)
            .into_iter()
            .filter(|&u| units.list[u].speed == 0.0)
            .filter_map(|u| nukes::warhead(&table, &units.list[u].name).map(|w| (u, w.damage * w.radius * w.radius)))
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(u, _)| u);
        self.nuke.kind = silo;
        let def = silo.map(|u| &self.hello.unit_defs[u]);
        self.nuke.engine = nukes::Nukes::new(format!("[territory ai={}]", self.hello.ai_id), self.hello.ai_id, &self.hello.unit_defs, &table, def, &self.hello.script);
        self.defence.table = Some(table);
        match (def, self.nuke.engine.warhead()) {
            (Some(d), Some(w)) => self.log(format_args!(
                "nuke: silo {} ({:.0} metal), warhead {:.0} at {:.0}, stockpile draw {:.0} energy a second{}",
                d.name,
                d.metal_cost,
                w.damage,
                w.radius,
                self.silo_draw_each(),
                if self.nuke.engine.flat() { "; the lobby keeps the ground flat: no verdicts" } else { "" }
            )),
            _ => self.log(format_args!("nuke: no silo in our tree")),
        }
    }

    /// A silo's stockpile draw, energy a second.
    fn silo_draw_each(&self) -> f32 {
        self.nuke.kind.map_or(0.0, |u| {
            let d = &self.hello.unit_defs[u];
            d.stockpile_energy / d.stockpile_seconds.max(1.0)
        })
    }

    fn is_silo(&self, u: &OwnUnit) -> bool {
        self.nuke.kind.is_some() && self.unit_index(u.def) == self.nuke.kind
    }

    /// T-SEEN, once a second with the sensors: his buildings in sight are remembered where they stand (not one seen
    /// destroyed: the engine lists a dying unit through its death), and one whose place is in our team's sight with no
    /// contact of that id is gone. A building the team saw keeps its type on our radar, at the radar's error from its
    /// place (`Unit_getDef` gives the type of a contact in radar once seen), so a known building's place is taken again
    /// only in sight.
    pub(crate) fn remember_buildings(&mut self, tick: &Tick) {
        if !self.options.nuke {
            return;
        }
        let Some(sensors) = &tick.snapshot.sensors else { return };
        for e in &tick.snapshot.enemies {
            if let Some(def) = e.def
                && self.def(def).is_some_and(|d| d.speed == 0.0)
                && !self.guard.dead.contains(&e.id)
            {
                let in_sight = sensors.los.at(e.pos.x, e.pos.z);
                match self.nuke.his_buildings.get_mut(&e.id) {
                    Some(known) if in_sight => *known = (e.pos, def),
                    Some(_) => {}
                    None => {
                        self.nuke.his_buildings.insert(e.id, (e.pos, def));
                    }
                }
            }
        }
        let listed: BTreeSet<UnitId> = tick.snapshot.enemies.iter().map(|e| e.id).collect();
        self.nuke.his_buildings.retain(|id, (pos, _)| listed.contains(id) || !sensors.los.at(pos.x, pos.z));
    }

    /// An enemy damaged or destroyed: for the strikes' tallies and his anti-nukes lost, then a building seen destroyed
    /// is forgotten.
    pub(crate) fn nuke_hit(&mut self, enemy: UnitId, attacker: Option<UnitId>, destroyed: bool) {
        let seen = self.nuke.his_buildings.get(&enemy).copied();
        self.nuke.engine.hit(enemy, seen, attacker, destroyed, self.frame);
        if destroyed {
            self.nuke.his_buildings.remove(&enemy);
        }
    }

    /// The shim's answer to a probe; a landing's ruins are forgotten (D-NUKE-RUINS).
    pub(crate) fn nuke_ground(&mut self, points: &[Vec3]) {
        let buildings = self.buildings_known();
        for id in self.nuke.engine.ground(points, &buildings, self.frame) {
            self.nuke.his_buildings.remove(&id);
        }
    }

    fn buildings_known(&self) -> Vec<nukes::Building> {
        self.nuke.his_buildings.iter().map(|(id, (pos, def))| nukes::Building { id: *id, pos: *pos, def: *def }).collect()
    }

    /// Our silos standing.
    fn silos(&self) -> Vec<&OwnUnit> {
        self.own.values().filter(|u| !u.being_built && self.is_silo(u)).collect()
    }

    /// The converters standing: the energy a second they can burn.
    fn converters_capacity(&self) -> f32 {
        let units = &self.game().units;
        self.own.values().filter(|u| !u.being_built).filter_map(|u| self.unit_index(u.def)).map(|i| units.list[i].conv_capacity as f32).sum()
    }

    /// What T-NUKE weighs, once: the bank against the bill, whether a place is worth striking, a silo framed or
    /// ordered, the silos standing sitting on missiles, and the spare energy (the converters' capacity, with the store
    /// above `SPARE_STORE`) against the silos' draw with one more.
    fn silo_want(&self) -> Option<SiloWant> {
        let silo = self.nuke.kind?;
        let bill = self.game().units.list[silo].metal_cost as f32;
        let pending = self.own.values().any(|u| u.being_built && self.is_silo(u)) || self.hands.orders.values().any(|o| matches!(&o.what, What::Build { unit, .. } if *unit == silo));
        let silos = self.silos();
        let piling = !silos.is_empty() && silos.iter().all(|s| s.stockpile >= 2);
        let drawing = silos.iter().filter(|s| !self.nuke.engine.is_paused(s.id)).count();
        let spare = if self.energy.current >= SPARE_STORE * self.energy.storage { self.converters_capacity() } else { 0.0 };
        Some(SiloWant { bill: bill.min(BANK_FULL * self.metal.storage), open: self.nuke.engine.open(), pending, piling, spare, needed: (drawing + 1) as f32 * self.silo_draw_each() })
    }

    /// T-NUKE: a builder of the silo near the farm starts one when one is wanted, on the farm's slot for big frames
    /// (the most construction turrets in reach), before the plan's flows.
    pub(crate) fn silo_order(&mut self, id: UnitId, unit: usize, commands: &mut Vec<Command>) -> bool {
        let Some(silo) = self.nuke.kind else { return false };
        // The cheap tests first: this runs for every advanced constructor the dispatch reaches each half second.
        if !self.options.nuke || !self.game().units.list[unit].builds.contains(&silo) || self.frame < self.nuke.no_slot_until {
            return false;
        }
        if self.hands.struck.contains(&silo) || self.hands.type_refused.get(&silo).is_some_and(|until| self.frame < *until) {
            return false;
        }
        let origin = self.farm_origin();
        if !self.own.get(&id).is_some_and(|b| b.pos.dist2d(origin) <= SILO_REACH) || !self.silo_want().is_some_and(|w| w.wanted(self.metal.current)) {
            return false;
        }
        if !self.energy_fits(silo, unit) {
            self.hands.energy_refused += 1;
            *self.hands.energy_refused_by.entry(silo).or_default() += 1;
            return false;
        }
        let Some(site) = self.farm_site(id, silo) else {
            self.nuke.no_slot_until = self.frame + NO_SLOT_WAIT;
            self.log(format_args!("nuke: no farm slot for the silo; looked for again in {} s", NO_SLOT_WAIT / SECOND));
            return false;
        };
        let def = self.hello.unit_defs[silo].id;
        let near = site.near;
        commands.push(Command::Build { unit: id, def, site: Some(site), queue: false });
        let what = What::Build { unit: silo, def, near, frame: None };
        self.note_order(id, &what, "nuke");
        self.hands.orders.insert(id, Order { what, given: self.frame, told: self.frame, arrived: None, step: None });
        *self.hands.ordered.entry((silo, "nuke")).or_default() += 1;
        true
    }

    /// What the strike engine sees: our silos standing, his buildings known, our units and our allies' on the ground
    /// (aircraft pass), his units in sight and the sensors. No scouts: the bot's line has no aircraft plant, and
    /// D-NUKE-LOOK does not run.
    fn nuke_view<'t>(&self, tick: &'t Tick) -> nukes::View<'t> {
        let grounded = |def| self.def(def).is_some_and(|d| !flies(d));
        let ours = self.own.values().filter(|u| grounded(u.def)).map(|u| u.pos);
        let allies = self.allies.iter().filter(|u| grounded(u.def)).map(|u| u.pos);
        nukes::View {
            frame: self.frame,
            silos: self.silos().into_iter().cloned().collect(),
            buildings: self.buildings_known(),
            friends: ours.chain(allies).collect(),
            enemies: &tick.snapshot.enemies,
            sensors: tick.snapshot.sensors.as_ref(),
            scouting: None,
        }
    }

    /// Once a second: our silos begun and standing to the log, and the strike engine's turn.
    pub(crate) fn run_nukes(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        if self.nuke.kind.is_none() || !self.options.nuke {
            return;
        }
        let mut news = Vec::new();
        let ours: Vec<(UnitId, Vec3, bool)> = self.own.values().filter(|u| self.is_silo(u)).map(|u| (u.id, u.pos, u.being_built)).collect();
        for (id, at, framed) in ours {
            if self.nuke.begun.insert(id) {
                news.push(format!("nuke: silo begun: unit {} at ({:.0},{:.0}), the bank {:.0} of {:.0}", id.0, at.x, at.z, self.metal.current, self.metal.storage));
            }
            if !framed && self.nuke.standing.insert(id) {
                news.push(format!("nuke: silo standing: unit {} at ({:.0},{:.0})", id.0, at.x, at.z));
            }
        }
        for line in news {
            self.log(format_args!("{line}"));
        }
        let view = self.nuke_view(tick);
        self.nuke.engine.run(&view, commands);
    }

    /// The minute's `nuke:` lines once a building of his is known or a silo begun: what T-NUKE weighs, and the engine's
    /// line.
    pub(crate) fn log_nukes(&self, tick: &Tick) {
        let Some(want) = self.silo_want().filter(|_| self.options.nuke) else { return };
        let all = self.own.values().filter(|u| self.is_silo(u)).count();
        if all == 0 && self.nuke.his_buildings.is_empty() {
            return;
        }
        self.log(format_args!(
            "nuke: want {} | his buildings known {}, a place worth striking {} | bank {:.0} of {:.0} | spare energy {:.0} of {:.0} | silo framed or ordered {} | standing on missiles {}",
            if want.wanted(self.metal.current) { "yes" } else { "no" },
            self.nuke.his_buildings.len(),
            if want.open { "yes" } else { "no" },
            self.metal.current,
            want.bill,
            want.spare,
            want.needed,
            if want.pending { "yes" } else { "no" },
            if want.piling { "yes" } else { "no" },
        ));
        if all > 0 {
            let view = self.nuke_view(tick);
            let drawing = view.silos.iter().filter(|s| !self.nuke.engine.is_paused(s.id)).count();
            self.nuke.engine.log_line(&view, all - view.silos.len(), drawing as f32 * self.silo_draw_each());
        }
    }
}

/// T-NUKE's tests, as weighed.
struct SiloWant {
    bill: f32,
    open: bool,
    pending: bool,
    piling: bool,
    spare: f32,
    needed: f32,
}

impl SiloWant {
    fn wanted(&self, bank: f32) -> bool {
        bank >= self.bill && self.open && !self.pending && !self.piling && self.spare >= self.needed
    }
}
