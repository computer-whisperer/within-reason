//! A heuristic bot distilled from the flagship's winning games. Lobby name `WRDistilled`.
//!
//! This is a bot of `bots/`: code decides everything here, and the flagship's rule against that (`docs/code-decisions.md`)
//! does not govern it (`bots/README.md`). Its rules and the games each was read from are registered in this
//! directory's `README.md` under `D-` ids; a rule added here is added there.
//!
//! The shape (`README.md`): the economy and production are the scripts the flagship's player wrote in every game it
//! won, as rules over what stands (`economy.rs`); the army is a few rules over the verbs its hands picked most
//! (`army.rs`), which are a first guess and not a fit.

mod army;
mod economy;

use std::collections::BTreeMap;

use bot_protocol::{Bot, Command, Commands, Event, Hello, OwnUnit, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};

pub(crate) const SECOND: i32 = 30;
pub(crate) const MINUTE: i32 = 60 * SECOND;

/// The units the rules name, for the faction the commander is of. `None` where the game has no such unit.
#[derive(Default)]
pub(crate) struct Kit {
    pub extractor: Option<UnitDefId>,
    pub solar: Option<UnitDefId>,
    pub plant: Option<UnitDefId>,
    pub sentry: Option<UnitDefId>,
    pub constructor: Option<UnitDefId>,
    pub scout: Option<UnitDefId>,
    pub raider: Option<UnitDefId>,
    pub tank: Option<UnitDefId>,
    pub artillery: Option<UnitDefId>,
}

pub(crate) struct Distilled {
    pub hello: Hello,
    pub defs: BTreeMap<UnitDefId, UnitDefInfo>,
    pub kit: Kit,
    /// Where the commander stood at the first tick, and where the enemy's start box lies.
    pub home: Vec3,
    pub enemy_home: Vec3,
    pub frame: i32,
    pub own: BTreeMap<UnitId, OwnUnit>,
    pub economy: economy::Economy,
    pub army: army::Army,
    started: bool,
}

impl Distilled {
    pub(crate) fn def(&self, id: UnitDefId) -> Option<&UnitDefInfo> {
        self.defs.get(&id)
    }

    pub(crate) fn finished(&self, def: Option<UnitDefId>) -> impl Iterator<Item = &OwnUnit> {
        self.own.values().filter(move |u| Some(u.def) == def && !u.being_built)
    }

    /// Units of a type, those under construction too.
    pub(crate) fn count(&self, def: Option<UnitDefId>) -> usize {
        self.own.values().filter(|u| Some(u.def) == def).count()
    }

    /// A point `distance` from `from` toward the enemy's start.
    pub(crate) fn toward_enemy(&self, from: Vec3, distance: f32) -> Vec3 {
        let (dx, dz) = (self.enemy_home.x - from.x, self.enemy_home.z - from.z);
        let length = dx.hypot(dz).max(1.0);
        Vec3 { x: from.x + dx / length * distance, y: from.y, z: from.z + dz / length * distance }
    }

    fn kit_of(&self, commander: &UnitDefInfo) -> Kit {
        let faction = &commander.name[..commander.name.len().min(3)];
        let named = |suffix: &str| self.defs.values().find(|d| d.name == format!("{faction}{suffix}")).map(|d| d.id);
        let (raider, tank, artillery) = match faction {
            "cor" => ("gator", "raid", "wolv"),
            _ => ("flash", "stump", "art"),
        };
        Kit {
            // The cheapest extractor the commander can build.
            extractor: commander.build_options.iter().filter_map(|id| self.def(*id)).filter(|d| d.extracts_metal > 0.0).min_by(|a, b| a.metal_cost.total_cmp(&b.metal_cost)).map(|d| d.id),
            solar: named("solar"),
            plant: named("vp"),
            sentry: named("llt"),
            constructor: named("cv"),
            scout: named("fav"),
            raider: named(raider),
            tank: named(tank),
            artillery: named(artillery),
        }
    }

    fn start(&mut self, tick: &Tick) {
        let Some(commander) = tick.snapshot.own_units.iter().find(|u| self.def(u.def).is_some_and(|d| !d.build_options.is_empty())) else { return };
        self.home = commander.pos;
        let ours = self.hello.ally_team;
        let map = Vec3 { x: self.hello.map.width, y: 0.0, z: self.hello.map.height };
        // His start box when the script gives one, else our start mirrored through the map's centre.
        self.enemy_home = self.hello.start_boxes.iter().find(|b| b.ally_team != ours).map_or(Vec3 { x: map.x - self.home.x, y: 0.0, z: map.z - self.home.z }, |b| b.centre());
        self.kit = self.def(commander.def).map(|d| self.kit_of(d)).unwrap_or_default();
        let kit = &self.kit;
        let missing: Vec<&str> = [("extractor", kit.extractor), ("solar", kit.solar), ("plant", kit.plant), ("sentry", kit.sentry), ("constructor", kit.constructor), ("scout", kit.scout), ("raider", kit.raider), ("tank", kit.tank), ("artillery", kit.artillery)]
            .iter()
            .filter(|(_, def)| def.is_none())
            .map(|(name, _)| *name)
            .collect();
        eprintln!("[distilled ai={}] home ({:.0}, {:.0}), enemy start taken as ({:.0}, {:.0}); kit lacks: {}", self.hello.ai_id, self.home.x, self.home.z, self.enemy_home.x, self.enemy_home.z, if missing.is_empty() { "nothing".into() } else { missing.join(", ") });
        self.started = true;
    }
}

impl Bot for Distilled {
    fn start(hello: Hello) -> Self {
        let defs = hello.unit_defs.iter().map(|d| (d.id, d.clone())).collect();
        let spots = hello.metal_spots.len();
        Distilled { hello, defs, kit: Kit::default(), home: Vec3::default(), enemy_home: Vec3::default(), frame: 0, own: BTreeMap::new(), economy: economy::Economy::new(spots), army: army::Army::default(), started: false }
    }

    fn tick(&mut self, tick: Tick) -> Commands {
        self.frame = tick.frame;
        self.own = tick.snapshot.own_units.iter().map(|u| (u.id, u.clone())).collect();
        if !self.started {
            self.start(&tick);
            if !self.started {
                return Commands::default();
            }
        }
        for event in &tick.events {
            match event {
                Event::BuildSiteNotFound { unit, def } => self.order_failed(*unit, Some(*def)),
                Event::CommandRejected { unit, .. } => self.order_failed(*unit, None),
                Event::EnemyDestroyed { enemy } => {
                    self.army.his_buildings.remove(enemy);
                }
                _ => {}
            }
        }
        let mut commands: Vec<Command> = Vec::new();
        // Twice a second for the economy, once for the army: nothing here needs the tick's 10 Hz.
        if tick.due() % (SECOND / 2) == 0 {
            self.run_builders(&tick, &mut commands);
            self.run_factories(&mut commands);
        }
        if tick.due() % SECOND == 0 {
            self.run_army(&tick, &mut commands);
        }
        if tick.due() % MINUTE == 0 && tick.frame > 0 {
            eprintln!(
                "[distilled ai={}] f={} extractors={} solars={} plants={} constructors={} sentries={} home_guard={} strike={} his_buildings_known={} metal {:.0}/{:.0} +{:.1} energy {:.0}/{:.0} +{:.0} -{:.0}",
                self.hello.ai_id, tick.frame, self.count(self.kit.extractor), self.count(self.kit.solar), self.count(self.kit.plant), self.count(self.kit.constructor), self.count(self.kit.sentry),
                self.army.home_guard.len(), self.army.strike.len(), self.army.his_buildings.len(),
                tick.snapshot.metal.current, tick.snapshot.metal.storage, tick.snapshot.metal.income, tick.snapshot.energy.current, tick.snapshot.energy.storage, tick.snapshot.energy.income, tick.snapshot.energy.usage
            );
        }
        Commands(commands)
    }
}

ai_host::export!(ai_host::InProcess<Distilled>);
