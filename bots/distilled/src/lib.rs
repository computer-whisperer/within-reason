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
mod lane;
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
    /// The anti-air turret (D-ANTI-AIR): Nettle, Thistle, Bramble.
    pub aa_turret: Option<UnitDefId>,
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
    /// The previous tick's units, for where a unit that died this tick stood (the death instrument).
    pub prev_own: BTreeMap<UnitId, OwnUnit>,
    /// This tick's metal income, for the rules that run without the tick (D-PRODUCTION).
    pub metal_income: f32,
    /// The lobby's income bonus for our seat as a multiplier (`Handicap=50` in our `[TEAMn]` of the start script is
    /// 1.5): what every side's army scales by, so the launch weights scale by it too (D-LAUNCH, 2026-10-13).
    pub income_multiplier: f32,
    pub economy: economy::Economy,
    pub army: army::Army,
    /// The control lane under the army's orders (D-LANE, `lane.rs`).
    pub control: lane::Control,
    /// The arms of the rows, from `WITHIN_REASON_DISTILLED`.
    pub options: tier2::Options,
    started: bool,
    /// The banner has been said in the game chat (once, at the first orders).
    announced: bool,
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
            aa_turret: named("rl"),
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
        let missing: Vec<&str> = [("extractor", kit.extractor), ("solar", kit.solar), ("advanced solar", kit.advanced_solar), ("construction turret", kit.turret), ("plant", kit.plant), ("sentry", kit.sentry), ("anti-air turret", kit.aa_turret), ("constructor", kit.constructor), ("scout", kit.scout), ("raider", kit.raider), ("tank", kit.tank), ("artillery", kit.artillery), ("advanced plant", kit.advanced_plant), ("advanced constructor", kit.advanced_constructor), ("tier-2 extractor", kit.moho), ("fusion", kit.fusion), ("tier-2 tank", kit.t2_tank)]
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

    /// What the bot says in the game chat at its first orders, as the flagship does (`crates/bot/src/main.rs`): the
    /// game names AIs at random, so the line says which name is ours, the commit it was built from, its arms, the
    /// seat's income bonus and the seat. A replay or an engine log then says which build played.
    fn banner(&self) -> String {
        let side = self.hello.teams.iter().find(|t| t.team == self.hello.team).map_or("?", |t| t.side.as_str());
        format!(
            "{{name}} is WRDistilled {} | arms: {} | bonus {:+.0}% | seat ai{} team {} {side}",
            ai_host::commit(),
            self.options.describe(),
            (self.income_multiplier - 1.0) * 100.0,
            self.hello.ai_id,
            self.hello.team
        )
    }
}

impl Distilled {
    /// A building's frame begun by one of our builders goes to the engine log with how long after the builder's
    /// order it came and how far the builder stood (D-ORDERS: what a builder waits for before the frame).
    fn frame_begun(&self, unit: UnitId, builder: Option<UnitId>) {
        let (Some(frame), Some(builder)) = (self.own.get(&unit), builder.and_then(|b| self.own.get(&b))) else { return };
        let Some(def) = self.def(frame.def).filter(|d| d.speed == 0.0) else { return };
        let age = self.economy.order_age(builder.id, self.frame).map_or("no order".to_string(), |f| format!("{:.1} s after its order", f as f32 / SECOND as f32));
        eprintln!("[distilled ai={}] f={} begun: {} by unit {} at {:.0}, {age}", self.hello.ai_id, self.frame, def.name, builder.id.0, builder.pos.dist2d(frame.pos));
    }
}

impl Distilled {
    /// The death instrument (2026-10-20): every soldier or scout of ours that dies goes to the engine log with where it
    /// stood, which body it was in and what that body was doing, what killed it (the type read off the weapon's
    /// name, which the game writes as `<unit>_<weapon>`), and what stood within 600 of it: his turrets and mobile
    /// units by weight, and ours. `run/bot_deaths.py` tallies the lines. Written because the exchange is the bot's
    /// wall (ours 15-29 dead by 10:00 for his 4-15 against BARb hard_aggressive, dist6-barb-hardagg-comet-1) and the
    /// census says nothing about where or to what.
    fn unit_died(&self, tick: &Tick, unit: UnitId, attacker: Option<UnitId>, death: &bot_protocol::Death) {
        let Some(fallen) = self.prev_own.get(&unit) else { return };
        let Some(def) = self.def(fallen.def) else { return };
        let at = fallen.pos;
        // Soldiers by body; builders and buildings by kind (2026-10-20 evening: the outposts and constructors die to
        // his raids and the soldiers' deaths alone said nothing of it, dist8-army-barb-hardagg-comet-1 game 00).
        let body = if def.speed == 0.0 {
            if def.build_speed > 0.0 && !def.build_options.is_empty() { "plant" } else if def.extracts_metal > 0.0 { "extractor" } else if def.weapon_count > 0 { "turret" } else { "building" }
        } else if !def.build_options.is_empty() {
            if Some(fallen.def) == self.kit.commander { "commander" } else { "builder" }
        } else if def.weapon_count == 0 {
            "other"
        } else if self.army.strike.contains(&unit) {
            "strike"
        } else if self.army.home_guard.contains(&unit) {
            "guard"
        } else if self.army.is_scout(unit) {
            "scout"
        } else if self.army.is_raider(unit) {
            "raiders"
        } else {
            "none"
        };
        let mode = self.stance_word();
        let killer = match death {
            bot_protocol::Death::Weapon(w) => self.defs.values().filter(|d| w.name.starts_with(&d.name)).max_by_key(|d| d.name.len()).map(|d| d.name.clone()).unwrap_or_else(|| w.name.clone()),
            other => format!("{other:?}").to_ascii_lowercase(),
        };
        let killer_kind = match death {
            bot_protocol::Death::Weapon(w) => match self.defs.values().filter(|d| w.name.starts_with(&d.name)).max_by_key(|d| d.name.len()) {
                Some(d) if d.speed == 0.0 => "turret",
                Some(d) if d.name.ends_with("com") => "commander",
                Some(_) => "mobile",
                None => "unknown",
            },
            _ => "cause",
        };
        let attacker_seen = attacker.and_then(|a| tick.snapshot.enemies.iter().find(|e| e.id == a)).and_then(|e| e.def).and_then(|d| self.def(d)).map_or("-", |d| d.name.as_str());
        let near = |p: Vec3| p.dist2d(at) < 600.0;
        let (mut turrets, mut turret_w, mut mobiles, mut mobile_w) = (0, 0.0, 0, 0.0);
        for e in tick.snapshot.enemies.iter().filter(|e| near(e.pos) && !e.being_built) {
            let Some(d) = e.def.and_then(|d| self.def(d)) else { mobiles += 1; mobile_w += 110.0; continue };
            if d.weapon_count == 0 {
                continue;
            }
            if d.speed == 0.0 {
                turrets += 1;
                turret_w += d.metal_cost;
            } else {
                mobiles += 1;
                mobile_w += d.metal_cost;
            }
        }
        let (mut ours, mut ours_w) = (0, 0.0);
        for u in self.prev_own.values().filter(|u| u.id != unit && near(u.pos)) {
            if let Some(d) = self.def(u.def).filter(|d| d.speed > 0.0 && d.weapon_count > 0 && d.build_options.is_empty()) {
                ours += 1;
                ours_w += d.metal_cost;
            }
        }
        let half = if at.dist2d(self.enemy_home) < at.dist2d(self.home) { "his" } else { "ours" };
        let order = self.army.strike_target().map_or("none".to_string(), |(p, _)| format!("({:.0},{:.0})", p.x, p.z));
        eprintln!(
            "[distilled ai={}] f={} died: {} unit {} at ({:.0},{:.0}) half={half} body={body} mode={mode} killer={killer} kind={killer_kind} attacker={attacker_seen} near600: turrets {turrets} w{:.0} mobiles {mobiles} w{:.0} ours {ours} w{:.0}; body n={} w{:.0} order={order}",
            self.hello.ai_id, self.frame, def.name, unit.0, at.x, at.z, turret_w, mobile_w, ours_w, self.army.strike.len(), self.strike_weight()
        );
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
        Distilled { hello, defs, kit: Kit::default(), home: Vec3::default(), enemy_home: Vec3::default(), frame: 0, own: BTreeMap::new(), prev_own: BTreeMap::new(), metal_income: 0.0, income_multiplier: 1.0, economy: economy::Economy::new(spots), army: army::Army::default(), control: lane::Control::default(), options: tier2::Options::from_env(), started: false, announced: false }
    }

    fn tick(&mut self, tick: Tick) -> Commands {
        self.frame = tick.frame;
        self.prev_own = std::mem::take(&mut self.own);
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
                Event::EnemyDestroyed { enemy, .. } => self.army.forget(*enemy),
                Event::UnitFinished { unit } => self.t2_milestone(*unit, "standing"),
                Event::UnitDestroyed { unit, attacker, death } => self.unit_died(&tick, *unit, *attacker, death),
                Event::UnitCreated { unit, builder } => {
                    self.t2_milestone(*unit, "begun");
                    self.frame_begun(*unit, *builder);
                }
                _ => {}
            }
        }
        let mut commands: Vec<Command> = Vec::new();
        // Once the engine takes orders (its first seconds drop them), the bot says who it is.
        if !self.announced && tick.frame >= 2 * SECOND {
            self.announced = true;
            commands.push(Command::Say { text: self.banner() });
        }
        // Twice a second for the economy, once for the army: nothing here needs the tick's 10 Hz.
        if tick.due() % (SECOND / 2) == 0 {
            self.run_builders(&tick, &mut commands);
            self.run_turrets(&mut commands);
            self.run_factories(&mut commands);
            self.run_priorities(&mut commands);
        }
        if tick.due() % SECOND == 0 {
            self.run_army(&tick, &mut commands);
            self.note_commitments();
        }
        // Every tick: the lane's footwork over the soldiers' standing orders (D-LANE).
        self.run_lane(&tick, &mut commands);
        if tick.due() % MINUTE == 0 && tick.frame > 0 {
            let project = self.project_frame().map_or("none".to_string(), |f| format!("{}:{:.0}%:{:.0}/{:.0}", self.def(f.def).map_or("?", |d| d.name.as_str()), 100.0 * f.health / f.max_health.max(1.0), self.power_on(f), self.power_wanted(f, &tick)));
            eprintln!(
                "[distilled ai={}] f={} extractors={} solars={}+{} plants={} turrets={} constructors={} sentries={} aa={} air_seen={} can_spend={:.1} home_guard={} strike={} raiders={} launch={:.0}/{:.0} field_won={} his_buildings_known={} made={} advplant={} acv={} mohos={} fusions={} afus={} converters={} assist={} t2_made={} paused={} project={} low={} metal {:.0}/{:.0} +{:.1} -{:.1} energy {:.0}/{:.0} +{:.0} -{:.0}",
                self.hello.ai_id, tick.frame, self.count(self.kit.extractor), self.count(self.kit.solar), self.count(self.kit.advanced_solar), self.count(self.kit.plant), self.count(self.kit.turret), self.count(self.kit.constructor), self.count(self.kit.sentry), self.count(self.kit.aa_turret), self.army.air_seen.map_or(0, |f| f / MINUTE), self.can_spend(),
                self.army.home_guard.len(), self.army.strike.len(), self.army.raiders.len(), self.home_guard_weight(), self.launch_weight(), u8::from(self.field_won()), self.army.his_buildings.len(), self.economy.made(),
                self.count(self.kit.advanced_plant), self.count(self.kit.advanced_constructor), self.count(self.kit.moho), self.count(self.kit.fusion), self.count(self.kit.advanced_fusion), self.count(self.kit.converter) + self.count(self.kit.advanced_converter), self.count(self.kit.assist), self.economy.t2_made, u8::from(self.plants_paused()), project, self.own.values().filter(|u| u.priority_low).count(),
                tick.snapshot.metal.current, tick.snapshot.metal.storage, tick.snapshot.metal.income, tick.snapshot.metal.usage, tick.snapshot.energy.current, tick.snapshot.energy.storage, tick.snapshot.energy.income, tick.snapshot.energy.usage
            );
        }
        Commands(commands)
    }
}

ai_host::export!(ai_host::InProcess<Distilled>);
