//! The arms of the rows, from `WITHIN_REASON_TERRITORY`: `key=value` pairs separated by commas
//! (`WITHIN_REASON_TERRITORY=end=1200,budget=400,spots=all`).

pub(crate) struct Options {
    /// The second the horizon runs to: the game's end as the arena caps it (`--max-minutes 20`).
    pub end: f64,
    /// Seconds of the income at the horizon added to the metal made (`Objective::Made`); 0 for the score.
    pub terminal: f64,
    /// The share of the energy storage the program keeps in stock, and the hands' energy budget counts from.
    pub energy_reserve: f64,
    /// The program's energy margin: production over the build draw, as a share of the draw (`buildorder::lp`).
    pub energy_margin: f64,
    /// The game second the margin applies from, and the second before which the program plans no factory.
    pub margin_from: f64,
    pub factory_from: f64,
    /// Wall for a re-plan beside the game, and for the first plan on the engine thread.
    pub budget_ms: u64,
    pub first_budget_ms: u64,
    pub threads: usize,
    /// Frames between re-plans.
    pub replan: i32,
    /// Every spot on the map rather than our half: the ceiling against an opponent that does nothing.
    pub all_spots: bool,
    /// Soldiers on the factories' palette (the territory stage); off, the factories make builders only.
    pub soldiers: bool,
    /// The basic factories the program may open with: `vehicle` (the default: vehicle and air plants, thebluegecko's
    /// line, whose constructors move at 54 against the bot lab's 36 and whose advanced constructor builds at 290
    /// against 210; terr62 against terr61 on Comet: 15:00 367-423 against 322-357, 20:00 881-980 against 825-908,
    /// the mohos 70-73 by 20:00 against 60-62, and 5:00 down 2-7 with the advanced plant committed at 3:48-4:33),
    /// `bot` (bot labs and air plants) or `any` (the program's own choice, which was the lab by its prices).
    pub line: String,
    /// Queues the search may write for factories and constructors not yet standing; the program's caps (constructors per tier).
    pub factories: usize,
    pub constructors: usize,
    /// Write the game's `Hello` with home and the commander's type beside the match for the oracle.
    pub dump: bool,
    /// The planner: the linear program (docs/design/2026-10-20-territory-lp.md) or the anneal of the first design.
    pub lp: bool,
    /// The program's buckets: `dt` seconds for the first `mid_from`, `mid_dt` to `coarse_from`, `coarse_dt` after.
    pub dt: f64,
    pub mid_from: f64,
    pub mid_dt: f64,
    pub coarse_from: f64,
    pub coarse_dt: f64,
    /// The program's bounds: turret help as a multiple of direct power, turrets a factory, the assist factor; the
    /// walk overheads in seconds a unit for a mobile builder (an extractor's besides the walk to its ring).
    pub help_factor: f64,
    pub nanos_per_factory: f64,
    /// The farm: turrets at most as a number (0: the bound per factory), and whether the program may take a
    /// standing building down for its metal (off until the hands give the order).
    pub max_nanos: f64,
    /// Under the farm, the build time from which a turret may help a building (the core's frames); 0 is off.
    pub farm_big: f64,
    /// The metal bank at the horizon counted as metal made (no free spending at the end; off: it hoarded, terr64).
    pub bank: bool,
    pub reclaim: bool,
    /// T-CHAIN: a constructor out among the spots takes extractor flows only while it has them (`chain=off` for the
    /// control).
    pub chain: bool,
    pub assist_factor: f64,
    pub walk_extractor: f64,
    /// The extractor's walk grows from `walk_extractor` at the start to `walk_late` at `walk_late_from` seconds of
    /// the game, by the plan's own t0 (the realized order age at a constructor's extractor frame: 12-19 s through
    /// 9:00, 28-42 s from 10:00; at one 15-s walk the plans from 10:00 wanted 21-24 constructors against 25-28 and
    /// 20:00 read 40-70 under, terr73-82).
    pub walk_late: f64,
    pub walk_late_from: f64,
    pub walk_upgrade: f64,
    pub walk_base: f64,
    pub walk_factory: f64,
    pub commander_walk_base: f64,
    pub commander_walk_factory: f64,
    /// The program's distance rings for extractors (one: the walk a constant), and the overhead beside a ring's walk.
    pub rings: usize,
    pub ring_overhead: f64,
    /// T-COMMIT's window and threshold (hands.rs): a factory type the program wants at `commit_at` of one within
    /// `commit_within` seconds is started whole, once the stock and `commit_afford` seconds of income cover its cost
    /// (no more than AFFORD_SECONDS, where order_one's own affordability binds).
    pub commit_within: f64,
    pub commit_at: f64,
    pub commit_afford: f64,
    /// An advanced plant's frame (one whose constructors build an upgrade) calls its helpers from constructors within
    /// this of home; a basic factory's from FACTORY_PLACER (hands.rs).
    pub plant_helpers_from: f32,
    /// T-OPENING (hands.rs): the commander's first minute is the free spots within this of home; where it holds
    /// three, his opening (the solars before the plant). 600 since terr87: his start's third spot is 537 from it and
    /// the south-west start's 562 (500 before, two spots at either).
    pub opening_spots: f32,
    /// The commander takes extractor spots and upgrades within this of home only (terr15-lp-nullai-comet-1:
    /// following its extractor flows spot by spot it crossed the map, set up a base in the far corner, and died to
    /// his commander 7,300 elmos from home, two games of four lost); the program's `commander_reach` too, and the
    /// radius of the free spots the program prices and bounds its extractors by (`commander_spots`). 700 since
    /// terr76; 1,300 reaches his commander's two spots at 1,007 and 1,215 (2:06-2:13 in his record) and was tried in
    /// terr90-92: one spot a trip and level (terr91), or the pair with the plant unguarded and worse (terr92).
    pub com_leash: f32,
    /// T-PRIORITY: the commander low from its first guard onward (`com_low=after`, the default since terr95: his
    /// record's flag from 2:50 to past 7:00, through its solars and the advanced plant; 5:00 and 8:00 up in all four
    /// games against the guard-only form), or low only while it guards (`guard`, terr94: the pros' claim raises it for
    /// a build of its own, and the distilled bot's commander low on its solars threw its energy store).
    pub com_low_after: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            end: 1500.0,
            terminal: 0.0,
            energy_reserve: 0.2,
            energy_margin: 0.25,
            margin_from: 240.0,
            factory_from: 0.0,
            budget_ms: 400,
            first_budget_ms: 1000,
            threads: 4,
            replan: 30 * 30,
            all_spots: false,
            soldiers: false,
            line: "vehicle".to_string(),
            factories: 4,
            constructors: 40,
            dump: true,
            lp: true,
            dt: 10.0,
            mid_from: 60.0,
            mid_dt: 60.0,
            coarse_from: 300.0,
            coarse_dt: 120.0,
            help_factor: 40.0,
            nanos_per_factory: 6.0,
            max_nanos: 300.0,
            farm_big: 0.0,
            bank: false,
            reclaim: false,
            chain: true,
            assist_factor: 2.0,
            walk_extractor: 15.0,
            walk_late: 40.0,
            walk_late_from: 600.0,
            walk_upgrade: 40.0,
            walk_base: 3.0,
            walk_factory: 40.0,
            commander_walk_base: 5.0,
            commander_walk_factory: 9.0,
            rings: 1,
            ring_overhead: 5.0,
            commit_within: 300.0,
            commit_at: 0.05,
            commit_afford: 45.0,
            plant_helpers_from: 4000.0,
            opening_spots: 600.0,
            com_leash: 700.0,
            com_low_after: true,
        }
    }
}

impl Options {
    pub(crate) fn from_env() -> Self {
        let mut options = Options::default();
        let Some(text) = std::env::var_os("WITHIN_REASON_TERRITORY") else { return options };
        for pair in text.to_string_lossy().split(',').filter(|p| !p.trim().is_empty()) {
            let Some((key, value)) = pair.split_once('=') else { continue };
            let (key, value) = (key.trim(), value.trim());
            let on = value == "on" || value == "1" || value == "true";
            match key {
                "end" => options.end = value.parse().unwrap_or(options.end),
                "terminal" => options.terminal = value.parse().unwrap_or(options.terminal),
                "reserve" => options.energy_reserve = value.parse().unwrap_or(options.energy_reserve),
                "emargin" => options.energy_margin = value.parse().unwrap_or(options.energy_margin),
                "emargin_from" => options.margin_from = value.parse().unwrap_or(options.margin_from),
                "factory_from" => options.factory_from = value.parse().unwrap_or(options.factory_from),
                "budget" => options.budget_ms = value.parse().unwrap_or(options.budget_ms),
                "first" => options.first_budget_ms = value.parse().unwrap_or(options.first_budget_ms),
                "threads" => options.threads = value.parse().unwrap_or(options.threads),
                "replan" => options.replan = value.parse::<i32>().map_or(options.replan, |s| s * 30),
                "spots" => options.all_spots = value == "all",
                "soldiers" => options.soldiers = on,
                "line" => options.line = value.to_string(),
                "factories" => options.factories = value.parse().unwrap_or(options.factories),
                "constructors" => options.constructors = value.parse().unwrap_or(options.constructors),
                "dump" => options.dump = on,
                "planner" => options.lp = value != "anneal",
                "dt" => options.dt = value.parse().unwrap_or(options.dt),
                "mid_from" => options.mid_from = value.parse().unwrap_or(options.mid_from),
                "mid_dt" => options.mid_dt = value.parse().unwrap_or(options.mid_dt),
                "coarse_from" => options.coarse_from = value.parse().unwrap_or(options.coarse_from),
                "coarse_dt" => options.coarse_dt = value.parse().unwrap_or(options.coarse_dt),
                "help" => options.help_factor = value.parse().unwrap_or(options.help_factor),
                "nanos" => options.nanos_per_factory = value.parse().unwrap_or(options.nanos_per_factory),
                "maxnanos" => options.max_nanos = value.parse().unwrap_or(options.max_nanos),
                "farmbig" => options.farm_big = value.parse().unwrap_or(options.farm_big),
                "bank" => options.bank = on,
                "reclaim" => options.reclaim = on,
                "chain" => options.chain = on,
                "assist" => options.assist_factor = value.parse().unwrap_or(options.assist_factor),
                "walk_extractor" => options.walk_extractor = value.parse().unwrap_or(options.walk_extractor),
                "walk_upgrade" => options.walk_upgrade = value.parse().unwrap_or(options.walk_upgrade),
                "walk_late" => options.walk_late = value.parse().unwrap_or(options.walk_late),
                "walk_late_from" => options.walk_late_from = value.parse().unwrap_or(options.walk_late_from),
                "walk_base" => options.walk_base = value.parse().unwrap_or(options.walk_base),
                "walk_factory" => options.walk_factory = value.parse().unwrap_or(options.walk_factory),
                "com_walk_base" => options.commander_walk_base = value.parse().unwrap_or(options.commander_walk_base),
                "com_walk_factory" => options.commander_walk_factory = value.parse().unwrap_or(options.commander_walk_factory),
                "rings" => options.rings = value.parse().unwrap_or(options.rings),
                "ring_overhead" => options.ring_overhead = value.parse().unwrap_or(options.ring_overhead),
                "commit_within" => options.commit_within = value.parse().unwrap_or(options.commit_within),
                "commit_at" => options.commit_at = value.parse().unwrap_or(options.commit_at),
                "commit_afford" => options.commit_afford = value.parse().unwrap_or(options.commit_afford),
                "plant_helpers_from" => options.plant_helpers_from = value.parse().unwrap_or(options.plant_helpers_from),
                "opening_spots" => options.opening_spots = value.parse().unwrap_or(options.opening_spots),
                "com_leash" => options.com_leash = value.parse().unwrap_or(options.com_leash),
                "com_low" => options.com_low_after = value == "after",
                _ => {}
            }
        }
        options
    }

    pub(crate) fn describe(&self) -> String {
        format!(
            "planner={}, end={:.0}, terminal={:.0}, reserve={:.2}, emargin={:.2}/{:.0}, factory_from={:.0}, budget={}, first={}, threads={}, replan={}, spots={}, soldiers={}, line={}, factories={}, constructors={}, dt={:.0}/{:.0}/{:.0}, help={:.0}, nanos={:.0}, maxnanos={:.0}, farmbig={:.0}, bank={}, reclaim={}, chain={}, assist={:.0}, walk={:.0}>{:.0}@{:.0}/{:.0}/{:.0}/{:.0}, com_walk={:.0}/{:.0}, rings={}/{:.0}, commit={:.0}/{:.2}/{:.0}, plant_helpers_from={:.0}, opening={:.0}, com_leash={:.0}, com_low={}",
            if self.lp { "lp" } else { "anneal" }, self.end, self.terminal, self.energy_reserve, self.energy_margin, self.margin_from, self.factory_from, self.budget_ms, self.first_budget_ms, self.threads, self.replan / 30, if self.all_spots { "all" } else { "half" }, if self.soldiers { "on" } else { "off" }, self.line, self.factories, self.constructors,
            self.dt, self.mid_dt, self.coarse_dt, self.help_factor, self.nanos_per_factory, self.max_nanos, self.farm_big, if self.bank { "on" } else { "off" }, if self.reclaim { "on" } else { "off" }, if self.chain { "on" } else { "off" }, self.assist_factor, self.walk_extractor, self.walk_late, self.walk_late_from, self.walk_upgrade, self.walk_base, self.walk_factory, self.commander_walk_base, self.commander_walk_factory, self.rings, self.ring_overhead, self.commit_within, self.commit_at, self.commit_afford, self.plant_helpers_from, self.opening_spots, self.com_leash, if self.com_low_after { "after" } else { "guard" }
        )
    }
}
