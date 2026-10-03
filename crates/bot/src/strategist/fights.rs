//! Fights as records, for the commander's report (`command.rs`): every death of ours and every death of his we
//! saw, with what stood near it, gathered by place and time into engagements: where, when, what of ours met what
//! of his, what each side lost. (duel-split-1: the commander sent Blitzes into six Janus at F5 twice; the first
//! loss reached it as a tally line and a smaller army number.)

use std::collections::BTreeMap;

/// A death joins a fight whose last death was within this distance and this long ago.
const NEAR: f32 = 1000.0;
const QUIET_FRAMES: i32 = 20 * 30;
/// What stood within this of a death is counted as there.
pub const THERE: f32 = 900.0;
/// A fight is written on a line of its own when either side lost this much metal; the rest are summed.
const WORTH_A_LINE: f32 = 200.0;
/// The most fights written, newest kept.
const LINES: usize = 12;

/// One death, as the seat that saw it published it.
#[derive(Clone, Debug, Default)]
pub struct Death {
    pub frame: i32,
    pub unit: u32,
    pub ours: bool,
    pub name: String,
    pub metal: f32,
    pub at: (f32, f32),
    pub cell: String,
    /// The publishing seat's soldiers within `THERE` of the death, the dead one not counted: how many, their metal.
    pub ours_there: (usize, f32),
    /// His units in sight within `THERE`, by type, and their metal.
    pub his_there: Vec<(String, usize)>,
    pub his_there_metal: f32,
}

#[derive(Debug, Default)]
struct Fight {
    from: i32,
    to: i32,
    at: (f32, f32),
    cells: BTreeMap<String, usize>,
    lost: BTreeMap<String, (usize, f32)>,
    killed: BTreeMap<String, (usize, f32)>,
    /// The largest count of ours seen there at one death (with the dead one), and its metal.
    ours_there: (usize, f32),
    his_there: BTreeMap<String, usize>,
    his_there_metal: f32,
}

fn gather(deaths: &[Death]) -> Vec<Fight> {
    let mut fights: Vec<Fight> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut ordered: Vec<&Death> = deaths.iter().collect();
    ordered.sort_by_key(|d| d.frame);
    for d in ordered {
        // His death is published by every seat that saw it: counted once.
        if !seen.insert((d.ours, d.unit)) {
            continue;
        }
        let open = fights.iter_mut().rev().find(|f| d.frame - f.to <= QUIET_FRAMES && ((f.at.0 - d.at.0).powi(2) + (f.at.1 - d.at.1).powi(2)).sqrt() <= NEAR);
        let fight = match open {
            Some(fight) => fight,
            None => {
                fights.push(Fight { from: d.frame, at: d.at, ..Fight::default() });
                fights.last_mut().expect("just pushed")
            }
        };
        fight.to = d.frame;
        fight.at = d.at;
        *fight.cells.entry(d.cell.clone()).or_default() += 1;
        let side = if d.ours { &mut fight.lost } else { &mut fight.killed };
        let entry = side.entry(d.name.clone()).or_default();
        *entry = (entry.0 + 1, entry.1 + d.metal);
        let ours = (d.ours_there.0 + usize::from(d.ours), d.ours_there.1 + if d.ours { d.metal } else { 0.0 });
        if ours.1 > fight.ours_there.1 {
            fight.ours_there = ours;
        }
        for (name, count) in &d.his_there {
            let most = fight.his_there.entry(name.clone()).or_default();
            *most = (*most).max(*count);
        }
        fight.his_there_metal = fight.his_there_metal.max(d.his_there_metal);
    }
    fights
}

fn clock(frame: i32) -> String {
    format!("{}:{:02}", frame / 30 / 60, frame / 30 % 60)
}

fn counted(units: &BTreeMap<String, (usize, f32)>) -> String {
    let mut list: Vec<(&String, &(usize, f32))> = units.iter().collect();
    list.sort_by(|a, b| b.1.1.total_cmp(&a.1.1));
    list.iter().map(|(name, (count, _))| format!("{count} {name}")).collect::<Vec<_>>().join(", ")
}

/// The block: a line a fight, oldest first, and one line for the small ones. Empty when nothing has died.
pub fn lines(deaths: &[Death], now: i32) -> Vec<String> {
    let fights = gather(deaths);
    if fights.is_empty() {
        return Vec::new();
    }
    // A fold from 0.0: an empty `sum` of floats is -0.0 and prints so.
    let metal = |units: &BTreeMap<String, (usize, f32)>| units.values().fold(0.0, |total, (_, m)| total + m);
    let (big, small): (Vec<&Fight>, Vec<&Fight>) = fights.iter().partition(|f| metal(&f.lost) >= WORTH_A_LINE || metal(&f.killed) >= WORTH_A_LINE);
    let mut lines = vec![format!(
        "fights so far (deaths gathered by place and time; \"there\" is within {THERE:.0} of a death, ours counted by the seat that lost or killed, his only what was in sight; metal in brackets){}:",
        if big.len() > LINES { format!(", the last {LINES} of {}", big.len()) } else { String::new() }
    )];
    for f in big.iter().skip(big.len().saturating_sub(LINES)) {
        let cell = f.cells.iter().max_by_key(|(_, n)| **n).map_or("?", |(c, _)| c.as_str());
        let mut his: Vec<(&String, &usize)> = f.his_there.iter().collect();
        his.sort_by(|a, b| b.1.cmp(a.1));
        let when = if f.to - f.from < 30 { clock(f.from) } else { format!("{}-{}", clock(f.from), clock(f.to)) };
        lines.push(format!(
            "  {when} at {cell}{}: ours there at the most {} soldiers ({:.0}); his seen there at the most {} ({:.0}). We lost {} ({:.0}); he lost {} ({:.0}).",
            if now - f.to <= QUIET_FRAMES { ", still going" } else { "" },
            f.ours_there.0, f.ours_there.1,
            if his.is_empty() { "nothing in sight".to_string() } else { his.iter().map(|(name, count)| format!("{count} {name}")).collect::<Vec<_>>().join(", ") },
            f.his_there_metal,
            if f.lost.is_empty() { "nothing".to_string() } else { counted(&f.lost) }, metal(&f.lost),
            if f.killed.is_empty() { "nothing we saw die".to_string() } else { counted(&f.killed) }, metal(&f.killed),
        ));
    }
    if !small.is_empty() {
        lines.push(format!(
            "  and {} smaller clashes (under {WORTH_A_LINE:.0} metal a side): we lost {:.0} in them, he lost {:.0}",
            small.len(), small.iter().fold(0.0, |total, f| total + metal(&f.lost)), small.iter().fold(0.0, |total, f| total + metal(&f.killed))
        ));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn death(second: i32, ours: bool, unit: u32, name: &str, metal: f32, at: (f32, f32), cell: &str) -> Death {
        Death { frame: second * 30, unit, ours, name: name.into(), metal, at, cell: cell.into(), ours_there: (10, 1100.0), his_there: vec![("armjanus".into(), 6), ("armllt".into(), 1)], his_there_metal: 1900.0 }
    }

    /// Deaths close in place and time are one fight with both sides' losses and what stood there; a death far off
    /// or long after is another; his death seen by two seats counts once; small ones are summed.
    #[test]
    fn deaths_gather_into_fights_by_place_and_time() {
        let mut deaths = vec![
            death(680, true, 1, "armflash", 110.0, (5000.0, 3000.0), "F5"),
            death(685, true, 2, "armflash", 110.0, (5100.0, 3050.0), "F5"),
            death(690, false, 900, "armjanus", 270.0, (5200.0, 3000.0), "F5"),
            death(690, false, 900, "armjanus", 270.0, (5200.0, 3000.0), "F5"),
            death(700, true, 3, "armflash", 110.0, (5150.0, 3100.0), "F5"),
            // Elsewhere at the same time: a lone extractor.
            death(690, true, 4, "armmex", 50.0, (1000.0, 800.0), "B2"),
            // The same place two minutes on: another fight.
            death(830, true, 5, "armstump", 240.0, (5100.0, 3000.0), "F5"),
        ];
        deaths.swap(0, 4);
        let lines = lines(&deaths, 2000 * 30);
        assert_eq!(lines.len(), 4, "{lines:#?}");
        assert_eq!(lines[1], "  11:20-11:40 at F5: ours there at the most 11 soldiers (1210); his seen there at the most 6 armjanus, 1 armllt (1900). We lost 3 armflash (330); he lost 1 armjanus (270).");
        assert!(lines[2].starts_with("  13:50 at F5: ") && lines[2].contains("We lost 1 armstump (240); he lost nothing we saw die (0)."), "{}", lines[2]);
        assert_eq!(lines[3], "  and 1 smaller clashes (under 200 metal a side): we lost 50 in them, he lost 0");
        assert!(super::lines(&deaths, 835 * 30)[2].contains("at F5, still going"));
        assert!(super::lines(&[], 0).is_empty());
    }
}
