//! Which duels to run: pairings, repetitions, and how many units make a fair fight.

/// A side of a pairing: one unit type (its count from the batch's sizing), or a mixed force written
/// `name*count+name*count`, spawned as written in list order (the first type stands in the front rank).
#[derive(Clone, Debug, PartialEq)]
pub struct Force {
    pub parts: Vec<(String, u32)>,
}

impl Force {
    pub fn parse(spec: &str) -> Force {
        let parts = spec
            .split('+')
            .map(|part| match part.split_once('*') {
                Some((name, count)) => (name.trim().to_string(), count.trim().parse().unwrap_or(1)),
                None => (part.trim().to_string(), 0),
            })
            .collect();
        Force { parts }
    }

    /// One type without a count: the batch's sizing decides how many.
    pub fn is_plain(&self) -> bool {
        self.parts.len() == 1 && self.parts[0].1 == 0
    }
}

/// Whether the control lane (`crates/micro`) drives an army's units over the director's orders (`--lane`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaneMode {
    /// The director's orders alone: the harness's behaviour before 2026-09-25.
    Off,
    /// The lane without H-MICRO-FORM: the bot's lane as it was before the formation behaviour.
    Old,
    /// The whole lane.
    On,
}

impl LaneMode {
    pub fn parse(text: &str) -> Option<LaneMode> {
        match text {
            "off" => Some(LaneMode::Off),
            "old" => Some(LaneMode::Old),
            "on" => Some(LaneMode::On),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            LaneMode::Off => "off",
            LaneMode::Old => "old",
            LaneMode::On => "on",
        }
    }
}

/// One fight to run: `x` against `y`. The repetition number decides who stands where.
#[derive(Clone, Debug)]
pub struct Job {
    pub x: String,
    pub y: String,
    pub rep: u32,
    /// Index into the batch's formations (`Batch::shapes`): how the two armies stand and advance.
    pub shape: usize,
    /// Times this job was handed out before (a match that aborts gives its running duels back).
    pub attempts: u32,
    /// The engine's `FixedRNGSeed` this duel must be fought under (`--seeds`); any match takes it when `None`.
    pub seed: Option<u32>,
}

impl Job {
    /// Even repetitions put `x` at the west end, odd ones at the east end.
    pub fn x_is_west(&self) -> bool {
        self.rep.is_multiple_of(2)
    }

    /// Repetitions 0-1 give `x` to team 0, 2-3 to team 1, and so on.
    pub fn x_team(&self) -> i32 {
        ((self.rep / 2) % 2) as i32
    }
}

/// Every unordered pair from `units`, a unit against itself included.
pub fn all_pairs(units: &[String]) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for (i, x) in units.iter().enumerate() {
        for y in &units[i..] {
            pairs.push((x.clone(), y.clone()));
        }
    }
    pairs
}

/// Every `ours` against every `theirs`, without repeating a pair that appears both ways round.
pub fn cross(ours: &[String], theirs: &[String]) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    for x in ours {
        for y in theirs {
            if !pairs.iter().any(|(a, b)| (a == x && b == y) || (a == y && b == x)) {
                pairs.push((x.clone(), y.clone()));
            }
        }
    }
    pairs
}

/// `reps` jobs per pair and shape (and per seed, when `seeds` names any), in an order that spreads a pair's
/// repetitions over sites and matches.
pub fn jobs(pairs: &[(String, String)], reps: u32, shapes: usize, seeds: &[u32]) -> Vec<Job> {
    let seeds: Vec<Option<u32>> = if seeds.is_empty() { vec![None] } else { seeds.iter().copied().map(Some).collect() };
    let mut jobs: Vec<Job> = (seeds.iter().copied())
        .flat_map(|seed| {
            (0..reps).flat_map(move |rep| {
                (0..shapes).flat_map(move |shape| {
                    pairs.iter().map(move |(x, y)| Job { x: x.clone(), y: y.clone(), rep, shape, attempts: 0, seed })
                })
            })
        })
        .collect();
    // A fixed shuffle (the same plan every run): long fights and short ones mix, so sites stay evenly loaded.
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    for i in (1..jobs.len()).rev() {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        jobs.swap(i, (state >> 33) as usize % (i + 1));
    }
    jobs
}

/// How many of each unit fight.
#[derive(Clone, Copy, Debug)]
pub enum Sizing {
    /// About this much metal a side, the two sides' totals as close as whole units allow.
    EqualMetal(f32),
    EqualCount(u32),
}

impl Sizing {
    pub fn counts(self, cost_x: f32, cost_y: f32) -> (u32, u32) {
        match self {
            Sizing::EqualCount(n) => (n, n),
            Sizing::EqualMetal(budget) => {
                // Among armies worth 60-125% of the budget, take the one whose two totals differ least (in steps
                // of 2%); on a tie, the one nearest the budget. A unit dearer than that fights alone.
                let mut best = (1, (cost_x / cost_y).round().max(1.0) as u32);
                let mut best_score = (f32::INFINITY, f32::INFINITY);
                for n in 1..=400u32 {
                    let value = n as f32 * cost_x;
                    let m = (value / cost_y).round().max(1.0);
                    let other = m * cost_y;
                    if value.min(other) < 0.6 * budget || value.max(other) > 1.25 * budget {
                        continue;
                    }
                    let mismatch = ((value - other).abs() / value.max(other) * 50.0).round();
                    let score = (mismatch, (value - budget).abs());
                    if score < best_score {
                        best_score = score;
                        best = (n, m as u32);
                    }
                }
                best
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_force_is_plain_or_mixed() {
        assert!(Force::parse("armstump").is_plain());
        let mixed = Force::parse("armflash*8+armstump*6");
        assert!(!mixed.is_plain());
        assert_eq!(mixed.parts, vec![("armflash".to_string(), 8), ("armstump".to_string(), 6)]);
    }
}
