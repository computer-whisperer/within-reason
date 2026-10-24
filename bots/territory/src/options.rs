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
    pub reclaim: bool,
    /// T-CHAIN: a constructor out among the spots takes extractor flows only while it has them (`chain=off` for the
    /// control).
    pub chain: bool,
    pub assist_factor: f64,
    pub walk_extractor: f64,
    pub walk_base: f64,
    pub walk_factory: f64,
    pub commander_walk_extractor: f64,
    pub commander_walk_base: f64,
    pub commander_walk_factory: f64,
    /// The program's distance rings for extractors (one: the walk a constant), and the overhead beside a ring's walk.
    pub rings: usize,
    pub ring_overhead: f64,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            end: 1200.0,
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
            factories: 4,
            constructors: 40,
            dump: true,
            lp: true,
            dt: 10.0,
            mid_from: 60.0,
            mid_dt: 60.0,
            coarse_from: 300.0,
            coarse_dt: 120.0,
            help_factor: 3.0,
            nanos_per_factory: 6.0,
            max_nanos: 0.0,
            reclaim: false,
            chain: true,
            assist_factor: 2.0,
            walk_extractor: 65.0,
            walk_base: 3.0,
            walk_factory: 40.0,
            commander_walk_extractor: 16.0,
            commander_walk_base: 5.0,
            commander_walk_factory: 9.0,
            rings: 1,
            ring_overhead: 5.0,
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
                "reclaim" => options.reclaim = on,
                "chain" => options.chain = on,
                "assist" => options.assist_factor = value.parse().unwrap_or(options.assist_factor),
                "walk_extractor" => options.walk_extractor = value.parse().unwrap_or(options.walk_extractor),
                "walk_base" => options.walk_base = value.parse().unwrap_or(options.walk_base),
                "walk_factory" => options.walk_factory = value.parse().unwrap_or(options.walk_factory),
                "com_walk_extractor" => options.commander_walk_extractor = value.parse().unwrap_or(options.commander_walk_extractor),
                "com_walk_base" => options.commander_walk_base = value.parse().unwrap_or(options.commander_walk_base),
                "com_walk_factory" => options.commander_walk_factory = value.parse().unwrap_or(options.commander_walk_factory),
                "rings" => options.rings = value.parse().unwrap_or(options.rings),
                "ring_overhead" => options.ring_overhead = value.parse().unwrap_or(options.ring_overhead),
                _ => {}
            }
        }
        options
    }

    pub(crate) fn describe(&self) -> String {
        format!(
            "planner={}, end={:.0}, terminal={:.0}, reserve={:.2}, emargin={:.2}/{:.0}, factory_from={:.0}, budget={}, first={}, threads={}, replan={}, spots={}, soldiers={}, factories={}, constructors={}, dt={:.0}/{:.0}/{:.0}, help={:.0}, nanos={:.0}, maxnanos={:.0}, reclaim={}, chain={}, assist={:.0}, walk={:.0}/{:.0}/{:.0}, com_walk={:.0}/{:.0}/{:.0}, rings={}/{:.0}",
            if self.lp { "lp" } else { "anneal" }, self.end, self.terminal, self.energy_reserve, self.energy_margin, self.margin_from, self.factory_from, self.budget_ms, self.first_budget_ms, self.threads, self.replan / 30, if self.all_spots { "all" } else { "half" }, if self.soldiers { "on" } else { "off" }, self.factories, self.constructors,
            self.dt, self.mid_dt, self.coarse_dt, self.help_factor, self.nanos_per_factory, self.max_nanos, if self.reclaim { "on" } else { "off" }, if self.chain { "on" } else { "off" }, self.assist_factor, self.walk_extractor, self.walk_base, self.walk_factory, self.commander_walk_extractor, self.commander_walk_base, self.commander_walk_factory, self.rings, self.ring_overhead
        )
    }
}
