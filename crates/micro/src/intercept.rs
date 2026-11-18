//! H-MICRO-INTERCEPT: a wing of fighters kept up by a place and flown at what flies into its keep
//! (`docs/design/2026-11-16-air-intercept.md`; the user, 2026-11-16: bombing runs "have to be intercepted with
//! fighters before they reach the base", and the player hands the wing to the lane as it hands a roving group).
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
//! A body slower than `SLOW` (a gunship that hovers) is attacked as it is. A wing with nothing flying in its keep
//! patrols a small loop round its place (`Command::Patrol`), each fighter beginning at the next corner: in the air,
//! where the drills' landed wing answered three to four seconds later and died in the block's own blast, and some of
//! it flying toward whatever comes.
//!
//! What flies: an aircraft by its type when the type is known, and a radar contact by its height over the ground
//! and its speed, which the snapshot carries before the type is seen
//! (K-snapshot-a-radar-contact-carries-its-height-and-its-velocity).
//!
//! Not in the law: his fighters with the bombers (it flies the same conversion on whatever flies), a wing's losses
//! to anti-air under the bombers' track, and a way back for a fighter rolled out far to the side other than the
//! engine's pursuit.

use std::collections::{HashMap, HashSet};

use bot_protocol::{Command, EnemyUnit, OwnUnit, UnitId, Vec3};

use crate::{FRAMES_PER_SECOND, Lane, View};

pub const INTERCEPT: &str = "H-MICRO-INTERCEPT";
/// The keep of a wing whose host gave none: about what a tier-2 radar by the place shows (3,500), and a second's
/// flight more.
pub const KEEP: f32 = 4500.0;
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
/// A move is sent again when its point has shifted this far, no sooner than this many frames after the last.
const RESEND_MOVED: f32 = 120.0;
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
/// A contact stays the wing's until it is this share of the keep away from the place.
const KEEP_OUT: f32 = 1.15;
/// The engine's manoeuvre move state: a patrolling fighter under it still answers what comes within sight of its leg
/// when the radar gave nothing.
const MOVESTATE_MANOEUVRE: i32 = 1;

/// What the lane reports of an intercept group, once each.
#[derive(Clone, Debug, PartialEq)]
pub struct InterceptEvent {
    pub group: String,
    pub what: InterceptWhat,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InterceptWhat {
    /// The wing left its loiter: this many flying contacts, their centre, and how far that is from its place.
    Scrambled { contacts: usize, at: Vec3, out: f32 },
    /// Nothing that flies is left in its keep; it is back on its loiter. `seconds` since the scramble.
    Returned { seconds: i32 },
}

/// What a wing is doing this tick, for the host's picture.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InterceptReport {
    /// Flying contacts in its keep.
    pub contacts: usize,
    /// Its fighters on the loiter, closing, turning in, and behind a contact attacking.
    pub loitering: usize,
    pub closing: usize,
    pub turning: usize,
    pub trailing: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Phase {
    Loiter,
    Close,
    Turn { since: i32 },
    Trail,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Sent {
    Loiter,
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
}

/// The lane's memory of one wing.
#[derive(Default)]
pub(crate) struct Wing {
    scrambled: Option<i32>,
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

/// Where a fighter at `f` flying `v` a second begins its turn on a body whose rearmost member is at `r`: the point
/// `across` to the left of the track (to its right when negative) and `lead` ahead of where `r` will be when the
/// fighter gets there.
fn turn_point(f: Vec3, v: f32, r: Vec3, u: (f32, f32), s: f32, lead: f32, across: f32) -> Vec3 {
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

    /// H-MICRO-INTERCEPT for one declared group of kind `Intercept` (the module's comment).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn intercept(&mut self, view: &dyn View, name: &str, place: Vec3, keep: f32, fighters: &[&OwnUnit], enemies: &[EnemyUnit], frame: i32, commands: &mut Vec<Command>, fired: &mut Vec<&'static str>, events: &mut Vec<InterceptEvent>, debug: bool) {
        let mut wing = self.wings.remove(name).unwrap_or_default();
        // What flies in the keep; once scrambled, what the wing is on stays its until it is well out of it.
        let out_to = if wing.scrambled.is_some() { keep * KEEP_OUT } else { keep };
        let contacts: Vec<Contact> = (enemies.iter())
            .filter(|e| e.pos.dist2d(place) <= out_to && flies(view, e))
            .map(|e| Contact { id: e.id, pos: e.pos, vel: (e.vel.x * FRAMES_PER_SECOND as f32, e.vel.z * FRAMES_PER_SECOND as f32), health: if e.def.is_some() && e.health > 0.0 { e.health } else { BLIP_HEALTH } })
            .collect();
        let mut report = InterceptReport { contacts: contacts.len(), ..InterceptReport::default() };
        for unit in fighters {
            self.interceptors.entry(unit.id).or_insert(Interceptor { phase: Phase::Loiter, side: 0.0, sent: None, target: None, ahead: 0 });
        }
        if contacts.is_empty() {
            if let Some(since) = wing.scrambled.take() {
                events.push(InterceptEvent { group: name.to_string(), what: InterceptWhat::Returned { seconds: (frame - since) / FRAMES_PER_SECOND } });
                if debug {
                    eprintln!("{} f={frame} micro: {name} back to its loiter after {} s", view.label(), (frame - since) / FRAMES_PER_SECOND);
                }
            }
            for (k, unit) in fighters.iter().enumerate() {
                let memory = self.interceptors.get_mut(&unit.id).expect("entered above");
                memory.phase = Phase::Loiter;
                memory.target = None;
                let due = match memory.sent {
                    Some((Sent::Loiter, at)) => unit.idle && frame - at >= LOITER_RESEND,
                    _ => true,
                };
                if due {
                    memory.sent = Some((Sent::Loiter, frame));
                    // A loop round the place, each fighter beginning at the next corner: at any moment some of the
                    // wing is flying toward whatever comes, from whichever side.
                    let half = LOITER_LOOP / 2.0;
                    let corners = [(-half, -half), (half, -half), (half, half), (-half, half)];
                    self.counts.1 += 5;
                    commands.push(Command::MoveState { unit: unit.id, state: MOVESTATE_MANOEUVRE });
                    for step in 0..4 {
                        let (dx, dz) = corners[(k + step) % 4];
                        commands.push(Command::Patrol { unit: unit.id, to: Vec3 { x: place.x + dx, y: place.y, z: place.z + dz }, queue: step > 0 });
                    }
                }
            }
            report.loitering = fighters.len();
            wing.report = report;
            self.wings.insert(name.to_string(), wing);
            return;
        }
        let bodies = bodies(&contacts);
        if wing.scrambled.is_none() {
            wing.scrambled = Some(frame);
            let n = contacts.len() as f32;
            let at = Vec3 { x: contacts.iter().map(|c| c.pos.x).sum::<f32>() / n, y: 0.0, z: contacts.iter().map(|c| c.pos.z).sum::<f32>() / n };
            fired.push(INTERCEPT);
            self.counts.0 += fighters.len() as u32;
            events.push(InterceptEvent { group: name.to_string(), what: InterceptWhat::Scrambled { contacts: contacts.len(), at, out: at.dist2d(place) } });
            if debug {
                let b = &bodies[0];
                eprintln!("{} f={frame} micro: {name} scrambles {} fighters at {} flying contacts {:.0} out (the first body: {} at {:.0} a second, {:.0} over the ground)", view.label(), fighters.len(), contacts.len(), at.dist2d(place), b.members.len(), b.speed, b.members[0].pos.y - ground_height(view, b.members[0].pos));
            }
        }
        // Each fighter's body: the one its contact is in, else the body with the most health to each fighter on it
        // (ties to the nearer).
        let body_of = |id: UnitId| bodies.iter().position(|b| b.members.iter().any(|c| c.id == id));
        let mut on: Vec<f32> = vec![0.0; bodies.len()];
        let mut chosen: HashMap<UnitId, usize> = HashMap::new();
        for unit in fighters {
            if let Some(b) = self.interceptors.get(&unit.id).and_then(|m| m.target).and_then(body_of) {
                chosen.insert(unit.id, b);
                on[b] += 1.0;
            }
        }
        for unit in fighters {
            if chosen.contains_key(&unit.id) {
                continue;
            }
            let weight = |b: &Body| b.members.iter().map(|c| c.health).sum::<f32>();
            let best = (0..bodies.len()).min_by(|&x, &y| ((on[x] + 1.0) / weight(&bodies[x])).total_cmp(&((on[y] + 1.0) / weight(&bodies[y]))).then(bodies[x].members[0].pos.dist2d(unit.pos).total_cmp(&bodies[y].members[0].pos.dist2d(unit.pos)))).expect("a contact is in a body");
            chosen.insert(unit.id, best);
            on[best] += 1.0;
        }
        // The fighters behind a body are dealt its members from the rear forward, each member the fighters that kill
        // it in `WINDOW`: first the fighters that keep the contact they have, then the nearest of the rest.
        let mut dealt: HashMap<UnitId, UnitId> = HashMap::new();
        for (index, body) in bodies.iter().enumerate() {
            let trailing: Vec<&OwnUnit> = fighters.iter().copied().filter(|f| chosen[&f.id] == index && self.interceptors.get(&f.id).is_some_and(|m| m.phase == Phase::Trail)).collect();
            let weight = |f: &OwnUnit| view.stats(f.def).map_or(0.0, |s| s.air_dps * WINDOW).max(1.0);
            let mut cover: HashMap<UnitId, f32> = HashMap::new();
            let mut free: HashSet<UnitId> = trailing.iter().map(|f| f.id).collect();
            for f in &trailing {
                let Some(target) = self.interceptors.get(&f.id).and_then(|m| m.target) else { continue };
                let Some(member) = body.members.iter().find(|c| c.id == target) else { continue };
                let had = cover.entry(target).or_insert(0.0);
                if *had < member.health && dot(sub(member.pos, f.pos), (f.vel.x, f.vel.z)) > 0.0 {
                    *had += weight(f);
                    dealt.insert(f.id, target);
                    free.remove(&f.id);
                }
            }
            // A fighter is dealt what is in front of it: with its contact behind it the engine flies it straight on.
            let in_front = |f: &OwnUnit, c: &Contact| dot(sub(c.pos, f.pos), (f.vel.x, f.vel.z)) > 0.0;
            for member in &body.members {
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
                let nearest = |front: bool| body.members.iter().filter(|c| !front || in_front(f, c)).min_by(|a, b| a.pos.dist2d(f.pos).total_cmp(&b.pos.dist2d(f.pos)));
                if let Some(member) = nearest(true).or_else(|| nearest(false)) {
                    dealt.insert(f.id, member.id);
                }
            }
        }
        for unit in fighters {
            let body = &bodies[chosen[&unit.id]];
            let Some(stats) = view.stats(unit.def) else { continue };
            let rear = body.members[0];
            let (u, s) = (body.u, body.speed);
            let v = (unit.vel.x * FRAMES_PER_SECOND as f32, unit.vel.z * FRAMES_PER_SECOND as f32);
            let speed = norm(v);
            let memory = self.interceptors.get_mut(&unit.id).expect("entered above");
            if memory.phase == Phase::Loiter {
                memory.phase = Phase::Close;
                memory.side = 0.0;
            }
            // Until it trails it is on the body's rearmost.
            if memory.phase != Phase::Trail {
                memory.target = Some(rear.id);
            }
            let slow = s < SLOW;
            if debug && frame % 15 == 0 {
                let h = dir(v).unwrap_or(u);
                eprintln!("{} f={frame} micro: #{} state {:?} speed {speed:.0} heading {:.0} off the body's ({} of {} at {s:.0}), {:.0} ahead of the rearmost, {:.0} across, last {:?}", view.label(), unit.id.0, memory.phase, dot(h, u).clamp(-1.0, 1.0).acos().to_degrees(), rear.id.0, body.members.len(), dot(sub(unit.pos, rear.pos), u), cross(u, sub(unit.pos, rear.pos)), memory.sent.map(|(what, at)| (match what { Sent::Loiter => "loiter", Sent::Move(_) => "move", Sent::Attack(_) => "attack" }, frame - at)));
            }
            if memory.phase == Phase::Close {
                // The side of the track it is on, chosen once (the left when it is on the track).
                if memory.side == 0.0 {
                    memory.side = if cross(u, sub(unit.pos, rear.pos)) >= 0.0 { 1.0 } else { -1.0 };
                }
                let lead = (s * TURN_SECONDS - TRAIL).max(0.0);
                let at = turn_point(unit.pos, speed.max(stats.speed * 0.8), rear.pos, u, s, lead, CLOSE_ACROSS * memory.side);
                // Its heading: its velocity, or the way to its point while it has none (on the ground, taking off).
                let h = if speed > 30.0 { dir(v) } else { dir(sub(at, unit.pos)) }.unwrap_or((-u.0, -u.1));
                // How fast the trail it would roll out to grows: the two speeds along the track.
                let closing = (s - dot(v, u)).max(0.0);
                // The turn begins when a turn begun now would roll it out the trail behind, and in any case once it
                // is no further ahead of them along the track than the turn's point: a fighter that came to the point
                // at an angle swung onto their heading after it, the first test (which reads its heading) never
                // held, and it flew on ahead of them for the rest of the fight, a third of the wing in every drill
                // up to air-I10 (air-I11-run2-rF-f12: three Falcons in `Close` 60 to 420 ahead for ten seconds).
                // From there the turn's own rule keeps it from rolling out ahead.
                let along = dot(sub(unit.pos, rear.pos), u);
                if slow || along <= lead + closing * LATENCY || rollout_behind(unit.pos, h, rear.pos, u, s, ROLL_IN) >= TRAIL - closing * LATENCY {
                    memory.phase = Phase::Turn { since: frame };
                    if debug {
                        eprintln!("{} f={frame} micro: #{} turns in on {} ({name}): {:.0} ahead of it along its track, {:.0} across", view.label(), unit.id.0, rear.id.0, dot(sub(unit.pos, rear.pos), u), cross(u, sub(unit.pos, rear.pos)));
                    }
                } else {
                    let to = match dir(sub(at, unit.pos)) {
                        Some(d) => carried(at, d, BEYOND),
                        None => at,
                    };
                    let due = match memory.sent {
                        Some((Sent::Move(last), sent)) => unit.idle || (frame - sent >= RESEND_FRAMES && last.dist2d(to) > RESEND_MOVED),
                        _ => true,
                    };
                    if due {
                        memory.sent = Some((Sent::Move(to), frame));
                        self.counts.1 += 1;
                        commands.push(Command::Move { unit: unit.id, to, queue: false });
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
                    let to = if behind < TRAIL - HOLD_BAND && turned < HOLD_SECONDS && against {
                        carried(unit.pos, (-u.0, -u.1), BEYOND)
                    } else if behind < TRAIL - HOLD_BAND && turned < HOLD_SECONDS {
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
                    let due = match memory.sent {
                        Some((Sent::Move(last), sent)) => unit.idle || (frame - sent >= RESEND_FRAMES && last.dist2d(to) > 2.0 * RESEND_MOVED),
                        _ => true,
                    };
                    if due {
                        memory.sent = Some((Sent::Move(to), frame));
                        self.counts.1 += 1;
                        commands.push(Command::Move { unit: unit.id, to, queue: false });
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
                // Dealt from the next tick on (the deal is of the fighters already behind the body); the rearmost
                // until then.
                let target = dealt.get(&unit.id).copied().unwrap_or(rear.id);
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
        wing.report = report;
        self.wings.insert(name.to_string(), wing);
    }
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
        let p = turn_point(f, v, r, u, s, lead, TURN_ACROSS);
        assert!((p.z - TURN_ACROSS).abs() < 1.0);
        let t = p.dist2d(f) / v;
        assert!((p.x - (lead + s * t)).abs() < 2.0, "the point {p:?} at t={t}");
        assert!(p.x > 1500.0 && p.x < 2200.0);
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
        lane.set_group("wing", ids.iter().map(|id| UnitId(*id)).collect(), GroupOrder::new(GroupKind::Intercept { place: PLACE, keep: KEEP }, Vec::new()));
        lane
    }

    #[test]
    fn a_wing_with_nothing_flying_patrols_a_loop_round_its_place_and_the_hosts_orders_to_it_are_dropped() {
        let host = Host::new(Vec::new());
        let mut lane = wing(&[1, 2]);
        // A tank in the keep is nothing to it.
        let pawn = crate::test_host::enemy(50, PAWN, 2300.0, 2000.0);
        let mut said = vec![Command::Move { unit: UnitId(1), to: at(100.0, 100.0), queue: false }];
        let out = lane.tick(&host, &tick(30, vec![falcon(1, 2000.0, 2000.0, (0.0, 0.0)), falcon(2, 2050.0, 2000.0, (0.0, 0.0))], vec![pawn.clone()]), &mut said, false);
        assert!(said.is_empty(), "the host's move to a fighter of the wing went out: {said:?}");
        assert!(out.intercepts.is_empty());
        for id in [1, 2] {
            let legs: Vec<Vec3> = out.commands.iter().filter_map(|c| match c { Command::Patrol { unit, to, .. } if unit.0 == id => Some(*to), _ => None }).collect();
            assert_eq!(legs.len(), 4, "{:?}", out.commands);
            assert!(legs.iter().all(|p| p.dist2d(PLACE) < 600.0));
        }
        // Not told again while it flies the loop.
        let again = lane.tick(&host, &tick(33, vec![falcon(1, 2000.0, 2000.0, (0.0, 0.0)), falcon(2, 2050.0, 2000.0, (0.0, 0.0))], vec![pawn]), &mut Vec::new(), false);
        assert!(again.commands.is_empty(), "{:?}", again.commands);
        assert_eq!(lane.intercept_report("wing").map(|r| (r.contacts, r.loitering)), Some((0, 2)));
    }

    /// At a radar contact that flies the wing leaves its loop for the point beside the contact's track and ahead of
    /// it; it is not ordered at the contact.
    #[test]
    fn a_wing_scrambles_at_a_radar_contact_that_flies_to_a_point_beside_its_track() {
        let host = Host::new(Vec::new());
        let mut lane = wing(&[1]);
        // The bomber 3,000 west of the place, flying east at it; the fighter over the place, a little north.
        let out = lane.tick(&host, &tick(30, vec![falcon(1, 2000.0, 1950.0, (-250.0, 0.0))], vec![bomber(60, -1000.0, 2000.0, false)]), &mut Vec::new(), false);
        assert!(matches!(out.intercepts.as_slice(), [InterceptEvent { what: InterceptWhat::Scrambled { contacts: 1, .. }, .. }]), "{:?}", out.intercepts);
        assert!(!out.commands.iter().any(|c| matches!(c, Command::Attack { .. })), "{:?}", out.commands);
        let to = out.commands.iter().find_map(|c| match c { Command::Move { to, .. } => Some(*to), _ => None }).expect("a move");
        // West of it, toward the bomber, and on its own (north) side of the track.
        assert!(to.x < 1500.0 && to.z < 2000.0, "{to:?}");
        assert_eq!(lane.intercept_report("wing").map(|r| (r.contacts, r.closing)), Some((1, 1)));
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
            across |= out.commands.iter().any(|c| matches!(c, Command::Move { to, .. } if (to.z - 1950.0).abs() > 500.0));
        }
        assert_eq!(attacks, 0);
        assert!(across, "never sent across the track");
    }
}
