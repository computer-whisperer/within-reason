//! H-MICRO-INTERCEPT: a wing of fighters kept up over a place and flown at the aircraft that come at it
//! (`docs/design/2026-11-16-air-intercept.md`; the user, 2026-11-16: bombing runs "have to be intercepted with
//! fighters before they reach the base", and the player hands the wing to the lane as it hands a roving group).
//!
//! Two parts. The **law** is how one fighter gets behind a body of aircraft and stays there (the stern
//! conversion, drilled against the two recorded runs and fifteen made-up raids, §6 of the note). The **loop** is
//! what the wing answers, with how many fighters, how far from its place, and how it comes back (§8, re-derived
//! after the first game, where a loop that answered whatever flew within 4,500 took fourteen Falcons 4,000 out at
//! one bomber milling over its own station and then chased it home).
//!
//! **Why a law and not an attack order.** A fighter ordered at a bomber that is coming at it meets it nose to nose:
//! the closing speed is both speeds together (a Falcon's 289 and a loaded Hailstorm's 200 to 250, 500 a second), its
//! missile reaches 680, so it fires two or three times, passes, flies straight on until the bomber is 617 behind it
//! (the engine locks a plane's turn while its goal is behind it and nearer than its turn radius), takes three to
//! four seconds to come round and is then 1,100 to 1,400 behind something it gains 60 a second on
//! (K-engine-a-fighter-sent-at-a-plane-flying-at-it-passes-it-and-is-left-behind). Behind a bomber the same fighter
//! fires every reload for as long as the bomber lives, and the engine itself throttles it back there
//! (K-engine-a-fighter-behind-its-target-is-held-there-and-fires-every-reload). So the wing does not fly at the
//! bombers. Each fighter flies to a point beside their track and ahead of them, turns before they pass, and rolls
//! out behind the last of them: the stern conversion.
//!
//! **The law.** For a body of contacts flying together, with the rearmost of them at `r`, their heading `u` and
//! speed `s`, a fighter whose half turn costs `TURN_SECONDS` of the body's flight and carries it `TURN_ACROSS`
//! sideways (K-engine-a-fighters-turn-costs-four-seconds-of-the-bombers-flight):
//! - **Close.** It flies straight at the point where it must begin its turn: `CLOSE_ACROSS` to its own side of the
//!   track (near enough for two shots on the way in) and `s x TURN_SECONDS - TRAIL` ahead of where `r` will be when
//!   it gets there (the point is solved for the two speeds each tick).
//! - **Turn.** The turn begins once it is no further ahead of them along the track than that point, or a turn begun
//!   now would roll it out `TRAIL` behind `r` (the turn it needs is read from its own heading). It goes round toward
//!   the track, by a move to a point abeam on that side (to a point dead astern the engine turns a plane either
//!   way), then to the point `TURN_AHEAD` ahead of where `r` will be when the turn is done. The turn is watched as
//!   it is flown: while it would roll the fighter out nearer than `TRAIL - HOLD_BAND` behind `r`, or ahead of it
//!   (the bombers slowed or turned meanwhile, as they do for their run), the fighter flies on against their heading,
//!   or across the track once it is round, and they pass.
//! - **Trail.** Once it flies within `ALONG_DEG` of their heading with a contact ahead of it and near, it attacks
//!   by id, and the engine's pursuit holds it there. Targets are dealt from the rear of the body forward (everything
//!   it has yet to kill is then in front of it), each contact covered by the fighters that kill it in `WINDOW` and
//!   no more, and a fighter is dealt only what is in front of it. A fighter with nothing of the body in front of it
//!   goes back to the turn's rule, which takes it across the track until they have passed.
//!
//! A body slower than `SLOW` (a gunship that hovers) is attacked as it is, and so is one the fighter cannot stay
//! behind (`CATCH`: a Liche or a Stiletto to a Falcon).
//!
//! **The loop.** The wing guards a place; it does not hunt.
//! - **What it answers.** A contact is a threat when it is coming at the place (`coming`: its course passes within
//!   `GUARD` of it and it is within `THREAT_SECONDS` of its nearest point to it) and has held that course for
//!   `HOLD_FRAMES` (within `STRAIGHT_DEG`: a raid on its leg flies straight, an aircraft milling points at the
//!   place a second in each round), or when it is inside `GUARD` whatever its course. A contact taken on stays the
//!   wing's until it has been beyond `LEASH` and not coming for `RELEASE_FRAMES`. Aircraft that pass abeam or mill
//!   about their own station are counted (`watched`) and left.
//! - **With how many.** Each body of threats is given the fighters that kill it: for each member, the fighters
//!   whose anti-air damage takes its health in `WINDOW`. Bodies are served soonest first; a fighter keeps the body
//!   it has; free fighters are taken nearest the body's rear. The rest stay on the loop.
//! - **How far.** No point and no target further than `LEASH` from the place: a fighter's point beyond it is
//!   pulled in to it along the way, a target beyond it and not coming is not followed, and a fighter found
//!   `SLACK` beyond it is in the return whatever it was doing.
//! - **The way back.** A fighter with no threat flies a move order to the place under hold position
//!   (K-engine-a-fighter-on-a-patrol-order-chases-what-it-sees: a patrol order from out there is a chase with no
//!   place in it), and inside `HOME` of it its loop (`Command::Patrol`, four corners, each fighter beginning at the
//!   next), also under hold position. In the air, where the drills' landed wing answered three to four seconds
//!   later and died in the block's own blast. A fighter sent at a threat flies under manoeuvre.
//!
//! What flies: an aircraft by its type when the type is known, and a radar contact by its height over the ground
//! and its speed, which the snapshot carries before the type is seen
//! (K-snapshot-a-radar-contact-carries-its-height-and-its-velocity).
//!
//! Not in it: his fighters with the bombers (it flies the same conversion on whatever flies, and counts a fighter
//! of his as one of ours' work), a wing's losses to anti-air under the bombers' track, a target of his further
//! than `GUARD` from the wing's place (the raid's course does not pass the place, so the wing stays: §6.4's third
//! limit, now by design), and a way back for a fighter rolled out far to the side other than the engine's pursuit.

use std::collections::{HashMap, HashSet};

use bot_protocol::{Command, EnemyUnit, OwnUnit, UnitId, Vec3};

use crate::{FRAMES_PER_SECOND, Lane, View};

pub const INTERCEPT: &str = "H-MICRO-INTERCEPT";
/// The loop's bounds (`docs/design/2026-11-16-air-intercept.md` §8.1). A contact is coming at the place when its
/// course passes within `GUARD` of it and it is within `THREAT_SECONDS` of its nearest point to it at its speed (a
/// Hailstorm flying straight in, 4,500 out: 18 seconds, which is what the conversion needs with what these radars
/// gave); anything that flies inside `GUARD` is the wing's whatever its course. No fighter is given a point or a
/// target further than `LEASH` from the place: the wing met a raid flying straight in 2,400 out, and a chase ends
/// there.
pub const GUARD: f32 = 1500.0;
pub const LEASH: f32 = 3000.0;
pub const THREAT_SECONDS: f32 = 18.0;
/// A fighter found this much beyond the leash is in the return whatever it was doing.
const SLACK: f32 = 500.0;
/// A fighter with no threat flies a move order to the place while it is further than this from it, and its loop
/// inside; one on the loop is in the return again when something has carried it half as far again.
const HOME: f32 = 1000.0;
const HOME_OUT: f32 = 1500.0;
/// A contact's course holds once it has been coming at the place this long with its heading no more than
/// `STRAIGHT_DEG` off what it was when that began: an aircraft milling about its own station turns thirty degrees
/// a second and points at the place a second in each round; a raid on its leg flies straight (air-patrol-1: one
/// Stormbringer milling 4,100 to 4,900 out took fourteen Falcons off their place).
const HOLD_FRAMES: i32 = FRAMES_PER_SECOND;
const STRAIGHT_DEG: f32 = 15.0;
/// A contact the wing has taken on stays its until it has been beyond the leash and not coming for this long.
const RELEASE_FRAMES: i32 = 3 * FRAMES_PER_SECOND / 2;
/// A contact out of sight is forgotten after this long.
const FORGET_FRAMES: i32 = 3 * FRAMES_PER_SECOND;
/// A fighter's half turn at speed: the seconds it costs and how far across it carries the plane. Read from the
/// drills' tracks of Falcons and Highwinds: 0.7 s before the nose moves, then 60 to 90 degrees a second, 350 to 620
/// across; and the plane comes out of it at 140 to 200 a second, slower than the bombers, and takes three seconds to
/// have its speed again, so the turn's cost in distance behind them is that of four seconds, not of the three it
/// takes to come round (at 3.4 the wing rolled out 300 behind as meant and was 450 to 620 behind two seconds later,
/// where a missile on a bomber flying away just reaches: air-I8-run2-f12).
pub const TURN_SECONDS: f32 = 4.0;
pub const TURN_ACROSS: f32 = 550.0;
/// Of the turn's seconds, what passes before the plane has banked and its nose begins to come round: every turn
/// costs it, however small.
const ROLL_IN: f32 = 0.7;
/// How far to its own side of the track a fighter closes: less than its turn carries it across, so the bombers are
/// within its missile's reach and its nose's cone from 680 out until it is well into the turn, two shots a fighter
/// (it then rolls out that much beyond the track, which the pursuit takes up). Closing 450 to the side took the
/// salvo away: the wing's damage before the first bomb fell from 83 % of the bombers' health to 69 % and run 1 from
/// 6 kept of 8 to 2 (air-I4-run1-f12, air-I6-run1-f12).
const CLOSE_ACROSS: f32 = 200.0;
/// How far behind the rearmost contact a fighter means to roll out: inside its missile's reach on something flying
/// away from it (a Falcon's missile flies 1.2 s, 820 elmos, and a Hailstorm 250 in that time), and outside the 300
/// under which the engine stops a fighter's attack run (BAR's `attacksafetydistance`).
pub const TRAIL: f32 = 300.0;
/// The order's delay the turn's trigger allows for: the lane's tick and the engine's frame.
const LATENCY: f32 = 0.2;
/// A fighter in its turn does not come round while the turn would roll it out this much nearer the rearmost contact
/// than `TRAIL`, or ahead of it (the bombers slowed or turned while it came round: in both recorded runs they
/// slow from 250 to 195 and turn 40 to 50 degrees for their run just where the wing meets them): while it still
/// flies against them it flies on against their heading, and once it is round past `INSIDE_DEG` it flies across the
/// track, where it gains nothing along it and they pass. Rolled out ahead of them it has no shot and the engine
/// flies it straight on until they are 617 behind it: the wing's fighters were ahead of the rearmost bomber, flying
/// its way, for 27 to 56 % of their seconds from the merge to the bombs at 0.1 hits a second, against 0.7 in the
/// 22 to 43 % they were behind it and in reach (air-I9-*).
/// Rolled out ahead of them it has no shot and the engine flies it straight on until they are 617 behind it
/// (air-I3-run1-f12: a Falcon seven seconds ahead of bombers that had jinked at the merge).
const HOLD_BAND: f32 = 200.0;
/// And for no longer than this: after it the turn goes on whatever it would come to.
const HOLD_SECONDS: f32 = 3.0;
/// While a turning fighter's heading is more than this off the body's, its move goes to a point this far to the
/// track's side of it and this far along the body's heading: the turn toward the track, said plainly.
const INSIDE_DEG: f32 = 110.0;
/// An attacking fighter with nothing of its body in front of it for this many ticks goes back to the turn.
const AHEAD_TICKS: i32 = 3;
const INSIDE_ACROSS: f32 = 700.0;
const INSIDE_ALONG: f32 = 350.0;
/// The turn's move goes to the point this far ahead of where the rearmost contact will be when the turn is done:
/// far enough from the fighter for the engine to turn it at once, near enough that a fighter beside the track comes
/// onto it (a point 1,500 ahead flew a fighter 400 beside the bombers level with them for four seconds).
const TURN_AHEAD: f32 = 500.0;
/// A contact slower than this (elmos a second) is attacked as it is.
const SLOW: f32 = 90.0;
/// So is one flying at this share of the fighter's own speed or more: behind it the fighter, slow from its turn,
/// is left at once (a Falcon at 289 on a Liche at 295 or a Stiletto at 300: 0.13 to 0.17 hits a second in the
/// place behind them, the bombers that dropped the same with the wing as without, air-R-liche-\*, air-R-stil-\*).
/// The pass nose to nose is all there is to have.
const CATCH: f32 = 0.93;
/// A radar contact flies when it is this high over the ground and moving at all, or faster than anything on the
/// ground.
const FLYING_HEIGHT: f32 = 60.0;
const FLYING_MOVING: f32 = 45.0;
const FLYING_FAST: f32 = 190.0;
/// Contacts within this of one another are one body.
const BODY: f32 = 700.0;
/// A contact whose type (and health) is not known yet is weighed as this much health.
const BLIP_HEALTH: f32 = 1000.0;
/// A contact is covered by the fighters whose fire kills it in this many seconds.
pub const WINDOW: f32 = 2.0;
/// The turn ends for the attack once the fighter flies within `ALONG_DEG` of the body's heading with a contact ahead
/// of it within this share of its reach, or after the turn's time and two seconds. (Without the heading's test the
/// attack began at the turn's first tick, with the bombers in front of a fighter flying at them: the pass nose to
/// nose again, air-I1-run2-f12.)
const ALONG_DEG: f32 = 60.0;
const NEAR_REACH: f32 = 1.5;
/// The close's move is a heading asked of the fighter toward a point far off: it is sent again when the bearing from
/// the fighter to the point is this far off the bearing to the point it was last sent, no sooner than
/// `CLOSE_RESEND_FRAMES` after; when the fighter is within `ARRIVING` of the point it has (an aircraft with no order
/// lands); or when it is idle. Not when the point has shifted 120, as the rule first was: by that a fighter was
/// sent eight to fourteen moves in the three to five seconds of a close on the two recorded runs (air-D0-*), most
/// of them a meeting point refined along its own bearing. At ten degrees and a second the wing kept what it kept
/// (run 1 37 of 40, run 2 39 of 40, the same shares of its seconds in each place from the merge to the bombs) on
/// half the conversion's moves (air-B2-*, air-B2b-* against air-B0-*, air-B0b-*;
/// `docs/design/2026-11-16-air-intercept.md` 8.6).
const CLOSE_RESEND_DEG: f32 = 10.0;
const CLOSE_RESEND_FRAMES: i32 = FRAMES_PER_SECOND;
const ARRIVING: f32 = 300.0;
/// The turn's move is a point beside the fighter itself, steering it tick by tick: again when the point has shifted
/// this far, no sooner than this many frames after the last. (By bearing the turn was sent twice as often: the
/// point moves with the fighter, and the bearing to the old one swings as it comes round; air-D1-*.)
const TURN_RESEND_MOVED: f32 = 240.0;
const RESEND_FRAMES: i32 = 6;
/// The move's point is carried this far beyond the turn's point along the fighter's way, so the plane never
/// arrives at it (an aircraft with no order lands).
const BEYOND: f32 = 800.0;
/// The side of the loiter's loop round the place, and how often an idle fighter is told it again.
const LOITER_LOOP: f32 = 700.0;
const LOITER_RESEND: i32 = 2 * FRAMES_PER_SECOND;
/// A standing attack goes out again this often, and this often to a fighter with no order.
const ATTACK_RESEND: i32 = 5 * FRAMES_PER_SECOND;
const ATTACK_IDLE_RESEND: i32 = FRAMES_PER_SECOND / 2;
/// The engine's move states. A fighter on a threat flies under manoeuvre, as every drill of the conversion did. On
/// the loop and on its way back it is on hold position: the engine's own search for targets, a thousand elmos round
/// the fighter wherever the fighter is, is a second loop with no place in it (air-patrol-1: a patrol order under
/// manoeuvre given 4,000 from home beside his fighters was a chase to his station, ten of fourteen Falcons). What
/// flies near the loop is inside `GUARD` and the wing's by its own rule.
const MOVESTATE_HOLD: i32 = 0;
const MOVESTATE_MANOEUVRE: i32 = 1;

/// The loop's bounds in force: the constants, or `WITHIN_REASON_INTERCEPT` for a drill
/// (`guard=2000,leash=3500,seconds=18,closedeg=6,closegap=0.5`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub guard: f32,
    pub leash: f32,
    pub seconds: f32,
    /// The close's sending rule: degrees (`closedeg`) and seconds (`closegap`).
    pub close_deg: f32,
    pub close_gap: f32,
}

pub fn bounds() -> Bounds {
    static BOUNDS: std::sync::OnceLock<Bounds> = std::sync::OnceLock::new();
    *BOUNDS.get_or_init(|| {
        let mut bounds = Bounds { guard: GUARD, leash: LEASH, seconds: THREAT_SECONDS, close_deg: CLOSE_RESEND_DEG, close_gap: CLOSE_RESEND_FRAMES as f32 / FRAMES_PER_SECOND as f32 };
        for pair in std::env::var("WITHIN_REASON_INTERCEPT").unwrap_or_default().split(',') {
            match pair.split_once('=').map(|(key, value)| (key.trim(), value.trim().parse::<f32>())) {
                Some(("guard", Ok(v))) => bounds.guard = v,
                Some(("leash", Ok(v))) => bounds.leash = v,
                Some(("seconds", Ok(v))) => bounds.seconds = v,
                Some(("closedeg", Ok(v))) => bounds.close_deg = v,
                Some(("closegap", Ok(v))) => bounds.close_gap = v,
                _ => {}
            }
        }
        bounds
    })
}

/// A fighter by its numbers, of what flies: it hits what flies harder than the ground. A bomber's bombs stand in
/// the table as hitting aircraft at their ground damage and a gunship's guns at a share of it, and neither is a
/// wing's: a wing takes the fighters of its group and leaves the rest to the host.
pub fn fighter(stats: &crate::Stats) -> bool {
    stats.air_dps > stats.dps
}

/// What the lane reports of an intercept group, once each.
#[derive(Clone, Debug, PartialEq)]
pub struct InterceptEvent {
    pub group: String,
    pub what: InterceptWhat,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InterceptWhat {
    /// Fighters left the loop for a threat: this many contacts coming at the place or inside its guard, their
    /// centre, how far that is from the place, and how many fighters went.
    Scrambled { contacts: usize, at: Vec3, out: f32, fighters: usize },
    /// No threat is left and every fighter is back on the loop. `seconds` since the scramble.
    Returned { seconds: i32 },
}

/// What a wing is doing this tick, for the host's picture.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InterceptReport {
    /// The contacts it has taken on (coming at its place, or inside its guard), and the nearest of them to the place.
    pub contacts: usize,
    pub nearest: Option<Vec3>,
    /// What else flies in its sight within two leashes of the place, and the nearest of those.
    pub watched: usize,
    pub nearest_watched: Option<Vec3>,
    /// Its fighters on the loop, on their way back to it, closing, turning in, and behind a contact attacking.
    pub loitering: usize,
    pub returning: usize,
    pub closing: usize,
    pub turning: usize,
    pub trailing: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Phase {
    Loiter,
    /// On a move order back to the place.
    Return,
    Close,
    Turn { since: i32 },
    Trail,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Sent {
    /// The loop round this place.
    Loiter(Vec3),
    /// The move back to this place.
    Home(Vec3),
    Move(Vec3),
    Attack(UnitId),
}

/// The lane's memory of one fighter of a wing.
pub(crate) struct Interceptor {
    phase: Phase,
    /// The side of the track it closes on (+1 or -1 along the track's left normal), kept while it closes.
    side: f32,
    sent: Option<(Sent, i32)>,
    /// The contact it is dealt.
    target: Option<UnitId>,
    /// The ticks running in which nothing of its body was in front of it while it attacked.
    ahead: i32,
    /// The members of the body it is on, as of the last tick: a body is the same body while one of them is in it.
    with: Vec<UnitId>,
}

/// What a wing remembers of one contact that flies.
#[derive(Clone, Copy, Debug)]
struct Track {
    /// Since when it has been coming at the place on one course, and its heading when that began.
    since: Option<(i32, (f32, f32))>,
    /// The wing has taken it on.
    taken: bool,
    /// Since when, taken on, it has been beyond the leash and not coming.
    gone: Option<i32>,
    seen: i32,
}

/// The lane's memory of one wing.
#[derive(Default)]
pub(crate) struct Wing {
    scrambled: Option<i32>,
    tracks: HashMap<UnitId, Track>,
    pub(crate) report: InterceptReport,
}

/// A contact that flies, as the wing reads it.
#[derive(Clone, Copy, Debug)]
struct Contact {
    id: UnitId,
    pos: Vec3,
    /// Elmos a second, in the plane.
    vel: (f32, f32),
    /// Its health when its type is known, else `BLIP_HEALTH`.
    health: f32,
}

/// Contacts flying together: the unit vector they fly along, their speed, and the members from the rear forward.
struct Body {
    u: (f32, f32),
    speed: f32,
    members: Vec<Contact>,
}

fn sub(a: Vec3, b: Vec3) -> (f32, f32) {
    (a.x - b.x, a.z - b.z)
}

fn dot(a: (f32, f32), b: (f32, f32)) -> f32 {
    a.0 * b.0 + a.1 * b.1
}

fn cross(a: (f32, f32), b: (f32, f32)) -> f32 {
    a.0 * b.1 - a.1 * b.0
}

fn norm(a: (f32, f32)) -> f32 {
    a.0.hypot(a.1)
}

fn dir(a: (f32, f32)) -> Option<(f32, f32)> {
    let n = norm(a);
    (n > 1e-3).then(|| (a.0 / n, a.1 / n))
}

fn carried(p: Vec3, d: (f32, f32), by: f32) -> Vec3 {
    Vec3 { x: p.x + d.0 * by, y: p.y, z: p.z + d.1 * by }
}

/// The ground's height at a point; 0 from a host without terrain.
fn ground_height(view: &dyn View, at: Vec3) -> f32 {
    let Some(t) = view.terrain() else { return 0.0 };
    if t.cell <= 0.0 || t.width == 0 || t.height == 0 {
        return 0.0;
    }
    let x = ((at.x / t.cell) as i64).clamp(0, t.width as i64 - 1) as usize;
    let z = ((at.z / t.cell) as i64).clamp(0, t.height as i64 - 1) as usize;
    t.heights.get(z * t.width as usize + x).map_or(0.0, |h| (*h as f32).max(0.0))
}

/// Whether this unit of theirs flies: an aircraft by its type, or a contact of no known type by its height and speed.
fn flies(view: &dyn View, e: &EnemyUnit) -> bool {
    if let Some(d) = e.def.and_then(|d| view.def(d)) {
        return d.speed > 0.0 && d.move_class.is_none();
    }
    let speed = e.vel.x.hypot(e.vel.z) * FRAMES_PER_SECOND as f32;
    speed >= FLYING_FAST || (speed >= FLYING_MOVING && e.pos.y - ground_height(view, e.pos) >= FLYING_HEIGHT)
}

/// The contacts as bodies: joined while within `BODY` of a member, each body's heading and speed the mean of its
/// members' velocities, its members from the rear forward.
fn bodies(contacts: &[Contact]) -> Vec<Body> {
    let mut left: Vec<Contact> = contacts.to_vec();
    let mut out = Vec::new();
    while let Some(first) = left.pop() {
        let mut members = vec![first];
        let mut k = 0;
        while k < members.len() {
            let at = members[k].pos;
            let (near, far): (Vec<Contact>, Vec<Contact>) = left.iter().partition(|c| c.pos.dist2d(at) <= BODY);
            members.extend(near);
            left = far;
            k += 1;
        }
        let n = members.len() as f32;
        let mean = (members.iter().map(|c| c.vel.0).sum::<f32>() / n, members.iter().map(|c| c.vel.1).sum::<f32>() / n);
        let speed = norm(mean);
        let u = dir(mean).unwrap_or((1.0, 0.0));
        members.sort_by(|a, b| dot((a.pos.x, a.pos.z), u).total_cmp(&dot((b.pos.x, b.pos.z), u)).then(a.id.0.cmp(&b.id.0)));
        out.push(Body { u, speed, members });
    }
    out.sort_by_key(|b| b.members[0].id.0);
    out
}

/// The turn's point for a fighter that comes to it flying `turn` radians off the body's heading (a half turn when it
/// flies against them, a quarter when it comes across their track): how far ahead of the rearmost contact along the
/// track (behind it when negative), and how far to the fighter's own side of the track. The lead is what the body
/// flies in the turn's time, less what the turn itself carries the fighter along the track, less the trail; the
/// side is the turn's own width for that angle, in the share `CLOSE_ACROSS` is of a half turn's.
fn turn_geometry(s: f32, turn: f32) -> (f32, f32) {
    let seconds = ROLL_IN + (TURN_SECONDS - ROLL_IN) * turn / std::f32::consts::PI;
    (s * seconds - TURN_ACROSS / 2.0 * turn.sin() - TRAIL, CLOSE_ACROSS * (1.0 - turn.cos()) / 2.0)
}

/// Where a fighter at `f` flying `v` a second begins its turn on a body whose rearmost member is at `r`, on `side`
/// of its track (+1 the left), and the turn it will need there. The point depends on the heading the fighter comes
/// to it with, and that on the point: a few rounds settle it. (For a half turn alone, as it first was, a wing that
/// came across the bombers' track was sent 600 ahead of them, crossed in front of them and came round wide: nothing
/// kept of a target 2,000 from its place in 8 of 8 with fifteen seconds of warning, air-R3-asideP-wing.)
fn turn_point(f: Vec3, v: f32, r: Vec3, u: (f32, f32), s: f32, side: f32) -> (Vec3, f32) {
    let mut turn = std::f32::consts::PI;
    let mut at = f;
    for _ in 0..6 {
        let (lead, across) = turn_geometry(s, turn);
        at = meeting_point(f, v, r, u, s, lead, across * side);
        let Some(course) = dir(sub(at, f)) else { break };
        let next = dot(course, u).clamp(-1.0, 1.0).acos();
        if (next - turn).abs() < 0.03 {
            turn = next;
            break;
        }
        turn = 0.5 * (turn + next);
    }
    (at, turn)
}

/// Where a fighter at `f` flying `v` a second meets the point `across` to the left of the track (to its right when
/// negative) and `lead` ahead of a contact now at `r` flying `u` at `s`.
fn meeting_point(f: Vec3, v: f32, r: Vec3, u: (f32, f32), s: f32, lead: f32, across: f32) -> Vec3 {
    let n = (-u.1, u.0);
    let start = Vec3 { x: r.x + u.0 * lead + n.0 * across, y: f.y, z: r.z + u.1 * lead + n.1 * across };
    // |start + u s t - f| = v t, for the least t that is not negative.
    let c = sub(start, f);
    let (a, b, k) = (s * s - v * v, 2.0 * dot(c, u) * s, dot(c, c));
    let t = if a.abs() < 1e-3 {
        if b.abs() < 1e-3 { 0.0 } else { (-k / b).max(0.0) }
    } else {
        let disc = b * b - 4.0 * a * k;
        if disc < 0.0 {
            norm(c) / v.max(1.0)
        } else {
            let (t1, t2) = ((-b - disc.sqrt()) / (2.0 * a), (-b + disc.sqrt()) / (2.0 * a));
            [t1, t2].into_iter().filter(|t| *t >= 0.0).fold(f32::INFINITY, f32::min)
        }
    };
    let t = if t.is_finite() { t.min(60.0) } else { norm(c) / v.max(1.0) };
    carried(start, u, s * t)
}

/// How far behind `r` a fighter at `f` heading `h` (a unit vector) rolls out if it turns onto the body's heading
/// from now, with `roll_in` seconds still to pass before its nose comes round (`ROLL_IN` for a turn not begun): the
/// body flies `s` for the turn's time, and the turn itself carries the fighter along the track by its radius times
/// the sine of the turn.
fn rollout_behind(f: Vec3, h: (f32, f32), r: Vec3, u: (f32, f32), s: f32, roll_in: f32) -> f32 {
    let turn = dot(h, u).clamp(-1.0, 1.0).acos();
    let seconds = roll_in + (TURN_SECONDS - ROLL_IN) * turn / std::f32::consts::PI;
    let along = dot(sub(f, r), u);
    s * seconds - along - TURN_ACROSS / 2.0 * turn.sin()
}

impl Lane {
    /// What a wing is doing as of the last tick.
    pub fn intercept_report(&self, name: &str) -> Option<&InterceptReport> {
        self.wings.get(name).map(|w| &w.report)
    }

    /// Every wing's report by its group's name, for a host that reads the lane from another thread.
    pub fn intercept_reports(&self) -> HashMap<String, InterceptReport> {
        self.wings.iter().map(|(name, w)| (name.clone(), w.report.clone())).collect()
    }

    /// H-MICRO-INTERCEPT for one declared group of kind `Intercept` (the module's comment).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn intercept(&mut self, view: &dyn View, name: &str, place: Vec3, fighters: &[&OwnUnit], enemies: &[EnemyUnit], frame: i32, commands: &mut Vec<Command>, fired: &mut Vec<&'static str>, events: &mut Vec<InterceptEvent>, debug: bool) {
        let b = bounds();
        let map = extent(view);
        let mut wing = self.wings.remove(name).unwrap_or_default();
        // What flies in the seat's sight, wherever it is, each with its track: whether its course holds at the
        // place, and whether the wing has taken it on.
        let flying: Vec<Contact> = (enemies.iter())
            .filter(|e| flies(view, e))
            .map(|e| Contact { id: e.id, pos: e.pos, vel: (e.vel.x * FRAMES_PER_SECOND as f32, e.vel.z * FRAMES_PER_SECOND as f32), health: if e.def.is_some() && e.health > 0.0 { e.health } else { BLIP_HEALTH } })
            .collect();
        for c in &flying {
            let track = wing.tracks.entry(c.id).or_insert(Track { since: None, taken: false, gone: None, seen: frame });
            track.seen = frame;
            let out = c.pos.dist2d(place);
            let is_coming = coming(c, place, b);
            match (is_coming, dir(c.vel), track.since) {
                (true, Some(h), Some((_, began))) if dot(h, began) >= STRAIGHT_DEG.to_radians().cos() => {}
                (true, Some(h), _) => track.since = Some((frame, h)),
                _ => track.since = None,
            }
            let held = track.since.is_some_and(|(since, _)| frame - since >= HOLD_FRAMES);
            if !track.taken && (out <= b.guard || held) {
                track.taken = true;
                if debug {
                    eprintln!("{} f={frame} micro: {name} takes on #{} {out:.0} out ({}), {:.0} a second", view.label(), c.id.0, if out <= b.guard { "inside its guard" } else { "its course holds at the place" }, norm(c.vel));
                }
            }
            if track.taken && out > b.leash && !is_coming {
                let since = *track.gone.get_or_insert(frame);
                if frame - since >= RELEASE_FRAMES {
                    track.taken = false;
                    track.gone = None;
                    if debug {
                        eprintln!("{} f={frame} micro: {name} lets #{} go, {out:.0} out and not coming", view.label(), c.id.0);
                    }
                }
            } else {
                track.gone = None;
            }
        }
        wing.tracks.retain(|_, t| frame - t.seen <= FORGET_FRAMES);
        // What a fighter may be flown at: a contact inside the leash, or one coming at the place (following that
        // one brings the fighter in).
        let reachable = |c: &Contact| c.pos.dist2d(place) <= b.leash || coming(c, place, b);
        // The threats: the bodies with a member taken on and a member to fly at, the soonest at the place first. A
        // body that has left the leash and is not coming takes no fighter, though the wing has not let it go yet.
        let taken = |c: &Contact| wing.tracks.get(&c.id).is_some_and(|t| t.taken);
        let (mut bodies, others): (Vec<Body>, Vec<Body>) = bodies(&flying).into_iter().partition(|body| body.members.iter().any(taken) && body.members.iter().any(reachable));
        let soon = |body: &Body| {
            let near = body.members.iter().map(|c| c.pos.dist2d(place)).fold(f32::INFINITY, f32::min);
            if near <= b.guard { 0.0 } else { (near - b.guard) / body.speed.max(1.0) }
        };
        bodies.sort_by(|x, y| soon(x).total_cmp(&soon(y)));
        let threats: Vec<&Contact> = bodies.iter().flat_map(|body| &body.members).collect();
        let watched: Vec<&Contact> = others.iter().flat_map(|body| &body.members).filter(|c| c.pos.dist2d(place) <= 2.0 * b.leash).collect();
        let nearest = |of: &[&Contact]| of.iter().map(|c| c.pos).min_by(|x, y| x.dist2d(place).total_cmp(&y.dist2d(place)));
        let mut report = InterceptReport { contacts: threats.len(), nearest: nearest(&threats), watched: watched.len(), nearest_watched: nearest(&watched), ..InterceptReport::default() };
        for unit in fighters {
            self.interceptors.entry(unit.id).or_insert(Interceptor { phase: Phase::Loiter, side: 0.0, sent: None, target: None, ahead: 0, with: Vec::new() });
        }
        // Who flies at what. A fighter on a threat stays on it; each threat then takes the free fighters nearest it
        // until it has the fighters whose fire kills its members in `WINDOW` each; the rest stay on the loop. A
        // fighter beyond the leash and its slack is on none.
        let within = |unit: &OwnUnit| unit.pos.dist2d(place) <= b.leash + SLACK;
        let body_of = |id: UnitId| bodies.iter().position(|body| body.members.iter().any(|c| c.id == id));
        let mut on: Vec<usize> = vec![0; bodies.len()];
        let mut chosen: HashMap<UnitId, usize> = HashMap::new();
        for unit in fighters.iter().filter(|unit| within(unit)) {
            if let Some(index) = self.interceptors.get(&unit.id).and_then(|m| m.target).and_then(body_of) {
                chosen.insert(unit.id, index);
                on[index] += 1;
            }
        }
        let fire = |f: &OwnUnit| view.stats(f.def).map_or(0.0, |s| s.air_dps * WINDOW).max(1.0);
        let mean_fire = fighters.iter().map(|f| fire(f)).sum::<f32>() / fighters.len().max(1) as f32;
        for (index, body) in bodies.iter().enumerate() {
            let need: usize = body.members.iter().map(|c| (c.health / mean_fire.max(1.0)).ceil().max(1.0) as usize).sum();
            if on[index] >= need {
                continue;
            }
            let rear = body.members[0].pos;
            let mut free: Vec<&OwnUnit> = fighters.iter().copied().filter(|unit| !chosen.contains_key(&unit.id) && within(unit)).collect();
            free.sort_by(|x, y| x.pos.dist2d(rear).total_cmp(&y.pos.dist2d(rear)).then(x.id.0.cmp(&y.id.0)));
            for unit in free.into_iter().take(need - on[index]) {
                chosen.insert(unit.id, index);
                on[index] += 1;
            }
        }
        if !chosen.is_empty() && wing.scrambled.is_none() {
            wing.scrambled = Some(frame);
            let n = threats.len().max(1) as f32;
            let at = Vec3 { x: threats.iter().map(|c| c.pos.x).sum::<f32>() / n, y: 0.0, z: threats.iter().map(|c| c.pos.z).sum::<f32>() / n };
            fired.push(INTERCEPT);
            self.counts.0 += chosen.len() as u32;
            events.push(InterceptEvent { group: name.to_string(), what: InterceptWhat::Scrambled { contacts: threats.len(), at, out: at.dist2d(place), fighters: chosen.len() } });
            if debug {
                let first = &bodies[0];
                eprintln!("{} f={frame} micro: {name} scrambles {} of {} fighters at {} contacts {:.0} out (the first body: {} at {:.0} a second, {:.0} over the ground)", view.label(), chosen.len(), fighters.len(), threats.len(), at.dist2d(place), first.members.len(), first.speed, first.members[0].pos.y - ground_height(view, first.members[0].pos));
            }
        }
        // A fighter given a body it was not on (its own is dead or gone and another threat took it the same tick, so
        // it never went by the loop) begins the conversion anew: its side of the track, its turn's clock and its
        // place behind were the other body's. Without this a fighter trailing the first wave was sent an attack by
        // id at the second from wherever it was, which is the nose-to-nose pass the conversion is there to avoid.
        for unit in fighters {
            let memory = self.interceptors.get_mut(&unit.id).expect("entered above");
            let Some(index) = chosen.get(&unit.id) else {
                memory.with.clear();
                continue;
            };
            let members = &bodies[*index].members;
            if !matches!(memory.phase, Phase::Loiter | Phase::Return) && !members.iter().any(|c| memory.with.contains(&c.id)) {
                if debug {
                    eprintln!("{} f={frame} micro: #{} is given another body ({name}): its conversion begins anew from {:?}", view.label(), unit.id.0, memory.phase);
                }
                memory.phase = Phase::Close;
                memory.side = 0.0;
                memory.ahead = 0;
                memory.target = None;
            }
            memory.with = members.iter().map(|c| c.id).collect();
        }
        // The fighters behind a body are dealt its members from the rear forward, each member the fighters that kill
        // it in `WINDOW`: first the fighters that keep the contact they have, then the nearest of the rest.
        let mut dealt: HashMap<UnitId, UnitId> = HashMap::new();
        for (index, body) in bodies.iter().enumerate() {
            let trailing: Vec<&OwnUnit> = fighters.iter().copied().filter(|f| chosen.get(&f.id) == Some(&index) && self.interceptors.get(&f.id).is_some_and(|m| m.phase == Phase::Trail)).collect();
            let weight = |f: &OwnUnit| view.stats(f.def).map_or(0.0, |s| s.air_dps * WINDOW).max(1.0);
            let mut cover: HashMap<UnitId, f32> = HashMap::new();
            let mut free: HashSet<UnitId> = trailing.iter().map(|f| f.id).collect();
            for f in &trailing {
                let Some(target) = self.interceptors.get(&f.id).and_then(|m| m.target) else { continue };
                let Some(member) = body.members.iter().find(|c| c.id == target && reachable(c)) else { continue };
                let had = cover.entry(target).or_insert(0.0);
                if *had < member.health && dot(sub(member.pos, f.pos), (f.vel.x, f.vel.z)) > 0.0 {
                    *had += weight(f);
                    dealt.insert(f.id, target);
                    free.remove(&f.id);
                }
            }
            // A fighter is dealt what is in front of it: with its contact behind it the engine flies it straight on.
            let in_front = |f: &OwnUnit, c: &Contact| dot(sub(c.pos, f.pos), (f.vel.x, f.vel.z)) > 0.0;
            for member in body.members.iter().filter(|c| reachable(c)) {
                let mut near: Vec<&OwnUnit> = trailing.iter().copied().filter(|f| free.contains(&f.id) && in_front(f, member)).collect();
                near.sort_by(|a, b| a.pos.dist2d(member.pos).total_cmp(&b.pos.dist2d(member.pos)).then(a.id.0.cmp(&b.id.0)));
                for f in near {
                    let had = cover.entry(member.id).or_insert(0.0);
                    if *had >= member.health {
                        break;
                    }
                    *had += weight(f);
                    dealt.insert(f.id, member.id);
                    free.remove(&f.id);
                }
            }
            // Fighters over (every member in front of them is covered, or none is in front): the nearest in front,
            // else the nearest.
            for f in trailing.iter().filter(|f| free.contains(&f.id)) {
                let nearest = |front: bool| body.members.iter().filter(|c| reachable(c) && (!front || in_front(f, c))).min_by(|a, b| a.pos.dist2d(f.pos).total_cmp(&b.pos.dist2d(f.pos)));
                if let Some(member) = nearest(true).or_else(|| nearest(false)) {
                    dealt.insert(f.id, member.id);
                }
            }
        }
        for (k, unit) in fighters.iter().enumerate() {
            let Some(body) = chosen.get(&unit.id).map(|index| &bodies[*index]) else {
                // No threat is its: the way back to the place, then the loop.
                let memory = self.interceptors.get_mut(&unit.id).expect("entered above");
                if home(memory, unit, k, place, map, frame, commands, &mut self.counts) {
                    report.returning += 1;
                } else {
                    report.loitering += 1;
                }
                continue;
            };
            let Some(stats) = view.stats(unit.def) else { continue };
            let rear = body.members[0];
            let (u, s) = (body.u, body.speed);
            let v = (unit.vel.x * FRAMES_PER_SECOND as f32, unit.vel.z * FRAMES_PER_SECOND as f32);
            let speed = norm(v);
            let memory = self.interceptors.get_mut(&unit.id).expect("entered above");
            if matches!(memory.phase, Phase::Loiter | Phase::Return) {
                memory.phase = Phase::Close;
                memory.side = 0.0;
                self.counts.1 += 1;
                commands.push(Command::MoveState { unit: unit.id, state: MOVESTATE_MANOEUVRE });
            }
            // Until it trails it is on the body's rearmost.
            if memory.phase != Phase::Trail {
                memory.target = Some(rear.id);
            }
            // Attacked as it is: a body that hovers, or one the fighter cannot stay behind.
            let slow = s < SLOW || s >= stats.speed * CATCH;
            if debug && frame % 15 == 0 {
                let h = dir(v).unwrap_or(u);
                eprintln!("{} f={frame} micro: #{} state {:?} speed {speed:.0} heading {:.0} off the body's ({} of {} at {s:.0}), {:.0} ahead of the rearmost, {:.0} across, last {:?}", view.label(), unit.id.0, memory.phase, dot(h, u).clamp(-1.0, 1.0).acos().to_degrees(), rear.id.0, body.members.len(), dot(sub(unit.pos, rear.pos), u), cross(u, sub(unit.pos, rear.pos)), memory.sent.map(|(what, at)| (match what { Sent::Loiter(_) => "loiter", Sent::Home(_) => "home", Sent::Move(_) => "move", Sent::Attack(_) => "attack" }, frame - at)));
            }
            if memory.phase == Phase::Close {
                // The side of the track it is on, chosen once (the left when it is on the track).
                if memory.side == 0.0 {
                    memory.side = if cross(u, sub(unit.pos, rear.pos)) >= 0.0 { 1.0 } else { -1.0 };
                }
                let fast = speed.max(stats.speed * 0.8);
                let (at, turn) = turn_point(unit.pos, fast, rear.pos, u, s, memory.side);
                // Never a point beyond the leash: one out there is flown to as far as the leash.
                let at = pulled(at, place, b.leash);
                // Its heading: its velocity, or the way to its point while it has none (on the ground, taking off).
                let h = if speed > 30.0 { dir(v) } else { dir(sub(at, unit.pos)) }.unwrap_or((-u.0, -u.1));
                // How fast the trail it would roll out to grows: the two speeds along the track.
                let closing = (s - dot(v, u)).max(0.0);
                // The turn begins when a turn begun now would roll it out the trail behind; and in any case at its
                // point, or once it is no further ahead of them along the track than a half turn's point while it
                // flies against them or is within a turn's width of their track: there no point is left to fly
                // to, and the turn's own rule keeps it from rolling out ahead. (By the first test alone a fighter
                // that came to its point at an angle swung onto their heading after it and flew on ahead of them
                // for the rest of the fight, a third of the wing in every drill up to air-I10: air-I11-run2-rF-f12,
                // three Falcons in `Close` 60 to 420 ahead for ten seconds.)
                let along = dot(sub(unit.pos, rear.pos), u);
                let across = cross(u, sub(unit.pos, rear.pos));
                let reached = unit.pos.dist2d(at) <= fast * LATENCY + 40.0;
                let against = dot(h, u) < INSIDE_DEG.to_radians().cos();
                let inside = along <= turn_geometry(s, std::f32::consts::PI).0 + closing * LATENCY && (against || across.abs() <= TURN_ACROSS);
                if slow || reached || inside || rollout_behind(unit.pos, h, rear.pos, u, s, ROLL_IN) >= TRAIL - closing * LATENCY {
                    memory.phase = Phase::Turn { since: frame };
                    if debug {
                        eprintln!("{} f={frame} micro: #{} turns in on {} ({name}): {along:.0} ahead of it along its track, {across:.0} across, a turn of {:.0} planned", view.label(), unit.id.0, rear.id.0, turn.to_degrees());
                    }
                } else {
                    let asked = pulled(
                        match dir(sub(at, unit.pos)) {
                            Some(d) => carried(at, d, BEYOND),
                            None => at,
                        },
                        place,
                        b.leash,
                    );
                    let to = onto_map(unit.pos, asked, map);
                    if let Some(why) = move_due(memory.sent, unit, to, frame, Resend::Heading(b.close_deg, (b.close_gap * FRAMES_PER_SECOND as f32) as i32)) {
                        if debug {
                            eprintln!("{} f={frame} micro: #{} sent a move (close) {why}{}", view.label(), unit.id.0, if to == asked { "" } else { ", its point off the map" });
                        }
                        memory.sent = Some((Sent::Move(to), frame));
                        self.counts.1 += 1;
                        commands.push(Command::MoveNow { unit: unit.id, to });
                    }
                    report.closing += 1;
                }
            }
            if let Phase::Turn { since } = memory.phase {
                // On the body's heading with a contact ahead of it and near: the attack. A contact that hovers is
                // attacked as it is.
                let h = dir(v).unwrap_or(u);
                let ahead_of_it = dot(h, u) >= ALONG_DEG.to_radians().cos() && body.members.iter().any(|c| {
                    let to = sub(c.pos, unit.pos);
                    norm(to) <= stats.air_reach * NEAR_REACH && dot(to, h) > 0.0
                });
                let turned = (frame - since) as f32 / FRAMES_PER_SECOND as f32;
                if slow || ahead_of_it || turned >= TURN_SECONDS + 2.0 {
                    memory.phase = Phase::Trail;
                    if debug {
                        eprintln!("{} f={frame} micro: #{} trails {} ({name}) after {turned:.1} s of turn: {:.0} behind it along its track, {:.0} across, {:.0} off", view.label(), unit.id.0, rear.id.0, -dot(sub(unit.pos, rear.pos), u), cross(u, sub(unit.pos, rear.pos)), rear.pos.dist2d(unit.pos));
                    }
                } else {
                    // The turn, to the point ahead of where the rearmost will be when it is done; or straight on
                    // while the turn would bring it out too near them or ahead of them.
                    let behind = rollout_behind(unit.pos, h, rear.pos, u, s, (ROLL_IN - turned).max(0.0));
                    let against = dot(h, u) < INSIDE_DEG.to_radians().cos();
                    // Read every tick, and it does change its mind while the estimate stands at the threshold: that is
                    // the rule holding the fighter's roll-out where it wants it. Decided once instead, the wing
                    // rolled out ahead of the bombers on a long approach (a quarter to a third of its seconds from
                    // the merge to the bombs on the northern raid and the line abreast, none before: air-S1-*).
                    let holds = behind < TRAIL - HOLD_BAND && turned < HOLD_SECONDS;
                    let branch = if holds && against { "holds on" } else if holds { "holds across" } else if against { "comes round" } else { "onto their track" };
                    let to = if holds && against {
                        carried(unit.pos, (-u.0, -u.1), BEYOND)
                    } else if holds {
                        // Across the track, the way it is already going (away from it when it is going neither).
                        let n = (-u.1, u.0);
                        let going = dot(h, n);
                        let way = if going.abs() >= 0.2 { going.signum() } else { let across = cross(u, sub(unit.pos, rear.pos)); if across.abs() >= 60.0 { across.signum() } else { memory.side } };
                        Vec3 { x: unit.pos.x + n.0 * way * INSIDE_ACROSS + u.0 * 100.0, y: unit.pos.y, z: unit.pos.z + n.1 * way * INSIDE_ACROSS + u.1 * 100.0 }
                    } else if against {
                        // Still flying against them: round toward the track, by a point abeam on that side. (To a
                        // point dead astern the engine turns a plane either way, and the turn away from the track
                        // rolled fighters out 600 to 800 beside the bombers: air-I7-run1-rF-f12.)
                        let across = cross(u, sub(unit.pos, rear.pos));
                        let inward = if across.abs() >= 60.0 { -across.signum() } else { -memory.side };
                        let n = (-u.1 * inward, u.0 * inward);
                        Vec3 { x: unit.pos.x + n.0 * INSIDE_ACROSS + u.0 * INSIDE_ALONG, y: unit.pos.y, z: unit.pos.z + n.1 * INSIDE_ACROSS + u.1 * INSIDE_ALONG }
                    } else {
                        carried(rear.pos, u, s * (TURN_SECONDS - turned).max(0.0) + TURN_AHEAD)
                    };
                    let asked = pulled(to, place, b.leash);
                    let to = onto_map(unit.pos, asked, map);
                    if let Some(why) = move_due(memory.sent, unit, to, frame, Resend::Shifted(TURN_RESEND_MOVED, RESEND_FRAMES)) {
                        if debug {
                            eprintln!("{} f={frame} micro: #{} sent a move (turn: {branch}) {why}{}", view.label(), unit.id.0, if to == asked { "" } else { ", its point off the map" });
                        }
                        memory.sent = Some((Sent::Move(to), frame));
                        self.counts.1 += 1;
                        commands.push(Command::MoveNow { unit: unit.id, to });
                    }
                    report.turning += 1;
                }
            }
            if memory.phase == Phase::Trail {
                // Nothing of the body in front of it for `AHEAD_TICKS`: it is ahead of them, and the turn's rule
                // takes it across the track until they have passed.
                if body.members.iter().any(|c| dot(sub(c.pos, unit.pos), v) > 0.0) || slow {
                    memory.ahead = 0;
                } else {
                    memory.ahead += 1;
                }
                if memory.ahead >= AHEAD_TICKS {
                    memory.ahead = 0;
                    memory.phase = Phase::Turn { since: frame };
                    memory.target = Some(rear.id);
                    if debug {
                        eprintln!("{} f={frame} micro: #{} is ahead of {} ({name}) by {:.0}: lets it pass", view.label(), unit.id.0, rear.id.0, dot(sub(unit.pos, rear.pos), u));
                    }
                    report.turning += 1;
                    continue;
                }
                // Dealt from the next tick on (the deal is of the fighters already behind the body); until then the
                // rearmost it may be flown at. Nothing beyond the leash that is not coming is followed (a threat has
                // a member that may be: that is what made it one).
                let may = |id: &UnitId| body.members.iter().any(|c| c.id == *id && reachable(c));
                let Some(target) = dealt.get(&unit.id).copied().filter(may).or_else(|| body.members.iter().find(|c| reachable(c)).map(|c| c.id)) else { continue };
                memory.target = Some(target);
                let due = match memory.sent {
                    Some((Sent::Attack(last), sent)) => last != target || frame - sent >= ATTACK_RESEND || (unit.idle && frame - sent >= ATTACK_IDLE_RESEND),
                    _ => true,
                };
                if due {
                    memory.sent = Some((Sent::Attack(target), frame));
                    self.counts.1 += 1;
                    commands.push(Command::Attack { unit: unit.id, target, queue: false });
                }
                report.trailing += 1;
            }
        }
        // Said once: every fighter back on the loop with no threat left. A wing with no fighter left is not back:
        // its scramble ends unsaid (the group's losses are the host's to say).
        if fighters.is_empty() {
            wing.scrambled = None;
        } else if chosen.is_empty()
            && report.returning == 0
            && let Some(since) = wing.scrambled.take()
        {
            events.push(InterceptEvent { group: name.to_string(), what: InterceptWhat::Returned { seconds: (frame - since) / FRAMES_PER_SECOND } });
            if debug {
                eprintln!("{} f={frame} micro: {name} back on its loop after {} s", view.label(), (frame - since) / FRAMES_PER_SECOND);
            }
        }
        wing.report = report;
        self.wings.insert(name.to_string(), wing);
    }
}

/// Whether a contact is coming at the place: it moves, the place is ahead of it, its course passes within the guard
/// of the place, and it is within the bound's seconds of its nearest point to the place at its speed.
fn coming(c: &Contact, place: Vec3, b: Bounds) -> bool {
    let speed = norm(c.vel);
    let Some(h) = dir(c.vel).filter(|_| speed >= FLYING_MOVING) else { return false };
    let to = sub(place, c.pos);
    let along = dot(to, h);
    along > 0.0 && cross(h, to).abs() <= b.guard && along / speed <= b.seconds
}

/// A point no further from the place than the leash: itself, or the point of the leash on the way to it.
/// When a move is sent again.
#[derive(Clone, Copy, Debug)]
enum Resend {
    /// The point has shifted this far, no sooner than this many frames after the last.
    Shifted(f32, i32),
    /// The bearing from the fighter to the point is this many degrees off the bearing to the point last sent, no
    /// sooner than this many frames after; or the fighter is about to arrive at the point it has.
    Heading(f32, i32),
}

/// Whether a fighter flown by moves is to be sent `to`, and why (the lane's debug line): its first move, it is
/// idle, or the rule says again.
fn move_due(sent: Option<(Sent, i32)>, unit: &OwnUnit, to: Vec3, frame: i32, rule: Resend) -> Option<&'static str> {
    let Some((Sent::Move(last), at)) = sent else { return Some("its first") };
    if unit.idle {
        return Some("idle");
    }
    match rule {
        Resend::Shifted(shift, frames) => (frame - at >= frames && last.dist2d(to) > shift).then_some("the point shifted"),
        Resend::Heading(degrees, frames) => {
            if last.dist2d(unit.pos) < ARRIVING {
                return Some("arriving");
            }
            let (Some(had), Some(asked)) = (dir(sub(last, unit.pos)), dir(sub(to, unit.pos))) else { return Some("no bearing") };
            (frame - at >= frames && dot(had, asked) < degrees.to_radians().cos()).then_some("another heading")
        }
    }
}

fn pulled(to: Vec3, place: Vec3, leash: f32) -> Vec3 {
    let out = to.dist2d(place);
    if out <= leash {
        return to;
    }
    Vec3 { x: place.x + (to.x - place.x) * leash / out, y: to.y, z: place.z + (to.z - place.z) * leash / out }
}

/// The map's width and height in elmos, from a host that has its terrain.
fn extent(view: &dyn View) -> Option<(f32, f32)> {
    view.terrain().filter(|t| t.cell > 0.0 && t.width > 0 && t.height > 0).map(|t| (t.width as f32 * t.cell, t.height as f32 * t.cell))
}

/// Where a fighter at `from` is sent for the point `to`. The engine drops a move to a place off the map and the
/// fighter flies on as it was (K-engine-an-order-to-a-place-off-the-map-is-dropped), and the wing's points lie
/// there whenever what it flies at is within `BEYOND` of the map's edge on the far side of the fighter, or outside
/// the map as aircraft can be (air-patrol-4-apex50, 21:32: twelve fighters circled for three seconds of a
/// ten-second warning). Off the map the point is where the fighter's line to it leaves the map: the heading asked,
/// a shorter way to fly. For a fighter already there (within `ARRIVING` of that point) or itself outside, it is
/// the map's nearest point to `to`, which takes it along the edge.
fn onto_map(from: Vec3, to: Vec3, map: Option<(f32, f32)>) -> Vec3 {
    let Some((width, height)) = map else { return to };
    let near = to.on_the_map(width, height);
    if near == to || from.on_the_map(width, height) != from {
        return near;
    }
    // The furthest along the line that is still on the map, each axis for itself.
    let along = |from: f32, to: f32, max: f32| if to > max { (max - from) / (to - from) } else if to < 0.0 { from / (from - to) } else { 1.0 };
    let t = along(from.x, to.x, width - 1.0).min(along(from.z, to.z, height - 1.0)).clamp(0.0, 1.0);
    let edge = Vec3 { x: from.x + (to.x - from.x) * t, y: to.y, z: from.z + (to.z - from.z) * t }.on_the_map(width, height);
    if edge.dist2d(from) >= ARRIVING { edge } else { near }
}

/// A fighter with no threat: a move order to the place under hold position while it is further than `HOME` from it
/// (a fighter on a move order searches for nothing), the loop round the place inside, each fighter beginning at the
/// next corner. Whether it is on its way back.
#[allow(clippy::too_many_arguments)]
fn home(memory: &mut Interceptor, unit: &OwnUnit, k: usize, place: Vec3, map: Option<(f32, f32)>, frame: i32, commands: &mut Vec<Command>, counts: &mut (u32, u32)) -> bool {
    memory.target = None;
    memory.ahead = 0;
    let out = unit.pos.dist2d(place);
    memory.phase = match memory.phase {
        Phase::Loiter if out <= HOME_OUT => Phase::Loiter,
        _ if out <= HOME => Phase::Loiter,
        _ => Phase::Return,
    };
    if memory.phase == Phase::Return {
        let due = match memory.sent {
            Some((Sent::Home(to), at)) => to != place || (unit.idle && frame - at >= LOITER_RESEND),
            _ => true,
        };
        if due {
            memory.sent = Some((Sent::Home(place), frame));
            counts.1 += 2;
            commands.push(Command::MoveState { unit: unit.id, state: MOVESTATE_HOLD });
            commands.push(Command::Move { unit: unit.id, to: place, queue: false });
        }
        return true;
    }
    let due = match memory.sent {
        // Its loop, unless the host has moved the wing's place since.
        Some((Sent::Loiter(over), at)) => over != place || (unit.idle && frame - at >= LOITER_RESEND),
        _ => true,
    };
    if due {
        memory.sent = Some((Sent::Loiter(place), frame));
        let half = LOITER_LOOP / 2.0;
        let corners = [(-half, -half), (half, -half), (half, half), (-half, half)];
        counts.1 += 5;
        commands.push(Command::MoveState { unit: unit.id, state: MOVESTATE_HOLD });
        for step in 0..4 {
            let (dx, dz) = corners[(k + step) % 4];
            // A corner off the map (a place within half the loop of the edge) is the map's nearest point to it: a
            // patrol order to a place off the map is dropped as a move is.
            let corner = Vec3 { x: place.x + dx, y: place.y, z: place.z + dz };
            commands.push(Command::Patrol { unit: unit.id, to: map.map_or(corner, |(width, height)| corner.on_the_map(width, height)), queue: step > 0 });
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    /// A fighter flying straight at bombers that come at it turns in before they pass: when it is the turn's lead
    /// ahead of them, and not sooner.
    #[test]
    fn a_fighter_meeting_a_body_head_on_turns_when_the_turn_would_roll_it_out_behind() {
        let (u, s) = ((1.0, 0.0), 200.0);
        let r = at(0.0, 0.0);
        let lead = s * TURN_SECONDS - TRAIL;
        // Flying against the track, the half turn: behind by the trail exactly at the lead.
        assert!((rollout_behind(at(lead, TURN_ACROSS), (-1.0, 0.0), r, u, s, ROLL_IN) - TRAIL).abs() < 1.0, "{}", rollout_behind(at(lead, TURN_ACROSS), (-1.0, 0.0), r, u, s, ROLL_IN));
        assert!(rollout_behind(at(lead + 300.0, TURN_ACROSS), (-1.0, 0.0), r, u, s, ROLL_IN) < TRAIL - 250.0);
        // Already behind and flying the track's way: no turn to wait for.
        assert!(rollout_behind(at(-500.0, 0.0), (1.0, 0.0), r, u, s, ROLL_IN) >= TRAIL);
        // Coming across the track it needs a quarter turn and begins later: abeam is still too soon.
        assert!(rollout_behind(at(0.0, TURN_ACROSS), (0.0, -1.0), r, u, s, ROLL_IN) < TRAIL);
    }

    /// The turn's point lies beside the track on the fighter's side, and is where the two meet: the fighter's time
    /// to it is the body's time to the lead behind it.
    #[test]
    fn the_turn_point_is_reached_by_both_at_once() {
        let (u, s, v) = ((1.0, 0.0), 200.0, 290.0);
        let (f, r) = (at(3500.0, 0.0), at(0.0, 0.0));
        let lead = s * TURN_SECONDS - TRAIL;
        let p = meeting_point(f, v, r, u, s, lead, TURN_ACROSS);
        assert!((p.z - TURN_ACROSS).abs() < 1.0);
        let t = p.dist2d(f) / v;
        assert!((p.x - (lead + s * t)).abs() < 2.0, "the point {p:?} at t={t}");
        assert!(p.x > 1500.0 && p.x < 2200.0);
    }

    /// A fighter flying at the bombers turns well ahead of them; one that comes across their track turns about
    /// level with the rearmost, a quarter turn from being behind it.
    #[test]
    fn the_turn_point_is_for_the_heading_the_fighter_comes_with() {
        let (u, s, v) = ((1.0, 0.0), 230.0, 289.0);
        let r = at(0.0, 0.0);
        let (head_on, turn) = turn_point(at(3500.0, 100.0), v, r, u, s, 1.0);
        assert!(turn.to_degrees() > 170.0, "{}", turn.to_degrees());
        let met = head_on.dist2d(at(3500.0, 100.0)) / v;
        assert!((head_on.x - s * met - (s * TURN_SECONDS - TRAIL)).abs() < 80.0, "{head_on:?}");
        // From 2,500 to the side, level with where they will be: it comes across.
        let from = at(1800.0, 2500.0);
        let (across, turn) = turn_point(from, v, r, u, s, 1.0);
        assert!((60.0..125.0).contains(&turn.to_degrees()), "{}", turn.to_degrees());
        let met = across.dist2d(from) / v;
        let lead = across.x - s * met;
        assert!(lead < 250.0, "a crossing fighter's point {lead} ahead of the rearmost");
        // And the turn's own test holds there: a turn begun at the point on that course rolls out the trail behind.
        let course = dir(sub(across, from)).unwrap();
        let rear_then = at(s * met, 0.0);
        assert!((rollout_behind(across, course, rear_then, u, s, ROLL_IN) - TRAIL).abs() < 40.0, "{}", rollout_behind(across, course, rear_then, u, s, ROLL_IN));
    }

    #[test]
    fn contacts_flying_together_are_one_body_from_the_rear_forward() {
        let c = |id: i32, x: f32, z: f32| Contact { id: UnitId(id), pos: at(x, z), vel: (0.0, -200.0), health: 1000.0 };
        let all = bodies(&[c(1, 0.0, 0.0), c(2, 100.0, 300.0), c(3, 50.0, -200.0), c(4, 5000.0, 0.0)]);
        assert_eq!(all.len(), 2);
        let ids: Vec<i32> = all[0].members.iter().map(|m| m.id.0).collect();
        // Heading -z: the rearmost is the one with the greatest z.
        assert_eq!(ids, vec![2, 1, 3]);
        assert!((all[0].speed - 200.0).abs() < 1.0);
    }

    use crate::test_host::{FALCON, HAILSTORM, Host, PAWN, tick};
    use crate::{GroupKind, GroupOrder};
    use bot_protocol::OwnUnit;

    const PLACE: Vec3 = Vec3 { x: 2000.0, y: 0.0, z: 2000.0 };

    /// A Falcon at a place, flying this way (elmos a second).
    fn falcon(id: i32, x: f32, z: f32, vel: (f32, f32)) -> OwnUnit {
        OwnUnit { id: UnitId(id), def: FALCON, pos: at(x, z), vel: at(vel.0 / 30.0, vel.1 / 30.0), health: 167.0, max_health: 167.0, being_built: false, idle: false, reload_frame: 0, facing: 0, stockpile: 0, stockpile_queued: 0, priority_low: false, build_progress: 1.0, stunned: 0.0 }
    }

    /// A Hailstorm flying +x at 230 a second, its type seen or not (a radar contact, 200 up).
    fn bomber(id: i32, x: f32, z: f32, seen: bool) -> EnemyUnit {
        EnemyUnit { id: UnitId(id), def: seen.then_some(HAILSTORM), pos: Vec3 { x, y: 200.0, z }, vel: at(230.0 / 30.0, 0.0), health: if seen { 1520.0 } else { 0.0 }, team: Some(1), being_built: false, stunned: 0.0 }
    }

    fn wing(ids: &[i32]) -> Lane {
        let mut lane = Lane::default();
        lane.set_group("wing", ids.iter().map(|id| UnitId(*id)).collect(), GroupOrder::new(GroupKind::Intercept { place: PLACE }, Vec::new()));
        lane
    }

    #[test]
    fn a_wing_with_nothing_flying_patrols_a_loop_round_its_place_and_the_hosts_orders_to_it_are_dropped() {
        let host = Host::new(Vec::new());
        let mut lane = wing(&[1, 2]);
        // A tank beside the place is nothing to it.
        let pawn = crate::test_host::enemy(50, PAWN, 2300.0, 2000.0);
        let mut said = vec![Command::Move { unit: UnitId(1), to: at(100.0, 100.0), queue: false }];
        let out = lane.tick(&host, &tick(30, vec![falcon(1, 2000.0, 2000.0, (0.0, 0.0)), falcon(2, 2050.0, 2000.0, (0.0, 0.0))], vec![pawn.clone()]), &mut said, false);
        assert!(said.is_empty(), "the host's move to a fighter of the wing went out: {said:?}");
        assert!(out.intercepts.is_empty());
        for id in [1, 2] {
            let legs: Vec<Vec3> = out.commands.iter().filter_map(|c| match c { Command::Patrol { unit, to, .. } if unit.0 == id => Some(*to), _ => None }).collect();
            assert_eq!(legs.len(), 4, "{:?}", out.commands);
            assert!(legs.iter().all(|p| p.dist2d(PLACE) < 600.0));
            // The loop is flown on hold position: the engine's own search for targets is off.
            assert!(out.commands.iter().any(|c| matches!(c, Command::MoveState { unit, state: 0 } if unit.0 == id)), "{:?}", out.commands);
        }
        // Not told again while it flies the loop.
        let again = lane.tick(&host, &tick(33, vec![falcon(1, 2000.0, 2000.0, (0.0, 0.0)), falcon(2, 2050.0, 2000.0, (0.0, 0.0))], vec![pawn]), &mut Vec::new(), false);
        assert!(again.commands.is_empty(), "{:?}", again.commands);
        assert_eq!(lane.intercept_report("wing").map(|r| (r.contacts, r.loitering)), Some((0, 2)));
        // The host moves the wing's place 5,000 off: each fighter flies a move order to it (a patrol never goes
        // idle, and a patrol order from there would be flown as a search on the way), and its loop once it is there.
        let moved = at(6000.0, 5000.0);
        lane.set_group("wing", vec![UnitId(1), UnitId(2)], GroupOrder::new(GroupKind::Intercept { place: moved }, Vec::new()));
        let out = lane.tick(&host, &tick(36, vec![falcon(1, 2000.0, 2000.0, (0.0, 0.0)), falcon(2, 2050.0, 2000.0, (0.0, 0.0))], Vec::new()), &mut Vec::new(), false);
        let moves: Vec<Vec3> = out.commands.iter().filter_map(|c| match c { Command::Move { to, .. } => Some(*to), _ => None }).collect();
        assert!(moves.len() == 2 && moves.iter().all(|p| *p == moved) && !out.commands.iter().any(|c| matches!(c, Command::Patrol { .. })), "{:?}", out.commands);
        assert_eq!(lane.intercept_report("wing").map(|r| (r.returning, r.loitering)), Some((2, 0)));
        let out = lane.tick(&host, &tick(39, vec![falcon(1, 5500.0, 4600.0, (200.0, 160.0)), falcon(2, 2600.0, 2400.0, (200.0, 160.0))], Vec::new()), &mut Vec::new(), false);
        let legs: Vec<(i32, Vec3)> = out.commands.iter().filter_map(|c| match c { Command::Patrol { unit, to, .. } => Some((unit.0, *to)), _ => None }).collect();
        assert!(legs.len() == 4 && legs.iter().all(|(id, p)| *id == 1 && p.dist2d(moved) < 600.0), "the one that is there: {:?}", out.commands);
    }

    /// What is not coming at the place moves nothing: a bomber whose course passes 2,500 abeam, and one whose nose
    /// swings through the place as it mills about (thirty degrees a second). One flying straight at the place is
    /// taken on once its course has held a second, and only the fighters that kill it in two seconds go: six
    /// Falcons a Hailstorm, the other six stay on the loop.
    #[test]
    fn a_wing_leaves_its_loop_for_what_comes_at_its_place_and_with_the_fighters_that_needs() {
        let host = Host::new(Vec::new());
        let ids: Vec<i32> = (1..=12).collect();
        let over = |ids: &[i32]| -> Vec<OwnUnit> { ids.iter().map(|id| falcon(*id, 2000.0 + 20.0 * *id as f32, 2000.0, (0.0, 0.0))).collect() };
        // 3,000 west of the place flying east, its course 2,500 south of it.
        let mut lane = wing(&ids);
        for frame in (30..=120).step_by(3) {
            let out = lane.tick(&host, &tick(frame, over(&ids), vec![bomber(60, -1000.0, 4500.0, true)]), &mut Vec::new(), false);
            assert!(out.intercepts.is_empty() && !out.commands.iter().any(|c| matches!(c, Command::Attack { .. })), "f={frame} {:?}", out.intercepts);
        }
        assert_eq!(lane.intercept_report("wing").map(|r| (r.contacts, r.watched, r.loitering)), Some((0, 1, 12)));
        // 3,000 west of the place, milling: its nose comes round three degrees a tick.
        let mut lane = wing(&ids);
        for (k, frame) in (30..=390).step_by(3).enumerate() {
            let angle = (k as f32 * 3.0).to_radians();
            let milling = EnemyUnit { vel: at(230.0 / 30.0 * angle.cos(), 230.0 / 30.0 * angle.sin()), ..bomber(60, -1000.0, 2000.0, true) };
            let out = lane.tick(&host, &tick(frame, over(&ids), vec![milling]), &mut Vec::new(), false);
            assert!(out.intercepts.is_empty(), "f={frame} {:?}", out.intercepts);
        }
        // 3,000 west of the place flying straight at it.
        let mut lane = wing(&ids);
        for frame in (30..60).step_by(3) {
            let out = lane.tick(&host, &tick(frame, over(&ids), vec![bomber(60, -1000.0, 2000.0, true)]), &mut Vec::new(), false);
            assert!(out.intercepts.is_empty(), "f={frame}: its course has not held a second yet");
        }
        let out = lane.tick(&host, &tick(60, over(&ids), vec![bomber(60, -770.0, 2000.0, true)]), &mut Vec::new(), false);
        assert!(matches!(out.intercepts.as_slice(), [InterceptEvent { what: InterceptWhat::Scrambled { contacts: 1, fighters: 6, .. }, .. }]), "{:?}", out.intercepts);
        assert_eq!(lane.intercept_report("wing").map(|r| (r.contacts, r.closing, r.loitering)), Some((1, 6, 6)));
        // The six that go fly under manoeuvre, as the conversion was drilled.
        assert_eq!(out.commands.iter().filter(|c| matches!(c, Command::MoveState { state: 1, .. })).count(), 6, "{:?}", out.commands);
    }

    /// The leash: a fighter behind a bomber that flies away is given no target further than the leash from the place
    /// and flies a move order back to it on hold position; the wing says it is back once every fighter is on the
    /// loop again, not when the threat is gone.
    #[test]
    fn a_fighter_follows_nothing_beyond_the_leash_and_the_return_is_a_move_to_the_place() {
        let host = Host::new(Vec::new());
        let mut lane = wing(&[1]);
        // A bomber 1,000 east of the place flying on east, the fighter 300 behind it: taken on (inside the guard),
        // attacked from behind.
        let mut events = Vec::new();
        let mut commands = Vec::new();
        for (k, frame) in (30..=390).step_by(3).enumerate() {
            let x = 3000.0 + 23.0 * k as f32;
            let out = lane.tick(&host, &tick(frame, vec![falcon(1, x - 300.0, 2000.0, (230.0, 0.0))], vec![bomber(60, x, 2000.0, true)]), &mut Vec::new(), false);
            events.extend(out.intercepts);
            commands.extend(out.commands.into_iter().map(|c| (x, c)));
        }
        // Attacked while the bomber was inside the leash (3,000 from the place: x 5,000), never after.
        let attacks: Vec<f32> = commands.iter().filter(|(_, c)| matches!(c, Command::Attack { .. })).map(|(x, _)| *x).collect();
        assert!(!attacks.is_empty() && attacks.iter().all(|x| *x <= 2000.0 + LEASH), "{attacks:?}");
        // Then the way back: hold position and a move to the place, and no patrol from out there.
        let back: Vec<&(f32, Command)> = commands.iter().filter(|(x, _)| *x > 2000.0 + LEASH).collect();
        assert!(back.iter().any(|(_, c)| matches!(c, Command::Move { to, .. } if *to == PLACE)) && back.iter().any(|(_, c)| matches!(c, Command::MoveState { state: 0, .. })), "{back:?}");
        assert!(!back.iter().any(|(_, c)| matches!(c, Command::Patrol { .. } | Command::Attack { .. })), "{back:?}");
        assert!(matches!(events.as_slice(), [InterceptEvent { what: InterceptWhat::Scrambled { .. }, .. }]), "not back while the fighter is 2,700 out: {events:?}");
        // The bomber gone and the fighter home: the loop, and the word that it is back.
        let out = lane.tick(&host, &tick(420, vec![falcon(1, 2300.0, 2000.0, (-280.0, 0.0))], Vec::new()), &mut Vec::new(), false);
        assert!(matches!(out.intercepts.as_slice(), [InterceptEvent { what: InterceptWhat::Returned { .. }, .. }]), "{:?}", out.intercepts);
        assert_eq!(out.commands.iter().filter(|c| matches!(c, Command::Patrol { .. })).count(), 4);
    }

    /// A fighter trailing one body whose last member dies, with a second body a threat the same tick, is not sent
    /// an attack at the second from where it is (it would meet it nose to nose): it closes on it as from the loop.
    #[test]
    fn a_fighter_given_another_body_begins_its_conversion_anew() {
        let host = Host::new(Vec::new());
        let mut lane = wing(&[1]);
        // Behind a bomber inside the guard, flying its way: the attack from the first tick.
        for frame in [30, 33] {
            let out = lane.tick(&host, &tick(frame, vec![falcon(1, 700.0, 2000.0, (280.0, 0.0))], vec![bomber(60, 1000.0, 2000.0, true)]), &mut Vec::new(), false);
            assert!(frame != 30 || out.commands.iter().any(|c| matches!(c, Command::Attack { target: UnitId(60), .. })), "{:?}", out.commands);
        }
        assert_eq!(lane.intercept_report("wing").map(|r| r.trailing), Some(1));
        // That bomber is gone, and another is inside the guard east of the place, flying west at the fighter.
        let second = EnemyUnit { vel: at(-230.0 / 30.0, 0.0), ..bomber(70, 3200.0, 2100.0, true) };
        let out = lane.tick(&host, &tick(36, vec![falcon(1, 760.0, 2000.0, (280.0, 0.0))], vec![second]), &mut Vec::new(), false);
        assert!(!out.commands.iter().any(|c| matches!(c, Command::Attack { .. })), "{:?}", out.commands);
        assert!(out.commands.iter().any(|c| matches!(c, Command::MoveNow { .. })), "{:?}", out.commands);
        assert_eq!(lane.intercept_report("wing").map(|r| (r.contacts, r.closing, r.trailing)), Some((1, 1, 0)));
        // One scramble said, and no return between the two bodies.
        assert!(out.intercepts.is_empty(), "{:?}", out.intercepts);
    }

    /// A wing whose every fighter is dead is not "back": nothing is said, with the threat there or gone.
    #[test]
    fn a_wing_with_no_fighter_left_does_not_say_it_is_back() {
        let host = Host::new(Vec::new());
        let mut lane = wing(&[1]);
        let out = lane.tick(&host, &tick(30, vec![falcon(1, 700.0, 2000.0, (280.0, 0.0))], vec![bomber(60, 1000.0, 2000.0, true)]), &mut Vec::new(), false);
        assert!(matches!(out.intercepts.as_slice(), [InterceptEvent { what: InterceptWhat::Scrambled { .. }, .. }]), "{:?}", out.intercepts);
        let out = lane.tick(&host, &tick(33, Vec::new(), vec![bomber(60, 1023.0, 2000.0, true)]), &mut Vec::new(), false);
        assert!(out.intercepts.is_empty(), "{:?}", out.intercepts);
        let out = lane.tick(&host, &tick(36, Vec::new(), Vec::new()), &mut Vec::new(), false);
        assert!(out.intercepts.is_empty(), "{:?}", out.intercepts);
    }

    /// At a radar contact that flies the wing leaves its loop for the point beside the contact's track and ahead of
    /// it; it is not ordered at the contact.
    /// A move is a heading asked: the same heading to a point further on is no new order, another heading is, and
    /// a fighter about to arrive at its point, or idle, is always sent on.
    #[test]
    fn a_move_is_sent_again_when_the_heading_it_asks_changes() {
        let fighter = falcon(1, 2000.0, 2000.0, (-250.0, 0.0));
        let sent = Some((Sent::Move(at(0.0, 2000.0)), 100));
        let rule = Resend::Heading(CLOSE_RESEND_DEG, CLOSE_RESEND_FRAMES);
        let due = |sent, unit: &OwnUnit, to, frame| move_due(sent, unit, to, frame, rule).is_some();
        assert!(due(None, &fighter, at(0.0, 2000.0), 100));
        assert!(!due(sent, &fighter, at(-150.0, 2000.0), 130), "the point 150 further on the same bearing");
        assert!(!due(sent, &fighter, at(0.0, 2300.0), 130), "300 aside at 2,000: eight and a half degrees");
        assert!(due(sent, &fighter, at(0.0, 2400.0), 130), "400 aside at 2,000: eleven degrees");
        assert!(!due(sent, &fighter, at(0.0, 2400.0), 100 + CLOSE_RESEND_FRAMES - 1), "not within a second of the last");
        let near = falcon(1, 250.0, 2000.0, (-250.0, 0.0));
        assert!(due(sent, &near, at(-800.0, 2000.0), 130), "about to arrive");
        let idle = OwnUnit { idle: true, ..falcon(1, 2000.0, 2000.0, (0.0, 0.0)) };
        assert!(due(sent, &idle, at(0.0, 2000.0), 130));
        // The turn's rule is the point's shift.
        assert!(move_due(sent, &fighter, at(0.0, 2300.0), 130, Resend::Shifted(TURN_RESEND_MOVED, RESEND_FRAMES)).is_some());
        assert!(move_due(sent, &fighter, at(0.0, 2200.0), 130, Resend::Shifted(TURN_RESEND_MOVED, RESEND_FRAMES)).is_none());
    }

    #[test]
    fn a_wing_scrambles_at_a_radar_contact_that_flies_to_a_point_beside_its_track() {
        let host = Host::new(Vec::new());
        let mut lane = wing(&[1]);
        // The bomber 3,000 west of the place, flying east at it; the fighter over the place, a little north. Its
        // course has to hold a second first.
        for frame in (30..60).step_by(3) {
            let out = lane.tick(&host, &tick(frame, vec![falcon(1, 2000.0, 1950.0, (-250.0, 0.0))], vec![bomber(60, -1000.0, 2000.0, false)]), &mut Vec::new(), false);
            assert!(out.intercepts.is_empty(), "f={frame}");
        }
        let out = lane.tick(&host, &tick(60, vec![falcon(1, 2000.0, 1950.0, (-250.0, 0.0))], vec![bomber(60, -1000.0, 2000.0, false)]), &mut Vec::new(), false);
        assert!(matches!(out.intercepts.as_slice(), [InterceptEvent { what: InterceptWhat::Scrambled { contacts: 1, fighters: 1, .. }, .. }]), "{:?}", out.intercepts);
        assert!(!out.commands.iter().any(|c| matches!(c, Command::Attack { .. })), "{:?}", out.commands);
        let to = out.commands.iter().find_map(|c| match c { Command::MoveNow { to, .. } => Some(*to), _ => None }).expect("a first-class move");
        // West of it, toward the bomber, and on its own (north) side of the track.
        assert!(to.x < 1500.0 && to.z < 2000.0, "{to:?}");
        assert_eq!(lane.intercept_report("wing").map(|r| (r.contacts, r.closing)), Some((1, 1)));
        // Two seconds in which it has not yet come round (it flies on east, as a fighter on the far side of its
        // loop does) and the bomber flies on: the meeting point falls back along the track with every tick the
        // fighter loses, and the heading asked of it stays west. One move more at most; by the point's shift
        // (120, as the rule first was) it was sent the point again and again.
        let mut more = 0;
        for k in 1..=20 {
            let (f, b) = (falcon(1, 2000.0 + 25.0 * k as f32, 1950.0, (250.0, 0.0)), bomber(60, -1000.0 + 23.0 * k as f32, 2000.0, false));
            let out = lane.tick(&host, &tick(60 + 3 * k, vec![f], vec![b]), &mut Vec::new(), false);
            more += out.commands.iter().filter(|c| matches!(c, Command::MoveNow { .. } | Command::Move { .. })).count();
        }
        assert!(more <= 1, "{more} moves in two seconds of coming round");
    }

    /// Behind the bombers and flying their way a fighter attacks, the rearmost first and each bomber covered by the
    /// six Falcons that kill it in two seconds; ahead of them it does not, and is sent across their track.
    #[test]
    fn a_fighter_behind_the_bombers_attacks_from_the_rear_and_one_ahead_of_them_lets_them_pass() {
        let host = Host::new(Vec::new());
        let ids: Vec<i32> = (1..=12).collect();
        let mut lane = wing(&ids);
        let bombers = vec![bomber(60, 1000.0, 2000.0, true), bomber(61, 1150.0, 2000.0, true)];
        let behind: Vec<OwnUnit> = ids.iter().map(|id| falcon(*id, 700.0 - 10.0 * *id as f32, 2000.0 + 5.0 * *id as f32, (280.0, 0.0))).collect();
        let mut targets: HashMap<UnitId, UnitId> = HashMap::new();
        for frame in [30, 33, 36] {
            let out = lane.tick(&host, &tick(frame, behind.clone(), bombers.clone()), &mut Vec::new(), false);
            for c in &out.commands {
                if let Command::Attack { unit, target, .. } = c {
                    targets.insert(*unit, *target);
                }
            }
        }
        let on = |b: i32| targets.values().filter(|t| t.0 == b).count();
        assert_eq!((on(60), on(61)), (6, 6), "{targets:?}");
        // One fighter 200 ahead of the rearmost, flying their way, with nothing of theirs in front of it.
        let mut lane = wing(&[1]);
        let lone = vec![bomber(60, 1000.0, 2000.0, true)];
        let mut attacks = 0;
        let mut across = false;
        for frame in (30..60).step_by(3) {
            let out = lane.tick(&host, &tick(frame, vec![falcon(1, 1200.0, 1950.0, (280.0, 0.0))], lone.clone()), &mut Vec::new(), false);
            attacks += out.commands.iter().filter(|c| matches!(c, Command::Attack { .. })).count();
            across |= out.commands.iter().any(|c| matches!(c, Command::MoveNow { to, .. } if (to.z - 1950.0).abs() > 500.0));
        }
        assert_eq!(attacks, 0);
        assert!(across, "never sent across the track");
    }

    /// A place off the map is an order the engine drops (K-engine-an-order-to-a-place-off-the-map-is-dropped): the
    /// point a fighter is sent for is where its line to it leaves the map, and along the edge for a fighter that is
    /// there already or outside.
    #[test]
    fn a_point_off_the_map_is_flown_at_as_far_as_the_map_goes() {
        let map = Some((4096.0, 4096.0));
        assert_eq!(onto_map(at(1000.0, 1000.0), at(-500.0, 1000.0), None), at(-500.0, 1000.0), "a host with no terrain: as asked");
        assert_eq!(onto_map(at(1000.0, 1000.0), at(3000.0, 4000.0), map), at(3000.0, 4000.0), "on the map: as asked");
        assert_eq!(onto_map(at(1000.0, 1000.0), at(-500.0, 1000.0), map), at(0.0, 1000.0));
        assert_eq!(onto_map(at(3000.0, 3000.0), at(5000.0, 5000.0), map), at(4095.0, 4095.0), "the engine's last place is a size less one");
        // The bearing asked stands: a point 2,000 south and 1,000 east leaves the map 1,095 south and 547.5 east.
        assert_eq!(onto_map(at(1000.0, 3000.0), at(2000.0, 5000.0), map), at(1547.5, 4095.0));
        // At the edge, a short way from it, or outside: the map's nearest point to what was asked.
        assert_eq!(onto_map(at(1000.0, 4095.0), at(2000.0, 5000.0), map), at(2000.0, 4095.0));
        assert_eq!(onto_map(at(1000.0, 3900.0), at(1100.0, 4500.0), map), at(1100.0, 4095.0));
        assert_eq!(onto_map(at(1000.0, 4300.0), at(2000.0, 5000.0), map), at(2000.0, 4095.0));
    }

    /// Bombers that come in from outside the map's edge: the point beside their track, and the 800 a move is
    /// carried beyond it, lie off the map, and the fighter is sent to the edge on that bearing (as the wing was
    /// built its first moves were to places off the map, and the fighters of air-patrol-4-apex50 circled on for
    /// three seconds at 21:32). A loop at the map's edge keeps its corners on it.
    #[test]
    fn a_wing_sends_no_fighter_to_a_place_off_the_map() {
        let mut host = Host::new(Vec::new());
        host.terrain = Some(crate::test_host::flat_terrain());
        let on_the_map = |commands: &[Command]| {
            for c in commands {
                if let Command::Move { to, .. } | Command::MoveNow { to, .. } | Command::Patrol { to, .. } = c {
                    assert!((0.0..=4095.0).contains(&to.x) && (0.0..=4095.0).contains(&to.z), "{c:?}");
                }
            }
        };
        let place = at(1500.0, 2000.0);
        let mut lane = Lane::default();
        lane.set_group("wing", vec![UnitId(1)], GroupOrder::new(GroupKind::Intercept { place }, Vec::new()));
        // The bomber 1,200 west of the map's western edge, flying east at the place; the fighter over the place, a
        // little north of the track.
        let mut moves = Vec::new();
        for k in 0..=20 {
            let out = lane.tick(&host, &tick(30 + 3 * k, vec![falcon(1, 1500.0, 1950.0, (-250.0, 0.0))], vec![bomber(60, -1200.0 + 23.0 * k as f32, 2000.0, true)]), &mut Vec::new(), false);
            on_the_map(&out.commands);
            moves.extend(out.commands.iter().filter_map(|c| match c { Command::MoveNow { to, .. } => Some(*to), _ => None }));
        }
        // On the western edge, on its own side of the track: the bearing of the point it was flown at.
        let first = moves.first().expect("a first-class move");
        assert!(first.x == 0.0 && (1600.0..1900.0).contains(&first.z), "{first:?}");
        let mut lane = Lane::default();
        lane.set_group("wing", vec![UnitId(1)], GroupOrder::new(GroupKind::Intercept { place: at(200.0, 2000.0) }, Vec::new()));
        let out = lane.tick(&host, &tick(30, vec![falcon(1, 200.0, 2000.0, (0.0, 0.0))], Vec::new()), &mut Vec::new(), false);
        assert_eq!(out.commands.iter().filter(|c| matches!(c, Command::Patrol { .. })).count(), 4, "{:?}", out.commands);
        on_the_map(&out.commands);
    }
}
