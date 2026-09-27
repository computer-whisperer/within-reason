//! The control lane: what a soldier does with its order between its commander's decisions, every tick
//! (`docs/design/2026-09-20-micro-lane.md`, `docs/design/2026-09-25-formation-micro.md`). The host (the bot's brain,
//! or the duel director) decides intent and gives orders; the lane may override a unit's order for as long as a
//! behaviour claims it, and gives the order back when none does. It never chooses a party's target and never
//! re-prices a fight: it spends a unit's order on staying alive, shooting well and standing in its slot. The one
//! exception is a rover (`rove.rs`, H-MICRO-ROVE): a unit the host hands over whole, which chooses where to look and
//! what to kill itself.
//!
//! The host is a [`View`]: unit definitions, each type's fighting numbers, the ground, and what it remembers of the
//! enemy. H-MICRO-LANE switches the whole lane off; each behaviour has its own ID under it.

pub mod form;
mod rove;
pub mod threat;

pub use rove::{RoveEvent, RoveGoal, RoveReport, RoveWhat};

use std::collections::{HashMap, HashSet};

use bot_protocol::{Command, EnemyUnit, OwnUnit, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};
use serde::{Deserialize, Serialize};

pub use threat::ThreatGrid;

pub const FRAMES_PER_SECOND: i32 = 30;

/// Only soldiers with an enemy this close (plus its reach) are looked at by the fighting behaviours; a body's
/// formation is engaged when an armed enemy is this close to its centre.
const HORIZON: f32 = 700.0;
/// The unit's next place is its position plus this many frames of its velocity: half a second.
const LOOKAHEAD_FRAMES: f32 = 15.0;
/// Beyond a threat's reach the grid slopes off over this much: what a Pawn walks in a second.
const TAIL: f32 = 90.0;
/// A flee is a step to the least threatened cell within this.
const STEP_RADIUS: f32 = 100.0;
/// The order goes this many times further along the step's direction than the chosen cell, so the unit is not
/// held in the engine's braking zone: after a step re-issued every 32 elmos Stouts moved 28-30 elmos/s and
/// Blitzes 25-40 against 48-55 on a far move (2v1b-hard, 2v1-hard_aggressive, bank-1).
const STEP_REACH: f32 = 2.5;
/// A fleeing unit's step is re-issued when its cell has moved this far, and no more often than this.
const REORDER_DISTANCE: f32 = 64.0;
const REORDER_FRAMES: i32 = 6;
/// A claimed unit keeps its step while the cell it has is within this share of the least threat it could reach:
/// the chooser flipped between cells of nearly equal threat every six frames (2v1-hard_aggressive 7:04-7:12, two
/// units re-stepped to points zigzagging within 50 elmos).
const STEP_KEEP: f32 = 1.2;
/// An armed enemy out of sight is remembered where it was seen for this long, fading.
const MEMORY_FRAMES: i32 = 10 * FRAMES_PER_SECOND;
/// The grid's cell when the host sent no terrain.
const DEFAULT_CELL: f32 = 16.0;
/// A source fainter than this (a memory nearly faded) stamps the grid but moves nobody.
const FAINT: f32 = 10.0;
/// A claim stands at least this long: a step's reversed velocity cleared the unit's predicted place at once, and
/// (before it was retired) focus took a fleeing unit back the tick after its step (bank-1 8:53: six Blitzes fled
/// twelve times a second and never left).
const CLAIM_FRAMES: i32 = 30;
/// A target this far beyond a shooter's reach still counts as in it (the engine closes the last few elmos).
const REACH_SLACK: f32 = 20.0;
/// A unit's friends within this count toward its side's strength (H-MICRO-FLEE's lethal branch).
const GROUP_RADIUS: f32 = 300.0;
/// A committed unit leaves when the threat where it is heading would kill it within this long.
const EXPOSURE_SECONDS: f32 = 1.5;
/// A wounded unit leaves a unit fight only when their strength on it (damage a second times health, over the
/// soldiers of theirs whose reach covers it) beats our strength round it by this factor. Damage a second alone
/// misjudged it: 22 Pawns out-shoot 11 Blitzes but not out-fight them (form-smoke-old: the Blitzes fled a fight
/// they win with the lane off, 33% of their in-reach seconds muzzled, all under the flee).
const ODDS_TO_STAY: f32 = 1.2;
/// A unit kites what it out-reaches by this much or more: a Rocketeer (475) a Hammer (380), not a Stout (350) a
/// Warrior (325), which stepped back every reload instead of firing (form8-scen1510: 33 of 42 muzzled seconds under
/// the kite, 0.46 shots per in-reach second against 0.68 with the lane off).
const KITE_MARGIN: f32 = 80.0;
/// Where a kiting unit keeps its enemy: this far inside its reach.
const KITE_EDGE: f32 = 15.0;
/// How far a kiting unit steps back while reloading, beyond what restores the edge.
const KITE_STEP: f32 = 40.0;
/// A commander's D-gun line is spaced out to this far beyond its reach: three seconds of a Pawn's approach. The
/// shot comes as the party enters the reach (matt-raidprice and matt-fan: every D-gun death was a unit closing at
/// full speed, dead at 240 to 280 from the commander, in a file with the rest), so a zone the size of the reach
/// itself catches a unit half a second before it dies (matt-fan: the rule fired ten times in twelve games).
const DGUN_MARGIN: f32 = 260.0;
/// The width of a D-gun shot: units of ours killed by one shot lay within 20 elmos either side of its line
/// (matt-plan, matt-raidprice, matt-fan2: six shots of two to six). Two lines to the commander that pass closer than
/// this at the edge of its reach are one line to the shot.
const DGUN_SHADOW: f32 = 48.0;
/// The gap a shadowed unit opens between its line and its friend's, at the edge of the reach.
const FAN_CLEAR: f32 = 64.0;
/// The most a fan step is, however far out the unit is.
const FAN_STEP_MAX: f32 = 150.0;
/// H-MICRO-FORM: a unit stands when an enemy is inside its reach by this much (the engine's range test and ours
/// disagree at the edge; the fire instrument calls the last 40 the edge), and keeps standing while one is within
/// its reach plus `STAND_KEEP`, and for `FORM_FRAMES` at least: every change of stance is an order, and an order
/// costs shots (form-smoke: Blitzes re-ordered between standing, stepping back and closing every few frames fired
/// 1.8 a second in reach against 6.6 with the lane off). A keep of 100 held Stouts standing out of reach of Thuds
/// at 380 (form5-stoutmix-on, -0.08): the keep is the reach slack now.
const STAND_INSIDE: f32 = 20.0;
const STAND_KEEP: f32 = REACH_SLACK;
/// A unit closing is sent this many seconds of its own speed forward along its body's heading (at least
/// `CLOSE_LEAD`), on its own line, not at its nearest enemy: an attack-move stops when a target is in range, a goal
/// short of the enemy is within the unit's braking distance for the whole chase (standing-1's Rover at 33-44% of
/// its speed), and a goal at the enemy folds a formed line into a ball in the last 200 elmos (u-blitz-on: spacing
/// 43 at contact with every unit sent at its nearest enemy).
const CLOSE_SECONDS: f32 = 3.0;
const CLOSE_LEAD: f32 = 150.0;
/// On the march a body forms its rank first, at a waypoint this many seconds of its fastest member's speed ahead
/// (at least `FORM_UP_LEAD`), and walks from there to its points at the destination on parallel lines, both in
/// one queued order: points at the destination alone spread the line only on arrival, and contact en route met
/// a body still at its spawn spacing (t-blitz-on: nearest friend 45 at contact against the loop's 59).
const FORM_UP_SECONDS: f32 = 3.0;
const FORM_UP_LEAD: f32 = 300.0;
/// H-MICRO-FORM-FLANK: a unit this near its flank slot has reached it and closes on the enemy from there.
const FLANK_ARRIVED: f32 = 48.0;
/// A stance lasts at least this long: a change is an order, and an order costs shots.
const FORM_FRAMES: i32 = 15;
/// H-MICRO-HUNT: a quarry out of sight and radar this long ends the hunt.
const HUNT_LOST_FRAMES: i32 = 6 * FRAMES_PER_SECOND;
/// A hunter under this share of its health drops out of the hunt.
const HUNT_DROP_HEALTH: f32 = 1.0 / 3.0;

/// What a soldier's group was priced against, so the lane knows which threats its host meant it to face.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Commitment {
    /// Nothing: a unit on a Move, a scout, a raider waiting for the rest. It flees any threat.
    #[default]
    None,
    /// Every soldier in sight, these turrets, and the commander when `commander`.
    Priced { turrets: Vec<UnitId>, commander: bool },
    /// Everything (a committed wave, a commander's squad, a duel's army).
    All,
    /// A hunter after one unit (H-MICRO-HUNT): it runs raw, under no footwork rule, and the lane's attack order
    /// does the closing.
    Hunt(Hunt),
    /// A rover (H-MICRO-ROVE): the lane runs it whole, and the host's orders to it are dropped.
    Rove,
}

/// One quarry for a set of hunters: the unit to kill, and the leash (from where the hunt began, or the group's
/// station) beyond which the hunt ends.
#[derive(Clone, Debug, PartialEq)]
pub struct Hunt {
    pub quarry: UnitId,
    pub leash_from: Vec3,
    pub leash: f32,
}

/// What the lane reports of a hunt: `hunter` is `None` when the hunt on `quarry` ended (`why`: "dead", "lost" (out
/// of sight and radar 6 s), "leash", "no hunters"), else that hunter dropped out ("hurt": under a third of its
/// health; "slower": not faster than the quarry). Each is reported once; the host clears the commitments.
#[derive(Clone, Debug, PartialEq)]
pub struct HuntEvent {
    pub quarry: UnitId,
    pub hunter: Option<UnitId>,
    pub why: &'static str,
}

/// The lane's memory of one hunt.
struct HuntState {
    started: i32,
    /// The hunt's leash point: a commitment with another one is a new hunt on the same quarry (the raid's second
    /// hunt after a leash end starts from where the hunters then stand).
    leash_from: Vec3,
    /// The quarry's last sighting (sight or radar): place, velocity, frame.
    last_seen: Option<(Vec3, Vec3, i32)>,
    quarry_def: Option<UnitDefId>,
    ended: Option<&'static str>,
    dropped: HashMap<UnitId, &'static str>,
    /// The frame of the sighting each hunter was last sent to when the quarry was out of sight.
    sent_to_sighting: HashMap<UnitId, i32>,
}

impl Commitment {
    fn covers(&self, source: &Source) -> bool {
        match self {
            Commitment::None => false,
            Commitment::All | Commitment::Hunt(_) | Commitment::Rove => true,
            Commitment::Priced { turrets, commander } => {
                if source.commander {
                    *commander
                } else if source.mobile {
                    true
                } else {
                    turrets.contains(&source.id)
                }
            }
        }
    }
}

/// Which footwork rules apply to a group's soldiers (H-HANDS-LANE): the lane's five, the march that keeps an
/// advancing group together, and the re-sending of an engaging group after its party. All on by default; `raw` is
/// none, and the group's orders reach the engine as the host gave them. `rove` is the other mode (H-MICRO-ROVE): no
/// rule and no order of the host's, the lane runs the group's soldiers whole.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Footwork {
    pub flee: bool,
    pub fan: bool,
    pub kite: bool,
    #[serde(default = "yes")]
    pub form: bool,
    pub march: bool,
    pub follow: bool,
    #[serde(default)]
    pub rove: bool,
}

fn yes() -> bool {
    true
}

impl Default for Footwork {
    fn default() -> Footwork {
        Footwork { flee: true, fan: true, kite: true, form: true, march: true, follow: true, rove: false }
    }
}

impl Footwork {
    pub const RULES: [&'static str; 6] = ["flee", "fan", "kite", "form", "march", "follow"];

    pub fn raw() -> Footwork {
        Footwork { flee: false, fan: false, kite: false, form: false, march: false, follow: false, rove: false }
    }

    /// The rover's setting: none of the rules, the lane runs the soldiers itself (H-MICRO-ROVE).
    pub fn rove() -> Footwork {
        Footwork { rove: true, ..Footwork::raw() }
    }

    /// The rules kept, by name.
    pub fn kept(&self) -> Vec<&'static str> {
        let flags = [self.flee, self.fan, self.kite, self.form, self.march, self.follow];
        Footwork::RULES.iter().zip(flags).filter(|(_, on)| *on).map(|(name, _)| *name).collect()
    }

    /// From the rules to keep, by name; an unknown name is the error.
    pub fn keeping(names: &[String]) -> Result<Footwork, String> {
        let mut footwork = Footwork::raw();
        for name in names {
            match name.as_str() {
                "flee" => footwork.flee = true,
                "fan" => footwork.fan = true,
                "kite" => footwork.kite = true,
                "form" => footwork.form = true,
                "march" => footwork.march = true,
                "follow" => footwork.follow = true,
                other => return Err(format!("{other} is not a footwork rule; the rules are {}", Footwork::RULES.join(", "))),
            }
        }
        Ok(footwork)
    }

    /// One line for the picture: what stands when it is not the default.
    pub fn words(&self) -> Option<String> {
        if *self == Footwork::default() {
            return None;
        }
        if self.rove {
            return Some("roving: its soldiers run on their own in code, ten times a second, and take no orders from your hands: each looks at the places nothing of ours has seen (his start box first), kills what it finds unguarded (extractors, radars, lone constructors) and keeps out of the reach of anything that can shoot it".to_string());
        }
        let kept = self.kept();
        Some(if kept.is_empty() { "raw: no footwork rules; its orders go to the engine as given".to_string() } else { format!("footwork rules {} only", kept.join(", ")) })
    }
}

/// A type's fighting numbers against ground: reach, damage a second, speed (elmos a second), hit points, the reach of
/// its D-gun (a `command_fire` weapon; 0 for everything but a commander), and its sight.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stats {
    pub reach: f32,
    pub dps: f32,
    pub speed: f32,
    pub health: f32,
    pub dgun: f32,
    pub sight: f32,
}

/// What the lane reads of its host.
pub trait View {
    fn def(&self, def: UnitDefId) -> Option<&UnitDefInfo>;
    /// A type's numbers; `None` for a type the host has no numbers for (it then neither threatens nor shoots).
    fn stats(&self, def: UnitDefId) -> Option<Stats>;
    /// The threat grid's cell size and dimensions.
    fn grid_spec(&self) -> (f32, usize, usize);
    /// One flag a cell in the grid's order: whether our soldiers can stand there; `None` when not known.
    fn passable(&self) -> Option<&[bool]>;
    /// The reachable ground nearest `pos`; `pos` itself when the host cannot tell.
    fn snap(&self, pos: Vec3) -> Vec3;
    fn home(&self) -> Vec3;
    /// Armed buildings of theirs the host remembers, in sight or not: id, type, place.
    fn remembered_buildings(&self) -> Vec<(UnitId, UnitDefId, Vec3)>;
    /// Where a remembered building of theirs stands (an Attack order's goal when the target is out of sight).
    fn building_at(&self, id: UnitId) -> Option<Vec3>;
    /// Whether a soldier of theirs seen lately is still believed alive.
    fn enemy_known(&self, id: UnitId) -> bool;
    /// The type a radar-only contact is taken for (the soldier of theirs seen most).
    fn blip_def(&self) -> Option<UnitDefId>;
    /// False when an ablation run has switched this rule off.
    fn enabled(&self, rule: &str) -> bool;
    /// Whether this unit of the team is the lane's to command: a duel director runs one lane per army and the
    /// team's snapshot holds every army of the team on every site (a lane that took the other site's units for its
    /// own fled them from everything: form2-blitz-old, seven losses of eight in fights twice as long).
    fn mine(&self, unit: UnitId) -> bool;
    /// What the debug lines are prefixed with.
    fn label(&self) -> String;
    /// The places a rover of this type may go and look at, each with when it was last within sight of anything of
    /// ours (H-MICRO-ROVE): the metal spots it can reach, and his base when one is known. None by default (a host
    /// without rovers).
    fn rove_goals(&self, _def: UnitDefId) -> Vec<RoveGoal> {
        Vec::new()
    }
}

/// A soldier the lane commands: armed, mobile, no builder, on the ground (aircraft are out: its steps are ground
/// cells and its threats ground guns); the commander is a builder.
pub fn is_soldier(d: &UnitDefInfo) -> bool {
    d.weapon_count > 0 && d.speed > 0.0 && d.build_speed == 0.0 && d.move_class.is_some()
}

fn is_commander(d: &UnitDefInfo) -> bool {
    d.name.ends_with("com") && d.speed > 0.0 && d.build_speed > 0.0
}

/// One thing that can hurt us this tick.
struct Source {
    id: UnitId,
    pos: Vec3,
    reach: f32,
    /// Damage a second, faded when it is a memory.
    weight: f32,
    /// Its hit points now (a memory's: its type's).
    health: f32,
    mobile: bool,
    commander: bool,
    /// The reach of its D-gun; 0 for everything but a commander.
    dgun: f32,
    /// Elmos a second; 0 for a building.
    speed: f32,
    def: Option<UnitDefId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Rule {
    Flee,
    Fan,
    Kite,
    Form,
    Hunt,
}

impl Rule {
    fn id(self) -> &'static str {
        match self {
            Rule::Flee => "H-MICRO-FLEE",
            Rule::Fan => "H-MICRO-FAN",
            Rule::Kite => "H-MICRO-KITE",
            Rule::Form => "H-MICRO-FORM",
            Rule::Hunt => "H-MICRO-HUNT",
        }
    }
}

/// What a unit did while the lane held it (the milling instrument): where the claim began, where it was last
/// seen, the direction of its last step, the path walked and the reversals of more than ninety degrees.
struct Motion {
    start: Vec3,
    last: Vec3,
    last_step: Option<(f32, f32)>,
    path: f32,
    reversals: u32,
}

/// The milling counters over the claims that ended: claims, the path those units walked while held, their net
/// displacement, and reversals of more than ninety degrees.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Milling {
    pub claims: u32,
    pub path: f32,
    pub net: f32,
    pub reversals: u32,
}

impl std::ops::AddAssign for Milling {
    fn add_assign(&mut self, other: Milling) {
        self.claims += other.claims;
        self.path += other.path;
        self.net += other.net;
        self.reversals += other.reversals;
    }
}

/// A unit the lane has taken over: which behaviour, where it was last sent and when.
struct Claim {
    rule: Rule,
    sent_to: Vec3,
    /// What it was told to shoot (a kite's shot).
    target: Option<UnitId>,
    frame: i32,
    /// A form claim's kind, so a change of kind is a fresh order.
    stance: Option<Stance>,
    /// A form claim's command kind (Fight, Move, Attack on whom, Stop), so a change of order is a fresh order too.
    kind: Option<Kind>,
}

/// The kind of an order, for telling a changed order from the same one again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Fight,
    Move,
    Attack(UnitId),
}

fn kind_of(command: &Command) -> Option<Kind> {
    match command {
        Command::Fight { .. } => Some(Kind::Fight),
        Command::Move { .. } => Some(Kind::Move),
        Command::Attack { target, .. } => Some(Kind::Attack(*target)),
        _ => None,
    }
}

/// What H-MICRO-FORM has a unit doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stance {
    /// On the march: the host's order rewritten to the unit's own point at the destination.
    March,
    /// An enemy in reach: standing where it is (or attacking its order's target).
    Stand,
    /// Its body is engaged and it has nothing in reach: walking to its place in the rank at the front's depth.
    Flank,
    /// At its place in the rank with nothing in reach: stepping forward along the body's heading on its own line.
    Close,
}

/// A form order for one unit this tick. A stand has no command: the unit keeps the order it has (a Fight at its
/// slot, which it has reached, or an Attack on its order's target) and the engine's own attack-move fires at will
/// from there; a Stop would drop the weapon's target, and every order costs shots (form6-stoutmix-dbg: standing
/// Stouts re-stopped fired 0.58 a second in reach against 0.77 with the lane off).
struct FormOrder {
    stance: Stance,
    command: Option<Command>,
    /// A second order queued behind the first (the march's destination point behind its form-up point).
    then: Option<Command>,
    /// Where the command sends the unit.
    at: Vec3,
}

/// What a tick of the lane produced.
#[derive(Default)]
pub struct Output {
    pub commands: Vec<Command>,
    /// Rule IDs that fired, once per firing.
    pub fired: Vec<&'static str>,
    pub milling: Milling,
    /// Hunts that ended and hunters that dropped out this tick (H-MICRO-HUNT), each once.
    pub hunts: Vec<HuntEvent>,
    /// What the rovers found, attacked, ran from and gave up this tick (H-MICRO-ROVE).
    pub rove: Vec<RoveEvent>,
}

#[derive(Default)]
pub struct Lane {
    grid: Option<ThreatGrid>,
    /// Each soldier's last order from the host, and its frame: the lane's terminal behaviour.
    standing: HashMap<UnitId, (Command, i32)>,
    commitment: HashMap<UnitId, Commitment>,
    /// Which footwork rules apply to each soldier; a unit not listed gets them all.
    footwork: HashMap<UnitId, Footwork>,
    claims: HashMap<UnitId, Claim>,
    /// The motion of every unit under a claim, for the milling counters.
    motion: HashMap<UnitId, Motion>,
    /// Armed mobile enemies as last seen: type, place, frame.
    seen: HashMap<UnitId, (UnitDefId, Vec3, i32)>,
    /// Per minute, for the log: claims made, orders issued.
    counts: (u32, u32),
    /// The hunts in progress, by quarry (H-MICRO-HUNT).
    hunts: HashMap<UnitId, HuntState>,
    /// Each rover's memory (H-MICRO-ROVE).
    rovers: HashMap<UnitId, rove::Rover>,
    /// Enemy units some rover has already reported finding.
    rove_found: HashSet<UnitId>,
}

/// The host gave the same order again (it re-issues standing orders every few seconds): no reason to let go.
fn same_order(a: &Command, b: &Command) -> bool {
    match (a, b) {
        (Command::Move { unit, to, queue }, Command::Move { unit: u, to: t, queue: q })
        | (Command::Fight { unit, to, queue }, Command::Fight { unit: u, to: t, queue: q }) => unit == u && to.dist2d(*t) < 1.0 && queue == q,
        (Command::Attack { unit, target, queue }, Command::Attack { unit: u, target: t, queue: q }) => unit == u && target == t && queue == q,
        (Command::Stop { unit }, Command::Stop { unit: u }) => unit == u,
        _ => false,
    }
}

fn unit_of(command: &Command) -> Option<UnitId> {
    match *command {
        Command::Move { unit, .. } | Command::Fight { unit, .. } | Command::Attack { unit, .. } | Command::Stop { unit } => Some(unit),
        _ => None,
    }
}

/// Distance from `p` to the segment `a`-`b`, in the ground plane.
pub fn to_segment(p: Vec3, a: Vec3, b: Vec3) -> f32 {
    let (dx, dz) = (b.x - a.x, b.z - a.z);
    let len2 = dx * dx + dz * dz;
    let t = if len2 <= 0.0 { 0.0 } else { (((p.x - a.x) * dx + (p.z - a.z) * dz) / len2).clamp(0.0, 1.0) };
    (p.x - (a.x + t * dx)).hypot(p.z - (a.z + t * dz))
}

impl Lane {
    /// Whether the lane has this soldier stepping out of a threat's reach, or holding at its edge, this tick (H-MICRO-FLEE).
    pub fn fleeing(&self, unit: UnitId) -> bool {
        self.claims.get(&unit).is_some_and(|c| c.rule == Rule::Flee)
    }

    /// The quarry this unit is hunting under the lane's orders this tick, if any (H-MICRO-HUNT).
    pub fn hunting(&self, unit: UnitId) -> Option<UnitId> {
        self.claims.get(&unit).filter(|c| c.rule == Rule::Hunt).and_then(|c| c.target)
    }

    /// What holds this unit, for an instrument and for a host's idle re-send: the rule's ID and, for a form
    /// claim, its stance. A formed unit on the march or walking to its flank slot is not held: its order is the
    /// host's, rewritten, and the host may re-send it as it re-sends any walking unit's.
    pub fn holding(&self, unit: UnitId) -> Option<String> {
        self.claims.get(&unit).and_then(|c| match c.stance {
            Some(Stance::March | Stance::Flank | Stance::Close) => None,
            Some(stance) => Some(format!("{} {stance:?}", c.rule.id())),
            None => Some(c.rule.id().to_string()),
        })
    }

    /// The host's orders this tick become the standing orders of the soldiers they went to (`soldier` says which
    /// units are soldiers the lane commands).
    pub fn note_standing_orders(&mut self, commands: &[Command], frame: i32, soldier: impl Fn(UnitId) -> bool) {
        for command in commands {
            let Some(unit) = unit_of(command) else { continue };
            if !soldier(unit) {
                continue;
            }
            // A new order from the host outranks the lane's step (the lane claims again if it must); the same
            // order again does not (micro-flee-debug3: a wounded Pawn held out of a fight was sent back into it
            // by the raid's four-second re-issue, every time). A formation claim survives: the slot is recomputed
            // from the new order every tick anyway, and a standing unit re-stopped at every re-order never fires
            // (a Stop drops the weapon's target).
            if self.standing.get(&unit).is_none_or(|(old, _)| !same_order(old, command)) && self.claims.get(&unit).is_none_or(|c| c.rule != Rule::Form) {
                self.claims.remove(&unit);
            }
            self.standing.insert(unit, (command.clone(), frame));
        }
    }

    /// What each soldier's group was priced against, and which footwork rules each is under (a unit not listed
    /// gets every rule).
    pub fn set_commitments(&mut self, commitment: HashMap<UnitId, Commitment>, footwork: HashMap<UnitId, Footwork>) {
        self.commitment = commitment;
        self.footwork = footwork;
    }

    /// The milling counters: every unit under a claim has its steps summed; a unit whose claim has ended (released,
    /// re-ordered, or dead) hands its path, net displacement and reversals to the output.
    fn track_milling(&mut self, own: &[OwnUnit]) -> Milling {
        let claimed: HashSet<UnitId> = self.claims.keys().copied().collect();
        let mut ended = Milling::default();
        self.motion.retain(|id, m| {
            if claimed.contains(id) {
                return true;
            }
            ended.claims += 1;
            ended.path += m.path;
            ended.net += m.start.dist2d(m.last);
            ended.reversals += m.reversals;
            false
        });
        for unit in own.iter().filter(|u| claimed.contains(&u.id)) {
            let m = self.motion.entry(unit.id).or_insert(Motion { start: unit.pos, last: unit.pos, last_step: None, path: 0.0, reversals: 0 });
            let step = (unit.pos.x - m.last.x, unit.pos.z - m.last.z);
            let length = step.0.hypot(step.1);
            if length < 1.0 {
                continue;
            }
            if let Some(last) = m.last_step
                && last.0 * step.0 + last.1 * step.1 < 0.0
            {
                m.reversals += 1;
            }
            m.last_step = Some(step);
            m.path += length;
            m.last = unit.pos;
        }
        ended
    }

    /// Every tick: the threat grid, then each soldier through the behaviours, in order: flee, fan, kite, the
    /// formation (a rewrite of the host's orders of this tick, `host`, into per-unit points), the standing order.
    /// `debug` prints each claim to stderr. The host notes its orders as standing orders before this
    /// (`note_standing_orders`) and appends the output's commands after its own.
    pub fn tick(&mut self, view: &dyn View, tick: &Tick, host: &mut Vec<Command>, debug: bool) -> Output {
        let mut out = Output::default();
        if !view.enabled("H-MICRO-LANE") {
            return out;
        }
        let frame = tick.frame;
        let snapshot = &tick.snapshot;
        self.standing.retain(|id, _| snapshot.own_units.iter().any(|u| u.id == *id));
        self.claims.retain(|id, _| snapshot.own_units.iter().any(|u| u.id == *id));
        out.milling = self.track_milling(&snapshot.own_units);
        let sources = self.threat_sources(view, snapshot.enemies.as_slice(), frame);
        self.rebuild_grid(view, &sources);
        let commands = &mut out.commands;
        let fired = &mut out.fired;

        let mut soldiers: Vec<&OwnUnit> = snapshot.own_units.iter().filter(|u| !u.being_built && view.mine(u.id) && view.def(u.def).is_some_and(is_soldier)).collect();
        // H-MICRO-ROVE before everything: a rover is the lane's alone, and nothing the host said to it this tick
        // reaches the engine.
        let rovers: Vec<&OwnUnit> = soldiers.iter().copied().filter(|u| matches!(self.commitment.get(&u.id), Some(Commitment::Rove))).collect();
        self.rovers.retain(|id, _| rovers.iter().any(|u| u.id == *id));
        if !rovers.is_empty() {
            host.retain(|c| unit_of(c).is_none_or(|u| !rovers.iter().any(|r| r.id == u)));
            for unit in &rovers {
                self.claims.remove(&unit.id);
            }
            self.rove(view, &rovers, snapshot.enemies.as_slice(), &sources, frame, commands, fired, &mut out.rove, debug);
            soldiers.retain(|u| !rovers.iter().any(|r| r.id == u.id));
        }
        // H-MICRO-HUNT first: a hunter is under no other rule and in no body.
        let hunters: Vec<(&OwnUnit, Hunt)> = soldiers.iter().filter_map(|u| match self.commitment.get(&u.id) { Some(Commitment::Hunt(h)) => Some((*u, h.clone())), _ => None }).collect();
        if !hunters.is_empty() || !self.hunts.is_empty() {
            self.hunt(view, &hunters, snapshot.enemies.as_slice(), frame, commands, fired, &mut out.hunts, debug);
            soldiers.retain(|u| !hunters.iter().any(|(h, _)| h.id == u.id));
        }
        // Each soldier's footwork rules, and the rules the host has switched off by ID (an ablation).
        let footwork = self.footwork.clone();
        let gate = Footwork {
            flee: view.enabled(Rule::Flee.id()),
            fan: view.enabled(Rule::Fan.id()),
            kite: view.enabled(Rule::Kite.id()),
            form: view.enabled(Rule::Form.id()),
            march: true,
            follow: true,
            rove: false,
        };
        let rules_of = |id: UnitId| {
            let f = footwork.get(&id).copied().unwrap_or_default();
            Footwork { flee: f.flee && gate.flee, fan: f.fan && gate.fan, kite: f.kite && gate.kite, form: f.form && gate.form, march: f.march, follow: f.follow, rove: false }
        };
        // H-MICRO-FORM's orders for every unit in a body, decided first: the host's group orders of this tick are
        // rewritten into them here (a standing unit's dropped), and the free units take them last.
        let form_orders = if gate.form { self.form(view, &soldiers, snapshot.enemies.as_slice(), &sources, &rules_of) } else { HashMap::new() };
        let mut host_ordered: HashSet<UnitId> = HashSet::new();
        let mut rewritten: Vec<Command> = Vec::with_capacity(host.len());
        for command in host.drain(..) {
            let unit = unit_of(&command);
            let order = unit.filter(|_| matches!(command, Command::Fight { .. } | Command::Move { .. })).and_then(|u| form_orders.get(&u));
            match (unit, order) {
                (Some(unit), Some(order)) => {
                    host_ordered.insert(unit);
                    rewritten.extend(order.command.clone());
                    rewritten.extend(order.then.clone());
                }
                _ => rewritten.push(command),
            }
        }
        *host = rewritten;
        let mut free: Vec<&OwnUnit> = Vec::new();
        for unit in &soldiers {
            let near = sources.iter().any(|s| s.pos.dist2d(unit.pos) < HORIZON + s.reach);
            let rules = rules_of(unit.id);
            if rules == Footwork::raw() {
                self.release(view, unit.id, frame, commands, debug);
                continue;
            }
            if !near {
                free.push(unit);
                continue;
            }
            if !rules.flee {
                free.push(unit);
                continue;
            }
            // Our strength round the unit, this one's included (damage a second times health): what a unit fight
            // there is worth staying in.
            let friends: f32 = soldiers.iter().filter(|f| f.pos.dist2d(unit.pos) < GROUP_RADIUS).filter_map(|f| Some(view.stats(f.def)?.dps * f.health)).sum();
            // A fresh flee claim stands its second: the tick after a step, the unit's predicted place is out of
            // the threat and the next behaviour would take it straight back (bank-1 8:53, under focus).
            if self.claims.get(&unit.id).is_some_and(|c| c.rule == Rule::Flee && frame - c.frame < CLAIM_FRAMES) {
                self.flee(view, unit, &sources, friends, frame, commands, fired, debug);
                continue;
            }
            if self.flee(view, unit, &sources, friends, frame, commands, fired, debug).is_none() {
                free.push(unit);
            }
        }
        let near = |u: &OwnUnit| sources.iter().any(|s| s.pos.dist2d(u.pos) < HORIZON + s.reach);
        let fannable: Vec<&OwnUnit> = free.iter().copied().filter(|u| near(u) && rules_of(u.id).fan).collect();
        let fanned = self.fan(view, &fannable, &soldiers, &sources, frame, commands, fired, debug);
        free.retain(|u| !fanned.contains(&u.id));
        for unit in &free {
            if near(unit) && rules_of(unit.id).kite && self.kite(view, unit, snapshot.enemies.as_slice(), frame, commands, fired, debug) {
                continue;
            }
            if let Some(order) = form_orders.get(&unit.id) {
                self.take_slot(view, unit, order, host_ordered.contains(&unit.id), frame, commands, fired, debug);
            } else if self.claims.get(&unit.id).is_some_and(|c| c.rule != Rule::Form && frame - c.frame < CLAIM_FRAMES) {
                // A claim stands at least this long.
            } else {
                self.release(view, unit.id, frame, commands, debug);
            }
        }
        if tick.due() % (60 * FRAMES_PER_SECOND) == 0 && self.counts != (0, 0) {
            let (claims, orders) = std::mem::take(&mut self.counts);
            eprintln!("{} f={frame} micro this minute: {claims} units took over, {orders} orders", view.label());
        }
        out
    }

    /// H-MICRO-HUNT: every hunter is sent at its quarry by id every tick while the quarry is in sight or on radar,
    /// and once to the last sighting (carried along its velocity) while it is not; the hunt ends when the quarry is
    /// dead, out of sight and radar for six seconds, or any hunter has passed the leash, and a hunter drops out
    /// under a third of its health or when it is not faster than the quarry. Each end and drop is reported once;
    /// the host clears the commitments (worlds-2: four Rovers at 168 never closed on a Pawn at 87 in 28 s under
    /// fight-to-point orders re-issued at the party's last place, the flank slots 400-500 short of it every tick).
    #[allow(clippy::too_many_arguments)]
    fn hunt(&mut self, view: &dyn View, hunters: &[(&OwnUnit, Hunt)], enemies: &[EnemyUnit], frame: i32, commands: &mut Vec<Command>, fired: &mut Vec<&'static str>, events: &mut Vec<HuntEvent>, debug: bool) {
        let quarries: Vec<UnitId> = hunters.iter().map(|(_, h)| h.quarry).collect();
        self.hunts.retain(|q, _| quarries.contains(q));
        let mut by_quarry: Vec<UnitId> = Vec::new();
        for q in &quarries {
            if !by_quarry.contains(q) {
                by_quarry.push(*q);
            }
        }
        for quarry in by_quarry {
            let leash_from = hunters.iter().find(|(_, h)| h.quarry == quarry).map(|(_, h)| h.leash_from).unwrap_or_default();
            let fresh_state = || HuntState { started: frame, leash_from, last_seen: None, quarry_def: None, ended: None, dropped: HashMap::new(), sent_to_sighting: HashMap::new() };
            if self.hunts.get(&quarry).is_some_and(|s| s.ended.is_some() && s.leash_from.dist2d(leash_from) > 1.0) {
                self.hunts.insert(quarry, fresh_state());
            }
            let state = self.hunts.entry(quarry).or_insert_with(fresh_state);
            if state.ended.is_some() {
                continue;
            }
            let seen = enemies.iter().find(|e| e.id == quarry);
            if let Some(e) = seen {
                state.last_seen = Some((e.pos, e.vel, frame));
                state.quarry_def = e.def.or(state.quarry_def);
            }
            if state.quarry_def.is_none() {
                state.quarry_def = self.seen.get(&quarry).map(|(d, _, _)| *d);
            }
            let quarry_speed = state.quarry_def.and_then(|d| view.stats(d)).map(|s| s.speed);
            let mut ended: Option<&'static str> = None;
            if seen.is_none() && !view.enemy_known(quarry) {
                ended = Some("dead");
            } else if seen.is_none() && state.last_seen.map_or(frame - state.started, |(_, _, f)| frame - f) >= HUNT_LOST_FRAMES {
                ended = Some("lost");
            }
            let mut active: Vec<&OwnUnit> = Vec::new();
            if ended.is_none() {
                for (unit, hunt) in hunters.iter().filter(|(_, h)| h.quarry == quarry) {
                    if state.dropped.contains_key(&unit.id) {
                        continue;
                    }
                    if hunt.leash_from.dist2d(unit.pos) > hunt.leash {
                        ended = Some("leash");
                        break;
                    }
                    let drop = if unit.health < unit.max_health * HUNT_DROP_HEALTH {
                        Some("hurt")
                    } else if let (Some(mine), Some(theirs)) = (view.stats(unit.def).map(|s| s.speed), quarry_speed)
                        && mine <= theirs
                    {
                        Some("slower")
                    } else {
                        None
                    };
                    if let Some(why) = drop {
                        state.dropped.insert(unit.id, why);
                        self.claims.remove(&unit.id);
                        events.push(HuntEvent { quarry, hunter: Some(unit.id), why });
                        if debug {
                            eprintln!("{} f={frame} micro: unit {} drops out of the hunt on {} ({why})", view.label(), unit.id.0, quarry.0);
                        }
                        continue;
                    }
                    active.push(unit);
                }
                if ended.is_none() && active.is_empty() {
                    ended = Some("no hunters");
                }
            }
            if let Some(why) = ended {
                state.ended = Some(why);
                for (unit, _) in hunters.iter().filter(|(_, h)| h.quarry == quarry) {
                    self.claims.remove(&unit.id);
                }
                events.push(HuntEvent { quarry, hunter: None, why });
                if debug {
                    eprintln!("{} f={frame} micro: the hunt on {} ends ({why}) after {} s", view.label(), quarry.0, (frame - state.started) / FRAMES_PER_SECOND);
                }
                continue;
            }
            for unit in active {
                let fresh = !self.claims.get(&unit.id).is_some_and(|c| c.rule == Rule::Hunt && c.target == Some(quarry));
                if fresh {
                    fired.push(Rule::Hunt.id());
                    self.counts.0 += 1;
                    if debug {
                        eprintln!("{} f={frame} micro: {}#{} hunts {}", view.label(), view.def(unit.def).map_or("?", |d| d.name.as_str()), unit.id.0, quarry.0);
                    }
                }
                match (seen, state.last_seen) {
                    (Some(e), _) => {
                        self.claims.insert(unit.id, Claim { rule: Rule::Hunt, sent_to: e.pos, target: Some(quarry), frame, stance: None, kind: None });
                        self.counts.1 += 1;
                        commands.push(Command::Attack { unit: unit.id, target: quarry, queue: false });
                    }
                    (None, Some((pos, vel, at))) => {
                        // Out of sight: once per sighting, to where it was heading.
                        if state.sent_to_sighting.get(&unit.id) != Some(&at) {
                            state.sent_to_sighting.insert(unit.id, at);
                            let ahead = (frame - at) as f32;
                            let to = view.snap(Vec3 { x: pos.x + vel.x * ahead, y: pos.y, z: pos.z + vel.z * ahead });
                            self.claims.insert(unit.id, Claim { rule: Rule::Hunt, sent_to: to, target: Some(quarry), frame, stance: None, kind: None });
                            self.counts.1 += 1;
                            commands.push(Command::Move { unit: unit.id, to, queue: false });
                        }
                    }
                    (None, None) => {}
                }
            }
        }
    }

    /// H-MICRO-FLEE for one unit: `Some` when it owns the unit this tick (a step, or a hold at the edge). A threat
    /// the host did not price the unit against (a turret or the commander its group was not sent at; anything at
    /// all for a unit on a walk), or the lethal branch: soldiers of theirs that would kill it before the host looks
    /// again while its side is losing there, for a unit not yet under their guns (a unit inside their reach stays
    /// and shoots: a unit that walks does not fire, and their fire on a ball at contact is "lethal" for every unit
    /// in it at once: form-smoke-old, seven to ten Blitzes fled a won fight).
    #[allow(clippy::too_many_arguments)]
    fn flee(&mut self, view: &dyn View, unit: &OwnUnit, sources: &[Source], friends: f32, frame: i32, commands: &mut Vec<Command>, fired: &mut Vec<&'static str>, debug: bool) -> Option<Rule> {
        let next = Vec3 { x: unit.pos.x + unit.vel.x * LOOKAHEAD_FRAMES, y: 0.0, z: unit.pos.z + unit.vel.z * LOOKAHEAD_FRAMES };
        let commitment = self.commitment.get(&unit.id).cloned().unwrap_or_default();
        let covers = |s: &Source, p: Vec3| s.pos.dist2d(p) < s.reach + TAIL;
        let unpriced_at = |p: Vec3| sources.iter().find(|s| !commitment.covers(s) && s.weight >= FAINT && covers(s, p));
        // Damage a second from soldiers of theirs at `p`, and their strength (damage a second times health): a
        // committed unit leaves a unit fight it would die in and its side is losing, but never a turret dive its
        // group was priced for (a Pawn cannot get out of a tower's reach in time whatever it does, and the dive
        // needs its gun; micro-flee-debug).
        let share = |s: &Source, p: Vec3| -> f32 {
            let d = s.pos.dist2d(p);
            if d <= s.reach { 1.0 } else { (s.reach + TAIL - d) / TAIL }
        };
        let mobile_threat_at = |p: Vec3| -> f32 { sources.iter().filter(|s| s.mobile && covers(s, p)).map(|s| s.weight * share(s, p)).sum() };
        let mobile_strength_at = |p: Vec3| -> f32 { sources.iter().filter(|s| s.mobile && covers(s, p)).map(|s| s.weight * s.health * share(s, p)).sum() };
        // A threat the host did not price this unit against, where it is heading (or where it stands, while it has
        // not got out yet); or soldiers of theirs that would kill it before the host looks again.
        let claimed = self.claims.get(&unit.id).is_some_and(|c| c.rule == Rule::Flee);
        let unpriced = unpriced_at(next).or_else(|| if claimed { unpriced_at(unit.pos) } else { None });
        let here = if claimed { unit.pos } else { next };
        let theirs = mobile_threat_at(next).max(mobile_threat_at(here));
        let outgunned = mobile_strength_at(next).max(mobile_strength_at(here)) > friends * ODDS_TO_STAY;
        let inside = sources.iter().any(|s| s.mobile && s.weight >= FAINT && s.pos.dist2d(unit.pos) < s.reach);
        let lethal = theirs * EXPOSURE_SECONDS >= unit.health && outgunned && (claimed || !inside);
        let enemies: Vec<EnemyUnit> = Vec::new();
        if let Some(why) = unpriced.map(|s| ("unpriced", s.id)).or(lethal.then_some(("lethal", UnitId(-1)))) {
            let goal = self.standing_goal(view, unit.id, &enemies).unwrap_or(view.home());
            let passable = view.passable();
            let grid = self.grid.as_ref()?;
            let cell = grid.lowest_within(unit.pos, STEP_RADIUS, passable, goal)?;
            let claim = self.claims.get(&unit.id);
            let fresh = !claimed;
            // The cell it has is nearly as safe as the best: keep it, rather than zigzag between equals.
            let keep = claim.is_some_and(|c| {
                c.rule == Rule::Flee && grid.at(c.sent_to) <= grid.at(cell) * STEP_KEEP + 1.0 && c.sent_to.dist2d(unit.pos) > REORDER_DISTANCE / 2.0
            });
            let cell = if keep { claim.map(|c| c.sent_to).unwrap_or(cell) } else { cell };
            let reorder = claim.is_none_or(|c| c.sent_to.dist2d(cell) > REORDER_DISTANCE && frame - c.frame >= REORDER_FRAMES);
            if fresh {
                fired.push(Rule::Flee.id());
                self.counts.0 += 1;
                if debug {
                    eprintln!(
                        "{} f={frame} micro: {}#{} flees ({why:?}) from ({:.0}, {:.0}) to ({:.0}, {:.0}), threat there {:.0}/s, health {:.0}",
                        view.label(), view.def(unit.def).map_or("?", |d| d.name.as_str()), unit.id.0, unit.pos.x, unit.pos.z, cell.x, cell.z,
                        grid.at(next), unit.health
                    );
                }
            }
            if fresh || reorder {
                self.counts.1 += 1;
                // The order goes further along the step's direction than the cell, so the unit is not held in the
                // engine's braking zone; the cell is what the claim remembers.
                let (dx, dz) = (cell.x - unit.pos.x, cell.z - unit.pos.z);
                let to = view.snap(Vec3 { x: unit.pos.x + dx * STEP_REACH, y: 0.0, z: unit.pos.z + dz * STEP_REACH });
                self.claims.insert(unit.id, Claim { rule: Rule::Flee, sent_to: cell, target: None, frame, stance: None, kind: None });
                commands.push(Command::Move { unit: unit.id, to, queue: false });
            }
            return Some(Rule::Flee);
        }
        // Out of danger, but its order leads straight back in (the goal, or the way to it), or back into a fight it
        // would die in: it stands where it is until the host orders otherwise, rather than walking in and out of
        // the tail every few seconds (micro-flee-debug2, -debug3).
        if claimed
            && let Some(goal) = self.standing_goal(view, unit.id, &enemies)
            && (sources.iter().any(|s| !commitment.covers(s) && s.weight >= FAINT && to_segment(s.pos, unit.pos, goal) < s.reach + TAIL)
                || (mobile_threat_at(goal) * EXPOSURE_SECONDS >= unit.health && mobile_strength_at(goal) > friends * ODDS_TO_STAY))
        {
            return Some(Rule::Flee);
        }
        None
    }

    /// H-MICRO-FAN: inside a commander's D-gun reach plus three seconds of approach, a unit whose line to the
    /// commander would pass within a shot's width of a nearer friend's line at the edge of the reach steps across
    /// its line, to the side with fewer of ours, far enough to open a gap there: the D-gun's shot passes through
    /// everything on its line (matt-plan 07 and 08: seven Pawns to one shot, twice), and lines to one point
    /// converge, so a gap here is a smaller one at the reach (matt-fan2: 48 elmos of sidestep at 500 out left the
    /// party 24 apart where the shot came, and one shot still took three). The unit nearest the commander on a line
    /// never steps, so the party fans out rather than withdrawing; commitment is not consulted, a sidestep leaves no
    /// fight. Returns who stepped or holds a fan claim inside the zone.
    #[allow(clippy::too_many_arguments)]
    fn fan(&mut self, view: &dyn View, free: &[&OwnUnit], soldiers: &[&OwnUnit], sources: &[Source], frame: i32, commands: &mut Vec<Command>, fired: &mut Vec<&'static str>, debug: bool) -> Vec<UnitId> {
        let mut owned = Vec::new();
        for commander in sources.iter().filter(|s| s.dgun > 0.0 && s.weight >= FAINT) {
            let zone = commander.dgun + DGUN_MARGIN;
            let inside: Vec<&OwnUnit> = soldiers.iter().copied().filter(|u| u.pos.dist2d(commander.pos) < zone).collect();
            for unit in free.iter().filter(|u| inside.iter().any(|i| i.id == u.id)) {
                let (dx, dz) = (unit.pos.x - commander.pos.x, unit.pos.z - commander.pos.z);
                let along = dx.hypot(dz).max(1.0);
                let (ux, uz) = (dx / along, dz / along);
                // Signed distance across the line; and what a gap here between two lines to the commander is at the
                // edge of its reach (the same when the unit is inside it).
                let across = |f: &OwnUnit| -> f32 { (f.pos.x - commander.pos.x) * -uz + (f.pos.z - commander.pos.z) * ux };
                let at_edge = |f: &OwnUnit| -> f32 { (commander.dgun / f.pos.dist2d(commander.pos).max(1.0)).min(1.0) };
                let shadowed = inside.iter().any(|f| f.id != unit.id && f.pos.dist2d(commander.pos) < along && across(f).abs() * at_edge(f) < DGUN_SHADOW);
                let claim = self.claims.get(&unit.id);
                if !shadowed {
                    if claim.is_some_and(|c| c.rule == Rule::Fan && frame - c.frame < CLAIM_FRAMES) {
                        owned.push(unit.id);
                    }
                    continue;
                }
                // Across by enough to open the gap at the reach, to the side with fewer of ours within that.
                let step_len = (FAN_CLEAR / at_edge(unit)).min(FAN_STEP_MAX);
                let left = inside.iter().filter(|f| f.id != unit.id && f.pos.dist2d(unit.pos) < 2.0 * step_len && across(f) > across(unit)).count();
                let right = inside.iter().filter(|f| f.id != unit.id && f.pos.dist2d(unit.pos) < 2.0 * step_len && across(f) < across(unit)).count();
                let side = if left <= right { 1.0 } else { -1.0 };
                let step = view.snap(Vec3 { x: unit.pos.x + -uz * step_len * side, y: 0.0, z: unit.pos.z + ux * step_len * side });
                let fresh = claim.is_none_or(|c| c.rule != Rule::Fan);
                let reorder = claim.is_none_or(|c| c.sent_to.dist2d(step) > REORDER_DISTANCE / 2.0 && frame - c.frame >= REORDER_FRAMES);
                if fresh {
                    fired.push(Rule::Fan.id());
                    self.counts.0 += 1;
                    if debug {
                        eprintln!(
                            "{} f={frame} micro: {}#{} fans out of the D-gun line of commander {} ({:.0} away) from ({:.0}, {:.0}) to ({:.0}, {:.0})",
                            view.label(), view.def(unit.def).map_or("?", |d| d.name.as_str()), unit.id.0, commander.id.0, along, unit.pos.x, unit.pos.z, step.x, step.z
                        );
                    }
                }
                if fresh || reorder {
                    self.counts.1 += 1;
                    self.claims.insert(unit.id, Claim { rule: Rule::Fan, sent_to: step, target: None, frame, stance: None, kind: None });
                    commands.push(Command::Move { unit: unit.id, to: step, queue: false });
                }
                owned.push(unit.id);
            }
        }
        owned
    }

    /// H-MICRO-KITE for one unit: the nearest soldier of theirs that can hurt it is one it outranges and is no
    /// slower than, inside its reach: while its weapon reloads it steps back to keep the enemy at the edge of its
    /// reach; when the weapon is ready it shoots it. True when it owns the unit.
    #[allow(clippy::too_many_arguments)]
    fn kite(&mut self, view: &dyn View, unit: &OwnUnit, enemies: &[EnemyUnit], frame: i32, commands: &mut Vec<Command>, fired: &mut Vec<&'static str>, debug: bool) -> bool {
        let Some(Stats { reach, dps, speed, .. }) = view.stats(unit.def) else { return false };
        if reach <= 0.0 || dps <= 0.0 {
            return false;
        }
        let nearest = enemies.iter().filter_map(|e| {
            let def = e.def?;
            let d = view.def(def)?;
            let theirs = view.stats(def)?;
            (d.speed > 0.0 && theirs.dps > 0.0 && d.build_speed == 0.0).then_some((e, theirs.reach, theirs.speed))
        }).min_by(|a, b| a.0.pos.dist2d(unit.pos).total_cmp(&b.0.pos.dist2d(unit.pos)));
        let Some((enemy, their_reach, their_speed)) = nearest else { return false };
        let distance = enemy.pos.dist2d(unit.pos);
        if !(reach >= their_reach + KITE_MARGIN && speed >= their_speed && distance < reach + REACH_SLACK) {
            return false;
        }
        // What the claim says now, copied out: the unit is claimed again below whatever it was.
        let claim: Option<(Rule, Vec3, Option<UnitId>, i32)> = self.claims.get(&unit.id).map(|c| (c.rule, c.sent_to, c.target, c.frame));
        let fresh = claim.is_none_or(|c| c.0 != Rule::Kite);
        if fresh {
            fired.push(Rule::Kite.id());
            self.counts.0 += 1;
            if debug {
                eprintln!("{} f={frame} micro: {}#{} kites enemy {} ({:.0} away, reach {reach:.0} against {their_reach:.0})", view.label(), view.def(unit.def).map_or("?", |d| d.name.as_str()), unit.id.0, enemy.id.0, distance);
            }
        }
        let ready = unit.reload_frame <= frame + 1;
        if ready {
            if claim.is_some_and(|c| c.0 == Rule::Kite && c.2 == Some(enemy.id)) {
                return true;
            }
            self.counts.1 += 1;
            self.claims.insert(unit.id, Claim { rule: Rule::Kite, sent_to: unit.pos, target: Some(enemy.id), frame, stance: None, kind: None });
            commands.push(Command::Attack { unit: unit.id, target: enemy.id, queue: false });
            return true;
        }
        // Reloading: back to the edge of the reach, away from the enemy.
        let (dx, dz) = (unit.pos.x - enemy.pos.x, unit.pos.z - enemy.pos.z);
        let len = dx.hypot(dz).max(1.0);
        let back = (reach - KITE_EDGE - distance).max(0.0) + KITE_STEP;
        let step = view.snap(Vec3 { x: unit.pos.x + dx / len * back, y: 0.0, z: unit.pos.z + dz / len * back });
        if claim.is_some_and(|c| c.0 == Rule::Kite && c.2.is_none() && c.1.dist2d(step) < REORDER_DISTANCE / 2.0 && frame - c.3 < REORDER_FRAMES) {
            return true;
        }
        self.counts.1 += 1;
        self.claims.insert(unit.id, Claim { rule: Rule::Kite, sent_to: step, target: None, frame, stance: None, kind: None });
        commands.push(Command::Move { unit: unit.id, to: step, queue: false });
        true
    }

    /// H-MICRO-FORM's orders for every soldier in a body this tick (`form.rs` for the geometry;
    /// `docs/design/2026-09-25-formation-as-a-transform.md`). On the march a body's units get their own points at
    /// the destination, ranks of six across the approach two hulls apart, assigned by least travel: the host's
    /// one point rewritten, nothing issued between the host's orders. At contact (armed enemies within 700 of the
    /// body's centre) the rank is one line across the enemy's direction at the depth of the standing front
    /// (H-MICRO-FORM-FLANK; ranks of six centred on the body without it): a unit with an enemy in reach stands
    /// (no order), one with nothing in reach walks to its place in the rank, and from there closes on the nearest
    /// enemy to a point past it. A Move order has no contact behaviour; an Attack on a unit is left to the engine
    /// on the march.
    fn form(&self, view: &dyn View, soldiers: &[&OwnUnit], enemies: &[EnemyUnit], sources: &[Source], rules_of: &dyn Fn(UnitId) -> Footwork) -> HashMap<UnitId, FormOrder> {
        let mut orders = HashMap::new();
        let members: Vec<form::Member> = soldiers.iter().filter_map(|u| {
            if !rules_of(u.id).form {
                return None;
            }
            let order = match self.standing.get(&u.id)?.0 {
                Command::Fight { to, .. } => form::Order::Fight(to),
                Command::Move { to, .. } => form::Order::Move(to),
                Command::Attack { target, .. } => form::Order::Attack(target),
                _ => return None,
            };
            let reach = view.stats(u.def)?.reach;
            Some(form::Member { id: u.id, pos: u.pos, raider: reach <= form::RAIDER_REACH, order })
        }).collect();
        let target_of = |id: UnitId| enemies.iter().find(|e| e.id == id).map(|e| e.pos).or_else(|| view.building_at(id));
        let by_id: HashMap<UnitId, &OwnUnit> = soldiers.iter().map(|u| (u.id, *u)).collect();
        for body in form::bodies(&members, target_of) {
            let Some(centre) = form::centroid(body.iter().map(|m| m.pos)) else { continue };
            let order = body[0].order;
            let goal = match order {
                form::Order::Fight(p) | form::Order::Move(p) => Some(p),
                form::Order::Attack(id) => target_of(id),
            };
            let walking = matches!(order, form::Order::Move(_));
            // Engaged: an armed enemy within the horizon of the body's centre; the rank then faces the enemy.
            let armed_near: Vec<&Source> = sources.iter().filter(|s| s.weight >= FAINT && s.pos.dist2d(centre) < HORIZON).collect();
            let engaged = !walking && !armed_near.is_empty();
            let enemy_centre = form::centroid(armed_near.iter().map(|s| s.pos));
            let h = match (enemy_centre, goal) {
                (Some(e), _) if engaged => form::heading(centre, e),
                (_, Some(g)) => form::heading(centre, g),
                _ => continue,
            };
            let Some(goal) = goal else { continue };
            if !engaged && matches!(order, form::Order::Attack(_)) {
                continue;
            }
            let flank = engaged && view.enabled("H-MICRO-FORM-FLANK");
            let along = |p: Vec3| p.x * h.0 + p.z * h.1;
            let reach_of = |m: &form::Member| by_id.get(&m.id).and_then(|u| view.stats(u.def)).map_or(0.0, |s| s.reach);
            // The rank's centre: the destination on the march; at contact the standing front's depth (the mean
            // position along the heading of the units with an enemy inside their reach, the frontmost unit when
            // none stands) with the flank rule, the body's centre without it.
            let anchor = if !engaged {
                goal
            } else if flank {
                let standing: Vec<f32> = body.iter().filter(|m| enemies.iter().any(|e| e.pos.dist2d(m.pos) < reach_of(m) - STAND_INSIDE)).map(|m| along(m.pos)).collect();
                let depth = if standing.is_empty() { body.iter().map(|m| along(m.pos)).fold(f32::NEG_INFINITY, f32::max) } else { standing.iter().sum::<f32>() / standing.len() as f32 };
                let shift = depth - along(centre);
                Vec3 { x: centre.x + h.0 * shift, y: centre.y, z: centre.z + h.1 * shift }
            } else {
                centre
            };
            let slots = form::slots(anchor, h, body.len(), form::SPACING, if flank { body.len() } else { form::FILES });
            let positions: Vec<Vec3> = body.iter().map(|m| m.pos).collect();
            let slot_of = form::assign(&positions, &slots, h);
            // The march's form-up rank: ahead of the body by seconds of its fastest member's speed, short of the
            // goal; the same slot indices, so each unit walks a parallel line from its form-up point to its point
            // at the destination.
            let form_up: Option<Vec<Vec3>> = (!engaged).then(|| {
                let fastest = body.iter().filter_map(|m| by_id.get(&m.id)).filter_map(|u| view.stats(u.def)).map(|s| s.speed).fold(0.0, f32::max);
                let ahead = FORM_UP_LEAD.max(fastest * FORM_UP_SECONDS).min(goal.dist2d(centre) * 0.5);
                let at = Vec3 { x: centre.x + h.0 * ahead, y: centre.y, z: centre.z + h.1 * ahead };
                form::slots(at, h, body.len(), form::SPACING, form::FILES)
            });
            for (i, member) in body.iter().enumerate() {
                let Some(unit) = by_id.get(&member.id) else { continue };
                let (reach, speed) = view.stats(unit.def).map_or((0.0, 0.0), |s| (s.reach, s.speed));
                let slot = view.snap(slots[slot_of[i]]);
                let form_order = if !engaged {
                    let make = |to: Vec3, queue: bool| match order {
                        form::Order::Move(_) => Command::Move { unit: unit.id, to, queue },
                        _ => Command::Fight { unit: unit.id, to, queue },
                    };
                    let first = form_up.as_ref().map(|f| view.snap(f[slot_of[i]])).filter(|f| f.dist2d(slot) > FLANK_ARRIVED && f.dist2d(unit.pos) > FLANK_ARRIVED);
                    match first {
                        Some(first) => FormOrder { stance: Stance::March, command: Some(make(first, false)), then: Some(make(slot, true)), at: slot },
                        None => FormOrder { stance: Stance::March, command: Some(make(slot, false)), then: None, at: slot },
                    }
                } else {
                    let nearest = enemies.iter().min_by(|a, b| a.pos.dist2d(unit.pos).total_cmp(&b.pos.dist2d(unit.pos)));
                    let distance = nearest.map_or(f32::INFINITY, |e| e.pos.dist2d(unit.pos));
                    let standing = self.claims.get(&unit.id).is_some_and(|c| c.rule == Rule::Form && c.stance == Some(Stance::Stand));
                    if reach > 0.0 && (distance < reach - STAND_INSIDE || (standing && distance < reach + STAND_KEEP)) {
                        // Standing: its order's target when that is what is in reach, else nothing: it fires at
                        // will where it is.
                        let command = match order {
                            form::Order::Attack(target) if nearest.is_some_and(|e| e.id == target) => Some(Command::Attack { unit: unit.id, target, queue: false }),
                            _ => None,
                        };
                        FormOrder { stance: Stance::Stand, command, then: None, at: unit.pos }
                    } else if unit.pos.dist2d(slot) > FLANK_ARRIVED {
                        FormOrder { stance: Stance::Flank, command: Some(Command::Fight { unit: unit.id, to: slot, queue: false }), then: None, at: slot }
                    } else {
                        // Forward along the body's heading from where it stands, by seconds of its own speed.
                        let lead = CLOSE_LEAD.max(speed * CLOSE_SECONDS);
                        let at = view.snap(Vec3 { x: unit.pos.x + h.0 * lead, y: 0.0, z: unit.pos.z + h.1 * lead });
                        FormOrder { stance: Stance::Close, command: Some(Command::Fight { unit: unit.id, to: at, queue: false }), then: None, at }
                    }
                };
                orders.insert(unit.id, form_order);
            }
        }
        orders
    }

    /// A free unit with a form order takes it. An order goes out when the claim is fresh (the body just formed,
    /// or the unit came back from a flee), when the stance or the order's kind changed, or when the host gave the
    /// unit a group order this tick (`host_ordered`: the host's command was rewritten in place, so nothing is
    /// pushed here). Never on a distance the point has moved: the formation is a transform of the host's order,
    /// not a control loop (`docs/design/2026-09-25-formation-as-a-transform.md`).
    #[allow(clippy::too_many_arguments)]
    fn take_slot(&mut self, view: &dyn View, unit: &OwnUnit, order: &FormOrder, host_ordered: bool, frame: i32, commands: &mut Vec<Command>, fired: &mut Vec<&'static str>, debug: bool) {
        let claim = self.claims.get(&unit.id);
        let fresh = claim.is_none_or(|c| c.rule != Rule::Form);
        let kind = order.command.as_ref().and_then(kind_of);
        let changed = claim.is_some_and(|c| c.rule == Rule::Form && (c.stance != Some(order.stance) || c.kind != kind) && frame - c.frame >= FORM_FRAMES);
        if !(fresh || changed || host_ordered) {
            return;
        }
        if fresh {
            fired.push(Rule::Form.id());
            self.counts.0 += 1;
        }
        if debug && (fresh || changed) {
            eprintln!(
                "{} f={frame} micro: {}#{} forms ({:?}) from ({:.0}, {:.0}) to ({:.0}, {:.0})",
                view.label(), view.def(unit.def).map_or("?", |d| d.name.as_str()), unit.id.0, order.stance, unit.pos.x, unit.pos.z, order.at.x, order.at.z
            );
        }
        self.claims.insert(unit.id, Claim { rule: Rule::Form, sent_to: order.at, target: None, frame, stance: Some(order.stance), kind });
        if let Some(command) = &order.command
            && !host_ordered
        {
            self.counts.1 += 1;
            commands.push(command.clone());
            commands.extend(order.then.clone());
        }
    }

    /// A claimed unit no behaviour wants any more gets its standing order back, once.
    fn release(&mut self, view: &dyn View, unit: UnitId, frame: i32, commands: &mut Vec<Command>, debug: bool) {
        if self.claims.remove(&unit).is_none() {
            return;
        }
        if let Some((order, _)) = self.standing.get(&unit) {
            if debug {
                eprintln!("{} f={frame} micro: unit {} released to its order", view.label(), unit.0);
            }
            commands.push(order.clone());
        }
    }

    /// Where the unit's standing order was taking it: what a flee step prefers among equally safe cells.
    fn standing_goal(&self, view: &dyn View, unit: UnitId, enemies: &[EnemyUnit]) -> Option<Vec3> {
        match self.standing.get(&unit)?.0 {
            Command::Move { to, .. } | Command::Fight { to, .. } => Some(to),
            Command::Attack { target, .. } => enemies.iter().find(|e| e.id == target).map(|e| e.pos).or_else(|| view.building_at(target)),
            _ => None,
        }
    }

    /// Everything that can hurt us this tick: armed enemies in sight, remembered armed buildings, and armed
    /// soldiers seen lately at their last place, fading.
    fn threat_sources(&mut self, view: &dyn View, enemies: &[EnemyUnit], frame: i32) -> Vec<Source> {
        let stats = |def: UnitDefId| -> Option<Stats> { view.stats(def).filter(|s| s.reach > 0.0) };
        let blip = view.blip_def();
        let mut sources = Vec::new();
        for enemy in enemies {
            let Some(def) = enemy.def.or(blip) else { continue };
            let Some(d) = view.def(def) else { continue };
            // A building still being built shoots nothing: stamped whole, a turret at 5% pushed a push off its
            // approach (the user, 2026-09-28: the fight logic had no idea of it).
            if d.weapon_count == 0 || enemy.being_built {
                continue;
            }
            let Some(s) = stats(def) else { continue };
            let mobile = d.speed > 0.0;
            if mobile {
                self.seen.insert(enemy.id, (def, enemy.pos, frame));
            }
            let health = if enemy.health > 0.0 { enemy.health } else { s.health };
            sources.push(Source { id: enemy.id, pos: enemy.pos, reach: s.reach, weight: s.dps, health, mobile, commander: is_commander(d), dgun: s.dgun, speed: s.speed, def: enemy.def });
        }
        for (id, def, pos) in view.remembered_buildings() {
            if sources.iter().any(|s| s.id == id) {
                continue;
            }
            let Some(s) = stats(def) else { continue };
            sources.push(Source { id, pos, reach: s.reach, weight: s.dps, health: s.health, mobile: false, commander: false, dgun: 0.0, speed: 0.0, def: Some(def) });
        }
        self.seen.retain(|id, (_, _, at)| frame - *at < MEMORY_FRAMES && view.enemy_known(*id));
        for (id, (def, pos, at)) in &self.seen {
            if sources.iter().any(|s| s.id == *id) {
                continue;
            }
            let Some(s) = stats(*def) else { continue };
            let fade = 1.0 - (frame - at) as f32 / MEMORY_FRAMES as f32;
            let commander = view.def(*def).is_some_and(is_commander);
            sources.push(Source { id: *id, pos: *pos, reach: s.reach, weight: s.dps * fade, health: s.health, mobile: true, commander, dgun: s.dgun, speed: s.speed, def: Some(*def) });
        }
        sources
    }

    fn rebuild_grid(&mut self, view: &dyn View, sources: &[Source]) {
        if self.grid.is_none() {
            let (cell, width, height) = view.grid_spec();
            let (cell, width, height) = if width > 0 && cell > 0.0 { (cell, width, height) } else { (DEFAULT_CELL, 1, 1) };
            self.grid = Some(ThreatGrid::new(cell, width, height));
        }
        let grid = self.grid.as_mut().expect("made above");
        grid.clear();
        for source in sources {
            grid.stamp(source.pos, source.reach, TAIL, source.weight);
        }
    }
}
