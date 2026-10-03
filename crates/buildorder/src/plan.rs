//! A build order: one queue per builder. Queue 0 is the commander's, then one per factory in the order the factories
//! finish, then one per constructor in the order the constructors leave the factory.

use crate::units::Units;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Item {
    /// Build this unit (index into the unit table).
    Build(usize),
    /// Mobile builders only: walk home and add build power to the first factory for `Scenario::assist_chunk` seconds.
    Assist,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Step {
    pub item: Item,
    /// Where to build. `None` lets the simulator choose: the nearest free metal spot for an extractor, a site in the
    /// base for everything else.
    pub site: Option<(f64, f64)>,
}

impl Step {
    pub fn build(unit: usize) -> Step {
        Step { item: Item::Build(unit), site: None }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub commander: Vec<Step>,
    pub factories: Vec<Vec<Step>>,
    pub constructors: Vec<Vec<Step>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueKind {
    Commander,
    Factory,
    Constructor,
}

impl Plan {
    pub fn empty(factories: usize, constructors: usize) -> Plan {
        Plan { commander: Vec::new(), factories: vec![Vec::new(); factories], constructors: vec![Vec::new(); constructors] }
    }

    pub fn queue_count(&self) -> usize {
        1 + self.factories.len() + self.constructors.len()
    }

    pub fn kind(&self, queue: usize) -> QueueKind {
        match queue {
            0 => QueueKind::Commander,
            q if q <= self.factories.len() => QueueKind::Factory,
            _ => QueueKind::Constructor,
        }
    }

    pub fn factory_queue(&self, nth: usize) -> usize {
        1 + nth
    }

    pub fn constructor_queue(&self, nth: usize) -> usize {
        1 + self.factories.len() + nth
    }

    /// A queue past the plan's constructors is empty: the simulator numbers a state's builders itself, and a plan
    /// sized from another count of them (player-43: a constructor in the state, none in the plan) must not panic
    /// (the panic killed the player's tool server for the rest of the game).
    pub fn queue(&self, queue: usize) -> &Vec<Step> {
        static EMPTY: Vec<Step> = Vec::new();
        match queue {
            0 => &self.commander,
            q if q <= self.factories.len() => &self.factories[q - 1],
            q => self.constructors.get(q - 1 - self.factories.len()).unwrap_or(&EMPTY),
        }
    }

    /// The queue, grown to exist when it is past the plan's constructors.
    pub fn queue_mut(&mut self, queue: usize) -> &mut Vec<Step> {
        let factories = self.factories.len();
        if queue > factories {
            while self.constructors.len() < queue - factories {
                self.constructors.push(Vec::new());
            }
        }
        match queue {
            0 => &mut self.commander,
            q if q <= factories => &mut self.factories[q - 1],
            q => &mut self.constructors[q - 1 - factories],
        }
    }

    /// Text form, one queue per line: `com: mex mex win lab`, `fac0: pw ck`, `con0: mex@2136,2136 assist`.
    /// Unit names are written without the faction prefix.
    pub fn to_text(&self, units: &Units) -> String {
        let mut out = String::new();
        for queue in 0..self.queue_count() {
            let label = match self.kind(queue) {
                QueueKind::Commander => "com".to_string(),
                QueueKind::Factory => format!("fac{}", queue - 1),
                QueueKind::Constructor => format!("con{}", queue - 1 - self.factories.len()),
            };
            if self.queue(queue).is_empty() {
                continue;
            }
            let words: Vec<String> = self
                .queue(queue)
                .iter()
                .map(|step| {
                    let name = match step.item {
                        Item::Build(unit) => units.list[unit].name[3..].to_string(),
                        Item::Assist => "assist".to_string(),
                    };
                    match step.site {
                        Some((x, z)) => format!("{name}@{x:.0},{z:.0}"),
                        None => name,
                    }
                })
                .collect();
            out.push_str(&format!("{label}: {}\n", words.join(" ")));
        }
        out
    }

    /// Inverse of `to_text`; `side` is the faction prefix (`arm`, `cor`). Lines starting with `#` are comments.
    pub fn from_text(text: &str, side: &str, units: &Units) -> Result<Plan, String> {
        let mut plan = Plan::empty(0, 0);
        for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let (label, rest) = line.split_once(':').ok_or_else(|| format!("no ':' in {line:?}"))?;
            let mut steps = Vec::new();
            for word in rest.split_whitespace() {
                let (name, site) = match word.split_once('@') {
                    Some((name, at)) => {
                        let (x, z) = at.split_once(',').ok_or_else(|| format!("bad site in {word:?}"))?;
                        let parse = |s: &str| s.parse::<f64>().map_err(|_| format!("bad site in {word:?}"));
                        (name, Some((parse(x)?, parse(z)?)))
                    }
                    None => (word, None),
                };
                let item = if name == "assist" {
                    Item::Assist
                } else {
                    Item::Build(units.index(name).or_else(|| units.index(&format!("{side}{name}"))).ok_or_else(|| format!("unknown unit {name}"))?)
                };
                steps.push(Step { item, site });
            }
            let slot = |prefix: &str| label.strip_prefix(prefix).and_then(|n| n.parse::<usize>().ok());
            if label == "com" {
                plan.commander = steps;
            } else if let Some(n) = slot("fac") {
                plan.factories.resize(plan.factories.len().max(n + 1), Vec::new());
                plan.factories[n] = steps;
            } else if let Some(n) = slot("con") {
                plan.constructors.resize(plan.constructors.len().max(n + 1), Vec::new());
                plan.constructors[n] = steps;
            } else {
                return Err(format!("unknown queue {label:?}"));
            }
        }
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A queue past the plan's constructors reads as empty and is grown when written: the simulator numbers a
    /// state's builders itself (player-43's `plan` call panicked here and took the tool server with it).
    #[test]
    fn a_queue_past_the_constructors_is_empty_and_grows_when_written() {
        let mut plan = Plan::empty(1, 0);
        assert_eq!(plan.queue_count(), 2);
        assert!(plan.queue(2).is_empty() && plan.queue(5).is_empty());
        plan.queue_mut(3).push(Step::build(0));
        assert_eq!(plan.queue_count(), 4);
        assert_eq!(plan.queue(3).len(), 1);
        assert!(plan.queue(2).is_empty());
    }
}
