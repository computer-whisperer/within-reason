//! The nuclear silos (`README.md` D-NUKE to D-NUKE-LOOK; the design `docs/design/2026-10-27-distilled-nukes.md`): a
//! later-game way to spend a bank nothing else spends. A silo is started when the bank holds its metal, the energy
//! covers its draw and there is something to strike (D-NUKE, here). The strikes, their verdicts off the ground, the
//! memory of his cover, the hold and the look are `crates/nukes`' (shared with the territory bot), over this bot's
//! knowledge of his buildings, its friends on the ground and its scouts.

use bot_protocol::{Command, OwnUnit, Tick, UnitDefInfo, UnitId, Vec3};

use crate::economy::Job;
use crate::tier2::BANK_FULL;
use crate::Distilled;

impl Distilled {
    /// The strike engine for this seat, with our faction's silo.
    pub(crate) fn init_nuke(&mut self, table: &combatsim::units::Units) {
        let silo = self.kit.silo.and_then(|d| self.def(d));
        self.nuke = nukes::Nukes::new(format!("[distilled ai={}]", self.hello.ai_id), self.hello.ai_id, &self.hello.unit_defs, table, silo, &self.hello.script);
    }

    /// D-NUKE: a silo is wanted when the bank holds its metal (or `BANK_FULL` of a store smaller than that), its site
    /// was not refused lately, there is a place worth striking, and the silos standing are not sitting on missiles.
    pub(crate) fn silo_wanted(&self, tick: &Tick) -> bool {
        let Some(bill) = self.kit.silo.and_then(|d| self.def(d)).map(|d| d.metal_cost) else { return false };
        let metal = tick.snapshot.metal;
        let piling = self.finished(self.kit.silo).next().is_some() && self.finished(self.kit.silo).all(|s| s.stockpile >= 2);
        self.options.nuke && self.frame >= self.economy.silo_refused && metal.current >= bill.min(BANK_FULL * metal.storage) && self.nuke.open() && !piling
    }

    /// D-ENERGY's share for the silos: each silo's stockpile draw, frames counted and the paused left out, and one
    /// more while a silo is wanted (the energy comes before the silo it is for).
    pub(crate) fn silo_draw(&self, tick: &Tick) -> f32 {
        let Some(def) = self.kit.silo.and_then(|d| self.def(d)) else { return 0.0 };
        let drawing = self.own.values().filter(|u| Some(u.def) == self.kit.silo && !self.nuke.is_paused(u.id)).count();
        let silos = drawing + usize::from(self.silo_wanted(tick));
        def.stockpile_energy / def.stockpile_seconds.max(1.0) * silos as f32
    }

    /// D-NUKE: a builder that can build the silo starts one when one is wanted, no project is under way, and the
    /// energy, with this silo's draw counted, is not short.
    pub(crate) fn silo_job(&self, builder: &UnitDefInfo, tick: &Tick) -> Option<Job> {
        let silo = self.kit.silo?;
        let wanted = builder.build_options.contains(&silo) && self.silo_wanted(tick) && !self.project_under_way() && !self.energy_short(tick);
        wanted.then_some(Job::Silo)
    }

    /// His buildings known with their types, as the strike engine takes them.
    fn buildings_known(&self) -> Vec<nukes::Building> {
        self.army.his_buildings.iter().filter_map(|(id, b)| b.def.map(|def| nukes::Building { id: *id, pos: b.pos, def })).collect()
    }

    /// What the strike engine sees this second: our silos standing, his buildings known, our friends on the ground
    /// (our units but the scouts, our allies', our strike body's order: aircraft pass), his units in sight, the sensors,
    /// and our scouts for the look (the aircraft plant's scout planes, then the army's scouts).
    fn nuke_view<'t>(&self, tick: &'t Tick) -> nukes::View<'t> {
        let grounded = |def| self.def(def).is_some_and(|d| !crate::air::flies(d));
        let ours = self.own.values().filter(|u| grounded(u.def) && !self.army.is_scout(u.id)).map(|u| u.pos);
        let allies = tick.snapshot.allies.iter().filter(|u| grounded(u.def)).map(|u| u.pos);
        let friends: Vec<Vec3> = ours.chain(allies).chain(self.army.strike_target().map(|(p, _)| p)).collect();
        let unit = |u: &OwnUnit| (u.id, u.pos, u.idle);
        nukes::View {
            frame: self.frame,
            silos: self.finished(self.kit.silo).cloned().collect(),
            buildings: self.buildings_known(),
            friends,
            enemies: &tick.snapshot.enemies,
            sensors: tick.snapshot.sensors.as_ref(),
            scouting: Some(nukes::Scouting {
                planes: self.finished(self.kit.air_scout).map(unit).collect(),
                ground: self.own.values().filter(|u| self.army.is_scout(u.id)).map(unit).collect(),
                home: self.home,
                enemy_home: self.enemy_home,
                map: (self.hello.map.width, self.hello.map.height),
            }),
        }
    }

    /// Once a second: the strike engine's turn (D-NUKE-FIRE to D-NUKE-LOOK).
    pub(crate) fn run_nukes(&mut self, tick: &Tick, commands: &mut Vec<Command>) {
        let view = self.nuke_view(tick);
        self.nuke.run(&view, commands);
    }

    /// The shim's answer to a probe; a landing's ruins leave `his_buildings` (D-NUKE-RUINS), so neither the silos nor
    /// the army go back for them until they are seen again.
    pub(crate) fn nuke_ground(&mut self, points: &[Vec3]) {
        let buildings = self.buildings_known();
        for id in self.nuke.ground(points, &buildings, self.frame) {
            self.army.his_buildings.remove(&id);
        }
    }

    /// One of our units fired: a silo's strike is launched.
    pub(crate) fn nuke_launched(&mut self, unit: UnitId) {
        self.nuke.launched(unit, self.frame);
    }

    /// An enemy damaged or destroyed, for the strikes' tallies and his anti-nukes lost. Before `army.forget`.
    pub(crate) fn nuke_hit(&mut self, enemy: UnitId, attacker: Option<UnitId>, destroyed: bool) {
        let seen = self.army.his_buildings.get(&enemy).and_then(|b| b.def.map(|d| (b.pos, d)));
        self.nuke.hit(enemy, seen, attacker, destroyed, self.frame);
    }

    /// D-NUKE-LOOK: the aircraft plant makes a scout plane while one is wanted and none is alive.
    pub(crate) fn air_scout_wanted(&self) -> bool {
        self.nuke.looking() && self.kit.air_scout.is_some() && self.count(self.kit.air_scout) == 0
    }

    /// The minute's `nuke:` line, once a silo has been begun.
    pub(crate) fn log_nukes(&self, tick: &Tick) {
        if self.count(self.kit.silo) == 0 {
            return;
        }
        let view = self.nuke_view(tick);
        self.nuke.log_line(&view, self.count(self.kit.silo) - view.silos.len(), self.silo_draw(tick));
    }
}
