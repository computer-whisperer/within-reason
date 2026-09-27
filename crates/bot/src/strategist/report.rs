//! The turn reports: terse text, and after the first only what changed. A full game of identical JSON dumps
//! teaches a model to answer "no change" by rote; a report that is short when nothing happened keeps its attention
//! on what did. The front of the report describes the game (the score, the trade, the economy, the ground, the
//! opponent); the tail is the player's hands.

use super::shared::{Briefing, Field, Hands, Side};

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
        s.enemy_spots_seen,
        s.soldiers,
        s.army_metal,
        s.soldiers_near_home,
        s.enemy_soldiers_seen,
        s.enemy_soldiers_seen_metal
    ));
    lines.push(format!(
        "traded: in the last 3 min we lost {} metal of units and buildings and destroyed {} of theirs that we saw die; whole game {} lost, {} destroyed{}",
        s.traded_3_min.0, s.traded_3_min.1, s.traded.0, s.traded.1,
        s.seconds_since_turn.map_or(String::new(), |t| format!(" | {} of game time since your last turn began", clock(t)))
    ));
    lines.push(format!(
        "eco: metal {:.0} ({:+.1}/-{:.1}), energy {:.0}/{:.0} ({:+.0}), wind now {:.0} of this map's {:.0} to {:.0} | extractors {} constructors {} labs {} turrets {} converters {}",
        briefing.metal.current, briefing.metal.income, briefing.metal.usage, briefing.energy.current, briefing.energy.storage,
        briefing.energy.income - briefing.energy.usage, briefing.wind, briefing.wind_range.0, briefing.wind_range.1, c.extractors, c.constructors, c.labs, c.turrets, c.converters
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
                format!("team {} at {} ({}{}): metal {:.0} ({:+.1}), {} extractors, {} soldiers", s.team, s.home.grid, faction, if s.colour.is_empty() { String::new() } else { format!(", {} to the people in the game", s.colour) }, s.metal_stored, s.metal_income, s.extractors, s.soldiers)
            })
            .collect();
        lines.push(format!("seats: you command {} seats, each with its own economy (people in the game know a seat by its lobby colour, so say \"our {} seat\" in chat, never t{}) | {}", seats.len(), briefing.seats[0].colour, briefing.seats[0].team, seats.join(" | ")));
    }
    // Evidence, never a guess (docs/design/2026-09-22-enemy-evidence.md): finding the opponent is the player's.
    let gone = if s.enemy_factories_gone.is_empty() { String::new() } else { format!("; seen destroyed: {}", s.enemy_factories_gone.iter().map(|(p, at)| format!("{} ({}, {}) at {}", p.grid, p.x, p.z, clock(*at))).collect::<Vec<_>>().join("; ")) };
    let boxes = if s.enemy_start_boxes.is_empty() { "the lobby gave it no start box".to_string() } else { format!("its commander was placed at 0:00 inside its lobby box, cells {}", s.enemy_start_boxes.join(" and ")) };
    let in_box = s.never_looked.iter().filter(|(_, _, inside)| *inside).count();
    let unseen: Vec<String> = s.never_looked.iter().filter(|(_, _, inside)| *inside).chain(s.never_looked.iter().filter(|(_, _, inside)| !*inside)).take(10).map(|(n, p, inside)| format!("#{n} {}{}", p.grid, if *inside { " (in its box)" } else { "" })).collect();
    lines.push(format!(
        "to win: its commander {}; its factories standing as far as we know: {}{}; {}; where it stands and builds now is known only from what our units see, and it may rebuild anywhere. Metal spots never within sight of a unit of ours: {} ({} inside its box){}",
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
        boxes,
        s.never_looked.len(),
        in_box,
        if unseen.is_empty() { String::new() } else { format!("; its box's first, then nearest home first: {}", unseen.join(", ")) }
    ));
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
            cluster.killing.as_ref().map_or(String::new(), |(what, metal)| format!("; killing {what} ({metal:.0} metal) now"))
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
            for key in ["allowed", "lane", "standing", "progress", "stuck", "yard", "footwork", "enemies_near", "enemies_at_our_extractors", "under_fire", "scouts_out", "detachments_out", "split_from", "nanos"] {
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
    {
        let (packet, tool) = hands.standing_counts;
        if !hands.packet_rules && full {
            lines.push(format!("your packet is read as prose every second: no standing orders are taken from it{}", if tool > 0 { format!("; {tool} from `standing` prune your hands' choices and set their defaults") } else { String::new() }));
        } else if packet + tool > 0 {
            lines.push(format!("standing orders: {packet} from your packet, {tool} from `standing`; they prune your hands' choices and set their defaults"));
            if full {
                lines.push(format!("standing orders in force:\n{}", hands.standing_text));
            }
        }
    }
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
