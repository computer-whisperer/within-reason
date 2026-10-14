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
mod tier2;

use std::collections::BTreeMap;

use bot_protocol::{Bot, Command, Commands, Event, Hello, OwnUnit, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};

pub(crate) const SECOND: i32 = 30;
pub(crate) const MINUTE: i32 = 60 * SECOND;

/// The units the rules name, for the faction the commander is of. `None` where the game has no such unit.
#[derive(Default)]
pub(crate) struct Kit {
    pub extractor: Option<UnitDefId>,
    pub solar: Option<UnitDefId>,
    pub advanced_solar: Option<UnitDefId>,
    /// The construction turret: build power that stands beside a plant.
    pub turret: Option<UnitDefId>,
    pub plant: Option<UnitDefId>,
    pub sentry: Option<UnitDefId>,
    pub constructor: Option<UnitDefId>,
    pub scout: Option<UnitDefId>,
    pub raider: Option<UnitDefId>,
    pub tank: Option<UnitDefId>,
    pub artillery: Option<UnitDefId>,
    /// Tier 2 (`tier2.rs`).
    pub commander: Option<UnitDefId>,
    pub advanced_plant: Option<UnitDefId>,
    pub advanced_constructor: Option<UnitDefId>,
    pub moho: Option<UnitDefId>,
    pub fusion: Option<UnitDefId>,
    pub advanced_fusion: Option<UnitDefId>,
    pub converter: Option<UnitDefId>,
    pub advanced_converter: Option<UnitDefId>,
    /// The tier-2 fast assist unit the advanced vehicle plant makes (Armada's Consul; Cortex vehicles have none).
    pub assist: Option<UnitDefId>,
    pub t2_tank: Option<UnitDefId>,
    pub t2_artillery: Option<UnitDefId>,
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
    /// This tick's metal income, for the rules that run without the tick (D-PRODUCTION).
    pub metal_income: f32,
    /// The lobby's income bonus for our seat as a multiplier (`Handicap=50` in our `[TEAMn]` of the start script is
    /// 1.5): what every side's army scales by, so the launch weights scale by it too (D-LAUNCH, 2026-10-13).
    pub income_multiplier: f32,
    pub economy: economy::Economy,
    pub army: army::Army,
    /// The arms of the rows, from `WITHIN_REASON_DISTILLED`.
    pub options: tier2::Options,
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
        // The suffixes a faction uses for each place. Legion's differ from the other two where it matters: the
        // Pharos `leglht` for the light tower, the Wheelie `legscout`, the converters `legeconv`/`legadveconv`;
        // its vehicles by the same places (docs/knowledge/legion.md, K-leg-role-words): Alaris raider, Decurion
        // tank, Barrage artillery, Prometheus heat-ray tank and Cleaver cluster artillery at tier 2, the Aceso buggy
        // as the fast assist constructor.
        let (sentry, scout, converter, advanced_converter, raider, tank, artillery, t2_tank, t2_artillery, assist) = match faction {
            "cor" => ("llt", "fav", "makr", "mmkr", "gator", "raid", "wolv", "reap", "mart", None),
            "leg" => ("lht", "scout", "econv", "adveconv", "hades", "gat", "bar", "aheattank", "amcluster", Some("afcv")),
            _ => ("llt", "fav", "makr", "mmkr", "flash", "stump", "art", "bull", "mart", Some("consul")),
        };
        Kit {
            // The cheapest extractor the commander can build.
            extractor: commander.build_options.iter().filter_map(|id| self.def(*id)).filter(|d| d.extracts_metal > 0.0).min_by(|a, b| a.metal_cost.total_cmp(&b.metal_cost)).map(|d| d.id),
            solar: named("solar"),
            advanced_solar: named("advsol"),
            turret: named("nanotc"),
            plant: named("vp"),
            sentry: named(sentry),
            constructor: named("cv"),
            scout: named(scout),
            raider: named(raider),
            tank: named(tank),
            artillery: named(artillery),
            commander: Some(commander.id),
            advanced_plant: named("avp"),
            advanced_constructor: named("acv"),
            moho: named("moho"),
            fusion: named("fus"),
            advanced_fusion: named("afus"),
            converter: named(converter),
            advanced_converter: named(advanced_converter),
            assist: assist.and_then(named),
            t2_tank: named(t2_tank),
            t2_artillery: named(t2_artillery),
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
        self.income_multiplier = 1.0 + handicap_percent(&self.hello.script, self.hello.team) / 100.0;
        let kit = &self.kit;
        let missing: Vec<&str> = [("extractor", kit.extractor), ("solar", kit.solar), ("advanced solar", kit.advanced_solar), ("construction turret", kit.turret), ("plant", kit.plant), ("sentry", kit.sentry), ("constructor", kit.constructor), ("scout", kit.scout), ("raider", kit.raider), ("tank", kit.tank), ("artillery", kit.artillery), ("advanced plant", kit.advanced_plant), ("advanced constructor", kit.advanced_constructor), ("tier-2 extractor", kit.moho), ("fusion", kit.fusion), ("tier-2 tank", kit.t2_tank)]
            .iter()
            .filter(|(_, def)| def.is_none())
            .map(|(name, _)| *name)
            .collect();
        eprintln!(
            "[distilled ai={}] home ({:.0}, {:.0}), enemy start taken as ({:.0}, {:.0}); kit lacks: {}; arms: {}",
            self.hello.ai_id, self.home.x, self.home.z, self.enemy_home.x, self.enemy_home.z, if missing.is_empty() { "nothing".into() } else { missing.join(", ") }, self.options.describe()
        );
        self.started = true;
    }
}

/// The `Handicap` of a team's block in the start script, in percent of income; 0 when it has none.
fn handicap_percent(script: &str, team: i32) -> f32 {
    let lower = script.to_ascii_lowercase();
    let Some(start) = lower.find(&format!("[team{team}]")) else { return 0.0 };
    let block = &lower[start..];
    let block = &block[..block.find('}').unwrap_or(block.len())];
    block
        .split(';')
        .filter_map(|field| field.split_once('='))
        .find(|(key, _)| key.trim().trim_start_matches('{').trim() == "handicap")
        .and_then(|(_, value)| value.trim().parse::<f32>().ok())
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::handicap_percent;

    #[test]
    fn the_handicap_is_read_from_our_team_block() {
        let script = "[GAME]\n{\n\t[TEAM0] { TeamLeader=0; AllyTeam=0; Side=Armada; Handicap=50; }\n\t[TEAM1] { TeamLeader=0; AllyTeam=1; Side=Cortex; Handicap=-50; }\n\t[TEAM2] { TeamLeader=0; AllyTeam=1; Side=Cortex; }\n}\n";
        assert_eq!(handicap_percent(script, 0), 50.0);
        assert_eq!(handicap_percent(script, 1), -50.0);
        assert_eq!(handicap_percent(script, 2), 0.0);
        assert_eq!(handicap_percent("", 0), 0.0);
    }
}

impl Bot for Distilled {
    fn start(hello: Hello) -> Self {
        let defs = hello.unit_defs.iter().map(|d| (d.id, d.clone())).collect();
        let spots = hello.metal_spots.len();
        Distilled { hello, defs, kit: Kit::default(), home: Vec3::default(), enemy_home: Vec3::default(), frame: 0, own: BTreeMap::new(), metal_income: 0.0, income_multiplier: 1.0, economy: economy::Economy::new(spots), army: army::Army::default(), options: tier2::Options::from_env(), started: false }
    }

    fn tick(&mut self, tick: Tick) -> Commands {
        self.frame = tick.frame;
        self.own = tick.snapshot.own_units.iter().map(|u| (u.id, u.clone())).collect();
        self.metal_income = tick.snapshot.metal.income;
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
                Event::UnitFinished { unit } => self.t2_milestone(*unit, "standing"),
                Event::UnitCreated { unit, .. } => self.t2_milestone(*unit, "begun"),
                _ => {}
            }
        }
        let mut commands: Vec<Command> = Vec::new();
        // Twice a second for the economy, once for the army: nothing here needs the tick's 10 Hz.
        if tick.due() % (SECOND / 2) == 0 {
            self.run_builders(&tick, &mut commands);
            self.run_turrets(&mut commands);
            self.run_factories(&mut commands);
            self.run_priorities(&mut commands);
        }
        if tick.due() % SECOND == 0 {
            self.run_army(&tick, &mut commands);
        }
        if tick.due() % MINUTE == 0 && tick.frame > 0 {
            let project = self.project_frame().map_or("none".to_string(), |f| format!("{}:{:.0}%:{:.0}/{:.0}", self.def(f.def).map_or("?", |d| d.name.as_str()), 100.0 * f.health / f.max_health.max(1.0), self.power_on(f), self.power_wanted(f, &tick)));
            eprintln!(
                "[distilled ai={}] f={} extractors={} solars={}+{} plants={} turrets={} constructors={} sentries={} can_spend={:.1} home_guard={} strike={} his_buildings_known={} made={} advplant={} acv={} mohos={} fusions={} afus={} converters={} assist={} t2_made={} paused={} project={} low={} metal {:.0}/{:.0} +{:.1} -{:.1} energy {:.0}/{:.0} +{:.0} -{:.0}",
                self.hello.ai_id, tick.frame, self.count(self.kit.extractor), self.count(self.kit.solar), self.count(self.kit.advanced_solar), self.count(self.kit.plant), self.count(self.kit.turret), self.count(self.kit.constructor), self.count(self.kit.sentry), self.can_spend(),
                self.army.home_guard.len(), self.army.strike.len(), self.army.his_buildings.len(), self.economy.made(),
                self.count(self.kit.advanced_plant), self.count(self.kit.advanced_constructor), self.count(self.kit.moho), self.count(self.kit.fusion), self.count(self.kit.advanced_fusion), self.count(self.kit.converter) + self.count(self.kit.advanced_converter), self.count(self.kit.assist), self.economy.t2_made, u8::from(self.plants_paused()), project, self.own.values().filter(|u| u.priority_low).count(),
                tick.snapshot.metal.current, tick.snapshot.metal.storage, tick.snapshot.metal.income, tick.snapshot.metal.usage, tick.snapshot.energy.current, tick.snapshot.energy.storage, tick.snapshot.energy.income, tick.snapshot.energy.usage
            );
        }
        Commands(commands)
    }
}

ai_host::export!(ai_host::InProcess<Distilled>);
