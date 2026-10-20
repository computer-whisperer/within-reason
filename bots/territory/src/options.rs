//! The arms of the rows, from `WITHIN_REASON_TERRITORY`: `key=value` pairs separated by commas
//! (`WITHIN_REASON_TERRITORY=end=1200,budget=400,spots=all`).

pub(crate) struct Options {
    /// The second the horizon runs to: the game's end as the arena caps it (`--max-minutes 20`).
    pub end: f64,
    /// Seconds of the income at the horizon added to the metal made (`Objective::Made`); 0 for the score.
    pub terminal: f64,
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
    /// Queues the search may write for factories and constructors not yet standing.
    pub factories: usize,
    pub constructors: usize,
    /// Write the game's `Hello` with home and the commander's type beside the match for the oracle.
    pub dump: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options { end: 1200.0, terminal: 0.0, budget_ms: 400, first_budget_ms: 1000, threads: 4, replan: 30 * 30, all_spots: false, soldiers: false, factories: 3, constructors: 14, dump: true }
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
                "budget" => options.budget_ms = value.parse().unwrap_or(options.budget_ms),
                "first" => options.first_budget_ms = value.parse().unwrap_or(options.first_budget_ms),
                "threads" => options.threads = value.parse().unwrap_or(options.threads),
                "replan" => options.replan = value.parse::<i32>().map_or(options.replan, |s| s * 30),
                "spots" => options.all_spots = value == "all",
                "soldiers" => options.soldiers = on,
                "factories" => options.factories = value.parse().unwrap_or(options.factories),
                "constructors" => options.constructors = value.parse().unwrap_or(options.constructors),
                "dump" => options.dump = on,
                _ => {}
            }
        }
        options
    }

    pub(crate) fn describe(&self) -> String {
        format!(
            "end={:.0}, terminal={:.0}, budget={}, first={}, threads={}, replan={}, spots={}, soldiers={}, factories={}, constructors={}",
            self.end, self.terminal, self.budget_ms, self.first_budget_ms, self.threads, self.replan / 30, if self.all_spots { "all" } else { "half" }, if self.soldiers { "on" } else { "off" }, self.factories, self.constructors
        )
    }
}
