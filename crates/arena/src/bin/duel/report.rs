//! `duels.csv` (one row per duel) and the tables made from it.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;

use crate::director::DuelResult;

pub const HEADER: &str = "match,site,sequence,x,y,rep,x_end,x_team,n_x,n_y,metal_x,metal_y,winner,reason,seconds,\
contact_seconds,survivors_x,survivors_y,value_left_x,value_left_y,damage_taken_x,damage_taken_y,spread_x,spread_y,\
form_x,form_y,reach_s_x,reach_s_y,shots_x,shots_y,muzzled_line_x,muzzled_line_y,muzzled_edge_x,muzzled_edge_y,\
muzzled_clear_x,muzzled_clear_y,ff_x,ff_y,dealt_x,dealt_y,health_err_x,health_err_y,\
lane_x,lane_y,nn_x,nn_y,fol_in_x,fol_in_y,fol_blocked_x,fol_blocked_y,xf_x,xf_y,engaged_s_x,engaged_s_y,queued_s_x,queued_s_y";

pub fn row(r: &DuelResult) -> String {
    let [fx, fy] = &r.fire;
    let fire = format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{:.0},{:.0},{:.0},{:.0},{:.3},{:.3}",
        r.formation[0], r.formation[1], fx.reach_seconds, fy.reach_seconds, fx.shots, fy.shots,
        fx.muzzled[0], fy.muzzled[0], fx.muzzled[1], fy.muzzled[1], fx.muzzled[2], fy.muzzled[2],
        fx.friendly_fire, fy.friendly_fire, fx.dealt, fy.dealt, r.health_error[0], r.health_error[1],
    );
    let [sx, sy] = &r.shape;
    let shape = format!(
        "{},{},{:.0},{:.0},{},{},{},{},{},{},{},{},{},{}",
        r.lane[0], r.lane[1], sx.median_nearest(), sy.median_nearest(), sx.in_reach, sy.in_reach, sx.blocked, sy.blocked, fx.victims_cell(), fy.victims_cell(),
        fx.engaged_seconds, fy.engaged_seconds, fx.queued_seconds, fy.queued_seconds
    );
    format!(
        "{},{},{},{},{},{},{},{},{},{},{:.0},{:.0},{},{},{:.1},{},{},{},{:.3},{:.3},{:.0},{:.0},{:.0},{:.0}",
        r.match_index, r.site, r.sequence, r.job.x, r.job.y, r.job.rep,
        if r.job.x_is_west() { "west" } else { "east" }, r.job.x_team(),
        r.count[0], r.count[1], r.metal[0], r.metal[1], r.winner, r.reason, r.seconds,
        r.contact_seconds.map_or(String::new(), |s| format!("{s:.1}")),
        r.survivors[0], r.survivors[1], r.value_left[0], r.value_left[1], r.damage_taken[0], r.damage_taken[1],
        r.spread_at_contact[0], r.spread_at_contact[1],
    ) + "," + &fire + "," + &shape
}

/// One pairing (in one pair of formations) seen from its first unit's side.
#[derive(Default, Clone)]
struct Tally {
    duels: u32,
    wins: u32,
    losses: u32,
    /// Sum over duels of (own value left - the other's value left).
    margin: f32,
    seconds: f32,
    own_count: u32,
    other_count: u32,
    /// The fire instrument summed over the duels that carry it (none in a batch from before 2026-09-24):
    /// duels, in-reach seconds, shots, muzzled seconds by cause, friendly fire, dealt, dealt by the other side.
    fire_duels: u32,
    reach_s: f32,
    shots: f32,
    muzzled: [f32; 3],
    ff: f32,
    dealt: f32,
    dealt_against: f32,
    /// The shape instrument (2026-09-25): the sum of the duels' median nearest-friend distances and how many
    /// carried one, and the friend-on-the-line counts, pooled.
    nn_sum: f32,
    nn_duels: u32,
    fol_in: f32,
    fol_blocked: f32,
}

/// Columns of the fire instrument, as `x` and `y`; batches from before 2026-09-24 have none of them.
const FIRE: [&str; 7] = ["reach_s", "shots", "muzzled_line", "muzzled_edge", "muzzled_clear", "ff", "dealt"];
/// Columns of the shape instrument (2026-09-25), likewise.
const SHAPE: [&str; 3] = ["nn", "fol_in", "fol_blocked"];

/// Reads raw duel rows (several files may be merged) and writes `pairs.csv`, `matrix.csv` and `matrix.md` to `out`.
pub fn write(inputs: &[&Path], out: &Path) -> io::Result<()> {
    let mut tallies: BTreeMap<(String, String, String, String, String), Tally> = BTreeMap::new();
    let mut units: Vec<String> = Vec::new();
    for input in inputs {
        let text = fs::read_to_string(input)?;
        let mut lines = text.lines();
        let header: Vec<&str> = lines.next().unwrap_or_default().split(',').collect();
        let column = |name: &str| header.iter().position(|h| *h == name).ok_or_else(|| io::Error::other(format!("{}: no column {name}", input.display())));
        let (x, y, n_x, n_y) = (column("x")?, column("y")?, column("n_x")?, column("n_y")?);
        let (winner, reason, seconds) = (column("winner")?, column("reason")?, column("seconds")?);
        let (left_x, left_y) = (column("value_left_x")?, column("value_left_y")?);
        let optional = |name: &str| header.iter().position(|h| *h == name);
        let forms = [optional("form_x"), optional("form_y")];
        let fire: Option<Vec<[usize; 2]>> =
            FIRE.iter().map(|c| Some([optional(&format!("{c}_x"))?, optional(&format!("{c}_y"))?])).collect();
        let shape: Option<Vec<[usize; 2]>> =
            SHAPE.iter().map(|c| Some([optional(&format!("{c}_x"))?, optional(&format!("{c}_y"))?])).collect();
        let lanes = [optional("lane_x"), optional("lane_y")];
        for line in lines {
            let f: Vec<&str> = line.split(',').collect();
            if f.len() != header.len() || f[reason] == "spawn_failed" {
                continue;
            }
            let number = |i: usize| f[i].parse::<f32>().unwrap_or(0.0);
            let margin = number(left_x) - number(left_y);
            let form = |i: usize| forms[i].map_or("", |c| f[c]);
            let lane = |i: usize| lanes[i].map_or("", |c| f[c]);
            // Per army: in-reach seconds, shots, muzzled by cause, friendly fire, dealt.
            let fired = fire.as_ref().map(|columns| [0, 1].map(|side| columns.iter().map(|c| number(c[side])).collect::<Vec<f32>>()));
            for name in [f[x], f[y]] {
                if !units.iter().any(|u| u == name) {
                    units.push(name.to_string());
                }
            }
            // Each duel counts once from either side; a unit against itself only once.
            // A mirror pairing in two different formations is two pairings, each seen from its first army.
            let views = [(f[x], f[y], margin, "x", n_x, n_y, 0), (f[y], f[x], -margin, "y", n_y, n_x, 1)];
            let mirror = f[x] == f[y] && form(0) == form(1) && lane(0) == lane(1);
            for (own, other, margin, won_as, own_n, other_n, side) in views.into_iter().take(if mirror { 1 } else { 2 }) {
                let key = (own.to_string(), other.to_string(), form(side).to_string(), form(1 - side).to_string(), lane(side).to_string());
                let tally = tallies.entry(key).or_default();
                tally.duels += 1;
                tally.wins += u32::from(f[winner] == won_as);
                tally.losses += u32::from(f[winner] != won_as && f[winner] != "draw");
                tally.margin += margin;
                tally.seconds += number(seconds);
                tally.own_count = number(own_n) as u32;
                tally.other_count = number(other_n) as u32;
                if let Some(fired) = &fired {
                    tally.fire_duels += 1;
                    // In a mirror both armies fought in the same condition, so both count.
                    let armies = if mirror { vec![0, 1] } else { vec![side] };
                    for army in armies {
                        let (mine, theirs) = (&fired[army], &fired[1 - army]);
                        tally.reach_s += mine[0];
                        tally.shots += mine[1];
                        for cause in 0..3 {
                            tally.muzzled[cause] += mine[2 + cause];
                        }
                        tally.ff += mine[5];
                        tally.dealt += mine[6];
                        tally.dealt_against += theirs[6];
                        if let Some(shape) = &shape {
                            let nn = f[shape[0][army]].parse::<f32>().unwrap_or(f32::NAN);
                            if nn.is_finite() {
                                tally.nn_sum += nn;
                                tally.nn_duels += 1;
                            }
                            tally.fol_in += number(shape[1][army]);
                            tally.fol_blocked += number(shape[2][army]);
                        }
                    }
                }
            }
        }
    }

    // The first ten columns are the 2026-09-19 layout (`combatsim` reads them by position); the rest were added
    // 2026-09-24 and are empty for duels without the fire instrument.
    let mut pairs = String::from(
        "unit,against,n_unit,n_against,duels,wins,losses,draws,mean_margin,mean_seconds,formation,formation_against,\
         shots_per_reach_s,muzzled_share,muzzled_line,muzzled_edge,muzzled_clear,ff_share,exchange,lane,nearest_friend,friend_on_line\n",
    );
    for ((own, other, form, form_against, lane), t) in &tallies {
        let shape = format!(
            "{lane},{},{}",
            if t.nn_duels > 0 { format!("{:.0}", t.nn_sum / t.nn_duels as f32) } else { String::new() },
            if t.fol_in > 0.0 { format!("{:.3}", t.fol_blocked / t.fol_in) } else { String::new() },
        );
        let fire = if t.fire_duels == 0 {
            ",,,,,,".to_string()
        } else {
            let muzzled: f32 = t.muzzled.iter().sum();
            let share = |n: f32| if muzzled > 0.0 { format!("{:.3}", n / muzzled) } else { String::new() };
            format!(
                "{:.3},{:.3},{},{},{},{:.3},{}",
                t.shots / t.reach_s.max(1.0), muzzled / t.reach_s.max(1.0), share(t.muzzled[0]), share(t.muzzled[1]),
                share(t.muzzled[2]), t.ff / (t.dealt + t.ff).max(1.0),
                if t.dealt_against > 0.0 { format!("{:.3}", t.dealt / t.dealt_against) } else { String::new() },
            )
        };
        let _ = writeln!(
            pairs, "{own},{other},{},{},{},{},{},{},{:.3},{:.1},{form},{form_against},{fire},{shape}",
            t.own_count, t.other_count, t.duels, t.wins, t.losses, t.duels - t.wins - t.losses,
            t.margin / t.duels as f32, t.seconds / t.duels as f32
        );
    }
    fs::write(out.join("pairs.csv"), pairs)?;

    // Rows fight columns: +100 means the row unit won untouched, -100 that it died without scratching the column unit.
    // Formations pooled.
    let cell = |own: &str, other: &str| {
        let views = tallies.iter().filter(|((o, a, _, _, _), _)| o == own && a == other).map(|(_, t)| t);
        let (margin, duels) = views.fold((0.0, 0), |(m, d), t| (m + t.margin, d + t.duels));
        (duels > 0).then(|| (100.0 * margin / duels as f32).round() as i32)
    };
    let mut csv = format!("unit,{}\n", units.join(","));
    let mut md = format!("| row vs column | {} | mean |\n|---|{}---|\n", units.join(" | "), "---|".repeat(units.len()));
    for own in &units {
        let cells: Vec<Option<i32>> = units.iter().map(|other| cell(own, other)).collect();
        let text: Vec<String> = cells.iter().map(|c| c.map_or(String::new(), |v| v.to_string())).collect();
        let known: Vec<i32> = cells.iter().flatten().copied().collect();
        let mean = known.iter().sum::<i32>() as f32 / known.len().max(1) as f32;
        let _ = writeln!(csv, "{own},{}", text.join(","));
        let _ = writeln!(md, "| **{own}** | {} | {mean:.0} |", text.join(" | "));
    }
    fs::write(out.join("matrix.csv"), csv)?;
    fs::write(out.join("matrix.md"), md)
}
