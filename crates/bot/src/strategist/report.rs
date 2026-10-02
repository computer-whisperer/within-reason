//! The turn reports: terse text, and after the first only what changed. A full game of identical JSON dumps
//! teaches a model to answer "no change" by rote; a report that is short when nothing happened keeps its attention
//! on what did. The front of the report describes the game (the score, the trade, the economy, the ground, the
//! opponent); the tail is the player's hands.

use super::shared::{Briefing, CellLook, Field, Hands, Side};

/// The hands' `did` lines a player's report carries at most.
const DONE_LINES: usize = 40;

/// What the last report showed, to say only what differs.
#[derive(Default)]
pub struct Seen {
    extractors: Vec<String>,
    hands: Vec<String>,
}

fn counted(items: &[(String, usize)]) -> String {
    if items.is_empty() {
        return "none".into();
    }
    items.iter().map(|(name, n)| format!("{name} {n}")).collect::<Vec<_>>().join(", ")
}

fn clock(seconds: i32) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// How much of a cell was ever seen, in words.
fn share_words(share: f32) -> &'static str {
    match share {
        s if s >= 0.9 => "all",
        s if s >= 0.5 => "most",
        s if s >= 0.15 => "part",
        _ => "a corner",
    }
}

/// Cells as runs down a column, in the order given (columns A to H, rows 1 to 8): G4, G5, G6, H2 is "G4-G6, H2".
fn cell_runs(cells: &[&CellLook]) -> String {
    let mut runs: Vec<(String, String)> = Vec::new();
    for cell in cells {
        let next_in_column = |last: &str| last.as_bytes().first() == cell.cell.as_bytes().first() && last.as_bytes().get(1).map(|row| row + 1) == cell.cell.as_bytes().get(1).copied();
        match runs.last_mut() {
            Some((_, last)) if next_in_column(last) => *last = cell.cell.clone(),
            _ => runs.push((cell.cell.clone(), cell.cell.clone())),
        }
    }
    runs.iter().map(|(first, last)| if first == last { first.clone() } else { format!("{first}-{last}") }).collect::<Vec<_>>().join(", ")
}

/// The `scouting` block (H-SCOUT-GLANCE, `docs/design/2026-10-01-scouting-glance.md`): every cell of the map by
/// when units of ours last saw it, how much of it, and what of his stood there. A cell holding something of his has
/// its own line; the others are grouped by the age of the look and written as runs; a cell not seen for six
/// minutes or never seen names its metal spots, which are what the player orders by.
fn scouting_lines(cells: &[CellLook], factory_known: bool) -> Vec<String> {
    if cells.is_empty() {
        return Vec::new();
    }
    let spots = |c: &CellLook| if c.spots.is_empty() { String::new() } else { c.spots.iter().map(|n| format!("spot_{n}")).collect::<Vec<_>>().join(", ") };
    let when = |ago: i32| if ago < 10 { "in sight now".to_string() } else { format!("seen {} ago", clock(ago)) };
    let look = |c: &CellLook| match c.ago {
        Some(ago) => format!("{} of it {}", share_words(c.seen_share), when(ago)),
        None => "never seen".to_string(),
    };
    let mut lines = vec!["scouting (each cell of the map by when units of ours last saw it; an old look says what was built there then, nothing about his army now):".to_string()];
    let holds = |c: &CellLook| !c.his.is_empty() || c.his_commander;
    for c in cells.iter().filter(|c| holds(c)) {
        let mut his: Vec<String> = c.his_commander.then(|| "his commander as last seen".to_string()).into_iter().collect();
        his.extend(c.his.iter().map(|(name, count)| format!("{count} {name}")));
        lines.push(format!("  his, in {} ({}): {}", c.cell, look(c), his.join(", ")));
    }
    let bands: [(i32, &str); 4] = [(10, "nothing of his, in sight now"), (60, "nothing of his when seen under 1 min ago"), (180, "nothing of his when seen 1 to 3 min ago"), (360, "nothing of his when seen 3 to 6 min ago")];
    let mut from = 0;
    for (until, words) in bands {
        let band: Vec<&CellLook> = cells.iter().filter(|c| !holds(c) && c.ago.is_some_and(|ago| ago >= from && ago < until)).collect();
        from = until;
        let groups: Vec<String> = ["all", "most", "part", "a corner"]
            .into_iter()
            .filter_map(|share| {
                let of: Vec<&CellLook> = band.iter().copied().filter(|c| share_words(c.seen_share) == share).collect();
                (!of.is_empty()).then(|| format!("{} ({share} of {})", cell_runs(&of), if of.len() == 1 { "it" } else { "each" }))
            })
            .collect();
        if !groups.is_empty() {
            lines.push(format!("  {words}: {}", groups.join("; ")));
        }
    }
    let named = |c: &CellLook, words: String| if c.spots.is_empty() { format!("{} ({words})", c.cell) } else { format!("{} ({words}; {})", c.cell, spots(c)) };
    let old: Vec<String> = cells.iter().filter(|c| !holds(c) && c.ago.is_some_and(|ago| ago >= 360)).map(|c| named(c, format!("{} of it, {} ago", share_words(c.seen_share), clock(c.ago.unwrap_or(0))))).collect();
    if !old.is_empty() {
        lines.push(format!("  nothing of his when seen over 6 min ago: {}", old.join(", ")));
    }
    // Never seen: his box's cells one by one with their spots (where a scout is sent), the rest as runs (at the
    // start that is the whole map).
    let never = |inside: bool| cells.iter().filter(move |c| !holds(c) && c.ago.is_none() && c.in_his_box == inside);
    let in_box: Vec<String> = never(true).map(|c| if c.spots.is_empty() { c.cell.clone() } else { format!("{} ({})", c.cell, spots(c)) }).collect();
    if !in_box.is_empty() {
        lines.push(format!("  never seen, in his box: {}", in_box.join(", ")));
    }
    let elsewhere: Vec<&CellLook> = never(false).collect();
    if !elsewhere.is_empty() {
        lines.push(format!("  never seen, elsewhere: {}", cell_runs(&elsewhere)));
    }
    if !factory_known {
        let mut least: Vec<&CellLook> = cells.iter().filter(|c| c.in_his_box).collect();
        least.sort_by(|a, b| a.seen_share.total_cmp(&b.seen_share).then(b.ago.unwrap_or(i32::MAX).cmp(&a.ago.unwrap_or(i32::MAX))));
        let least: Vec<String> = least.iter().take(4).map(|c| format!("{} ({})", c.cell, look(c))).collect();
        lines.push(format!("  no factory of his has been seen: it stands on ground we have not seen, or was built after we looked{}", if least.is_empty() { String::new() } else { format!("; the cells of his box seen least: {}", least.join(", ")) }));
    }
    lines
}

/// The lines every report opens with, changed or not: what is not shown is not weighed.
fn front(briefing: &Briefing, field: &Field, fights: &[String]) -> Vec<String> {
    let mut lines = Vec::new();
    let c = &briefing.counts;
    let s = &field.score;
    lines.push(format!(
        "score: extractors {} (most held {}, no new high for {}; free spots we can walk to {}, nearest on foot: {}; the opponent is known to hold {}) | army {} soldiers worth {} metal, {} of them within 800 of our start | opponent: we see only what our units see. Its soldiers seen in the last 3 min and not seen to die: {} worth {} metal; its army is at least that and may be much more",
        s.extractors,
        s.extractor_peak,
        clock(s.seconds_since_growth),
        s.free_spots,
        if s.next_free.is_empty() { "none".to_string() } else { s.next_free.iter().map(|(n, p, walk)| format!("#{n} {} {walk}", p.grid)).collect::<Vec<_>>().join(", ") },
        if s.enemy_spots.is_empty() { "0".to_string() } else { format!("{}, nearest our start first: {}", s.enemy_spots.len(), s.enemy_spots.iter().map(|(n, p)| format!("#{n} {}", p.grid)).collect::<Vec<_>>().join(", ")) },
        s.soldiers,
        s.army_metal,
        s.soldiers_near_home,
        s.enemy_soldiers_seen,
        s.enemy_soldiers_seen_metal
    ));
    lines.push(format!(
        "traded: in the last 3 min we lost {} metal of units and buildings and destroyed {} of theirs that we saw die (each death counted once across our seats); whole game {} lost, {} destroyed{}",
        s.traded_3_min.0, s.traded_3_min.1, s.traded.0, s.traded.1,
        s.seconds_since_turn.map_or(String::new(), |t| format!(" | {} of game time since your last turn began", clock(t)))
    ));
    // A floor on his spending: what of his we saw die, what stands or was seen alive. Read against our own income
    // per extractor it says how big his economy is at least (game 9: 37k of his seen dead by 13:24 implied 18
    // extractors while the line said "known to hold 7").
    let seconds = (s.frame / 30).max(1);
    let spent = s.traded.1 + s.enemy_soldiers_seen_metal + s.enemy_buildings_metal;
    if spent > 0 && seconds > 60 {
        let per_second = spent as f32 / seconds as f32;
        let ours_per_extractor = if s.extractors > 0 { briefing.metal.income / s.extractors as f32 } else { 0.0 };
        lines.push(format!(
            "his spending seen: at least {spent} metal by {} ({} seen dying, {} of soldiers seen in the last 3 min, {} of buildings seen standing): at least {per_second:.0} a second over the game{}; a floor, since we see little of his side",
            clock(seconds), s.traded.1, s.enemy_soldiers_seen_metal, s.enemy_buildings_metal,
            if ours_per_extractor > 0.0 { format!(", about {:.0} extractors' worth at our {ours_per_extractor:.1} an extractor", per_second / ours_per_extractor) } else { String::new() }
        ));
    }
    // The energy stall is said in the word the player's prompt keys its solar rule on (player-12 1:45: the line read
    // "energy 1/1300 (+0)" behind an assisted constructor, the prompt's "the energy line reads STALLING" never
    // appearing in the report; the engine caps spending at income once the store is empty, so the net figure reads 0).
    let e = &briefing.energy;
    let stalling = e.storage > 0.0 && e.current / e.storage < 0.05 && e.usage >= e.income * 0.9;
    lines.push(format!(
        "eco: metal {:.0} ({:+.1}/-{:.1}), energy {:.0}/{:.0} ({:+.0}){}, wind now {:.0} of this map's {:.0} to {:.0} | extractors {} constructors {} labs {} turrets {} converters {}",
        briefing.metal.current, briefing.metal.income, briefing.metal.usage, e.current, e.storage,
        e.income - e.usage, if stalling { " STALLING: the energy store is empty and everything that needs it builds slowly" } else { "" }, briefing.wind, briefing.wind_range.0, briefing.wind_range.1, c.extractors, c.constructors, c.labs, c.turrets, c.converters
    ));
    if !field.wreck_fields.is_empty() || field.resurrection_bots > 0 {
        let fields: Vec<String> = field.wreck_fields.iter().take(5).map(|(at, metal, safe)| format!("{} {metal} metal{}", at.grid, if *safe { "" } else { " (not safe)" })).collect();
        lines.push(format!("wrecks: {} | resurrection bots {}", if fields.is_empty() { "none known".to_string() } else { fields.join(", ") }, field.resurrection_bots));
    }
    // Who is in the game (H-PLAYER-SIDES; human-9: a 2v1 against a person and BARb, and the player was never told).
    if !briefing.sides.is_empty() {
        let ours: Vec<&Side> = briefing.sides.iter().filter(|s| s.ours).collect();
        let theirs: Vec<&Side> = briefing.sides.iter().filter(|s| !s.ours).collect();
        let our_seats: usize = ours.iter().map(|s| s.seats.len()).sum();
        let their_seats: usize = theirs.iter().map(|s| s.seats.len()).sum();
        let shape = if theirs.len() > 1 { format!("{our_seats} of ours against {} sides of {their_seats} seats", theirs.len()) } else { format!("a {our_seats}v{their_seats}") };
        lines.push(format!(
            "sides ({shape}): yours: {}{} | against you: {}",
            ours.iter().flat_map(|s| s.seats.iter()).cloned().collect::<Vec<_>>().join(", "),
            if our_seats > 1 { " (allies fight beside you; their units are theirs to order)" } else { "" },
            if theirs.is_empty() { "nobody".to_string() } else { theirs.iter().map(|s| s.seats.join(", ")).collect::<Vec<_>>().join(" | ") }
        ));
    }
    if briefing.seats.len() > 1 {
        let seats: Vec<String> = briefing
            .seats
            .iter()
            .map(|s| {
                let faction = field.factions.iter().find(|(team, _)| *team == s.team).map(|(_, f)| f.as_str()).unwrap_or("faction not yet known");
                format!("team {} at {} ({}{}): metal {:.0} ({:+.1}), {} extractors, {} constructors, {} soldiers", s.team, s.home.grid, faction, if s.colour.is_empty() { String::new() } else { format!(", {} to the people in the game", s.colour) }, s.metal_stored, s.metal_income, s.extractors, s.constructors, s.soldiers)
            })
            .collect();
        lines.push(format!("seats: you command {} seats, each with its own economy (people in the game know a seat by its lobby colour, so say \"our {} seat\" in chat, never t{}) | {}", seats.len(), briefing.seats[0].colour, briefing.seats[0].team, seats.join(" | ")));
    }
    // Evidence, never a guess (docs/design/2026-09-22-enemy-evidence.md): finding the opponent is the player's.
    let gone = if s.enemy_factories_gone.is_empty() { String::new() } else { format!("; seen destroyed: {}", s.enemy_factories_gone.iter().map(|(p, at)| format!("{} ({}, {}) at {}", p.grid, p.x, p.z, clock(*at))).collect::<Vec<_>>().join("; ")) };
    let boxes = if s.enemy_start_boxes.is_empty() { "the lobby gave it no start box".to_string() } else { format!("its commander was placed at 0:00 inside its lobby box, cells {}", s.enemy_start_boxes.join(" and ")) };
    lines.push(format!(
        "to win: its commander {}; its factories standing as far as we know: {}{}; {}; where it stands and builds now is known only from what our units see, and it may rebuild anywhere",
        s.enemy_commander.as_ref().map_or("has never been seen".to_string(), |(p, ago)| format!("was last seen at {} ({}, {}) {} ago{}", p.grid, p.x, p.z, clock(*ago), if s.enemy_commander_afloat { ", in the water or on ground our bots cannot walk to (it is amphibious; our soldiers are not)" } else { "" })),
        if s.enemy_factories.is_empty() {
            "none seen standing".to_string()
        } else {
            // Named by type (human-9: an Aircraft Plant seen at 2:16 was reported as "A4 (992, 2488)" and the first
            // anti-air came after the first gunship).
            s.enemy_factories
                .iter()
                .map(|(name, p)| format!("{} {name} at {} ({}, {})", crate::brain::pianist::glossary::entry(name).map_or(name.as_str(), |e| e.name.as_str()), p.grid, p.x, p.z))
                .collect::<Vec<_>>()
                .join("; ")
        },
        gone,
        boxes
    ));
    lines.extend(scouting_lines(&s.scouting, !s.enemy_factories.is_empty()));
    if !s.trend.is_empty() {
        let then = |pick: &dyn Fn(&(i32, usize, f32, u32)) -> String| s.trend.iter().map(|t| format!("{} ({} min ago)", pick(t), t.0)).collect::<Vec<_>>().join(", ");
        lines.push(format!(
            "curves: extractors {} now, {}; metal income {:.0} now, {}; army metal {} now, {}; extractors lost in the last 3 min: {}",
            s.extractors,
            then(&|t| t.1.to_string()),
            s.metal_income,
            then(&|t| format!("{:.0}", t.2)),
            s.army_metal,
            then(&|t| t.3.to_string()),
            s.extractors_lost_3_min
        ));
    }
    if !fights.is_empty() {
        lines.push(format!("fights since last turn: {}", fights.join(", ")));
    }
    lines
}

/// Enemies in sight and our extractors with enemies near them.
fn contact(briefing: &Briefing, field: &Field) -> Vec<String> {
    let mut lines = Vec::new();
    for cluster in &briefing.enemies_visible {
        lines.push(format!(
            "enemy in sight: {} at {} ({}, {}), {} from home: {}{}",
            cluster.units,
            cluster.at.grid,
            cluster.at.x,
            cluster.at.z,
            cluster.distance_from_home,
            counted(&cluster.composition),
            cluster.harming.as_ref().map_or(String::new(), |(what, metal)| format!("; {what} ({metal:.0} metal) now"))
        ));
    }
    let threatened: Vec<String> = field
        .extractors
        .iter()
        .filter(|x| x.enemies_within_600 > 0)
        .map(|x| format!("{} ({} enemies{})", x.at.grid, x.enemies_within_600, if x.turret_within_300 { ", turret" } else { ", NO turret" }))
        .collect();
    if !threatened.is_empty() {
        lines.push(format!("extractors under threat: {}", threatened.join("; ")));
    }
    lines
}

/// Our extractors, in full at first and then as gains and losses.
fn extractor_lines(seen: &mut Seen, field: &Field, full: bool, lines: &mut Vec<String>) {
    let extractors: Vec<String> = field
        .extractors
        .iter()
        .map(|x| format!("{}{} ({}, {}){}", x.spot.map_or(String::new(), |n| format!("#{n} ")), x.at.grid, x.at.x, x.at.z, if x.turret_within_300 { " T" } else { "" }))
        .collect();
    if full {
        lines.push(format!("our extractors (T = turret within 300): {}", extractors.join("; ")));
    } else {
        let gained: Vec<&String> = extractors.iter().filter(|x| !seen.extractors.contains(x)).collect();
        let lost: Vec<&String> = seen.extractors.iter().filter(|x| !extractors.contains(x)).collect();
        if !gained.is_empty() {
            lines.push(format!("extractors new or changed: {}", gained.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("; ")));
        }
        if !lost.is_empty() {
            lines.push(format!("extractors gone or changed: {}", lost.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("; ")));
        }
    }
    seen.extractors = extractors;
}

/// The player's report: the front, then its hands: every actor as the picture has it (in full at first, then the
/// ones whose entry changed), Jev's judgements when they are high, and what the hands did since the last turn.
/// The faction's whole roster once a session (docs/design/2026-09-22-full-roster.md, decision 7): one line a unit
/// from the glossary, by tier; what stands now can build the ones marked. The `units` tool has the prose.
fn roster_lines(field: &Field) -> String {
    use crate::brain::pianist::glossary;
    let now: Vec<&str> = field.buildable.iter().map(|(n, _)| n.as_str()).collect();
    let mut entries: Vec<(u8, String)> = field
        .roster
        .iter()
        .map(|(name, metal)| match glossary::entry(name) {
            Some(e) => (e.tier, format!("{}{}", e.line(name), if now.contains(&name.as_str()) { " [now]" } else { "" })),
            None => (9, format!("{name} ({metal} metal){}", if now.contains(&name.as_str()) { " [now]" } else { "" })),
        })
        .collect();
    entries.sort();
    let sides: Vec<&str> = field.factions.iter().map(|(_, side)| side.as_str()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    let mut lines = vec![format!(
        "our roster ({} units the commander reaches by build lists; [now] marks what a builder or factory standing now can build; `units` gives any entry's full prose){}:",
        entries.len(),
        if sides.len() > 1 {
            format!(
                "; the seats are of two factions ({}), so the roster holds both: a seat's builders and factories take their own faction's names (arm... or cor...) and skip the other's",
                field.factions.iter().map(|(team, side)| format!("t{team} {side}")).collect::<Vec<_>>().join(", ")
            )
        } else {
            String::new()
        }
    )];
    lines.extend(entries.into_iter().map(|(_, line)| format!("  {line}")));
    lines.join("\n")
}

pub fn player_report(seen: &mut Seen, briefing: &Briefing, field: &Field, fights: &[String], hands: &Hands, chat: &[String], full: bool) -> String {
    let mut lines = front(briefing, field, fights);
    lines.extend(contact(briefing, field));
    extractor_lines(seen, field, full, &mut lines);
    if full {
        lines.push(roster_lines(field));
    }
    // Each actor on one line, as the hands see it: the words the instructions have to speak to.
    let mut actors: Vec<String> = Vec::new();
    if let Some(entries) = hands.picture["actors"].as_object() {
        for (name, entry) in entries {
            let field = |key: &str| entry[key].as_str().map(str::to_string);
            let mut parts: Vec<String> = Vec::new();
            if let Some(units) = field("units") {
                parts.push(units);
            }
            if let Some(at) = field("at") {
                parts.push(format!("at {at}"));
            }
            if let Some(doing) = field("doing") {
                parts.push(doing);
            }
            if let Some(health) = field("health").filter(|h| !h.starts_with("full")) {
                parts.push(format!("health {health}"));
            }
            // `last_pick` and `reached` are the hands' registers; `best_rated` is what the gate last rated best for the
            // actor: a paragraph that is not being read as meant shows here within a turn.
            for key in ["allowed", "lane", "progress", "stuck", "yard", "footwork", "reinforcements", "enemies_near", "enemies_at_our_extractors", "under_fire", "unseen_shooter", "rove", "scouts_out", "detachments_out", "split_from", "nanos", "reached", "last_pick", "best_rated", "reads"] {
                match &entry[key] {
                    serde_json::Value::String(text) => parts.push(format!("{key}: {text}")),
                    serde_json::Value::Array(items) => parts.push(format!("{key}: {}", items.iter().filter_map(|i| i.as_str()).collect::<Vec<_>>().join("; "))),
                    _ => {}
                }
            }
            actors.push(format!("{name}: {}", parts.join("; ")));
        }
    }
    let changed: Vec<&String> = actors.iter().filter(|a| full || !seen.hands.contains(a)).collect();
    if !changed.is_empty() {
        lines.push(format!("your hands' actors{}:", if full { "" } else { " (those whose entry changed)" }));
        lines.extend(changed.iter().map(|a| format!("  {a}")));
    }
    let name = |line: &String| line.split(':').next().unwrap_or_default().to_string();
    let gone: Vec<String> = seen.hands.iter().map(name).filter(|n| !actors.iter().any(|a| name(a) == *n)).collect();
    if !gone.is_empty() {
        lines.push(format!("actors gone (dead, merged or split): {}", gone.join(", ")));
    }
    seen.hands = actors;
    if !chat.is_empty() {
        lines.push("chat since your last turn (people in the game; `say` answers them):".to_string());
        lines.extend(chat.iter().map(|c| format!("  {c}")));
    }
    if !hands.done.is_empty() {
        let skipped = hands.done.len().saturating_sub(DONE_LINES);
        lines.push(format!("what your hands did since your last turn{}:", if skipped > 0 { format!(" (the last {DONE_LINES} of {})", hands.done.len()) } else { String::new() }));
        lines.extend(hands.done.iter().skip(skipped).map(|d| format!("  {d}")));
    } else if !full {
        lines.push("your hands played nothing new since your last turn (every actor carried on)".into());
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{CellLook, cell_runs, scouting_lines};

    fn cell(name: &str, seen_share: f32, ago: Option<i32>) -> CellLook {
        CellLook { cell: name.to_string(), seen_share, ago, in_his_box: name.starts_with('G') || name.starts_with('H'), ..CellLook::default() }
    }

    /// The scouting block says what player-33's report did not at 5:52: the south-east cells were seen, three
    /// minutes ago, with nothing of his in them; his commander's cell holds no factory; the cells around his base
    /// were never seen.
    #[test]
    fn the_scouting_block_says_what_was_seen_empty_and_what_was_never_seen() {
        let mut h3 = cell("H3", 0.95, Some(177));
        h3.his = vec![("armllt".to_string(), 1), ("armmex".to_string(), 2), ("armrad".to_string(), 1)];
        h3.his_commander = true;
        let mut g1 = cell("G1", 0.0, None);
        g1.spots = vec![5, 10];
        let mut old = cell("F1", 0.3, Some(425));
        old.spots = vec![3];
        let cells = vec![cell("A7", 1.0, Some(0)), cell("A8", 1.0, Some(2)), cell("B8", 0.6, Some(0)), cell("C1", 0.0, None), cell("C2", 0.0, None), old, g1, cell("G6", 0.7, Some(190)), cell("G7", 0.8, Some(200)), cell("G8", 0.6, Some(205)), cell("H1", 0.0, None), cell("H2", 0.1, Some(182)), h3, cell("H8", 0.55, Some(215))];
        let lines = scouting_lines(&cells, false);
        assert!(lines[0].starts_with("scouting (each cell of the map by when units of ours last saw it"));
        assert_eq!(lines[1], "  his, in H3 (all of it seen 2:57 ago): his commander as last seen, 1 armllt, 2 armmex, 1 armrad");
        assert_eq!(lines[2], "  nothing of his, in sight now: A7-A8 (all of each); B8 (most of it)");
        assert_eq!(lines[3], "  nothing of his when seen 3 to 6 min ago: G6-G8, H8 (most of each); H2 (a corner of it)");
        assert_eq!(lines[4], "  nothing of his when seen over 6 min ago: F1 (part of it, 7:05 ago; spot_3)");
        assert_eq!(lines[5], "  never seen, in his box: G1 (spot_5, spot_10), H1");
        assert_eq!(lines[6], "  never seen, elsewhere: C1-C2");
        assert_eq!(lines[7], "  no factory of his has been seen: it stands on ground we have not seen, or was built after we looked; the cells of his box seen least: G1 (never seen), H1 (never seen), H2 (a corner of it seen 3:02 ago), H8 (most of it seen 3:35 ago)");
        assert_eq!(lines.len(), 8);
        // With a factory of his on record the last line is not said; before the first survey nothing is.
        assert_eq!(scouting_lines(&cells, true).len(), 7);
        assert!(scouting_lines(&[], false).is_empty());
    }

    #[test]
    fn cells_are_written_as_runs_down_a_column() {
        let cells: Vec<CellLook> = ["G4", "G5", "G6", "H2", "H4", "H5"].iter().map(|name| cell(name, 1.0, Some(0))).collect();
        assert_eq!(cell_runs(&cells.iter().collect::<Vec<_>>()), "G4-G6, H2, H4-H5");
    }
}
