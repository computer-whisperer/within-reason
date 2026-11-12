//! The turn reports: terse text, and after the first only what changed. A full game of identical JSON dumps
//! teaches a model to answer "no change" by rote; a report that is short when nothing happened keeps its attention
//! on what did. The front of the report describes the game (the score, the trade, the economy, the ground, the
//! opponent); the tail is the player's hands.

use crate::brain::pianist::flow_words;
use serde_json::Value;
use super::shared::{Bins, Briefing, CellLook, Field, Hands, Side, Traded};

/// The hands' `did` lines a player's report carries at most.
const DONE_LINES: usize = 40;

/// What the last report showed, to say only what differs.
#[derive(Default)]
pub struct Seen {
    /// The frame the newest part of the commander's direction shown in full came into force.
    pub direction: i32,
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

/// Cells as runs down a column, in the order given (column by column, north to south): G4, G5, G6, H2 is
/// "G4-G6, H2"; a row of two digits (E10) reads as the picture writes it.
fn cell_runs(cells: &[&CellLook]) -> String {
    let column_row = |name: &str| -> Option<(char, u32)> {
        let mut chars = name.chars();
        let column = chars.next()?;
        Some((column, chars.as_str().parse().ok()?))
    };
    let mut runs: Vec<(String, String)> = Vec::new();
    for cell in cells {
        let next_in_column = |last: &str| matches!((column_row(last), column_row(&cell.cell)), (Some((a, r)), Some((b, s))) if a == b && r + 1 == s);
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
/// A constructor's internal name from its entry's `is` words ("Construction Vehicle (armcv; constructor vehicle,
/// 135 metal): ..." gives "armcv"), for the actor line's suffix: an advanced constructor is told from the others on
/// every turn, not only on its first appearance.
fn constructor_kind(is: &str) -> Option<String> {
    let (_, rest) = is.split_once('(')?;
    let kind = rest.split([';', ')']).next().unwrap_or_default().trim();
    (!kind.is_empty()).then(|| kind.to_string())
}

/// On a metal map, the `metal` line (the metal-maps design §1.4): each cell's share of the map's metal and the
/// extractors standing in it, ours / allied / his as seen; where the metal lies evenly over every cell (a plate)
/// only the cells with extractors are named. Nothing on a spot map.
/// Cells whose metal shares are all within this ratio of each other hold the metal "evenly" (a plate): no cell's
/// share is worth saying (the same ratio as `terrain::sheet`'s).
const EVEN_METAL: f32 = 1.5;

fn metal_line(cells: &[CellLook]) -> Option<String> {
    let with_metal: Vec<&CellLook> = cells.iter().filter(|c| c.metal_share > 0.0).collect();
    if with_metal.is_empty() {
        return None;
    }
    let (low, high) = with_metal.iter().fold((f32::MAX, 0f32), |(lo, hi), c| (lo.min(c.metal_share), hi.max(c.metal_share)));
    let even = with_metal.len() == cells.len() && high <= low * EVEN_METAL;
    let counts = |c: &CellLook| format!("{}/{}/{}", c.extractors[0], c.extractors[1], c.extractors[2]);
    let words = if even {
        let held: Vec<String> = with_metal.iter().filter(|c| c.extractors.iter().any(|n| *n > 0)).map(|c| format!("{} {}", c.cell, counts(c))).collect();
        format!("the metal lies evenly over every cell ({:.1}% each); cells with extractors standing: {}", 100.0 / cells.len() as f32, if held.is_empty() { "none yet".to_string() } else { held.join(", ") })
    } else {
        with_metal.iter().map(|c| format!("{} {:.0}% {}", c.cell, c.metal_share * 100.0, counts(c))).collect::<Vec<_>>().join(", ")
    };
    Some(format!("metal (each cell's share of the map's metal, then the extractors standing in it: ours/allied/his as our units last saw them): {words}"))
}

fn scouting_lines(cells: &[CellLook], factory_known: bool) -> Vec<String> {
    if cells.is_empty() {
        return Vec::new();
    }
    // What a cell holds for the economy: its spots by number, or on a metal map its share of the metal (the
    // metal-maps design §1.4).
    let with_metal: Vec<&CellLook> = cells.iter().filter(|c| c.metal_share > 0.0).collect();
    let even_metal = !with_metal.is_empty() && with_metal.len() == cells.len() && {
        let (low, high) = with_metal.iter().fold((f32::MAX, 0f32), |(lo, hi), c| (lo.min(c.metal_share), hi.max(c.metal_share)));
        high <= low * EVEN_METAL
    };
    let spots = |c: &CellLook| {
        if even_metal {
            String::new()
        } else if c.metal_share > 0.0 {
            format!("{:.0}% of the metal", c.metal_share * 100.0)
        } else if c.spots.is_empty() {
            String::new()
        } else {
            c.spots.iter().map(|n| format!("spot_{n}")).collect::<Vec<_>>().join(", ")
        }
    };
    let has_metal = |c: &CellLook| (c.metal_share > 0.0 && !even_metal) || !c.spots.is_empty();
    let when = |ago: i32| if ago < 10 { "in sight now".to_string() } else { format!("seen {} ago", clock(ago)) };
    let look = |c: &CellLook| match c.ago {
        Some(ago) => format!("{} of it {}", share_words(c.seen_share), when(ago)),
        None => "never seen".to_string(),
    };
    let mut lines = vec!["scouting (each cell of the map by when units of ours last saw it; an old look says what was built there then, nothing about his army now):".to_string()];
    let holds = |c: &CellLook| !c.his.is_empty() || c.his_commander || !c.guns.is_empty();
    for c in cells.iter().filter(|c| holds(c)) {
        let mut his: Vec<String> = c.his_commander.then(|| "his commander as last seen".to_string()).into_iter().collect();
        his.extend(c.his.iter().map(|(name, count)| format!("{count} {name}")));
        // Fire from out of sight laid to the cell (H-GUNS): his guns stand about here whatever was seen.
        if !c.guns.is_empty() {
            his.push(format!("fire from out of sight laid here: {}", c.guns.join(", ")));
        }
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
    let named = |c: &CellLook, words: String| if has_metal(c) { format!("{} ({words}; {})", c.cell, spots(c)) } else { format!("{} ({words})", c.cell) };
    let old: Vec<String> = cells.iter().filter(|c| !holds(c) && c.ago.is_some_and(|ago| ago >= 360)).map(|c| named(c, format!("{} of it, {} ago", share_words(c.seen_share), clock(c.ago.unwrap_or(0))))).collect();
    if !old.is_empty() {
        lines.push(format!("  nothing of his when seen over 6 min ago: {}", old.join(", ")));
    }
    // Never seen: his box's cells one by one with their spots (where a scout is sent), the rest as runs (at the
    // start that is the whole map).
    let never = |inside: bool| cells.iter().filter(move |c| !holds(c) && c.ago.is_none() && c.in_his_box == inside);
    let in_box: Vec<String> = never(true).map(|c| if has_metal(c) { format!("{} ({})", c.cell, spots(c)) } else { c.cell.clone() }).collect();
    if !in_box.is_empty() {
        lines.push(format!("  never seen, in his box: {}", in_box.join(", ")));
    }
    let elsewhere: Vec<&CellLook> = never(false).collect();
    if !elsewhere.is_empty() {
        lines.push(format!("  never seen, elsewhere: {}", cell_runs(&elsewhere)));
    }
    lines.extend(metal_line(cells));
    // Radar now (H-REPORT-RADAR-WORDS, the engine's own grid, cut by hills): of the cells not in sight this moment,
    // which our radar covers and what stands on them, which a seen jammer of his hides, and which are dark to both
    // eyes and radar. "Nothing of his" in a dark cell says nothing about it.
    if cells.iter().any(|c| c.radar_share.is_some()) {
        let quiet = |c: &&CellLook| c.ago.is_none_or(|ago| ago >= 10);
        let covered: Vec<&CellLook> = cells.iter().filter(quiet).filter(|c| !c.jammed && c.radar_share.is_some_and(|s| s >= 0.5)).collect();
        let empty: Vec<&CellLook> = covered.iter().copied().filter(|c| c.contacts == 0).collect();
        let with: Vec<String> = covered.iter().filter(|c| c.contacts > 0).map(|c| format!("{} ({} contacts)", c.cell, c.contacts)).collect();
        let jammed: Vec<&CellLook> = cells.iter().filter(quiet).filter(|c| c.jammed).collect();
        let dark: Vec<&CellLook> = cells.iter().filter(quiet).filter(|c| !c.jammed && c.radar_share.is_some_and(|s| s < 0.1)).collect();
        let out_of_sight = cells.iter().filter(quiet).count();
        let mut parts: Vec<String> = Vec::new();
        if !empty.is_empty() {
            parts.push(format!("under our radar with nothing on it: {}", cell_runs(&empty)));
        }
        if !with.is_empty() {
            parts.push(format!("under our radar with contacts of his: {}", with.join(", ")));
        }
        if !jammed.is_empty() {
            parts.push(format!("hidden from radar by a jammer of his: {}", cell_runs(&jammed)));
        }
        if !dark.is_empty() {
            // Most of the map at the start: the count, not sixty cells by runs.
            let which = if dark.len() * 4 >= out_of_sight * 3 {
                format!("{} ({} cells)", if parts.is_empty() { "every cell out of our sight now" } else { "all the rest out of our sight" }, dark.len())
            } else {
                cell_runs(&dark)
            };
            parts.push(format!("dark to our radar and sight (hills or range: nothing known of them now): {which}"));
        }
        if !parts.is_empty() {
            lines.push(format!("  radar now (the engine's coverage, cut by hills): {}", parts.join(" | ")));
        }
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
/// `seats_line`: the line a seat of a player with several (the commander's report writes its own).
/// "lost 2245 metal in soldiers and 0 in buildings; destroyed 640 metal of his in soldiers and 0 in buildings": the
/// builders (constructors, the commander) named only when some died.
fn traded_words(t: &Traded) -> String {
    fn bins(b: &Bins, head: &str) -> String {
        let builders = if b.builders > 0.0 { format!(", {:.0} in builders", b.builders) } else { String::new() };
        format!("{:.0} {head} in soldiers and {:.0} in buildings{builders}", b.soldiers, b.buildings)
    }
    format!("lost {}; destroyed {}", bins(&t.lost, "metal"), bins(&t.killed, "metal of his"))
}

/// The eco line: both stores with the engine's income and usage, what allies pour in beside the income when they do
/// (gifts and overflow, which the engine's income leaves out: the Glitters 2v6 of 2026-10-25 read "in balance" and
/// "FULL" together while 30 to 47 a second arrived unread), the flow words on the whole inflow, the metal store's
/// cap and the word when it is full (the impedance tally, 2026-10-16: the hands' picture said "2096 of 2100 stored
/// (full: what comes in beyond this is wasted)" while the line here read "metal 1794 (+43.3/-35.3)"; jev114-player-1
/// sat at the cap a fifth of the game), the wind and the counts.
fn eco_line(briefing: &Briefing) -> String {
    let (m, e, c) = (&briefing.metal, &briefing.energy, &briefing.counts);
    let (rm, re) = briefing.received;
    let from_allies = |r: f32| if r >= 0.5 { format!(", and {r:+.0} a second arriving from allies") } else { String::new() };
    let full = m.storage > 0.0 && m.current / m.storage >= 0.98;
    // What the converters took and made against what they could take, so "in balance" with converters standing reads
    // as their work and a converter past the energy reads as a body (econ-flagship-14: 153 converters that could take
    // 31,380 on an energy income of 23,270, 101 of them built in the last three minutes against a metal float). Under
    // three quarters of the store the gadget converts nothing, and the line says that instead.
    let conv = &briefing.conversion;
    let conversion = if conv.converters == 0 {
        String::new()
    } else if conv.energy >= 1.0 {
        let (them, they) = if conv.converters == 1 { ("the 1 converter".to_string(), "it") } else { (format!("the {} converters", conv.converters), "they") };
        format!("; {:.0} of it to {them} making {:.0} metal ({they} could take {:.0}), the other {:.0} to building and upkeep", conv.energy, conv.metal, conv.capacity, (e.usage - conv.energy).max(0.0))
    } else if e.storage > 0.0 && e.current / e.storage < 0.75 {
        format!("; the {} converter{} taking nothing: the store is under three quarters, where conversion begins", conv.converters, if conv.converters == 1 { "" } else { "s" })
    } else {
        String::new()
    };
    format!(
        "eco: metal {:.0}/{:.0} ({:+.1}/-{:.1}{}; {}){}, energy {:.0}/{:.0} ({:+.0}/-{:.0}{}; {}{}), wind now {:.0} of this map's {:.0} to {:.0} | extractors {} constructors {} labs {} turrets {} converters {}",
        m.current, m.storage, m.income, m.usage, from_allies(rm), flow_words(m.income + rm, m.usage, m.current, m.storage), if full { " FULL: what comes in beyond the store is wasted" } else { "" },
        e.current, e.storage, e.income, e.usage, from_allies(re), flow_words(e.income + re, e.usage, e.current, e.storage), conversion,
        briefing.wind, briefing.wind_range.0, briefing.wind_range.1, c.extractors, c.constructors, c.labs, c.turrets, c.converters
    )
}

pub(super) fn front(briefing: &Briefing, field: &Field, fights: &[String], seats_line: bool) -> Vec<String> {
    let mut lines = Vec::new();
    let s = &field.score;
    // On a metal map (the metal-maps design §1.4) the spot words are room on the metal and his extractors by cell.
    let metal = s.room.map(|room| {
        format!(
            "room on the metal for about {room} more tier-1 extractors; his extractors as our units last saw them, by cell nearest our start first: {}",
            if s.enemy_extractor_cells.is_empty() { "none seen standing".to_string() } else { s.enemy_extractor_cells.iter().map(|(cell, count, ago)| format!("{cell} {count} ({} ago)", clock(*ago))).collect::<Vec<_>>().join(", ") }
        )
    });
    let spots = metal.unwrap_or_else(|| format!(
        "free spots {}: {} in our half our bots walk to, {} in our half under water or off their ground{}, {} in his half; nearest, our half first: {}; his extractors as our units last saw them, nearest our start first: {}",
        s.free_spots,
        s.free_on_foot_ours,
        s.free_other_ours,
        s.other_reach.as_ref().map_or(String::new(), |r| format!(" ({r})")),
        s.free_in_his_half,
        if s.next_free.is_empty() { "none".to_string() } else { s.next_free.iter().map(|f| format!("#{} {} {}{}", f.n, f.place.grid, f.distance, f.how.as_ref().map_or(String::new(), |h| format!(" [{h}]")))).collect::<Vec<_>>().join(", ") },
        if s.enemy_spots.is_empty() { "none seen standing".to_string() } else { s.enemy_spots.iter().map(|(n, p, ago)| format!("#{n} {} ({} ago)", p.grid, clock(*ago))).collect::<Vec<_>>().join(", ") },
    ));
    lines.push(format!(
        "score: extractors {} (most held {}, no new high for {}; {spots}) | army {} soldiers worth {} metal, {} of them within 800 of our start, {} | opponent: we see only what our units see. Its soldiers seen in the last 3 min and not seen to die: {} worth {} metal; {}",
        s.extractors,
        s.extractor_peak,
        clock(s.seconds_since_growth),
        s.soldiers,
        s.army_metal,
        s.soldiers_near_home,
        if s.soldiers_in_his_half > 0 { format!("{} worth {} in his half", s.soldiers_in_his_half, s.army_metal_in_his_half) } else { "none in his half".to_string() },
        s.enemy_soldiers_seen,
        s.enemy_soldiers_seen_metal,
        match s.seconds_since_ours_in_his_half {
            None => "nothing of ours has stood in his half yet".to_string(),
            Some(ago) if ago < 30 => "ours stand in his half now".to_string(),
            Some(ago) => format!("the last of ours stood in his half {} ago", clock(ago)),
        }
    ));
    // The side's share of the map's spots (H-REPORT-SIDE-SPOTS; human-8v8-1: 43 of 84 for five seats, all taken by
    // 12:00, read by both players as being "stuck" and by neither as the tier-1 ceiling).
    if let Some((near, total, seats)) = briefing.side_spots {
        lines.push(format!(
            "spots: {near} of the map's {total} are nearer our side's start box than his, for the {seats} seat{} of our side ({}); past them income grows only by upgrading",
            if seats == 1 { "" } else { "s" },
            if seats > 1 { format!("about {} each", (near as f32 / seats as f32).round() as usize) } else { "all of them yours to take".to_string() }
        ));
    }
    if let Some((percent, seats)) = briefing.side_metal {
        lines.push(format!(
            "metal: about {percent}% of the map's metal lies in cells nearer our side's start box than his, for the {seats} seat{} of our side; income here grows by the room on the metal and by build power",
            if seats == 1 { "" } else { "s" }
        ));
    }
    // The trade by our own units' weapons, soldiers and buildings apart (H-PLAYER-OWN-TRADE): a seat's player
    // reads its own last minute, the commander the side's last three with each death counted once. Until
    // 2026-10-18 the destroyed side was every enemy death in the alliance's sight, the people's and the other
    // bots' kills among them, against this seat's own losses (human-8v8-1: 2 to 1 for us on the line while the
    // seat's own exchange at G2 ran 4 to 7 to 1 against).
    let (whose, window, recent) = if seats_line { ("this seat's own units", "the last minute", &s.traded_1_min) } else { ("our seats' units", "the last 3 min", &s.traded_3_min) };
    lines.push(format!(
        "traded: by {whose} in {window}, {} | whole game: {}{}{}",
        traded_words(recent),
        traded_words(&s.traded),
        if seats_line { "" } else { " (each death counted once across our seats)" },
        s.seconds_since_turn.map_or(String::new(), |t| format!(" | {} of game time since your last turn began", clock(t)))
    ));
    // Both stores carry the hands' own flow words (`flow_words`: STALLING, running down with the seconds to empty,
    // banking), so the prompt's "the energy line reads STALLING" and the player's projection read one source
    // (H-PLAYER-ECO-WORDS; player-12 1:45: the line read "energy 1/1300 (+0)" behind an assisted constructor and the
    // word never appeared; the engine caps spending at income once the store is empty, so a net figure reads 0;
    // chase-player-2 5:08: "energy 1110/1550 (-89)" was twelve seconds from empty and the turn went to a plant).
    lines.push(eco_line(briefing));
    lines.extend(spending_line(briefing));
    if let Some(line) = stockpile_line(briefing) {
        lines.push(line);
    }
    if let Some(line) = unfinished_line(briefing) {
        lines.push(line);
    }
    // The block: where the construction turrets and frames stand (the base layout design §1.1, step 3).
    if let Some(line) = &briefing.block {
        lines.push(line.clone());
    }
    // The lists line: builders whose lists ran out with nothing in their place (H-WAKE-LISTS-ENDED).
    if let Some(line) = &briefing.lists {
        lines.push(line.clone());
    }
    if !field.wreck_fields.is_empty() || field.resurrection_bots > 0 {
        let fields: Vec<String> = field.wreck_fields.iter().take(5).map(|(at, metal, safe)| format!("{} {metal} metal{}", at.grid, if *safe { "" } else { " (not safe)" })).collect();
        lines.push(format!("wrecks: {} | resurrection bots {}", if fields.is_empty() { "none known".to_string() } else { fields.join(", ") }, field.resurrection_bots));
    }
    // Who is in the game (H-PLAYER-SIDES; human-9: a 2v1 against a person and BARb, and the player was never told).
    lines.extend(sides_line(&briefing.sides));
    // The lobby's game-end rule (H-REPORT-GAME-END; human-8v8-1 ran `deathmode=com` with five seats a side, and the
    // role text said "lose when ours dies" of a game a side loses only when it has no commander left).
    if !briefing.game_end.is_empty() {
        lines.push(format!("game end: {}", briefing.game_end));
    }
    if !briefing.jammers.is_empty() {
        lines.push(format!("jammers of his (each hides what stands within its radius from radar, not from sight; a Juno pulse kills them): {}", briefing.jammers.join(" | ")));
    }
    if !briefing.guns.is_empty() {
        lines.push(format!("fire from out of sight (standing places `guns_N`, from the hits' bearings): {}", briefing.guns.join(" | ")));
    }
    if !briefing.shields.is_empty() {
        lines.push(format!("shields of his: {}", briefing.shields.join(" | ")));
    }
    if let Some(line) = allies_line(briefing) {
        lines.push(line);
    }
    if seats_line && briefing.seats.len() > 1 {
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
/// Our silos and anti-nukes with their rounds, in every report: the player fires, and a round it is not told of is a
/// round it does not fire (nuke-2-hard, 2026-10-09: two missiles ready five minutes before his anti-nuke was begun, the
/// reports naming neither, the player's note "silo had 3 missiles ready (unseen in reports)").
/// Every unfinished structure of ours, those nobody is building first, with the handle a `finish <handle>` list
/// step or a `remove` takes (the unfinished-structures sweep, 2026-10-17: a frame nobody was building was in no
/// report until it decayed, 205 such in 23 games).
/// The sides line: every seat of the game by who plays it, its faction (a person's is the lobby's word until its
/// units are seen) and its lobby colour, ours first. The same words before the game (the opening prompt: the hello
/// carries the lobby's factions and skills, and 1793584014-human told the player its opponent's only at 0:10, after
/// it had written the opening) and in the running report.
pub(crate) fn sides_line(sides: &[Side]) -> Option<String> {
    if sides.is_empty() {
        return None;
    }
    let ours: Vec<&Side> = sides.iter().filter(|s| s.ours).collect();
    let theirs: Vec<&Side> = sides.iter().filter(|s| !s.ours).collect();
    let our_seats: usize = ours.iter().map(|s| s.seats.len()).sum();
    let their_seats: usize = theirs.iter().map(|s| s.seats.len()).sum();
    let shape = if theirs.len() > 1 { format!("{our_seats} of ours against {} sides of {their_seats} seats", theirs.len()) } else { format!("a {our_seats}v{their_seats}") };
    Some(format!(
        "sides ({shape}): yours: {}{} | against you: {}",
        ours.iter().flat_map(|s| s.seats.iter()).cloned().collect::<Vec<_>>().join(", "),
        if our_seats > 1 { " (allies fight beside you; their units are theirs to order)" } else { "" },
        if theirs.is_empty() { "nobody".to_string() } else { theirs.iter().map(|s| s.seats.join(", ")).collect::<Vec<_>>().join(" | ") }
    ))
}

pub(super) fn unfinished_line(briefing: &Briefing) -> Option<String> {
    if briefing.frames.is_empty() {
        return None;
    }
    // The turrets' work on a frame shows in its share alone (the hands see no turret's target), so the line says what
    // the share is doing before who is on it: "nobody is building it" only when nothing is (econ-flagship-8, 12:48:
    // an advanced fusion rising a percent a second under fourteen turrets read so, and the player bought builders).
    const STILL: f32 = 0.0005;
    let entry = |f: &super::shared::UnfinishedFrame| {
        let head = format!("{} at {} ({:.0}% built, {:.0} of {:.0} metal in it", f.handle, f.at.grid, f.share * 100.0, f.share * f.cost, f.cost);
        let turrets = |n: usize| if n == 1 { "1 construction turret".to_string() } else { format!("{n} construction turrets") };
        let rising = format!("rising {:.1} % a second", f.rate * 100.0);
        // The measured rate against what the power told onto it would give (F-eco-build): at that pace nothing is
        // short; well under it, an empty store is named when one is. A frame begun this second has no rate yet
        // (econ-flagship-14, 16:10: "0 % built, standing still, 50 turrets on it" on a frame placed that second, and
        // the player reclaimed the turret it became two seconds later).
        let store_empty = || {
            let empty = |r: &bot_protocol::Resource| r.storage > 0.0 && r.current / r.storage < 0.02;
            if empty(&briefing.metal) { ", the metal store is empty" } else if empty(&briefing.energy) { ", the energy store is empty" } else { "" }
        };
        let against = |words: String| {
            if f.expected_rate <= 0.0 {
                words
            } else if f.rate >= 0.7 * f.expected_rate {
                format!("{words}, the pace of what is told onto it")
            } else {
                format!("{words} against the {:.1} % what is told onto it would give{}", f.expected_rate * 100.0, store_empty())
            }
        };
        let young = f.age_seconds.is_some_and(|age| age < 2);
        let trend = if young {
            format!(", begun {} s ago", f.age_seconds.unwrap_or(0))
        } else if f.rate > STILL {
            format!(", {}", against(rising.clone()))
        } else if f.rate < -STILL {
            ", falling".to_string()
        } else {
            format!(", {}", against("standing still".to_string()))
        };
        // The turrets in reach, and how many of them the hands have on this frame: "in reach" is not "on it" (a
        // turret takes one thing at a time), and reading it so bought 72 solars in econ-flagship-12.
        let elsewhere = || if f.turrets_elsewhere.is_empty() { String::new() } else { format!(" (the rest: {})", f.turrets_elsewhere.iter().map(|(what, n)| format!("{n} on {what}")).collect::<Vec<_>>().join(", ")) };
        let reach = || {
            if f.turrets_on == f.turrets {
                format!("{} in reach, all on it", turrets(f.turrets))
            } else {
                format!("{} of the {} in reach on it{}", f.turrets_on, turrets(f.turrets), elsewhere())
            }
        };
        if !f.builders.is_empty() {
            let with = if f.turrets > 0 { format!(" with {}", reach()) } else { String::new() };
            format!("{head}{trend}) by {}{with}", f.builders.join(", "))
        } else if f.rate > STILL && f.turrets > 0 {
            format!("{head}; {}: no builder of ours is on it, {})", against(rising.clone()), reach())
        } else if f.rate > STILL {
            format!("{head}; {rising} with no builder of ours on it)")
        } else {
            format!(
                "{head}; nobody is building it{}{}, {})",
                f.alone_seconds.map_or(String::new(), |s| format!(", {s} s since anything was")),
                if f.turrets > 0 { format!(" ({} in reach, on other work{})", turrets(f.turrets), elsewhere()) } else { String::new() },
                if f.rate < -STILL { "decaying" } else { "standing still" }
            )
        }
    };
    let alone = briefing.frames.iter().filter(|f| f.builders.is_empty() && f.rate <= STILL).count();
    let hint = if alone > 0 { "; `finish <handle>` on any builder's list resumes one as that builder's own build, the pick offers a builder near it the same, `remove` reclaims its metal" } else { "" };
    Some(format!("unfinished structures of ours: {}{hint}", briefing.frames.iter().map(entry).collect::<Vec<_>>().join("; ")))
}

pub(super) fn stockpile_line(briefing: &Briefing) -> Option<String> {
    if briefing.stockpilers.is_empty() {
        return None;
    }
    let parts: Vec<String> = briefing
        .stockpilers
        .iter()
        .map(|s| {
            let round = if s.anti_nuke { "interceptor" } else { "missile" };
            let plural = if s.ready == 1 { "" } else { "s" };
            let piling = if s.queued > 0 { format!(", {} stockpiling (one every {:.0} s at full energy)", s.queued, s.seconds) } else { String::new() };
            let tail = if s.anti_nuke {
                format!("; takes a missile aimed within {:.0} of it", s.coverage)
            } else if s.ready > 0 {
                "; `fire` sends one at a place".to_string()
            } else {
                String::new()
            };
            format!("{} holds {} {round}{plural} ready{piling}{tail}", s.handle, s.ready)
        })
        .collect();
    Some(format!("missiles: {}", parts.join(" | ")))
}

pub(super) fn contact(briefing: &Briefing, field: &Field) -> Vec<String> {
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
/// What a builder or factory standing now can build, once a session (the one thing the prompt's tables cannot
/// say; the roster that stood here until 2026-11-04 is the tables now, docs/design/2026-11-04-unit-context.md §5).
fn now_line(field: &Field) -> String {
    let names: Vec<&str> = field.buildable.iter().map(|(n, _)| n.as_str()).collect();
    let sides: Vec<&str> = field.factions.iter().map(|(_, side)| side.as_str()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    format!(
        "can build now (a builder or factory standing now makes these; your tables hold every unit of your faction){}: {}",
        if sides.len() > 1 {
            format!(
                "; the seats are of more than one faction ({}): a seat's builders and factories take their own faction's names (arm..., cor... or leg...) and skip the others'",
                field.factions.iter().map(|(team, side)| format!("t{team} {side}")).collect::<Vec<_>>().join(", ")
            )
        } else {
            String::new()
        },
        if names.is_empty() { "nothing yet".to_string() } else { names.join(", ") }
    )
}

/// The types this report meets first (H-PLAYER-UNIT-CONTEXT): one glossary line each, once in the game.
fn new_unit_lines(briefing: &Briefing, field: &Field) -> Vec<String> {
    let fresh = super::unit_context::fresh_lines(briefing.frame, field);
    if fresh.is_empty() {
        return Vec::new();
    }
    let mut lines = vec!["new units this report (the first time a type is seen of his or buildable of ours; its row is in your next session's tables; `units` has the prose):".to_string()];
    lines.extend(fresh.into_iter().map(|l| format!("  {l}")));
    lines
}

/// The parties out of sight now, as the hands' picture remembers them (H-HANDS-ENEMY-MEMORY): the picture had
/// "party_1 (9 armfav) last seen 0:23 ago at (G6), then moving toward our side" at human-26 3:27 and the report
/// said nothing, so red's player wrote a route for a quiet base while seven Rovers crossed the middle toward it.
pub(super) fn recently_seen(picture: &serde_json::Value) -> Vec<String> {
    let Some(parties) = picture["enemy"]["out_of_sight"].as_array() else { return Vec::new() };
    let mut lines = vec!["recently seen, out of sight now (a party that left one base intact is on its way somewhere; the hands' picture keeps it three minutes):".to_string()];
    lines.extend(parties.iter().filter_map(|p| p.as_str()).map(|p| format!("  {p}")));
    if lines.len() > 1 { lines } else { Vec::new() }
}

/// The player's `spending` line (H-PLAYER-SPENDING-LINE): what the factories and builders would take a second at
/// full speed on what they build now, each factory with the power helping it, and the seat's build power by kind;
/// beside the eco line's income that says whether the store fills or the energy runs out. Numbers, no advice
/// (docs/design/2026-10-17-spending-line.md: the bank of chase-player-1 and -2 was build power missing, and the draws
/// reached the hands and not the player).
fn spending_line(briefing: &Briefing) -> Option<String> {
    let s = &briefing.spending;
    let p = &briefing.build_power;
    if s.draws.is_empty() && s.builders_helping == 0 && s.builders_without_a_build == 0 {
        return None;
    }
    let plural = |n: usize| if n == 1 { "" } else { "s" };
    let scale = |d: &super::shared::Draw| if d.factory && d.own_power > 0.0 { (d.own_power + d.helping) / d.own_power } else { 1.0 };
    let metal: f32 = s.draws.iter().map(|d| d.metal * scale(d)).sum();
    let energy: f32 = s.draws.iter().map(|d| d.energy * scale(d)).sum();
    let mut parts: Vec<String> = Vec::new();
    for d in s.draws.iter().filter(|d| d.factory) {
        parts.push(if d.helping > 0.0 {
            format!("{} {:.1} metal and {:.0} energy on {} alone, {:.0} build power helping it: {:.1} and {:.0}", d.who, d.metal, d.energy, d.target, d.helping, d.metal * scale(d), d.energy * scale(d))
        } else {
            format!("{} {:.1} metal and {:.0} energy on {}, nothing helping it", d.who, d.metal, d.energy, d.target)
        });
    }
    // The builders by what they build: "2 constructors on an extractor 5.0 and 50".
    let mut by_target: Vec<(String, String, usize, f32, f32)> = Vec::new();
    for d in s.draws.iter().filter(|d| !d.factory) {
        let kind = if d.who.starts_with("constructor") { "constructor".to_string() } else { d.who.clone() };
        match by_target.iter_mut().find(|(k, t, ..)| *k == kind && *t == d.target) {
            Some(row) => {
                row.2 += 1;
                row.3 += d.metal;
                row.4 += d.energy;
            }
            None => by_target.push((kind, d.target.clone(), 1, d.metal, d.energy)),
        }
    }
    for (kind, target, n, m, e) in by_target {
        parts.push(if kind == "constructor" { format!("{n} constructor{} on {target} {m:.1} and {e:.0}", plural(n)) } else { format!("the {kind} on {target} {m:.1} and {e:.0}") });
    }
    if s.builders_helping > 0 {
        parts.push(format!("{} builder{} helping (counted with what they help)", s.builders_helping, plural(s.builders_helping)));
    }
    if s.builders_without_a_build > 0 {
        parts.push(format!("{} builder{} walking or idle (take nothing)", s.builders_without_a_build, plural(s.builders_without_a_build)));
    }
    let factories = s.draws.iter().filter(|d| d.factory).count();
    let own: f32 = p.factories.iter().map(|f| f.own).sum();
    Some(format!(
        "spending: at full speed our {} factor{} and {} builders would take about {metal:.0} metal and {energy:.0} energy a second on what they build now: {}. Build power: {own:.0} in the factories, {:.0} in {} builders that walk, {:.0} in {} construction turrets",
        factories,
        if factories == 1 { "y" } else { "ies" },
        p.mobile.0,
        parts.join("; "),
        p.mobile.1,
        p.mobile.0,
        p.turrets.1,
        p.turrets.0
    ))
}

/// The other seats of our side, one line's worth each, as the allied units in sight show them (H-REPORT-ALLIES-LINE;
/// human-8v8-1: three Apex allies placed advanced labs at 5:20, 8:00 and 8:30, upgraded our extractors 22 times, and
/// no report of either reader showed any of it).
pub(super) fn allies_line(briefing: &Briefing) -> Option<String> {
    if briefing.allies.is_empty() {
        return None;
    }
    let parts: Vec<String> = briefing
        .allies
        .iter()
        .map(|a| {
            let tier = match a.tier2_since {
                // `clock` takes seconds; `tier2_since` and `frame` are frames at 30 a second.
                Some(since) => format!("tier 2 since {}{}", clock(since / 30), if briefing.frame - since < 100 * 30 { ", new" } else { "" }),
                None => "tier 1".to_string(),
            };
            let mut who = a.who.clone();
            if !a.colour.is_empty() {
                who += &format!(", {}", a.colour);
            }
            if !a.faction.is_empty() {
                who += &format!(", {}", a.faction);
            }
            format!(
                "t{} ({who}): {} factor{} ({tier}), {} extractors{} (about {:.0} metal a second from its extractors), {} constructors{}, {} nanos{}{}, army {} soldiers worth {}{}",
                a.team,
                a.factories,
                if a.factories == 1 { "y" } else { "ies" },
                a.extractors,
                if a.mohos > 0 { format!(" + {} mohos", a.mohos) } else { String::new() },
                a.income_estimate,
                a.constructors,
                if a.advanced_constructors > 0 { format!(" + {} advanced", a.advanced_constructors) } else { String::new() },
                a.nanos,
                if a.big_energy > 0 { format!(", {} fusions or advanced solars", a.big_energy) } else { String::new() },
                if a.converters > 0 { format!(", {} converters", a.converters) } else { String::new() },
                a.soldiers,
                a.army_metal,
                if a.army_at.is_empty() { String::new() } else { format!(" mostly at {}", a.army_at) }
            )
        })
        .collect();
    Some(format!("allies (the other seats of our side, theirs to order; `transfer` sends them metal and energy by tag): {}", parts.join(" | ")))
}

/// A seat of ours on one line, for the commander's report and the other seats' players (H-PLAYER-OTHER-SEATS; human-8v8-1:
/// no report of a per-seat player named the other seat's bank, so neither could tell what a transfer would feed).
pub(super) fn seat_block_line(team: i32, view: &super::seats::SeatView) -> String {
    let b = &view.briefing;
    let faction = view.field.factions.iter().find(|(t, _)| *t == team).map(|(_, f)| f.as_str()).unwrap_or("faction not yet known");
    let stalling = b.energy.storage > 0.0 && b.energy.current / b.energy.storage < 0.05 && b.energy.usage >= b.energy.income * 0.9;
    format!(
        "seat t{team} ({faction}{}, home {}): metal {:.0} banked ({:+.1}/-{:.1}), energy {:.0}/{:.0} ({:+.0}){} | extractors {}, {} | constructors {}, factories {}, turrets {} | soldiers {} worth {} metal",
        if b.colour.is_empty() { String::new() } else { format!(", {} to the people in the game", b.colour) },
        b.home.grid, b.metal.current, b.metal.income, b.metal.usage, b.energy.current, b.energy.storage, b.energy.income - b.energy.usage,
        if stalling { " STALLING" } else { "" },
        b.counts.extractors,
        match view.field.score.room {
            Some(room) => format!("room on the metal for about {room} more"),
            None => format!("free spots in its reach {}", view.field.score.free_spots),
        },
        b.counts.constructors, b.counts.labs, b.counts.turrets, view.field.score.soldiers, view.field.score.army_metal
    )
}

/// What the report carries from outside the briefing and the field: the fights since the last turn, the people's chat,
/// and the other seats of ours under the same commander (one `seat_block_line` each, for a player with one seat).
pub struct Aside<'a> {
    pub fights: &'a [String],
    pub chat: &'a [String],
    pub others: &'a [String],
    /// Standing transfers touching this player's seats (H-PLAYER-STANDING-TRANSFER).
    pub standing: &'a [String],
}

/// Each actor on one line, as the hands see it: the words the instructions have to speak to. A builder's line
/// carries the build queued behind its current one and the steps its list has left (the 2v2 of 2026-11-04 on
/// Ravaged: the picture had both for the hands and the report showed neither, so the player re-sent its opening
/// from a 27-second-old picture and two solars went up twice before the plant).
fn actor_lines(picture: &Value) -> Vec<String> {
    let mut actors: Vec<String> = Vec::new();
    if let Some(entries) = picture["actors"].as_object() {
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
            if let Some(next) = field("next") {
                parts.push(next);
            }
            if let Some(list) = field("list") {
                parts.push(list.replacen("the player's list, done by the bot without asking: ", "its list, the steps left, done in order without asking: ", 1));
            }
            if let Some(health) = field("health").filter(|h| !h.starts_with("full")) {
                parts.push(format!("health {health}"));
            }
            // `last_pick` and `reached` are the hands' registers; `best_rated` is what the gate last rated best for the
            // actor: a paragraph that is not being read as meant shows here within a turn.
            for key in ["allowed", "lane", "progress", "stuck", "yard", "street", "reinforcements", "stragglers", "enemies_near", "enemies_at_our_extractors", "under_fire", "unseen_shooter", "rove", "scouts_out", "slots", "answer_to", "nanos", "reached", "last_pick", "best_rated", "reads"] {
                match &entry[key] {
                    serde_json::Value::String(text) => parts.push(format!("{key}: {text}")),
                    serde_json::Value::Array(items) => parts.push(format!("{key}: {}", items.iter().filter_map(|i| i.as_str()).collect::<Vec<_>>().join("; "))),
                    _ => {}
                }
            }
            // A factory's draw at full speed with what each 100 build power helping it adds (H-PLAYER-SPENDING-LINE).
            if (name.starts_with("plant_") || name.starts_with("lab_") || name.starts_with("factory_"))
                && let Some(draw) = field("draw")
            {
                parts.push(draw);
            }
            // A constructor's type beside its name (the impedance tally, 2026-10-16: an advanced constructor read
            // "constructor_N" like the others, and tier-1 steps went to it and were skipped).
            let kind = field("is").filter(|_| name.starts_with("constructor_")).and_then(|is| constructor_kind(&is)).map(|k| format!(" ({k})")).unwrap_or_default();
            actors.push(format!("{name}{kind}: {}", parts.join("; ")));
        }
    }
    actors
}

pub fn player_report(seen: &mut Seen, briefing: &Briefing, field: &Field, hands: &Hands, aside: &Aside, full: bool) -> String {
    let mut lines = front(briefing, field, aside.fights, true);
    if !aside.others.is_empty() {
        lines.push(format!("our other seats (under the same commander; `transfer` reaches them by tag, and a seat whose plant is going up is fed by the others): {}", aside.others.join(" | ")));
    }
    if !aside.standing.is_empty() {
        lines.push(format!("standing transfers: {}", aside.standing.join("; ")));
    }
    lines.extend(contact(briefing, field));
    lines.extend(recently_seen(&hands.picture));
    extractor_lines(seen, field, full, &mut lines);
    if full {
        lines.push(now_line(field));
    }
    lines.extend(new_unit_lines(briefing, field));
    let actors = actor_lines(&hands.picture);
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
    if !aside.chat.is_empty() {
        lines.push("chat since your last turn (people in the game; `say` answers them):".to_string());
        lines.extend(aside.chat.iter().map(|c| format!("  {c}")));
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
    use super::{CellLook, cell_runs, constructor_kind, recently_seen, scouting_lines};

    /// A builder's line carries the build queued behind its current one and the steps its list has left, in the
    /// picture's words with the list's label turned to the player; an actor without them says nothing of them.
    #[test]
    fn a_builders_line_says_what_is_queued_and_what_its_list_has_left() {
        let picture = serde_json::json!({ "actors": {
            "commander": { "at": "home (E4)", "doing": "building a Solar Collector (legsolar) at spot_21 (E4) (53 % built)", "health": "full (100%)",
                "next": "queued to start the moment this is done: build a legsolar at home (E4)",
                "list": "the player's list, done by the bot without asking: plant #vp1, assist 30 low, extractor" },
            "group_A": { "units": "3 Rovers", "at": "C3" },
        } });
        let lines = super::actor_lines(&picture);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert_eq!(lines[0], "commander: at home (E4); building a Solar Collector (legsolar) at spot_21 (E4) (53 % built); queued to start the moment this is done: build a legsolar at home (E4); its list, the steps left, done in order without asking: plant #vp1, assist 30 low, extractor");
        assert_eq!(lines[1], "group_A: 3 Rovers; at C3");
    }

    /// The actor line's type suffix is the internal name alone, from the entry's gloss.
    #[test]
    fn a_constructors_kind_is_its_internal_name() {
        assert_eq!(constructor_kind("Construction Vehicle (armcv; constructor vehicle, 135 metal): tier-1 constructor vehicle, 90 build power").as_deref(), Some("armcv"));
        assert_eq!(constructor_kind("Advanced Construction Vehicle (armacv)").as_deref(), Some("armacv"));
        assert_eq!(constructor_kind("no gloss"), None);
    }

    fn cell(name: &str, seen_share: f32, ago: Option<i32>) -> CellLook {
        CellLook { cell: name.to_string(), seen_share, ago, in_his_box: name.starts_with('G') || name.starts_with('H'), ..CellLook::default() }
    }

    /// The picture's out-of-sight parties are said to the player, one line each; the picture's "none" string is
    /// not an array and says nothing.
    /// The missiles line names every silo and anti-nuke with its rounds, and is absent without one.
    #[test]
    fn the_missiles_line_says_what_each_silo_holds() {
        use super::super::shared::{Briefing, Stockpiler};
        use super::stockpile_line;
        assert!(stockpile_line(&Briefing::default()).is_none());
        let briefing = Briefing {
            stockpilers: vec![
                Stockpiler { handle: "armsilo_282".into(), anti_nuke: false, ready: 2, queued: 8, seconds: 120.0, coverage: 0.0 },
                Stockpiler { handle: "armamd_7".into(), anti_nuke: true, ready: 1, queued: 0, seconds: 90.0, coverage: 2000.0 },
            ],
            ..Briefing::default()
        };
        let line = stockpile_line(&briefing).unwrap();
        assert!(line.starts_with("missiles: armsilo_282 holds 2 missiles ready, 8 stockpiling (one every 120 s at full energy); `fire` sends one at a place | armamd_7 holds 1 interceptor ready; takes a missile aimed within 2000 of it"), "{line}");
    }

    #[test]
    fn the_recently_seen_lines_carry_the_pictures_out_of_sight_parties() {
        let picture = serde_json::json!({ "enemy": { "out_of_sight": ["party_1 (9 armfav, 279 metal) last seen 0:23 ago at (G6), then moving toward our side", "party_8 (1 armcv, 135 metal) last seen 0:15 ago at spot_73 (B7), then standing still"] } });
        let lines = recently_seen(&picture);
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("recently seen, out of sight now"));
        assert_eq!(lines[1], "  party_1 (9 armfav, 279 metal) last seen 0:23 ago at (G6), then moving toward our side");
        assert!(recently_seen(&serde_json::json!({ "enemy": { "out_of_sight": "no party seen in the last three minutes is out of sight" } })).is_empty());
        assert!(recently_seen(&serde_json::json!({})).is_empty());
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

#[cfg(test)]
mod unfinished_tests {
    use super::super::shared::{Briefing, Place, UnfinishedFrame};
    use super::{eco_line, unfinished_line};

    /// Every unfinished structure of ours is in the report, those nobody is building first, with the handle a
    /// `finish` step or a `remove` takes.
    #[test]
    fn the_report_lists_every_unfinished_structure_the_orphans_first() {
        assert!(unfinished_line(&Briefing::default()).is_none());
        let at = |grid: &str| Place { grid: grid.to_string(), ..Place::default() };
        let briefing = Briefing {
            frames: vec![
                UnfinishedFrame { handle: "armnanotc_17819".into(), unit: "armnanotc".into(), at: at("C8"), share: 0.45, cost: 230.0, builders: Vec::new(), rate: -0.002, turrets: 0, turrets_on: 0, turrets_elsewhere: Vec::new(), alone_seconds: Some(40), expected_rate: 0.0, age_seconds: None },
                UnfinishedFrame { handle: "armsolar_26962".into(), unit: "armsolar".into(), at: at("B8"), share: 0.86, cost: 155.0, builders: vec!["constructor_26737".into()], rate: 0.02, turrets: 0, turrets_on: 0, turrets_elsewhere: Vec::new(), alone_seconds: None, expected_rate: 0.0, age_seconds: None },
            ],
            ..Briefing::default()
        };
        let line = unfinished_line(&briefing).unwrap();
        assert_eq!(line, "unfinished structures of ours: armnanotc_17819 at C8 (45% built, 104 of 230 metal in it; nobody is building it, 40 s since anything was, decaying); armsolar_26962 at B8 (86% built, 133 of 155 metal in it, rising 2.0 % a second) by constructor_26737; `finish <handle>` on any builder's list resumes one as that builder's own build, the pick offers a builder near it the same, `remove` reclaims its metal");
    }

    /// A frame the construction turrets are raising is not an orphan: the share says what the turrets do (the hands
    /// see no turret's target; econ-flagship-8, 12:48: 17 %, rising a percent a second under 14 turrets, read "nobody
    /// is building it, decaying"), and a frame the turrets in reach have left for other work says so.
    #[test]
    fn a_frame_rising_under_the_turrets_is_said_to_be_built_by_them() {
        let at = |grid: &str| Place { grid: grid.to_string(), ..Place::default() };
        let briefing = Briefing {
            frames: vec![
                UnfinishedFrame { handle: "armafus_5949".into(), unit: "armafus".into(), at: at("B3"), share: 0.04, cost: 9700.0, builders: Vec::new(), rate: 0.0, turrets: 11, turrets_on: 11, turrets_elsewhere: Vec::new(), alone_seconds: Some(12), expected_rate: 0.0, age_seconds: None },
                UnfinishedFrame { handle: "armafus_54".into(), unit: "armafus".into(), at: at("B3"), share: 0.17, cost: 9700.0, builders: Vec::new(), rate: 0.011, turrets: 14, turrets_on: 14, turrets_elsewhere: Vec::new(), alone_seconds: None, expected_rate: 0.0, age_seconds: None },
                UnfinishedFrame { handle: "armafus_21961".into(), unit: "armafus".into(), at: at("B3"), share: 0.62, cost: 9700.0, builders: vec!["commander".into(), "constructor_1774".into()], rate: 0.013, turrets: 13, turrets_on: 13, turrets_elsewhere: Vec::new(), alone_seconds: None, expected_rate: 0.0, age_seconds: None },
            ],
            ..Briefing::default()
        };
        let line = unfinished_line(&briefing).unwrap();
        assert!(line.contains("armafus_54 at B3 (17% built, 1649 of 9700 metal in it; rising 1.1 % a second: no builder of ours is on it, 14 construction turrets in reach, all on it)"), "{line}");
        assert!(line.contains("armafus_21961 at B3 (62% built, 6014 of 9700 metal in it, rising 1.3 % a second) by commander, constructor_1774 with 13 construction turrets in reach, all on it"), "{line}");
        assert!(line.contains("armafus_5949 at B3 (4% built, 388 of 9700 metal in it; nobody is building it, 12 s since anything was (11 construction turrets in reach, on other work), standing still)"), "{line}");
        assert!(line.contains("`finish <handle>`"), "{line}");
    }

    /// The rate is read against the power told onto the frame: at its pace nothing is short, well under it the empty
    /// store is named, and a frame begun this second has no rate to read (econ-flagship-14, 15:08 and 16:10).
    #[test]
    fn a_frames_rate_is_read_against_the_power_told_onto_it() {
        let at = |grid: &str| Place { grid: grid.to_string(), ..Place::default() };
        let frame = |handle: &str, share: f32, rate: f32, expected: f32, age: Option<i32>, turrets: usize| UnfinishedFrame {
            handle: handle.into(),
            unit: handle.split('_').next().unwrap_or_default().into(),
            at: at("C3"),
            share,
            cost: if handle.starts_with("armafus") { 9700.0 } else { 230.0 },
            builders: vec!["constructor_19709".into()],
            rate,
            turrets,
            turrets_on: turrets,
            turrets_elsewhere: Vec::new(),
            alone_seconds: None,
            expected_rate: expected,
            age_seconds: age,
        };
        let briefing = Briefing {
            frames: vec![frame("armafus_13005", 0.51, 0.009, 0.0115, Some(50), 18), frame("armafus_7477", 0.21, 0.003, 0.012, Some(40), 9), frame("armnanotc_14631", 0.0, 0.0, 0.03, Some(0), 50)],
            metal: bot_protocol::Resource { current: 10.0, income: 400.0, usage: 500.0, storage: 28200.0 },
            ..Briefing::default()
        };
        let line = unfinished_line(&briefing).unwrap();
        assert!(line.contains("armafus_13005 at C3 (51% built, 4947 of 9700 metal in it, rising 0.9 % a second, the pace of what is told onto it) by constructor_19709 with 18 construction turrets in reach, all on it"), "{line}");
        assert!(line.contains("armafus_7477 at C3 (21% built, 2037 of 9700 metal in it, rising 0.3 % a second against the 1.2 % what is told onto it would give, the metal store is empty) by constructor_19709 with 9 construction turrets in reach, all on it"), "{line}");
        assert!(line.contains("armnanotc_14631 at C3 (0% built, 0 of 230 metal in it, begun 0 s ago) by constructor_19709 with 50 construction turrets in reach, all on it"), "{line}");
    }

    /// The eco line's energy clause says what the converters took and made against what they could take, so a
    /// converter past the energy reads as a body (econ-flagship-14: 153 that could take 31,380 on 23,270 of income).
    #[test]
    fn the_eco_line_says_what_the_converters_took_and_could_take() {
        let briefing = Briefing {
            energy: bot_protocol::Resource { current: 66000.0, income: 23270.0, usage: 23213.0, storage: 77300.0 },
            conversion: super::super::shared::Conversion { energy: 21000.0, metal: 362.0, capacity: 31380.0, converters: 153 },
            ..Briefing::default()
        };
        let line = eco_line(&briefing);
        assert!(line.contains("energy 66000/77300 (+23270/-23213; in balance; 21000 of it to the 153 converters making 362 metal (they could take 31380), the other 2213 to building and upkeep)"), "{line}");
        let low = Briefing { energy: bot_protocol::Resource { current: 30000.0, income: 9000.0, usage: 8000.0, storage: 77300.0 }, conversion: super::super::shared::Conversion { energy: 0.0, metal: 0.0, capacity: 1960.0, converters: 28 }, ..Briefing::default() };
        assert!(eco_line(&low).contains("; the 28 converters taking nothing: the store is under three quarters, where conversion begins)"), "{}", eco_line(&low));
        let one = Briefing { energy: bot_protocol::Resource { current: 70000.0, income: 9000.0, usage: 8000.0, storage: 77300.0 }, conversion: super::super::shared::Conversion { energy: 600.0, metal: 10.3, capacity: 600.0, converters: 1 }, ..Briefing::default() };
        assert!(eco_line(&one).contains("; 600 of it to the 1 converter making 10 metal (it could take 600), the other 7400 to building and upkeep)"), "{}", eco_line(&one));
    }
}

#[cfg(test)]
mod sightings_tests {
    use super::super::shared::{Bins, Briefing, CellLook, Field, Place, Score, Traded};
    use super::{front, metal_line, scouting_lines};

    fn cell(name: &str, seen_share: f32, ago: Option<i32>) -> CellLook {
        CellLook { cell: name.to_string(), seen_share, ago, ..CellLook::default() }
    }

    /// The score line gives his extractors as sightings with their age and no count of what he holds, and his
    /// soldiers seen as a count and nothing more; no line estimates his spending (the user, 2026-10-17: what he
    /// has is the player's estimate from what was seen; the harness has no authority to say it).
    #[test]
    fn the_spending_line_sums_the_draws_with_what_helps_a_factory() {
        use super::super::shared::{Briefing, BuildPower, Draw, FactoryPower, Spending};
        use super::spending_line;
        let draw = |who: &str, factory: bool, target: &str, own_power: f32, energy: f32, metal: f32, helping: f32| Draw { who: who.into(), factory, target: target.into(), own_power, energy, metal, helping };
        let briefing = Briefing {
            spending: Spending {
                draws: vec![
                    draw("plant_15129", true, "a Blitz (armflash)", 150.0, 68.0, 8.2, 300.0),
                    draw("commander", false, "an extractor (armmex)", 300.0, 83.0, 8.3, 0.0),
                    draw("constructor_1", false, "an extractor (armmex)", 90.0, 25.0, 2.5, 0.0),
                    draw("constructor_2", false, "an extractor (armmex)", 90.0, 25.0, 2.5, 0.0),
                ],
                builders_helping: 1,
                builders_without_a_build: 1,
            },
            build_power: BuildPower { mobile: (5, 660.0), turrets: (0, 0.0), factories: vec![FactoryPower { name: "armvp_15129".into(), own: 150.0, ..Default::default() }], mobile_away: 270.0 },
            ..Briefing::default()
        };
        let line = spending_line(&briefing).unwrap();
        assert!(line.starts_with("spending: at full speed our 1 factory and 5 builders would take about 38 metal and 337 energy a second on what they build now: "), "{line}");
        assert!(line.contains("plant_15129 8.2 metal and 68 energy on a Blitz (armflash) alone, 300 build power helping it: 24.6 and 204; the commander on an extractor (armmex) 8.3 and 83; 2 constructors on an extractor (armmex) 5.0 and 50; 1 builder helping (counted with what they help); 1 builder walking or idle (take nothing). Build power: 150 in the factories, 660 in 5 builders that walk, 0 in 0 construction turrets"), "{line}");
        assert!(spending_line(&Briefing::default()).is_none());
    }

    #[test]
    fn the_score_line_dates_each_sighting_and_claims_nothing_of_his_state() {
        let place = |grid: &str| Place { grid: grid.to_string(), x: 0, z: 0 };
        let score = Score { frame: 10 * 1800, extractors: 24, enemy_spots: vec![(40, place("E4"), 14), (35, place("D4"), 141)], enemy_soldiers_seen: 12, enemy_soldiers_seen_metal: 3756, ..Score::default() };
        let field = Field { score, ..Field::default() };
        let lines = front(&Briefing::default(), &field, &[], false);
        let score_line = &lines[0];
        assert!(score_line.contains("his extractors as our units last saw them, nearest our start first: #40 E4 (0:14 ago), #35 D4 (2:21 ago))"), "{score_line}");
        assert!(score_line.contains("Its soldiers seen in the last 3 min and not seen to die: 12 worth 3756 metal; "), "{score_line}");
        for word in ["known to hold", "may be much more", "his spending", "a floor", "extractors' worth"] {
            assert!(lines.iter().all(|l| !l.contains(word)), "{word}: {lines:?}");
        }
    }

    /// On a metal map (the metal-maps design §1.4) the score line says the room left on the metal and his extractors
    /// by cell, the seat line says room, the side line says the metal's share, and the scouting block carries each
    /// cell's share of the metal and a metal line of shares and extractors; no spot words anywhere.
    #[test]
    fn a_metal_map_reads_room_and_cells_where_a_spot_map_reads_spots() {
        let score = Score { frame: 10 * 1800, extractors: 24, room: Some(3_120), enemy_extractor_cells: vec![("G1".to_string(), 4, 14), ("L2".to_string(), 9, 141)], ..Score::default() };
        let field = Field { score, ..Field::default() };
        let briefing = Briefing { side_metal: Some((42, 1)), ..Briefing::default() };
        let lines = front(&briefing, &field, &[], false);
        assert!(lines[0].contains("extractors 24 (most held 0, no new high for 0:00; room on the metal for about 3120 more tier-1 extractors; his extractors as our units last saw them, by cell nearest our start first: G1 4 (0:14 ago), L2 9 (2:21 ago)) | army"), "{}", lines[0]);
        assert!(lines.iter().any(|l| l == "metal: about 42% of the map's metal lies in cells nearer our side's start box than his, for the 1 seat of our side; income here grows by the room on the metal and by build power"), "{lines:?}");
        for word in ["spot", "free spots"] {
            assert!(lines.iter().filter(|l| !l.starts_with("metal:")).all(|l| !l.contains(word)), "{word}: {lines:?}");
        }
        let mut a1 = cell("A1", 1.0, Some(0));
        a1.metal_share = 0.08;
        a1.extractors = [6, 0, 0];
        let mut l1 = cell("L1", 0.0, None);
        l1.metal_share = 0.08;
        l1.extractors = [0, 0, 3];
        l1.in_his_box = true;
        let mut c1 = cell("C1", 0.0, None);
        c1.metal_share = 0.01;
        let scouting = scouting_lines(&[a1, c1, l1], true);
        assert!(scouting.iter().any(|l| l == "  never seen, in his box: L1 (8% of the metal)"), "{scouting:?}");
        assert!(scouting.iter().any(|l| l == "  never seen, elsewhere: C1"), "{scouting:?}");
        assert!(scouting.iter().any(|l| l == "metal (each cell's share of the map's metal, then the extractors standing in it: ours/allied/his as our units last saw them): A1 8% 6/0/0, C1 1% 0/0/0, L1 8% 0/0/3"), "{scouting:?}");
        // A plate: the metal even over every cell, only the cells with extractors named.
        let mut plate: Vec<CellLook> = ["A1", "B1", "A2", "B2"].iter().map(|n| cell(n, 1.0, Some(0))).collect();
        for c in &mut plate {
            c.metal_share = 0.25;
        }
        plate[0].extractors = [12, 0, 0];
        plate[3].extractors = [0, 0, 7];
        let line = metal_line(&plate).unwrap();
        assert_eq!(line, "metal (each cell's share of the map's metal, then the extractors standing in it: ours/allied/his as our units last saw them): the metal lies evenly over every cell (25.0% each); cells with extractors standing: A1 12/0/0, B2 0/0/7");
        assert!(metal_line(&[cell("A1", 1.0, Some(0))]).is_none());
        let mut plate_unseen: Vec<CellLook> = ["A1", "B1", "A2", "B2"].iter().map(|n| cell(n, 0.0, None)).collect();
        for c in &mut plate_unseen {
            c.metal_share = 0.25;
        }
        let lines = scouting_lines(&plate_unseen, true);
        assert!(lines.iter().any(|l| l == "  never seen, elsewhere: A1, B1, A2, B2"), "{lines:?}");
    }

    /// H-REPORT-FREE-SPOTS-BY-REACH: the free spots by who reaches them and by half, the nearest of our half first with
    /// what of ours gets to one under water; "we can walk to" and "nearest on foot" are gone.
    #[test]
    fn the_score_line_counts_free_spots_by_reach_and_names_the_nearest_under_water() {
        use super::super::shared::NextFree;
        let place = |grid: &str| Place { grid: grid.to_string(), x: 0, z: 0 };
        let reach = "under water: our corcom can take it now; a cormuskrat (from corvp) would";
        let score = Score {
            frame: 11 * 1800,
            extractors: 17,
            free_spots: 28,
            free_on_foot_ours: 3,
            free_other_ours: 13,
            free_in_his_half: 12,
            other_reach: Some(reach.to_string()),
            next_free: vec![
                NextFree { n: 57, place: place("G7"), distance: 1176, how: Some(format!("straight line; {reach}")) },
                NextFree { n: 12, place: place("F2"), distance: 2761, how: None },
                NextFree { n: 38, place: place("C5"), distance: 4100, how: Some("in his half".to_string()) },
            ],
            ..Score::default()
        };
        let field = Field { score, ..Field::default() };
        let line = &front(&Briefing::default(), &field, &[], false)[0];
        assert!(line.contains("free spots 28: 3 in our half our bots walk to, 13 in our half under water or off their ground (under water: our corcom can take it now; a cormuskrat (from corvp) would), 12 in his half; nearest, our half first: #57 G7 1176 [straight line; under water: our corcom can take it now; a cormuskrat (from corvp) would], #12 F2 2761, #38 C5 4100 [in his half];"), "{line}");
        for gone in ["we can walk to", "nearest on foot"] {
            assert!(!line.contains(gone), "{gone}: {line}");
        }
    }

    #[test]
    fn the_trade_line_is_the_seats_own_minute_with_soldiers_and_buildings_apart() {
        let bins = |soldiers: f32, builders: f32, buildings: f32| Bins { soldiers, builders, buildings };
        let score = Score {
            frame: 17 * 1800,
            traded_1_min: Traded { lost: bins(2245.0, 0.0, 0.0), killed: bins(640.0, 0.0, 0.0) },
            traded_3_min: Traded { lost: bins(13345.0, 0.0, 640.0), killed: bins(21529.0, 0.0, 0.0) },
            traded: Traded { lost: bins(28592.0, 120.0, 1830.0), killed: bins(9900.0, 0.0, 14200.0) },
            ..Score::default()
        };
        let field = Field { score, ..Field::default() };
        let line = |seats_line: bool| front(&Briefing::default(), &field, &[], seats_line).into_iter().find(|l| l.starts_with("traded:")).expect("a trade line");
        assert_eq!(
            line(true),
            "traded: by this seat's own units in the last minute, lost 2245 metal in soldiers and 0 in buildings; destroyed 640 metal of his in soldiers and 0 in buildings | whole game: lost 28592 metal in soldiers and 1830 in buildings, 120 in builders; destroyed 9900 metal of his in soldiers and 14200 in buildings"
        );
        let side = line(false);
        assert!(side.starts_with("traded: by our seats' units in the last 3 min, lost 13345 metal in soldiers and 640 in buildings; destroyed 21529 metal of his in soldiers and 0 in buildings | whole game: "), "{side}");
        assert!(side.ends_with(" (each death counted once across our seats)"), "{side}");
    }

    /// The side's lines (H-REPORT-ALLIES-LINE, H-REPORT-GAME-END, H-REPORT-SIDE-SPOTS): each allied seat on one line's
    /// worth with its tier and when it reached it, the lobby's game-end rule, and the spots our side's box is nearer.
    #[test]
    fn the_side_lines_say_the_allies_tier_the_game_end_rule_and_the_spots_share() {
        use super::super::shared::AllySeat;
        let briefing = Briefing {
            frame: 9 * 1800,
            allies: vec![
                AllySeat { team: 7, who: "Apex (an AI)".into(), colour: "orange".into(), faction: "Legion".into(), extractors: 8, mohos: 1, factories: 2, tier2_since: Some(5 * 1800 + 20 * 30), constructors: 20, advanced_constructors: 1, nanos: 4, big_energy: 2, converters: 6, soldiers: 25, army_metal: 2135, army_at: "G4".into(), income_estimate: 41.8 },
                AllySeat { team: 9, who: "Within Reason (another seat of this bot)".into(), extractors: 9, factories: 2, constructors: 8, nanos: 1, soldiers: 58, army_metal: 5933, army_at: "G3".into(), income_estimate: 29.9, ..AllySeat::default() },
            ],
            game_end: "a side loses when it has no commanders left (the lobby's rule), so a seat whose commander dies plays on while another of ours stands; our side has 5 commanders standing of the 5 its seats started with".into(),
            side_spots: Some((43, 84, 5)),
            ..Briefing::default()
        };
        let lines = front(&briefing, &Field::default(), &[], true);
        let allies = lines.iter().find(|l| l.starts_with("allies (")).expect("an allies line");
        assert!(allies.contains("t7 (Apex (an AI), orange, Legion): 2 factories (tier 2 since 5:20), 8 extractors + 1 mohos (about 42 metal a second from its extractors), 20 constructors + 1 advanced, 4 nanos, 2 fusions or advanced solars, 6 converters, army 25 soldiers worth 2135 mostly at G4"), "{allies}");
        assert!(allies.contains(" | t9 (Within Reason (another seat of this bot)): 2 factories (tier 1), 9 extractors (about 30 metal a second from its extractors), 8 constructors, 1 nanos, army 58 soldiers worth 5933 mostly at G3"), "{allies}");
        assert!(lines.iter().any(|l| l == "game end: a side loses when it has no commanders left (the lobby's rule), so a seat whose commander dies plays on while another of ours stands; our side has 5 commanders standing of the 5 its seats started with"), "{lines:?}");
        assert!(lines.iter().any(|l| l == "spots: 43 of the map's 84 are nearer our side's start box than his, for the 5 seats of our side (about 9 each); past them income grows only by upgrading"), "{lines:?}");
        // A tier-2 factory seen standing within the last 100 s is new; no allies, no rule, no boxes: no lines.
        let fresh = Briefing { frame: 5 * 1800 + 30 * 30, allies: vec![AllySeat { team: 7, factories: 1, tier2_since: Some(5 * 1800 + 20 * 30), ..AllySeat::default() }], ..Briefing::default() };
        assert!(super::allies_line(&fresh).unwrap().contains("(tier 2 since 5:20, new)"));
        let bare = front(&Briefing::default(), &Field::default(), &[], true);
        assert!(bare.iter().all(|l| !l.starts_with("allies (") && !l.starts_with("game end:") && !l.starts_with("spots:")), "{bare:?}");
    }

    /// With the engine's grids the scouting block says, of the cells not in sight now, which our radar covers and
    /// what is on them, which a jammer of his hides, and which are dark to both; without the grids it says nothing of
    /// radar (H-REPORT-RADAR-WORDS). The jammers, guns and shields lines appear only with something to say.
    #[test]
    fn the_scouting_block_tells_radar_covered_cells_from_dark_ones() {
        use super::super::shared::CellLook;
        use super::scouting_lines;
        let cell = |name: &str, seen_share: f32, ago: Option<i32>| CellLook { cell: name.to_string(), seen_share, ago, in_his_box: name.starts_with('G') || name.starts_with('H'), ..CellLook::default() };
        let radar = |name: &str, share: f32, contacts: usize, jammed: bool| CellLook { radar_share: Some(share), contacts, jammed, ..cell(name, 0.5, Some(400)) };
        let cells = vec![radar("A3", 1.0, 0, false), radar("A4", 0.9, 0, false), radar("F1", 0.8, 27, false), radar("G2", 1.0, 0, true), radar("H2", 0.0, 0, false), cell("B4", 1.0, Some(3))];
        let lines = scouting_lines(&cells, true);
        let radar_line = lines.iter().find(|l| l.contains("radar now")).expect("a radar line");
        assert!(radar_line.contains("under our radar with nothing on it: A3-A4"), "{radar_line}");
        assert!(radar_line.contains("under our radar with contacts of his: F1 (27 contacts)"), "{radar_line}");
        assert!(radar_line.contains("hidden from radar by a jammer of his: G2"), "{radar_line}");
        assert!(radar_line.contains("dark to our radar and sight (hills or range: nothing known of them now): H2"), "{radar_line}");
        assert!(!radar_line.contains("B4"), "a cell in sight now is not a radar matter: {radar_line}");
        // Most of the cells out of sight dark (the start of a game): the count, not the runs; all of them: said so.
        let mut mostly: Vec<CellLook> = ["A1", "A2", "A3", "B1", "B2", "B3", "C1", "C2", "C3"].iter().map(|c| radar(c, 0.0, 0, false)).collect();
        mostly.push(radar("D4", 1.0, 0, false));
        let lines = scouting_lines(&mostly, true);
        let radar_line = lines.iter().find(|l| l.contains("radar now")).expect("a radar line");
        assert!(radar_line.contains("under our radar with nothing on it: D4 | dark to our radar and sight (hills or range: nothing known of them now): all the rest out of our sight (9 cells)"), "{radar_line}");
        mostly.pop();
        let lines = scouting_lines(&mostly, true);
        let radar_line = lines.iter().find(|l| l.contains("radar now")).expect("a radar line");
        assert!(radar_line.ends_with("dark to our radar and sight (hills or range: nothing known of them now): every cell out of our sight now (9 cells)"), "{radar_line}");
        let plain = scouting_lines(&[cell("A3", 1.0, Some(400))], true);
        assert!(plain.iter().all(|l| !l.contains("radar now")), "{plain:?}");
        let mut briefing = super::super::shared::Briefing::default();
        assert!(super::front(&briefing, &super::super::shared::Field::default(), &[], false).iter().all(|l| !l.starts_with("jammers of his") && !l.starts_with("fire from out of sight") && !l.starts_with("shields of his")));
        briefing.jammers = vec!["Erebus at G2 (6192, 1088) hides 760 around it from radar (seen 9:26 ago)".into()];
        briefing.guns = vec!["guns_1: legbastion (a turret) (reach 1100) about F2 (5650, 1500)".into()];
        briefing.shields = vec!["Soteria at F1 (5424, 480): shield 550 around it".into()];
        let lines = super::front(&briefing, &super::super::shared::Field::default(), &[], false);
        assert!(lines.iter().any(|l| l.starts_with("jammers of his (each hides") && l.contains("Erebus at G2")), "{lines:?}");
        assert!(lines.iter().any(|l| l.starts_with("fire from out of sight (standing places `guns_N`") && l.contains("guns_1")), "{lines:?}");
        assert!(lines.iter().any(|l| l.starts_with("shields of his: Soteria")), "{lines:?}");
    }
}

#[cfg(test)]
mod inflow_tests {
    use super::*;
    use bot_protocol::Resource;

    /// What allies pour into our stores is said on the eco line beside the engine's income and counted in the flow
    /// words, and nothing is said when nothing arrives (the Glitters 2v6 of 2026-10-25 read "in balance" and "FULL"
    /// together while 30 to 47 a second arrived unread).
    #[test]
    fn the_eco_line_says_what_allies_pour_in() {
        let briefing = Briefing {
            metal: Resource { current: 14_800.0, storage: 14_900.0, income: 177.0, usage: 150.0 },
            energy: Resource { current: 1_000.0, storage: 3_000.0, income: 400.0, usage: 300.0 },
            received: (40.0, 0.0),
            ..Briefing::default()
        };
        let line = eco_line(&briefing);
        assert!(line.contains("metal 14800/14900 (+177.0/-150.0, and +40 a second arriving from allies; "), "{line}");
        assert!(line.contains("FULL") && line.contains("energy 1000/3000 (+400/-300; "), "{line}");
        let quiet = eco_line(&Briefing { received: (0.0, 0.0), ..briefing.clone() });
        assert!(!quiet.contains("allies"), "{quiet}");
    }
}
