//! Unit context (docs/design/2026-11-04-unit-context.md, H-PLAYER-UNIT-CONTEXT): what the player is told about
//! units, and when. The prompt's unit tables are generated here from the glossary for the factions in the game, each
//! unit at a level the rules below give it from the game's state as it stood when the session spawned (`refresh`,
//! the snapshot the prompt reads until the next spawn, so the prompt's digest holds still within a session); between
//! spawns the report introduces a type the first time it is seen of his or buildable of ours (`fresh_lines`), and the
//! next session's tables carry it. Nothing the player has met is ever dropped from a later prompt (`introduced`).

use std::collections::BTreeSet;
use std::sync::Mutex;

use crate::brain::pianist::glossary::{self, Entry};
use crate::strategist::shared::Field;

const FACTIONS: [&str; 3] = ["armada", "cortex", "legion"];
const DOMAINS: [&str; 5] = ["economy", "land", "structures", "air", "sea"];

/// The game as the rules read it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct State {
    /// The factions of our seats, our allies' and our enemies', lowercase; a seat on side Random counts as all three
    /// until the field names its faction.
    pub ours: BTreeSet<String>,
    pub allies: BTreeSet<String>,
    pub enemies: BTreeSet<String>,
    /// Percent of the map's cells under water.
    pub water_share: f32,
    pub minute: f32,
    /// What our standing builders and factories can build now, by internal name.
    pub buildable: BTreeSet<String>,
    /// Types of his seen so far, accumulated over the game.
    pub seen: BTreeSet<String>,
    /// Types whose words the player has had, by a table row or a report line.
    pub introduced: BTreeSet<String>,
    /// Units the brief's prose names, when `WITHIN_REASON_UNIT_REFERENCED` is set; empty otherwise.
    pub referenced: BTreeSet<String>,
}

/// What the prompt carries of a unit: nothing, or its row with the gloss. A prose level (the row and the paragraph
/// behind `units`) is the design's hook for the "prose ahead of the articles" question (§3), added when a rule needs it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Omitted,
    Row,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);
static SNAPSHOT: Mutex<Option<State>> = Mutex::new(None);

fn with_state<T>(f: impl FnOnce(&mut State) -> T) -> T {
    let mut guard = STATE.lock().unwrap();
    f(guard.get_or_insert_with(State::default))
}

/// The seats' factions and the map's water, from the hello (once per seat; the sets merge).
pub fn note(hello: &bot_protocol::Hello) {
    let heights = &hello.terrain.heights;
    let under = heights.iter().filter(|h| **h < 0).count();
    let water_share = if heights.is_empty() { 0.0 } else { under as f32 / heights.len() as f32 * 100.0 };
    with_state(|s| {
        s.water_share = water_share;
        // Gaia (the engine's neutral team, in the list with no side) reaches no faction: unit-context-apex-1 read its
        // empty side as Random and carried all three factions' tables in a mirror.
        for team in hello.teams.iter().filter(|t| t.controller != bot_protocol::Controller::Gaia) {
            let into = if team.team == hello.team {
                &mut s.ours
            } else if team.ally_team == hello.ally_team {
                &mut s.allies
            } else {
                &mut s.enemies
            };
            for f in factions_of_side(&team.side) {
                into.insert(f);
            }
        }
    });
}

/// A lobby side's factions: its own, all three for Random, none for an empty side (Gaia's).
fn factions_of_side(side: &str) -> Vec<String> {
    let low = side.trim().to_ascii_lowercase();
    if FACTIONS.contains(&low.as_str()) {
        vec![low]
    } else if low.is_empty() {
        Vec::new()
    } else {
        FACTIONS.iter().map(|f| f.to_string()).collect()
    }
}

/// The game at a report: the minute, what we can build, what of his is seen, and our seats' factions once known
/// (a seat on Random names its faction from its first unit).
pub fn note_field(frame: i32, field: &Field) {
    with_state(|s| {
        s.minute = frame as f32 / 1800.0;
        s.buildable = field.buildable.iter().map(|(n, _)| n.clone()).collect();
        s.seen.extend(field.enemy_types_seen.iter().cloned());
        let named: BTreeSet<String> = field.factions.iter().map(|(_, f)| f.to_ascii_lowercase()).filter(|f| FACTIONS.contains(&f.as_str())).collect();
        if !named.is_empty() {
            s.ours = named;
        }
    });
}

/// The lines a report carries for the types it meets first: his seen and ours buildable now, each once in the game.
pub fn fresh_lines(frame: i32, field: &Field) -> Vec<String> {
    note_field(frame, field);
    with_state(|s| {
        let mut fresh: Vec<String> = Vec::new();
        for name in field.buildable.iter().map(|(n, _)| n).chain(field.enemy_types_seen.iter()) {
            if s.introduced.insert(name.clone()) {
                fresh.push(match glossary::entry(name) {
                    Some(e) => e.line(name),
                    None => format!("{name} (not in the glossary)"),
                });
            }
        }
        fresh.sort();
        fresh
    })
}

/// Takes the snapshot a session's prompt reads: the state now, with every unit the rules show marked introduced.
/// Called by `Launch::spawn`, before the prompt is built.
pub fn refresh() {
    let state = with_state(|s| {
        if std::env::var_os("WITHIN_REASON_UNIT_REFERENCED").is_some() {
            s.referenced = referenced_by_the_brief();
        }
        let shown: Vec<String> = glossary::entries().filter(|(name, e)| level(s, name, e) >= Level::Row).map(|(n, _)| n.to_string()).collect();
        s.introduced.extend(shown);
        s.clone()
    });
    let text = part_of(&state);
    eprintln!(
        "[strategist] unit context: {} rows for {} (minute {:.1}, {} seen of his, {} introduced), {} chars",
        text.lines().filter(|l| l.starts_with("| ") && !l.starts_with("| unit")).count(),
        factions_present(&state).join(", "),
        state.minute,
        state.seen.len(),
        state.introduced.len(),
        text.len()
    );
    *SNAPSHOT.lock().unwrap() = Some(state);
}

/// The units the brief's documents name, by internal or display name (the flag's rule).
fn referenced_by_the_brief() -> BTreeSet<String> {
    let Ok(documents) = crate::texts::documents(crate::texts::manifest(&crate::texts::brief())) else { return BTreeSet::new() };
    let mut text = String::new();
    for document in documents.iter().filter(|d| !d.starts_with("generated:")) {
        if let Ok(t) = crate::texts::read_required(document) {
            text.push_str(&crate::texts::strip(&t));
            text.push('\n');
        }
    }
    referenced_in(&text)
}

fn referenced_in(text: &str) -> BTreeSet<String> {
    // Every word, and each word with a plural ending taken off (Blitzes, Stouts), so a display name matches either way.
    let mut words: BTreeSet<String> = BTreeSet::new();
    for word in text.split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '-') {
        words.insert(word.to_string());
        if let Some(stem) = word.strip_suffix("es").or_else(|| word.strip_suffix('s')) {
            words.insert(stem.to_string());
        }
    }
    glossary::entries()
        .filter(|(name, e)| words.contains(*name) || (e.name.len() >= 4 && words.contains(&e.name)))
        .map(|(n, _)| n.to_string())
        .collect()
}

/// The level of one unit under the rules (the design's §3, in order; first match wins).
pub fn level(s: &State, name: &str, e: &Entry) -> Level {
    let faction = e.faction.to_ascii_lowercase();
    let reachable = |set: &BTreeSet<String>| set.contains(&faction);
    if !(reachable(&s.ours) || reachable(&s.allies) || reachable(&s.enemies)) {
        return Level::Omitted;
    }
    if s.introduced.contains(name) || s.seen.contains(name) || s.buildable.contains(name) {
        return Level::Row;
    }
    if reachable(&s.ours) {
        return Level::Row;
    }
    let domain = domain_of(&e.class);
    if domain == "sea" && s.water_share < 3.0 {
        return Level::Omitted;
    }
    if s.referenced.contains(name) {
        return Level::Row;
    }
    if e.tier <= 1 && matches!(domain, "land" | "economy" | "structures") {
        return Level::Row;
    }
    if domain == "air" {
        return if s.minute >= 8.0 || air_plant_stands(s) { Level::Row } else { Level::Omitted };
    }
    if e.tier == 2 {
        return if s.minute >= 12.0 || his_tier2_seen(s) { Level::Row } else { Level::Omitted };
    }
    if e.tier >= 3 {
        return if s.minute >= 20.0 || his_gantry_seen(s) { Level::Row } else { Level::Omitted };
    }
    Level::Row
}

fn air_plant_stands(s: &State) -> bool {
    let is_air_plant = |n: &String| glossary::entry(n).is_some_and(|e| e.class.contains("aircraft factory") || e.class.contains("seaplane platform"));
    s.buildable.iter().any(|n| glossary::entry(n).is_some_and(|e| domain_of(&e.class) == "air")) || s.seen.iter().any(is_air_plant)
}

fn his_tier2_seen(s: &State) -> bool {
    s.seen.iter().any(|n| glossary::entry(n).is_some_and(|e| e.tier >= 2))
}

fn his_gantry_seen(s: &State) -> bool {
    s.seen.iter().any(|n| glossary::entry(n).is_some_and(|e| e.tier >= 3 || e.class.contains("gantry")))
}

/// A unit's domain by words of its class, as `run/fact_units.py` sorts the tables; first match wins.
pub fn domain_of(class: &str) -> &'static str {
    let low = class.to_ascii_lowercase();
    // Mobile anti-nuke and mobile support fight on land; the mine family stands still.
    if low.starts_with("mobile ") {
        return "land";
    }
    const WORDS: [(&str, &[&str]); 4] = [
        ("economy", &["constructor", "factory", "shipyard", "gantry", "extractor", "solar", "wind", "tidal", "geothermal", "fusion", "converter", "storage", "construction turret", "commander", "resurrection", "assist bot"]),
        ("air", &["aircraft", "bomber", "gunship", "plane", "drone"]),
        ("sea", &["ship", "submarine", "cruiser", "destroyer", "frigate", "boat", "battleship", "flagship"]),
        ("structures", &["turret", "tower", "launcher", "silo", "wall", "shield", "anti-nuke", "facility", "station", "platform", "decoy building", "hive", "mine"]),
    ];
    for (domain, words) in WORDS {
        if words.iter().any(|w| low.contains(w)) {
            return domain;
        }
    }
    "land"
}

fn factions_present(s: &State) -> Vec<&'static str> {
    FACTIONS.iter().copied().filter(|f| s.ours.contains(*f) || s.allies.contains(*f) || s.enemies.contains(*f)).collect()
}

fn title(faction: &str) -> &str {
    match faction {
        "armada" => "Armada",
        "cortex" => "Cortex",
        "legion" => "Legion",
        other => other,
    }
}

/// The part the prompt carries: the tables from the snapshot, or from the pre-game state of a seat whose sides are
/// unknown (`bot --print-prompt` before any hello: all three factions at the tier-1 level).
pub fn part() -> String {
    let snapshot = SNAPSHOT.lock().unwrap().clone();
    part_of(&snapshot.unwrap_or_else(pre_game))
}

/// The state `bot --print-prompt` shows without a hello: every faction his at minute 0, or the sides
/// `WITHIN_REASON_UNIT_SIDES=ours:his[,his]` (`armada:cortex`) for measuring a game's tables.
fn pre_game() -> State {
    let mut s = State::default();
    match std::env::var("WITHIN_REASON_UNIT_SIDES") {
        Ok(sides) if sides.contains(':') => {
            let (ours, his) = sides.split_once(':').unwrap_or_default();
            s.ours = ours.split(',').flat_map(factions_of_side).collect();
            s.enemies = his.split(',').flat_map(factions_of_side).collect();
        }
        _ => s.enemies = FACTIONS.iter().map(|f| f.to_string()).collect(),
    }
    s
}

const LEGEND: &str = "Metal and energy are the costs. Build time is in build units: a builder with build power P finishes the unit in build time / P seconds, and draws the costs over those seconds. Speed is elmos a second; range and sight are elmos; dps is damage a second against ground targets. The ratios are per metal of cost. The last column is what the unit is for, in our words.";

fn intro(domain: &str) -> &'static str {
    match domain {
        "economy" => "What produces, stores and builds: extractors, energy makers, converters, storages, construction turrets, factories and constructors, with the commanders.",
        "land" => "The land fighters: bots, tanks, mechs, hovercraft, and the mobile support beside them (radar, jammers, minelayers; mines and bombs are under structures).",
        "structures" => "What stands still and fights, sees or hides: defence and anti-air turrets, artillery and long-range cannon, missile launchers and silos, anti-nuke, shields, walls, radar, jammers, cameras, sonar, mines.",
        "air" => "What flies: fighters, bombers, gunships, radar and transport planes, drones.",
        _ => "What floats or dives: warships of every class and submarines.",
    }
}

pub fn part_of(s: &State) -> String {
    let present = factions_present(s);
    let mut out = String::from("## Unit facts\n\n");
    out.push_str(&format!(
        "The units of the factions in this game ({}), one table a domain and faction, from the game's unit definitions. {LEGEND} \
Held back until the game reaches them: the other sides' tier-2 rows (shown from minute 12, or when a tier-2 factory or unit of his is seen), their air (from minute 8, or when an aircraft plant stands on either side), tier 3 (from minute 20) and the sea on a dry map; every unit of your own faction is here. \
A type first seen of his or first buildable of ours arrives in the report with one line (`new units this report`), and the `units` tool gives any name's full entry, ours or theirs.\n",
        if present.is_empty() { "none known yet".to_string() } else { present.iter().map(|f| title(f)).collect::<Vec<_>>().join(", ") }
    ));
    for domain in DOMAINS {
        let mut sections: Vec<String> = Vec::new();
        for faction in &present {
            let all: Vec<(&str, &Entry)> = glossary::entries().filter(|(_, e)| e.faction.eq_ignore_ascii_case(faction) && domain_of(&e.class) == domain).collect();
            let mut shown: Vec<(&str, &Entry)> = all.iter().copied().filter(|(n, e)| level(s, n, e) >= Level::Row).collect();
            if shown.is_empty() {
                continue;
            }
            shown.sort_by(|a, b| (a.1.tier, a.1.class.as_str(), a.1.metal as u32, a.0).cmp(&(b.1.tier, b.1.class.as_str(), b.1.metal as u32, b.0)));
            let heading = if shown.len() == all.len() { format!("{} units", all.len()) } else { format!("{} of {} units", shown.len(), all.len()) };
            sections.push(format!("### {}: {domain} ({heading})\n\n{}\n", title(faction), table(domain, &shown)));
        }
        if !sections.is_empty() {
            out.push_str(&format!("\n{}\n\n{}", intro(domain), sections.join("\n")));
        }
    }
    out
}

fn fmt(v: f32) -> String {
    if v == 0.0 {
        String::new()
    } else if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v:.1}")
    }
}

fn opt(v: Option<f32>) -> String {
    v.filter(|x| *x != 0.0).map(fmt).unwrap_or_default()
}

fn ratio(a: f32, b: f32) -> String {
    if a != 0.0 && b != 0.0 { format!("{:.2}", a / b) } else { String::new() }
}

fn table(domain: &str, units: &[(&str, &Entry)]) -> String {
    let (head, row): (Vec<&str>, Box<dyn Fn(&str, &Entry) -> Vec<String>>) = match domain {
        "economy" => (
            vec!["unit", "name", "tier", "class", "from", "metal", "energy", "build time", "health", "speed", "build power", "energy make", "energy upkeep", "e storage", "m storage", "flags", "what it is for"],
            Box::new(|k, u| {
                vec![k.into(), u.name.clone(), u.tier.to_string(), u.class.clone(), u.made_by.join("/"), fmt(u.metal), fmt(u.energy), fmt(u.build_time), fmt(u.health), fmt(u.speed), fmt(u.build_power), opt(u.energy_make), opt(u.energy_upkeep), opt(u.energy_storage), opt(u.metal_storage), u.flags.join(", "), u.gloss.clone()]
            }),
        ),
        "structures" => (
            vec!["unit", "name", "tier", "class", "metal", "energy", "build time", "health", "range", "dps", "dps vs air", "sight", "dps per metal", "flags", "what it is for"],
            Box::new(|k, u| vec![k.into(), u.name.clone(), u.tier.to_string(), u.class.clone(), fmt(u.metal), fmt(u.energy), fmt(u.build_time), fmt(u.health), fmt(u.range), opt(u.dps), opt(u.dps_air), fmt(u.sight), ratio(u.dps.unwrap_or(0.0), u.metal), u.flags.join(", "), u.gloss.clone()]),
        ),
        _ => (
            vec!["unit", "name", "tier", "class", "from", "metal", "energy", "build time", "health", "speed", "range", "dps", "dps vs air", "sight", "health per metal", "dps per metal", "flags", "what it is for"],
            Box::new(|k, u| {
                vec![k.into(), u.name.clone(), u.tier.to_string(), u.class.clone(), u.made_by.join("/"), fmt(u.metal), fmt(u.energy), fmt(u.build_time), fmt(u.health), fmt(u.speed), fmt(u.range), opt(u.dps), opt(u.dps_air), fmt(u.sight), ratio(u.health, u.metal), ratio(u.dps.unwrap_or(0.0), u.metal), u.flags.join(", "), u.gloss.clone()]
            }),
        ),
    };
    let mut lines = vec![format!("| {} |", head.join(" | ")), format!("|{}", "---|".repeat(head.len()))];
    for (k, u) in units {
        lines.push(format!("| {} |", row(k, u).iter().map(|c| c.replace('|', "/")).collect::<Vec<_>>().join(" | ")));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(ours: &[&str], enemies: &[&str]) -> State {
        State { ours: ours.iter().map(|s| s.to_string()).collect(), enemies: enemies.iter().map(|s| s.to_string()).collect(), ..State::default() }
    }

    fn lvl(s: &State, name: &str) -> Level {
        level(s, name, glossary::entry(name).expect(name))
    }

    #[test]
    fn a_faction_nobody_reaches_is_omitted_and_ours_is_whole() {
        let s = state(&["armada"], &["cortex"]);
        assert_eq!(lvl(&s, "leggob"), Level::Omitted);
        assert_eq!(lvl(&s, "armpw"), Level::Row);
        assert_eq!(lvl(&s, "armbull"), Level::Row, "our tier 2 is a row from the start");
        assert_eq!(lvl(&s, "armbanth"), Level::Row, "our tier 3 too");
    }

    #[test]
    fn his_late_units_wait_for_the_clock_or_a_sighting() {
        let mut s = state(&["armada"], &["cortex"]);
        assert_eq!(lvl(&s, "corak"), Level::Row, "his tier-1 raider");
        assert_eq!(lvl(&s, "corllt"), Level::Row, "his tier-1 turret");
        assert_eq!(lvl(&s, "corsumo"), Level::Omitted, "his tier-2 mech before minute 12");
        assert_eq!(lvl(&s, "corshad"), Level::Omitted, "his bomber before minute 8");
        assert_eq!(lvl(&s, "corpship"), Level::Omitted, "his ship on a dry map");
        s.minute = 8.0;
        assert_eq!(lvl(&s, "corshad"), Level::Row);
        assert_eq!(lvl(&s, "corsumo"), Level::Omitted);
        s.seen.insert("coralab".into());
        assert_eq!(lvl(&s, "corsumo"), Level::Row, "his advanced lab seen");
        s.water_share = 20.0;
        assert_eq!(lvl(&s, "corpship"), Level::Row);
        assert_eq!(lvl(&s, "corjugg"), Level::Omitted, "tier 3 before minute 20 and no gantry");
        s.minute = 20.0;
        assert_eq!(lvl(&s, "corjugg"), Level::Row);
    }

    #[test]
    fn a_seen_or_buildable_type_is_a_row_whatever_the_rules_say() {
        let mut s = state(&["armada"], &["cortex"]);
        s.seen.insert("corshad".into());
        assert_eq!(lvl(&s, "corshad"), Level::Row);
        s.introduced.insert("corsumo".into());
        assert_eq!(lvl(&s, "corsumo"), Level::Row);
    }

    #[test]
    fn a_random_side_counts_as_every_faction_until_named() {
        assert_eq!(factions_of_side("Random"), vec!["armada", "cortex", "legion"]);
        assert_eq!(factions_of_side("Cortex"), vec!["cortex"]);
        assert!(factions_of_side("").is_empty(), "Gaia's empty side is nobody's faction");
        let mut s = state(&["armada", "cortex", "legion"], &["cortex"]);
        let field = Field { factions: vec![(0, "Legion".into())], ..Field::default() };
        // The field names the seat's faction; the state keeps it alone.
        s.minute = 0.0;
        let mut copy = s.clone();
        let named: BTreeSet<String> = field.factions.iter().map(|(_, f)| f.to_ascii_lowercase()).collect();
        copy.ours = named;
        assert_eq!(lvl(&copy, "armpw"), Level::Omitted, "Armada is nobody's once the seat is Legion against Cortex");
        assert_eq!(lvl(&copy, "leggob"), Level::Row);
    }

    #[test]
    fn the_part_holds_the_factions_present_with_the_gloss_and_counts_what_is_held_back() {
        let s = state(&["armada"], &["cortex"]);
        let text = part_of(&s);
        assert!(text.contains("### Armada: land (51 units)"), "{text}");
        assert!(text.contains("### Cortex: land ("), "{text}");
        assert!(!text.contains("### Legion"), "{text}");
        assert!(text.contains("| armstump | Stout | 1 | line tank | armvp |"), "{text}");
        assert!(text.contains("the vehicle line unit, loses to Maces and Pounders"), "the gloss column: {text}");
        assert!(!text.contains("| corsumo |"), "his tier-2 mech held back: {text}");
        assert!(!text.contains("### Cortex: sea"), "the sea on a dry map: {text}");
        assert!(text.contains("### Cortex: land ("), "{text}");
        let cortex_land = text.split("### Cortex: land (").nth(1).unwrap().split(')').next().unwrap();
        assert!(cortex_land.ends_with(" of 52 units"), "{cortex_land}");
    }

    #[test]
    fn the_domains_sort_as_the_generator_does() {
        assert_eq!(domain_of("line tank"), "land");
        assert_eq!(domain_of("constructor vehicle"), "economy");
        assert_eq!(domain_of("aircraft factory"), "economy");
        assert_eq!(domain_of("fighter aircraft"), "air");
        assert_eq!(domain_of("defence turret"), "structures");
        assert_eq!(domain_of("mobile anti-nuke"), "land");
        assert_eq!(domain_of("destroyer"), "sea");
        assert_eq!(domain_of("mine"), "structures");
    }

    #[test]
    fn the_brief_scan_finds_internal_and_display_names() {
        let found = referenced_in("Two Blitzes and a Stout meet corak at the spot; armpw too.");
        assert!(found.contains("armflash") && found.contains("armstump") && found.contains("corak") && found.contains("armpw"), "{found:?}");
        assert!(!found.contains("corsumo"));
    }
}
