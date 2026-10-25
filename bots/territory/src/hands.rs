//! The hands: the plan in force carried out one order per builder, placed as the simulator places things
//! (docs/design/2026-10-20-territory-bot.md, decision 5; the README's T-ORDERS, T-SITES).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use bot_protocol::{BuildSite, Command, Lane, OwnUnit, Placement, UnitDefId, UnitId, Vec3};
use buildorder::lp::{Class, Kind, LpPlan};
use buildorder::plan::{Item, Plan, Step};
use buildorder::units::Role;
use terrain::Field;

use crate::{Territory, MINUTE, SECOND};

/// Frames a unit is left alone after an order: the engine's idle flag clears a few frames after the order lands.
const GRACE: i32 = 45;
/// T-ORDERS: a build order with no frame begun after this long is held for nothing and dropped (the engine starts no
/// frame while it reclaims a wreck on the site or waits for a blocker, and drops a blocked build itself after thirty
/// tries; the distilled bot's D-ORDERS found a shorter watchdog took orders from builders in range of their spots).
const HELD: i32 = 2 * MINUTE;
/// ... counted from the first frame the builder stands this near the site (terr14-lp-nullai-comet-1: counted from
/// the order, every walk past two minutes was dropped as the builder came within 300 of a far spot, 130 extractor
/// orders in four games, the begun ages a wall at 120 s).
const HELD_NEAR: f32 = 300.0;
/// A step that came to nothing is given once more, at another site, then skipped.
const ATTEMPTS: u32 = 2;
/// The simulator's `assist_chunk`.
const ASSIST_CHUNK: i32 = 20 * SECOND;
/// Frames within which a priority order already sent is not sent again while the flag has not read back yet (the
/// engine applies it a frame or two after the tick that sent it).
const PRIORITY_RESEND: i32 = SECOND;
/// The simulator's `base_radius`: a builder this near home puts an economy building beside itself.
const BESIDE: f32 = 600.0;
/// The engine drops orders given in its first seconds; what comes to nothing before this counts no attempt.
const FIRST_ORDERS: i32 = 3 * SECOND;
/// A construction turret's build distance (its def's 400; the engine adds the target's own radius): what a turret
/// helps within, and what a frame counts its turrets within (T-FARM).
pub(crate) const NANO_REACH: f32 = 400.0;
/// A construction turret is asked for this far behind its factory: outside the yard's keep-out, inside its reach.
const NANO_BEHIND: f32 = 250.0;
/// T-PLANT: a turret the program's factory assist asks for goes round a factory, on a 16-grid point within this of
/// its centre (outside the yard and the exit lane with the rest of keep_out): the turret's reach is 400 to the
/// product on the pad; thebluegecko's 29 stood 97-362 from his advanced plant's centre, behind it and to its sides.
const PLANT_RING: f32 = 330.0;
/// Seconds ahead the program's turret assist is read over, for how many turrets the plants want: a turret is a
/// walk and 18-59 s of building away.
const PLANT_LEAD: f64 = 120.0;
/// Nothing is placed within this of a factory's centre: a site tested against an open yard passes and the engine
/// then drops the order (terr15-lp-nullai-comet-1: seven second labs and 48 construction turrets dropped against
/// the first lab, its yard open round a constructor's frame).
const FACTORY_YARD: f32 = 200.0;
/// A factory's own product inside its yard, told to guard it, walks off first and is given the guard this much
/// later: a newborn guarding its own factory from the pad blocks the next unit (terr28-lp-nullai-comet-1, game 01:
/// the plant's two orders held two minutes each from 9:45 with 28-37 guards standing, the income flat at 190 for
/// four minutes). Its own products only: moved off the yard at every chunk, the commander guarding the first lab
/// lost a third of its power to walks (terr29: nine constructors at 5:00 against fifteen); and the yard, not a
/// hundred of the centre, since the advanced vehicle plant's pad lies past that (terr32 game 01).
const PAD_CLEAR: i32 = 10 * SECOND;
/// Builders at most on a factory's frame: a committed factory (T-COMMIT) is counted standing by the program from
/// its order, so its frame is helped before anything else, this many on it (the yard crowds past that). Left to
/// the flows, the plant's frame kept one builder from 5:02 past 10:00 while five to eight constructors built solars
/// beside it (terr35-lp-nullai-comet-1 game 03; terr28 game 03 the same).
const FACTORY_HELPERS: usize = 6;
/// A building site placed but not yet begun keeps other sites this far off: the host's search knows nothing of
/// sites chosen seconds before (terr16-lp-nullai-comet-1: 72 economy orders in four games dropped by the engine
/// 5-30 s after the order with the builder idle a few hundred elmos from where it was ordered, forty constructors
/// placing solars beside themselves round one base).
const PENDING_SITE: f32 = 70.0;
/// T-OPENING: in the game's first minute the commander takes the free spots within this of home before anything
/// else, nearest first: the pros' opening (K-open-comet-pro-order; thebluegecko's extractors at 0:03, 0:10 and 0:25
/// beside his start, the plant at 1:01), where the plan's lab-first opening put the bot's first three at 0:52,
/// 1:06 and 1:23 (terr47). The program prices the commander's every extractor at one walk (16 s); the spots beside
/// the start cost it six, and on three of them fifty seconds earlier it is a thousand metal by 5:00. The radius is
/// the arm `opening_spots` (500). A spot is taken while the energy stock covers half an extractor's 500 (the walk's
/// income covers the rest). The plant comes next, before the energy for it: his three solars before the plant at
/// 1:01 put the plant's first constructor at 1:21, and with the hands' four spots and three or four solars before
/// the plant it came at 2:17 (terr78-79, behind at every mark to 8:00), where the plant at 0:32 on an energy stock
/// of 900, stalled fourteen seconds, has it at 1:07; the annealer's five-minute opening is one spot, the factory at
/// once, then the solars.
const OPENING: i32 = MINUTE;
/// The opening's energy: before the first factory stands, the commander's solars while the energy income, the
/// energy buildings ordered or framed counted, is under that factory's draw at the commander's power (his three
/// solars before the plant at 1:01: 90 a second against the vehicle plant's 1,550 over 19 s). Off unless the
/// opening takes three spots (`opening_spots` past the third): with two spots the plant at 0:32 stands (terr77-80).
const OPENING_PLANT: i32 = 2 * MINUTE;
/// To this second a constructor with a spot's credit chains spots at the base too, as the far ones do (T-CHAIN): his
/// constructors built nothing but extractors to 5:35 and his commander every solar; the hands' first constructor took
/// the plan's own solar at 1:53 and its first extractor at 2:48 against his first constructor's 1:56 (terr87). Three
/// minutes, not five: shut at 5:00 the window let five minutes of unbuilt energy credit spill into the constructors
/// at once, 16-19 solar orders to them in 5:00-8:00 against 12-13 and 8:00 down 7 in all four games (terr88).
const OPENING_CHAIN: i32 = 3 * MINUTE;
/// A factory's commit or class pick goes to a builder within this of home (T-COMMIT).
const FACTORY_PLACER: f32 = 1000.0;
/// T-SITES: a factory is sited within FACTORY_SEARCH of the point FACTORY_OUT from home toward his start (each
/// factory after the first a quarter turn further round home, and a dropped order one more), on a FACTORY_GRID
/// lattice of the engine's build grid, FACTORY_GAP clear of every footprint (the engine's 8-square gap the host was
/// asked for before the bot chose the site itself).
const FACTORY_OUT: f32 = 350.0;
const FACTORY_SEARCH: f32 = 900.0;
const FACTORY_GRID: f32 = 32.0;
const FACTORY_GAP: f32 = 64.0;
/// A factory's exit lane: the strip its products leave by, LANE_LENGTH out from its centre the way it faces and
/// LANE_HALF to each side. A site's lane is read every LANE_STEP along it, at its middle and LANE_SIDE to either side.
const LANE_LENGTH: f32 = 400.0;
const LANE_HALF: f32 = 120.0;
const LANE_STEP: f32 = 16.0;
const LANE_SIDE: f32 = 64.0;
/// The engine's facings (0 south +z, 1 east +x, 2 north -z, 3 west -x) as directions on the ground.
const FACING_DIR: [(f32, f32); 4] = [(0.0, 1.0), (1.0, 0.0), (0.0, -1.0), (-1.0, 0.0)];
/// T-FARM: every static building but an extractor or a factory goes on one lattice, the farm, laid from an origin
/// FARM_OFFSETS from home on the flattest ground off the line to his start: rows along the map's x axis, a row of
/// turret slots (FARM_TURRET apart) and a row of building slots (FARM_SLOT apart) in turn, FARM_ROW from one turret
/// row to the next, every slot on the engine's 16-elmo build grid (a six-square footprint centres on an odd
/// multiple of 8, a twelve-square one on a multiple of 16) with 16 elmos between neighbours. The building slots
/// within FARM_CORE of the origin are the core, the turrets' ground: a turret takes the free turret slot nearest
/// the origin, a building that takes long enough to want them (FARM_BIG of build time: fusions, advanced
/// converters) the free building slot with the most turrets in reach, and the cheap rest (solars, converters,
/// storages) the free building slot nearest home outside the core. A full lattice holds 32 turrets in reach of a
/// building slot (the rows beside it and the rows beyond) against thebluegecko's median of 39 within 300 of a
/// fusion at its placing; terr54's sunflower of everything in one sequence (80 times the square root of n from
/// home) gave its seventeen fusions 0-5 each, out to 1,400 elmos from home, and the constructors walked 24-33 s
/// an order.
const FARM_ROW: f32 = 176.0;
const FARM_SLOT: f32 = 112.0;
const FARM_TURRET: f32 = 64.0;
/// One building column in this many is a street, kept clear of turrets too: the lattice's gaps are 16 elmos, no
/// vehicle's width, and a slot inside the block could otherwise be reached by nothing on the ground; a slot is at
/// most 168 from a street's centre, within a constructor's reach of 128 and the frame's own size.
const FARM_STREET: i32 = 3;
const FARM_CORE: f32 = 800.0;
/// Two buildings of a 48-elmo footprint share a building slot, FARM_PAIR either side of its centre (converters).
const FARM_PAIR: f32 = 32.0;
const FARM_OFFSETS: [f32; 3] = [800.0, 1000.0, 1200.0];
/// Slots are read this far round the origin (and round home, for the cheap rest).
const FARM_EXTENT: f32 = 1600.0;
/// The host's search round a slot: the build grid's snap and no more, so that a building lands in its slot.
const FARM_SEARCH: f32 = 24.0;
/// A building of this much build time or more goes where the turrets are.
const FARM_BIG: f64 = 20_000.0;
/// A slot the host found no site at is left alone this long (something the bot does not see stands there).
const FARM_DEAD: i32 = 2 * MINUTE;
/// The engine's slope value (255 is vertical) the ground under a slot's footprint may have, and the gap kept
/// between footprints.
const FARM_SLOPE: u8 = 8;
const FARM_GAP: f32 = 8.0;
/// A mobile unit of ours this near a slot's footprint holds it for now: the engine's site search refuses ground
/// with a unit on it, and the host then walked its search out 200 elmos (terr55: 50 of 254 farm orders placed
/// 200 off their slot, all solars and converters by the base, where the pad-keeper parks the idle constructors;
/// the host placed later orders on the same ground and the engine dropped them against the first's building,
/// seven converters in game 00, which struck the type at 14:45).
const FARM_MOBILE: f32 = 32.0;
/// Mobile builders guarding one factory at most: past a yard's worth they crowd the pad and the exit, and the
/// products cannot leave (terr52 game 00: the advanced lab made two constructors at 6:15 and 6:53 and nothing
/// until 11:25, its orders dropped as held with a finished product of its own on the site, 9-24 constructors
/// guarding the two factories round it; game 01's, with the same counts and more room, made one every twenty
/// seconds). The program's assist factor is build power, not bodies.
const ASSIST_GUARDS: usize = 6;
/// A spot with an extractor, a frame or an order of ours this near is taken.
const SPOT_TAKEN: f32 = 100.0;
/// A refused spot is left alone this long.
const SPOT_REFUSED: i32 = MINUTE;
/// A unit type the host found no site for this many times within the window is left alone for the window by every
/// builder (terr18-lp-nullai-comet-1: the geothermal plant, with no vent on the map, asked for by every constructor
/// every half second, 5,500 drops a game and nothing else built for ten minutes). Not on one failure: a converter
/// the host could not place beside one builder in a crowd banned converters for everyone for a minute, three of the
/// last five minutes in terr27-lp-nullai-comet-1's game 00, where the plan wanted a hundred a minute and the
/// hands began thirteen frames of anything in minute nineteen.
const TYPE_REFUSED: i32 = MINUTE;
const NO_SITE_STRIKES: usize = 3;
/// The energy class fires only while the energy stock is under this share of its storage: with the stock at the
/// cap the program's planned energy is for spending that is not happening, and solars ahead of the flows left the
/// metal at zero from 2:00 to 8:00 (terr18-lp-nullai-comet-1). Ahead of the flows only under `ENERGY_STALL`, a
/// stall; else after them, a backstop (terr24: the lab drains the starting energy below half, and the class ahead
/// of the flows built four solars before any extractor in every Cortex opening).
const ENERGY_LOW: f64 = 0.5;
const ENERGY_STALL: f64 = 0.2;
/// ... and only while fewer than this many energy orders stand without a frame.
const ENERGY_PENDING: usize = 3;
/// A unit costing this little energy starts whatever the energy budget says (a solar, a wind, a basic extractor):
/// the stall's own remedies and the cheapest metal.
const ENERGY_FREE: f64 = 500.0;
/// T-DISPATCH: the program's targets are read this far ahead of the clock, so a builder starts what the plan wants
/// next rather than what it wanted at the last whole bucket. Not shorter: the program's counts ramp from the
/// snapshot, and every re-solve restarts the ramp, so a target read 20 s ahead never crossed its threshold for
/// anything the program wanted slowly (terr6-lp-nullai-comet-1: no factory in four games, the commander alone).
pub(crate) const LEAD: f64 = 45.0;
/// T-DISPATCH: a class of lumpy things (the factories as a whole; a factory's products of one role) starts a unit
/// when the program's count of the class runs this far ahead of what stands, is framed or is ordered.
const START_AT: f64 = 0.5;
/// T-DISPATCH: a constructor's no-credit fallbacks (the count rule, whose sites are the farm's, and the dearest
/// frame) reach as far as it walks in LEAD seconds, and the farm must be in reach for either; with nothing, it walks
/// to the farm's core under a Walk order that ends there or after this many frames, so the walker is offered the
/// flows and the count on the way. The count rule had no reach: a constructor that had finished the last spot on the far side took the farm's turret
/// 4,500-7,300 elmos off and walked 87-144 s, and its order covered the count and the column's credit meanwhile,
/// so the constructors standing at the farm took moho jobs; the turrets the plan wanted eleven more of within a
/// minute at 10:15 stood 2 at 10:00 and 3 at 11:00, the metal at its cap from 10:00 to 12:00 in all four games
/// (terr100 game 00: 7,040 / 7,050 and 10,514 / 11,000; terr97 the same), his 29 turrets standing at 10:00 on a
/// bank spent to 88. The dearest rule sent helpers to the advanced fusion frame from 5,700-6,800 elmos the same way.
const WALK_STEP: i32 = 15 * SECOND;
/// What stands within this of a dropped order's site is named in its log line (the held extractor orders).
const ON_SITE: f32 = 80.0;
/// A unit starts only when the metal stock and this many seconds of income cover its metal cost: the program builds
/// fractions of expensive things from the first minute (a tenth of a fusion in the ceiling's second minute, half by
/// 5:00; terr17-lp-nullai-comet-1 committed the advanced plant whole at 3:55 on an income of ten, and the early game
/// starved: income 7-23 at 5:00 against the ceiling's 51). Ninety seconds, not forty-five: at forty-five the plant
/// waited to 9:04-10:27 (terr20) where the ceiling takes its first third of one at 7:00.
pub(crate) const AFFORD_SECONDS: f64 = 90.0;
/// T-COMMIT: a factory type the program wants at all within this long, whose products include one that nothing
/// standing, framed or ordered makes, is started whole: the program builds fractions of a factory for minutes (0.3
/// of a lab in the first minute of the ceiling, 0.3 of an advanced plant from 7:00 for every moho it makes), and a
/// fraction of a factory makes nothing in the engine. Five minutes: within three the Armada plans' vehicle plant,
/// wanted at 0.5-0.9 by the end from 3:35, stayed under the threshold until 6:20 while the Cortex plans' whole
/// advanced lab passed it at 4:17 (terr49: 566 / 623 at 20:00 against 733 / 800). The window, the threshold and the
/// seconds of income the stock must cover the factory's cost with are the arms `commit_within` (300), `commit_at`
/// (0.05) and `commit_afford` (90).
/// T-STRUCK: a unit type whose orders have come to nothing this many times, more often than its frames have begun,
/// is struck from the palette for the rest of the game, and its remaining steps are passed over (terr1-nullai-comet-1:
/// every wind generator order on Comet was dropped by the engine, 9-21 a game, and every Cortex tier-2 extractor,
/// 94 in one game; the search kept writing them).
const STRIKE: u32 = 4;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum What {
    /// `unit` indexes the simulator's unit table; `near` is where the site was asked for; `frame` the building
    /// begun, once the engine names it.
    Build { unit: usize, def: UnitDefId, near: Vec3, frame: Option<UnitId> },
    Assist { factory: UnitId, until: i32 },
    Help { frame: UnitId },
    /// The walk of a constructor with no work where it stands to the farm's core (WALK_STEP): `to` is the farm's
    /// origin. No count and no credit is covered by it.
    Walk { to: Vec3 },
}

#[derive(Clone, Debug)]
pub(crate) struct Order {
    pub what: What,
    pub given: i32,
    /// The last frame the engine was told (the first order, or a repair re-issued to an idle builder).
    pub told: i32,
    /// The first frame the builder stood within `HELD_NEAR` of a build site (the held clock starts there).
    pub arrived: Option<i32>,
    /// The step of the builder's queue it carries out, so a failed one can be given again; none once the plan has
    /// changed under it.
    pub step: Option<usize>,
}

enum Review {
    Keep,
    Done,
    Failed(&'static str),
}

/// A frame under way, as the energy budget reads it (T-ENERGY).
struct FrameDraw {
    /// A producer's or a factory's frame (the producers' budget counts these alone).
    own: bool,
    /// Energy a second its builders draw on it at full speed.
    rate: f64,
    /// Energy left to pay on it.
    left: f64,
}

/// A queue index meaning "no queue in the plan": a constructor past the plan's queues takes the nearest free spot.
const NO_QUEUE: usize = usize::MAX;

pub(crate) struct Hands {
    pub plan: Plan,
    /// The program's targets (T-DISPATCH), when the planner is the program.
    pub targets: Option<LpPlan>,
    /// Build-power-seconds committed per unit type since the targets were adopted: an order's cost to its builder's
    /// type, a guard's chunk of its power.
    pub committed: BTreeMap<(usize, Class), f64>,
    pub orders: BTreeMap<UnitId, Order>,
    /// Per builder: its queue in the plan and the index of its next step.
    pub queues: BTreeMap<UnitId, (usize, usize)>,
    /// The plan's queues for factories and constructors that did not stand when it was made, in order.
    pub spare_factory_queues: VecDeque<usize>,
    pub spare_constructor_queues: VecDeque<usize>,
    pub attempts: BTreeMap<UnitId, u32>,
    /// Spots (indices into the game's list) refused by the host or the engine, until this frame.
    pub refused: BTreeMap<usize, i32>,
    /// Unit types the host found no site for, until this frame.
    pub type_refused: BTreeMap<usize, i32>,
    /// The frames of the "no site" drops of each type within the window (NO_SITE_STRIKES).
    pub no_site: BTreeMap<usize, Vec<i32>>,
    /// Build orders given since the last log line, by type and the dispatch rule that gave them.
    pub ordered: BTreeMap<(usize, &'static str), usize>,
    pub nano_told: BTreeMap<UnitId, i32>,
    /// Per turret, the frame it was last put on and the frame (of the clock) it was told: the turrets' power counts
    /// on a frame when a helper picks the frame to finish (`power_on`; terr60 counted it in the Frames credits too,
    /// which left the plan's turret help short of the turrets in reach for the constructors to make up).
    pub nano_frame: BTreeMap<UnitId, (UnitId, i32)>,
    /// Per builder, the frames it has begun (frame, unit): what a plan searched from an older snapshot has to skip.
    pub begun_log: BTreeMap<UnitId, Vec<(i32, usize)>>,
    /// Per unit type: orders that came to nothing, and frames begun (T-STRUCK).
    pub failures: BTreeMap<usize, u32>,
    pub successes: BTreeMap<usize, u32>,
    pub struck: BTreeSet<usize>,
    /// The farm's origin, chosen on first use (T-FARM), and the farm slots and factory sites the host found no site
    /// at, until a frame.
    pub farm_origin: Option<Vec3>,
    pub farm_dead: Vec<(Vec3, i32)>,
    /// Per factory type, the walking field from home of the class its sites are read for (T-SITES), made on first use.
    pub factory_ground: BTreeMap<usize, Option<Arc<Field>>>,
    pub dropped: usize,
    pub begun: usize,
    pub skipped: usize,
    /// Orders the energy budget refused (T-ENERGY), and by type since the last log line.
    pub energy_refused: usize,
    pub energy_refused_by: BTreeMap<usize, usize>,
    /// Per unit, the priority flag last sent and the frame (T-PRIORITY), and whether the commander has guarded yet.
    pub priority_sent: BTreeMap<UnitId, (bool, i32)>,
    pub commander_guarded: bool,
    /// T-LAGS: the frame of the last `lags:` line.
    pub lag_logged: i32,
}

impl Default for Hands {
    fn default() -> Self {
        Hands {
            plan: Plan::empty(0, 0),
            targets: None,
            committed: BTreeMap::new(),
            orders: BTreeMap::new(),
            queues: BTreeMap::new(),
            spare_factory_queues: VecDeque::new(),
            spare_constructor_queues: VecDeque::new(),
            attempts: BTreeMap::new(),
            refused: BTreeMap::new(),
            type_refused: BTreeMap::new(),
            no_site: BTreeMap::new(),
            ordered: BTreeMap::new(),
            nano_told: BTreeMap::new(),
            nano_frame: BTreeMap::new(),
            begun_log: BTreeMap::new(),
            failures: BTreeMap::new(),
            successes: BTreeMap::new(),
            struck: BTreeSet::new(),
            farm_origin: None,
            farm_dead: Vec::new(),
            factory_ground: BTreeMap::new(),
            dropped: 0,
            begun: 0,
            skipped: 0,
            energy_refused: 0,
            energy_refused_by: BTreeMap::new(),
            priority_sent: BTreeMap::new(),
            commander_guarded: false,
            lag_logged: i32::MIN / 2,
        }
    }
}

/// A factory's exit lane (LANE_LENGTH, LANE_HALF).
fn exit_lane(plant: &OwnUnit) -> Lane {
    lane_at(plant.pos, plant.facing)
}

fn lane_at(centre: Vec3, facing: i32) -> Lane {
    let (dx, dz) = FACING_DIR[facing.rem_euclid(4) as usize];
    Lane { from: centre, to: Vec3 { x: centre.x + dx * LANE_LENGTH, y: centre.y, z: centre.z + dz * LANE_LENGTH }, half_width: LANE_HALF }
}

/// The farm's slots of one kind within `radius` of `centre`, laid from `origin` (T-FARM): turret slots at
/// (8 + FARM_TURRET k, 8 + FARM_ROW r) from the origin, building slots at (FARM_SLOT k, FARM_ROW / 2 + 8 + FARM_ROW r),
/// so that a turret row (footprint 48) and a building row (footprint up to 96) alternate with 16 elmos between.
pub(crate) fn farm_slots(origin: Vec3, centre: Vec3, radius: f32, turret: bool) -> Vec<Vec3> {
    let (pitch, x0, z0) = if turret { (FARM_TURRET, 8.0, 8.0) } else { (FARM_SLOT, 0.0, FARM_ROW / 2.0 + 8.0) };
    // The street columns: every FARM_STREET-th building column, and the turret slots whose footprint reaches it.
    let street = |x: f32| {
        let column = ((x - origin.x) / FARM_SLOT).round() as i32;
        column.rem_euclid(FARM_STREET) == FARM_STREET - 1 && (x - (origin.x + FARM_SLOT * column as f32)).abs() < FARM_SLOT / 2.0 + 24.0
    };
    let mut out = Vec::new();
    let rows = ((centre.z - radius - origin.z - z0) / FARM_ROW).floor() as i32..=((centre.z + radius - origin.z - z0) / FARM_ROW).ceil() as i32;
    for r in rows {
        let z = origin.z + z0 + FARM_ROW * r as f32;
        let cols = ((centre.x - radius - origin.x - x0) / pitch).floor() as i32..=((centre.x + radius - origin.x - x0) / pitch).ceil() as i32;
        for k in cols {
            let p = Vec3 { x: origin.x + x0 + pitch * k as f32, y: origin.y, z };
            if p.dist2d(centre) <= radius && !street(p.x) {
                out.push(p);
            }
        }
    }
    out
}

impl Territory {
    /// A plan found from a snapshot taken at `snapshot` whose builders stood in `builders`' order replaces the plan
    /// in force (docs/design/2026-10-20-territory-bot.md, decision 1): each standing builder takes the queue of its
    /// rank, the queues of builders not yet standing wait for them, and a builder's leading steps are skipped as far
    /// as they are the frames it has begun since the snapshot and then the order it holds (the re-basing: a plan
    /// arrives 20-35 game-seconds after its snapshot at speed 50, a step or two of a constructor's queue).
    pub(crate) fn adopt(&mut self, plan: Plan, builders: &[UnitId], snapshot: i32) {
        let role_of = |me: &Self, id: UnitId| me.own.get(&id).and_then(|u| me.role_of_def(u.def));
        let (mut factories, mut constructors) = (0, 0);
        let mut queues = BTreeMap::new();
        for id in builders {
            let queue = match role_of(self, *id) {
                Some(Role::Commander) => 0,
                Some(Role::Factory) => {
                    factories += 1;
                    plan.factory_queue(factories - 1)
                }
                Some(Role::Builder) => {
                    constructors += 1;
                    plan.constructor_queue(constructors - 1)
                }
                _ => continue,
            };
            let steps = plan.queue(queue);
            let mut next = 0;
            for (_, unit) in self.hands.begun_log.get(id).map_or(&[][..], Vec::as_slice).iter().filter(|(frame, _)| *frame > snapshot) {
                match steps.get(next) {
                    Some(Step { item: Item::Build(u), .. }) if u == unit => next += 1,
                    _ => break,
                }
            }
            if let Some(order) = self.hands.orders.get_mut(id) {
                let begun = matches!(&order.what, What::Build { frame: Some(_), .. });
                let same = match (&order.what, steps.get(next).map(|s| s.item)) {
                    // A frame begun since the snapshot is in the log above; one begun before it was the snapshot's job.
                    (What::Build { unit, .. }, Some(Item::Build(u))) => *unit == u && !begun,
                    (What::Assist { .. }, Some(Item::Assist)) | (What::Help { .. }, Some(Item::Help)) => true,
                    _ => false,
                };
                order.step = if same { Some(next) } else { None };
                next += usize::from(same);
            }
            queues.insert(*id, (queue, next));
        }
        self.hands.spare_factory_queues = (factories..plan.factories.len()).map(|nth| plan.factory_queue(nth)).collect();
        self.hands.spare_constructor_queues = (constructors..plan.constructors.len()).map(|nth| plan.constructor_queue(nth)).collect();
        // A builder finished since the snapshot keeps no queue of the old plan; it takes a spare one when it is next
        // looked at.
        for (id, order) in self.hands.orders.iter_mut() {
            if !queues.contains_key(id) {
                order.step = None;
            }
        }
        self.hands.queues = queues;
        self.hands.plan = plan;
        for log in self.hands.begun_log.values_mut() {
            log.retain(|(frame, _)| *frame > snapshot);
        }
    }

    /// Every half second: each builder's order reviewed, and the next step given to one that has none.
    pub(crate) fn run_hands(&mut self, commands: &mut Vec<Command>) {
        self.log_lags();
        let mut builders: Vec<(UnitId, Role)> = self
            .own
            .values()
            .filter(|u| !u.being_built)
            .filter_map(|u| self.role_of_def(u.def).filter(|r| matches!(r, Role::Commander | Role::Factory | Role::Builder)).map(|r| (u.id, r)))
            .collect();
        builders.sort_by_key(|(id, _)| id.0);
        for (id, role) in builders {
            // The queues are the anneal's; the program's targets need none (terr7-lp-nullai-comet-1: a factory
            // with no spare queue was passed over before the dispatch, and no game made a constructor).
            if self.hands.targets.is_none() && !self.hands.queues.contains_key(&id) {
                let spare = match role {
                    Role::Commander => Some(0),
                    Role::Factory => self.hands.spare_factory_queues.pop_front(),
                    _ => self.hands.spare_constructor_queues.pop_front(),
                };
                match (spare, role) {
                    (Some(queue), _) => self.hands.queues.insert(id, (queue, 0)),
                    (None, Role::Builder) => self.hands.queues.insert(id, (NO_QUEUE, 0)),
                    (None, _) => continue,
                };
            }
            if let Some(order) = self.hands.orders.get(&id).cloned() {
                match self.review(id, &order, commands) {
                    Review::Keep => continue,
                    Review::Done => {
                        self.hands.orders.remove(&id);
                    }
                    Review::Failed(why) => self.failed(id, why),
                }
            }
            if self.hands.targets.is_some() {
                self.dispatch(id, role, commands);
            } else {
                self.give_next(id, role, commands);
            }
        }
    }

    /// T-PRIORITY: the game's builder-priority flag (K-mech-builder-priority-is-a-builder-flag: a low builder draws
    /// only the team's surplus, full speed or nothing; among high builders the engine feeds the oldest first and skips
    /// a step it cannot pay whole). Every plant and construction turret low from birth, as the stock widget sets a
    /// person's; the commander low while it guards a plant and high on a build of its own; constructors high: his
    /// record's flags (the vehicle plant low from 1:02, the constructors high throughout, the commander high through
    /// the opening and the trip and low from 2:50, the second it was back guarding, to past 7:00). With nobody low
    /// (terr89) a stall fed the commander's solars, then the plant, then the constructors' extractors last: 24-32 s
    /// an extractor frame against 20 at 2:37-3:03 of game 00, while his ran 19.7 s every one on a bank of 70-130;
    /// low only while guarding halved that (terr94, level); low from its first guard onward, his flag, took every
    /// extractor frame to its 20 s and 5:00 and 8:00 up in all four games (terr95, the default `com_low=after`).
    /// The flag is read back each tick (`OwnUnit::priority_low`), so an order goes only when it differs, and not
    /// again within PRIORITY_RESEND while the engine applies it; a frame still being built gets none.
    pub(crate) fn run_priorities(&mut self, commands: &mut Vec<Command>) {
        let frame = self.frame;
        self.hands.priority_sent.retain(|_, (_, at)| frame - *at < PRIORITY_RESEND);
        let mut send: Vec<(UnitId, bool)> = Vec::new();
        let mut guarded = self.hands.commander_guarded;
        for unit in self.own.values() {
            if unit.being_built {
                continue;
            }
            let low = match self.role_of_def(unit.def) {
                Some(Role::Factory | Role::Nano) => true,
                Some(Role::Commander) => {
                    let guarding = matches!(self.hands.orders.get(&unit.id).map(|o| &o.what), Some(What::Assist { .. }));
                    guarded |= guarding;
                    // `com_low=after`: low from its first guard onward, as his stayed from 2:50 to past 7:00.
                    guarding || (self.options.com_low_after && guarded)
                }
                Some(Role::Builder) => false,
                _ => continue,
            };
            if unit.priority_low != low && self.hands.priority_sent.get(&unit.id).is_none_or(|(l, _)| *l != low) {
                send.push((unit.id, low));
            }
        }
        self.hands.commander_guarded = guarded;
        for (unit, low) in send {
            commands.push(Command::Priority { unit, low });
            self.hands.priority_sent.insert(unit, (low, frame));
        }
    }

    /// The committed power when targets are adopted: what each builder's standing order still costs its type (the
    /// whole cost with no frame yet, the unbuilt share with one), since the program's new flows carry that remainder
    /// as work to do and would otherwise buy it twice.
    pub(crate) fn seed_committed(&mut self) {
        let mut committed: BTreeMap<(usize, Class), f64> = BTreeMap::new();
        let Some(targets) = self.hands.targets.as_ref() else { return };
        for (id, order) in &self.hands.orders {
            let Some(builder) = self.own.get(id).and_then(|u| self.unit_index(u.def)) else { continue };
            let What::Build { unit, frame, near, .. } = &order.what else { continue };
            // Extractor orders only, in their ring: the program's Frames columns carry every other frame, and the
            // power on those is read live (`frames_power`).
            if self.game().units.list[*unit].extracts_metal == 0.0 {
                continue;
            }
            let class = Class::Ring(targets.ring_of(near.dist2d(self.home) as f64));
            let left = frame.and_then(|f| self.own.get(&f)).map_or(1.0, |f| 1.0 - (f.health / f.max_health.max(1.0)).clamp(0.0, 1.0) as f64);
            let cost = targets.flows.iter().find(|f| f.builder == builder && f.unit == *unit && f.class == class).map_or(self.game().units.list[*unit].build_time, |f| f.cost);
            *committed.entry((*unit, class)).or_default() += cost * left;
        }
        self.hands.committed = committed;
    }

    /// T-DISPATCH: the idle builder follows the program. The lumpy things go by class: the factories as a whole,
    /// and a factory's products by role, start a unit when the program's count of the class runs half a unit ahead
    /// of what stands, is framed or is ordered, and the type started is the one of the class the program wants
    /// most. Read by type nothing ever crossed a threshold: the program is indifferent between factory types and
    /// spreads a plant over all of them, and grows its constructors through a fractional advanced plant
    /// (terr13-lp-nullai-comet-1, game 01: the lab idle from 7:00 to 14:00 with both stocks at their caps, the
    /// program asking three constructors more each minute, all of the tier-2 kind, from a plant nobody started).
    /// Everything else goes by the program's flows: a unit type's credit is the power the program has every builder
    /// type spend on it to the clock plus the lead, less what has been committed to it since the plan was adopted
    /// (the standing orders' unbuilt cost seeds it); the builder takes the largest credit as a share of the unit's
    /// cost among what it can build or assist, however small, since standing idle is never what the program
    /// planned: a building built where T-SITES puts it, a factory product as a guard on the factory that makes it.
    /// By unit type across builders, not by the builder's own type: the program's extractors in the first minute
    /// are on the fractional constructors it imagines and the commander's power on solars, and following its own
    /// flows the commander built four solars and no extractor while the plan wanted two of each
    /// (terr23-lp-nullai-comet-1). With no credit, the largest count shortfall of a type it builds; with none,
    /// help on the frame with the most metal left per builder on it.
    fn dispatch(&mut self, id: UnitId, role: Role, commands: &mut Vec<Command>) {
        let Some(builder) = self.own.get(&id).cloned() else { return };
        let Some(unit) = self.unit_index(builder.def) else { return };
        let now = self.frame as f64 / SECOND as f64;
        let ahead = now + LEAD;
        // T-OPENING: the commander's first minute is the spots beside the start, while the energy stock covers half
        // an extractor's.
        if role == Role::Commander && self.frame < OPENING && let Some(extractor) = self.game().units.extractor(unit) && (self.energy.current as f64) >= self.game().units.list[extractor].energy_cost / 2.0 && self.free_spot(builder.pos, None, Some((self.home, self.options.opening_spots)), None).is_some() {
            let cost = self.game().units.list[extractor].build_time;
            if self.order_one(id, unit, role, extractor, cost, Some(self.ring_of_pos(self.home)), "opening", commands) {
                return;
            }
        }
        let shortfalls = self.shortfalls(ahead);
        // T-COMMIT: a factory type the program wants at all within the window, among whose products is one the
        // program wants that no factory standing, framed or ordered makes, is started whole, before anything else:
        // the first lab, and later the plant of the tier-2 constructors the upgrades need (terr16-lp-nullai-comet-1:
        // the program wanted 0.3 of an advanced plant from 7:00 beside more labs, the class rule took the larger
        // fraction, four labs stood by 9:48 and no upgrade all game; its own ceiling runs on 0.8 of an advanced
        // vehicle plant). Not a kind whose products are all made already: the program spreads fractions over
        // every factory type, and terr7-lp-nullai-comet-1 stood three kinds by 5:00 on one commander's income.
        let commit: Option<usize> = {
            let units = &self.game().units;
            let def = &units.list[unit];
            let targets = self.hands.targets.as_ref().expect("dispatching");
            let ordered = self.hands.orders.values().filter_map(|o| match &o.what { What::Build { unit, frame: None, .. } => Some(*unit), _ => None });
            // Covered by tier: a product's role and whether it builds an upgrade. By exact type the second basic kind
            // was committed as soon as the first stood (terr33 game 03: a lab at 0:18, a vehicle plant at 0:42, the
            // commander on both to 5:00); the intent is the plant of the tier the upgrades need.
            let tier = |u: usize| (units.list[u].role, units.list[u].builds.iter().any(|b| self.is_upgrade(*b)));
            let covered: Vec<(Role, bool)> = self.own.values().filter_map(|f| self.unit_index(f.def)).chain(ordered).filter(|f| units.list[*f].role == Role::Factory).flat_map(|f| units.list[f].builds.iter().map(|p| tier(*p))).collect();
            let (within, at) = (self.options.commit_within, self.options.commit_at);
            let wanted_soon = |u: usize| (0..=6).any(|k| targets.count_at(u, now + k as f64 * within / 6.0) >= at);
            // One commitment at a time: with a factory framed or ordered, none (terr32: the program wanting both
            // constructor kinds, a lab and a vehicle plant were committed in the first minute, and the commander
            // built both and assisted them to 5:00 on an income of seven).
            let pending_factory = self.hands.orders.values().any(|o| matches!(&o.what, What::Build { unit, frame, .. } if units.list[*unit].role == Role::Factory && frame.is_none_or(|f| self.own.get(&f).is_some_and(|u| u.being_built))));
            // No factory before `factory_from` (the arm; the program holds its factory columns to the same second).
            let too_early = now < self.options.factory_from;
            targets
                .types
                .iter()
                .copied()
                .filter(|_| !pending_factory && !too_early)
                .filter(|f| units.list[*f].metal_cost <= self.metal.current as f64 + self.metal.income as f64 * self.options.commit_afford)
                .filter(|f| units.list[*f].role == Role::Factory && def.builds.contains(f) && !self.hands.struck.contains(f))
                .filter(|f| wanted_soon(*f) && units.list[*f].builds.iter().any(|p| !covered.contains(&tier(*p)) && wanted_soon(*p)))
                .max_by(|a, b| targets.count_at(*a, now + within).total_cmp(&targets.count_at(*b, now + within)))
        };
        // The opening's energy (OPENING_PLANT): his solars before the plant, when the opening takes three spots.
        if role == Role::Commander && self.frame < OPENING_PLANT && self.opening_three() && let Some(f) = commit {
            let solar: Option<(usize, f64)> = {
                let units = &self.game().units;
                let factories_standing = self.own.values().any(|u| self.unit_index(u.def).is_some_and(|i| units.list[i].role == Role::Factory));
                let draw = units.list[f].energy_cost / (units.list[f].build_time / units.list[unit].worker_time.max(1.0));
                // The supply counted from the buildings, not the engine's income, which lags a finished solar by
                // seconds and bought a fourth where three cover the plant (terr86, the plant twelve seconds later).
                let standing: f64 = self.own.values().filter(|u| !u.being_built).filter_map(|u| self.unit_index(u.def)).filter(|i| self.is_energy(*i)).map(|i| units.list[i].energy_make).sum();
                let pending: f64 = self.hands.orders.values().filter_map(|o| match &o.what { What::Build { unit, .. } if self.is_energy(*unit) => Some(units.list[*unit].energy_make), _ => None }).sum();
                let supply = units.list[unit].energy_make + standing + pending;
                if !factories_standing && supply < draw { self.opening_energy(unit).map(|u| (u, units.list[u].build_time)) } else { None }
            };
            if let Some((solar, cost)) = solar && self.order_one(id, unit, role, solar, cost, None, "opening", commands) {
                return;
            }
        }
        // The classes: the factories as a whole for a mobile builder; a factory's products by role.
        let class_pick: Option<usize> = commit.or_else(|| {
            let units = &self.game().units;
            let def = &units.list[unit];
            let mine = |u: &usize| def.builds.contains(u) && !self.hands.struck.contains(u);
            // A product's class is its role and its tier (an advanced constructor builds an upgrade): a lab
            // substitutes its kind for the program's basic wants only, never for the advanced plant's (terr20:
            // substituting for the program's fractional advanced vehicle constructors, which nothing stood to
            // make, the lab ran to 136 constructors once the caps no longer stopped it).
            let advanced = |u: usize| units.list[u].builds.iter().any(|b| self.is_upgrade(*b));
            let mut classes: Vec<(Role, bool)> = Vec::new();
            if role == Role::Factory {
                for (u, _) in shortfalls.iter().filter(|(u, _)| mine(u)) {
                    let class = (units.list[*u].role, advanced(*u));
                    if !classes.contains(&class) {
                        classes.push(class);
                    }
                }
            } else {
                classes.push((Role::Factory, false));
            }
            // The type started is the one of the class the program wants most. A builder that cannot build it leaves
            // the class to one that can; a factory that does not make it substitutes its own best of the class only
            // while no factory standing, framed or ordered makes it (terr15-lp-nullai-comet-1: the lab filled the
            // forty-constructor cap with its own kind while the program wanted the advanced plant's, which then made
            // seven and stopped; terr19: substituting through the plant's frame it stood 51 against the tier's cap
            // of forty, and the program had no feasible point from then on).
            let ordered = self.hands.orders.values().filter_map(|o| match &o.what { What::Build { unit, frame: None, .. } => Some(*unit), _ => None });
            let covered: Vec<usize> = self.own.values().filter_map(|f| self.unit_index(f.def)).chain(ordered).filter(|f| units.list[*f].role == Role::Factory).flat_map(|f| units.list[f].builds.iter().copied()).collect();
            let made_by_a_standing_factory = |u: usize| covered.contains(&u);
            classes
                .into_iter()
                .filter_map(|(class, tier)| {
                    let in_class = |u: usize| units.list[u].role == class && (class != Role::Builder || advanced(u) == tier);
                    let of_class = |u: &usize| in_class(*u) && !self.hands.struck.contains(u);
                    let total: f64 = shortfalls.iter().filter(|(u, _)| in_class(*u)).map(|(_, s)| *s).sum();
                    if total < START_AT {
                        return None;
                    }
                    let wanted = shortfalls.iter().filter(|(u, _)| of_class(u)).max_by(|a, b| a.1.total_cmp(&b.1)).map(|(u, _)| *u)?;
                    let pick = if mine(&wanted) {
                        wanted
                    } else if role == Role::Factory && !made_by_a_standing_factory(wanted) {
                        shortfalls.iter().filter(|(u, _)| of_class(u) && mine(u)).max_by(|a, b| a.1.total_cmp(&b.1)).map(|(u, _)| *u)?
                    } else {
                        return None;
                    };
                    Some((pick, total))
                })
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(u, _)| u)
        });
        // A factory is placed by a builder at the base: a chaining constructor 2,000 elmos out took the tier-2 plant's
        // commit and walked 88 s to place it (terr49 game 00, the plant begun 7:48, its first constructor at 9:00;
        // game 03's stood from 4:34 and made 800 at 20:00 against 566), where the commander or a base builder is
        // idle within seconds.
        let factory_pick = class_pick.is_some_and(|u| self.game().units.list[u].role == Role::Factory);
        let at_base = role == Role::Commander || builder.pos.dist2d(self.home) <= FACTORY_PLACER;
        if let Some(u) = class_pick && (at_base || !factory_pick) && self.order_one(id, unit, role, u, self.game().units.list[u].build_time, None, if commit.is_some() { "commit" } else { "class" }, commands) {
            return;
        }
        if role == Role::Factory {
            return;
        }
        // A factory's frame first (FACTORY_HELPERS), on the producers' budget; builders at the base only (terr70: one
        // to three constructors 2,000 out walked 45-100 s to the plant's frame and back out to the spots), but an
        // advanced plant's frame calls them from `plant_helpers_from` (his nine constructors came home for the plant
        // at 5:35 and built it in 43 s; alone its placer is 300 s).
        let from_home = builder.pos.dist2d(self.home);
        let factory_frame: Option<UnitId> = {
            let units = &self.game().units;
            let advanced = |i: usize| units.list[i].builds.iter().any(|c| units.list[*c].builds.iter().any(|u| self.is_upgrade(*u)));
            self.own
                .values()
                .filter(|f| f.being_built)
                .filter_map(|f| {
                    let i = self.unit_index(f.def)?;
                    let near_enough = from_home <= FACTORY_PLACER || (advanced(i) && from_home <= self.options.plant_helpers_from);
                    (units.list[i].role == Role::Factory && near_enough && self.helpers_on(f.id) < FACTORY_HELPERS && self.energy_fits(i, unit)).then_some((f.id, f.pos.dist2d(builder.pos)))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(id, _)| id)
        };
        if let Some(frame) = factory_frame {
            commands.push(Command::Repair { unit: id, target: frame, queue: false });
            self.hands.orders.insert(id, Order { what: What::Help { frame }, given: self.frame, told: self.frame, arrived: None, step: None });
            self.note_order(id, &What::Help { frame }, "plant-frame");
            return;
        }
        // The credits by unit type over every builder type's flows (turret help apart), the largest in power-seconds
        // first, and at a credit tied the one the plan wants sooner (its credit at a third of the lead: terr24, two
        // solars and two extractors wanted by one minute tied, and unit order put the solars first); this builder's
        // part is to build it or to guard the factory that makes it. In power-seconds, not units of the type: by
        // units a cheap type was always first (terr57-58: the plan's turrets and converters, five to seventy units
        // behind, ahead of its upgrades at five, and the upgrades it wanted at 12-17 a minute came at 5-10, the base
        // constructors on turrets while the chains alone took the spots).
        // By column: a whole unit's flow is built or assisted, a ring's is an extractor on a spot of that ring, and a
        // Frames column's is help on a frame of the type (the program's finishing work, in place of the dearest
        // frame), its commitment read live from the power standing on those frames.
        let mut flows: Vec<(usize, Class, Kind, f64, f64, f64)> = {
            let targets = self.hands.targets.as_ref().expect("dispatching");
            let units = &self.game().units;
            let def = &units.list[unit];
            let mut per_column: BTreeMap<(usize, Class), (f64, f64, f64, f64, f64)> = BTreeMap::new();
            // A reclaim flow (the program taking a building down for its metal) is no build; the hands give no reclaim
            // order yet, and the bot's arm leaves the program without the column until they do. The turrets' help
            // counts toward a whole unit's planned power (T-FARM): with the turrets doing forty times the placer's
            // work on a farm frame, a whole unit that waited for half its cost in the placers' own flows would wait
            // minutes. Not toward a Frames column's: there the turrets' planned help, less the turrets that reach
            // the frame, read as power for constructors to make up, and 14-27 of them walked to advanced fusion
            // frames at 16-17 min that twenty turrets stood on (terr59-60); the turrets go to every frame in their
            // reach on their own (run_nanos), and the constructors' part is the plan's own mobile power on the type.
            // Nor the turrets' assist on a factory (T-PLANT): in a product's credit it would send constructors to guard
            // the plant in the turrets' place; the plant's turrets are placed from those flows (farm_site).
            for f in targets.flows.iter().filter(|f| f.kind != Kind::Reclaim && units.list[f.unit].role != Role::Factory && !(units.list[f.builder].role == Role::Nano && (f.class == Class::Frames || f.kind == Kind::Assist))) {
                let e = per_column.entry((f.unit, f.class)).or_insert((0.0, 0.0, f64::MAX, 0.0, 0.0));
                e.0 += targets.planned_power_class(f.builder, f.unit, f.class, ahead);
                e.1 += targets.planned_power_class(f.builder, f.unit, f.class, now + LEAD / 3.0);
                e.2 = e.2.min(f.cost);
                // This type's own help on a type it cannot start (the basic constructor on the moho frames,
                // LpOptions::mobile_help): its planned power alone, since the column's total is the starters' and
                // the helper cannot deliver it, against this type's own commitment on the frames. Any type's help
                // flow on the column would turn every type's direct flow on it into help (terr98: the advanced
                // constructor's and the turrets' help flows made the constructors' extractor flows help, and the
                // extractors fell to the count rule, 5:00 at 29-32 against 48-55).
                if f.kind == Kind::Help && f.builder == unit {
                    e.3 += targets.planned_power_class(unit, f.unit, f.class, ahead);
                    e.4 += targets.planned_power_class(unit, f.unit, f.class, now + LEAD / 3.0);
                }
            }
            per_column
                .into_iter()
                .filter_map(|((u, class), (planned, soon, cost, own_help, own_help_soon))| {
                    let help = own_help > 0.0;
                    let (planned, soon) = if help { (own_help, own_help_soon) } else { (planned, soon) };
                    let kind = if (class == Class::Frames || help) && units.list[u].speed == 0.0 {
                        Kind::Help
                    } else if def.builds.contains(&u) {
                        Kind::Direct
                    } else if units.list[u].speed > 0.0 && self.factory_making(u, builder.pos).is_some() {
                        Kind::Assist
                    } else {
                        return None;
                    };
                    let committed = if help {
                        self.frames_power(u, targets.t0, Some(unit))
                    } else if class == Class::Frames {
                        self.frames_power(u, targets.t0, None)
                    } else {
                        self.hands.committed.get(&(u, class)).copied().unwrap_or(0.0)
                    };
                    let credit = planned - committed;
                    // A whole unit starts at half its cost of credit; help on frames at any. With the frames in their
                    // own column the standing orders no longer seed the whole-unit credits, and on any credit the
                    // hands started an advanced fusion at every re-plan (terr31: four in minute thirteen, 39,000
                    // metal, the upgrades refused by the consumers' budget for three minutes, the metal at zero).
                    let enough = if class == Class::Frames || help { credit > 0.0 } else { credit >= START_AT * cost };
                    enough.then_some((u, class, kind, credit, cost, soon - committed))
                })
                .collect()
        };
        flows.sort_by(|a, b| b.3.total_cmp(&a.3).then(b.5.total_cmp(&a.5)));
        // T-CHAIN: a constructor out among the spots (past BESIDE from home) chains them: while an extractor or
        // upgrade flow of its own has credit it takes no other order, so that a solar at the base does not walk it
        // home between two spots. Read from thebluegecko's record against the bot's (2026-10-24): his constructors
        // began an extractor every 31 s with a step of 528 elmos, six of his nine doing nothing else; the bot's every
        // 62 s with a step of 1,110, solars, advanced solars and turrets between (terr45 game 00); and the program's
        // own 5:00 is 62, his figure, at a walk of 36 s an extractor against 50 at the hands' realized 65.
        let far = self.options.chain && role == Role::Builder && builder.pos.dist2d(self.home) > BESIDE;
        // The commander out at a spot chains like a constructor: past BESIDE from home with a free spot within its
        // leash nearer than home, it takes that before any guard or frame at the base (terr90, the leash at 1,300:
        // the program's spots for it came one at a time and the guard's credit walked it 568-953 home between them,
        // in game 00 out and back twice; his commander's two at 1,007 and 1,215 lie 210 apart and were one trip,
        // 1:40-2:57). Inert at the default leash of 700, where no spot past the opening's three is in reach: with
        // the leash at 1,300 the trip was one spot (terr91, level: the second constructor takes the pair's nearer
        // spot at 2:06, before the commander sets out) or, sent ahead of the guard and the solars, the pair and then
        // a third spot 1,850 away with the plant unguarded to 3:00 (terr92, 5:00 down 7-18 in all four games).
        let com_far = role == Role::Commander
            && builder.pos.dist2d(self.home) > BESIDE
            && self.free_spot(builder.pos, None, Some((self.home, self.options.com_leash)), None).is_some_and(|i| self.hello.metal_spots[i].dist2d(builder.pos) < builder.pos.dist2d(self.home));
        let far = far || com_far;
        // A constructor with a spot's credit, at the base too: a new constructor's first order was the stall's
        // solar beside the plant (terr75-81, 1:43-1:49, the first extractor at 2:33 against his first constructor's
        // at 1:56), where the commander's solars stand a minute later anyway.
        let spots_credit = self.options.chain && role == Role::Builder && flows.iter().any(|f| matches!(f.1, Class::Ring(_)) && f.2 == Kind::Direct);
        let chain = (far || self.frame < OPENING_CHAIN) && spots_credit;
        if chain {
            flows.retain(|f| matches!(f.1, Class::Ring(_)));
        } else if far {
            // The far end of the chain: with no extractor flow in credit at this instant, a constructor out among the
            // spots keeps the rings' flows and the cheap buildings it places beside itself (a solar for the energy the
            // plan is short of) and takes no turret, guard or frame at the base; after the flows, the nearest free
            // spot or upgrade nearer than home (chain_on). Not ahead of the stall's solar and the flows (terr71: the
            // extractors ran to 26-35 by 7:00 against 21-24 and the energy sat at zero from 5:00 to 7:00 in three
            // games, 5:00 down to 26-32 in two).
            let units = &self.game().units;
            flows.retain(|f| matches!(f.1, Class::Ring(_)) || (f.2 == Kind::Direct && units.list[f.0].speed == 0.0 && units.list[f.0].role != Role::Nano && units.list[f.0].build_time < FARM_BIG));
        }
        // The energy as a class ahead of the flows only in a stall (ENERGY_STALL), else after them; a chaining
        // constructor leaves the stall's solar to the builders at the base.
        if !spots_credit && self.in_stall() && self.energy_class(id, unit, role, ahead, "stall", commands) {
            return;
        }
        for (u, class, kind, _, cost, _) in flows {
            match kind {
                Kind::Direct => {
                    let ring = match class {
                        Class::Ring(k) => Some(k),
                        _ => None,
                    };
                    if self.order_one(id, unit, role, u, cost, ring, "flows", commands) {
                        return;
                    }
                }
                Kind::Help => {
                    if let Some(frame) = self.frame_to_finish(u, unit, builder.pos) {
                        commands.push(Command::Repair { unit: id, target: frame, queue: false });
                        self.hands.orders.insert(id, Order { what: What::Help { frame }, given: self.frame, told: self.frame, arrived: None, step: None });
                        self.note_order(id, &What::Help { frame }, "flows");
                        return;
                    }
                }
                // Never made above (the program's reclaim flows are dropped at the gate); the hands' reclaim is to come.
                Kind::Reclaim => continue,
                Kind::Assist => {
                    let Some(factory) = self.factory_making(u, builder.pos) else { continue };
                    // Its maker's own product never guards it (a newborn guarding the factory it came from stands
                    // on the pad whatever it is told first), and a factory takes ASSIST_GUARDS guards at most.
                    let made_here = self.own.get(&factory).and_then(|f| self.unit_index(f.def)).is_some_and(|fi| self.game().units.list[fi].builds.contains(&unit));
                    let guards = self.hands.orders.values().filter(|o| matches!(&o.what, What::Assist { factory: f, .. } if *f == factory)).count();
                    if made_here || guards >= ASSIST_GUARDS {
                        continue;
                    }
                    // Only while the stock and the chunk's income cover two of the product, the factory's own and
                    // the guard's: a guard on a factory the metal cannot feed only spreads the stall to every frame
                    // (terr76 game 00: the commander's two chunks on the vehicle plant at 2:06 and 2:26 pumped a
                    // constructor every 16 s on an income of 9, the stock at zero from 2:00 to 4:00, the
                    // constructors' extractor frames 40 s each and 3 extractors at 3:00 against 6), and against the
                    // stock alone the commander never guarded (terr81: the stock under 135 at every idle moment,
                    // eight constructors by 4:53 against terr76's nine by 4:51 and 5:00 down 2-7 in three games).
                    let budget = self.metal.current as f64 + self.metal.income as f64 * f64::from(ASSIST_CHUNK / SECOND);
                    if budget < 2.0 * self.game().units.list[u].metal_cost {
                        continue;
                    }
                    // And the energy stock half the product's: a guarded plant draws a constructor's 1,950 in eleven
                    // seconds (terr86 game 02: the stock at 361 at 2:00, the first constructor sent to a solar; his
                    // guard ran at 1:21-1:35 on a full stock and again from 2:49).
                    if (self.energy.current as f64) < self.game().units.list[u].energy_cost / 2.0 {
                        continue;
                    }
                    if !self.energy_fits(u, unit) {
                        self.hands.energy_refused += 1;
                        *self.hands.energy_refused_by.entry(u).or_default() += 1;
                        continue;
                    }
                    // Its own product inside its yard is on or beside the pad (the advanced vehicle plant's pad lies
                    // past 100 from its centre: terr32 game 01, three orders held two minutes each), and any guard on
                    // the exit lane stands where the products must leave (terr43 game 01: the plant's first product
                    // stood finished on the pad from 7:06 to 11:42 behind the lab's constructors guarding from the
                    // exit, the plant's orders dropped as held twice); the rest guard from where they stand.
                    let own_product = |f: &OwnUnit| self.unit_index(f.def).is_some_and(|fi| self.game().units.list[fi].builds.contains(&unit));
                    if let Some(f) = self.own.get(&factory) && ((own_product(f) && builder.pos.dist2d(f.pos) < FACTORY_YARD) || exit_lane(f).contains(builder.pos)) {
                        let to = self.beside_factory(f, id);
                        commands.push(Command::Move { unit: id, to, queue: false });
                        self.hands.orders.insert(id, Order { what: What::Assist { factory, until: self.frame + PAD_CLEAR }, given: self.frame, told: self.frame, arrived: None, step: None });
                        self.note_order(id, &What::Assist { factory, until: self.frame + PAD_CLEAR }, "pad");
                        return;
                    }
                    commands.push(Command::Guard { unit: id, target: factory });
                    self.hands.orders.insert(id, Order { what: What::Assist { factory, until: self.frame + ASSIST_CHUNK }, given: self.frame, told: self.frame, arrived: None, step: None });
                    self.note_order(id, &What::Assist { factory, until: self.frame + ASSIST_CHUNK }, "flows");
                    if class != Class::Frames {
                        *self.hands.committed.entry((u, class)).or_default() += self.game().units.list[unit].worker_time * ASSIST_CHUNK as f64 / SECOND as f64;
                    }
                    return;
                }
            }
        }
        // The far end of the chain: the nearest free spot of any ring, or an upgrade, nearer than home (chain_on).
        if far && self.chain_on(id, unit, role, &builder, commands) {
            return;
        }
        if self.energy_class(id, unit, role, ahead, "energy", commands) {
            return;
        }
        // No credit: a constructor's fallbacks reach as far as it walks in LEAD seconds (WALK_STEP): the largest count
        // shortfall of a type it builds, in build-power-seconds, with the farm in reach, since the count rule's sites
        // are the farm's; then the dearest frame in reach, with the farm in reach; with nothing, the walk to the farm.
        // The commander keeps the fallbacks ungated (it is never far long).
        let reach = (self.game().units.list[unit].speed * LEAD) as f32;
        let farm = self.farm_origin();
        let farm_in_reach = role != Role::Builder || builder.pos.dist2d(farm) <= reach;
        let fallback: Option<usize> = {
            let units = &self.game().units;
            let def = &units.list[unit];
            shortfalls
                .iter()
                .filter(|(u, short)| *short > 0.0 && units.list[*u].role != Role::Factory && def.builds.contains(u) && !self.hands.struck.contains(u))
                .max_by(|a, b| (a.1 * units.list[a.0].build_time).total_cmp(&(b.1 * units.list[b.0].build_time)))
                .map(|(u, _)| *u)
        };
        if farm_in_reach && let Some(u) = fallback && self.order_one(id, unit, role, u, self.game().units.list[u].build_time, None, "count", commands) {
            return;
        }
        // T-ENERGY: in a stall the help goes to the energy frame that finishes soonest with this builder on it, else
        // to the dearest frame in reach the energy budget allows a helper on; with none, and the budget refusing, a
        // solar.
        let frame = if self.in_stall() { self.energy_frame_to_finish(unit) } else if farm_in_reach { self.dearest_frame(unit, builder.pos, reach) } else { None };
        if let Some(frame) = frame {
            commands.push(Command::Repair { unit: id, target: frame, queue: false });
            self.hands.orders.insert(id, Order { what: What::Help { frame }, given: self.frame, told: self.frame, arrived: None, step: None });
            self.note_order(id, &What::Help { frame }, if self.in_stall() { "stall-frame" } else { "dearest" });
            return;
        }
        if (self.energy.current as f64) < ENERGY_LOW * self.energy.storage as f64 && self.energy_budget(true) < 0.0 && self.free_energy_building(id, unit, role, commands) {
            return;
        }
        // Nothing to do out here: a constructor walks to the farm's core, where every frame of the farm is in its reach
        // (terr101: the walk ended at the reach's rim and far constructors helped the metal makers beside them out on
        // the map, 226 orders of seconds each in game 00, where before they walked to the advanced fusions).
        if role == Role::Builder && builder.pos.dist2d(farm) > FARM_CORE {
            commands.push(Command::Move { unit: id, to: farm, queue: false });
            self.hands.orders.insert(id, Order { what: What::Walk { to: farm }, given: self.frame, told: self.frame, arrived: None, step: None });
            self.note_order(id, &What::Walk { to: farm }, "home");
            *self.hands.ordered.entry((unit, "home")).or_default() += 1;
            return;
        }
        // Nothing to do: a builder standing in a factory's yard or exit lane steps beside it (the pad-keeper), so
        // that a newborn with no order of its own does not hold the next product on the pad.
        if role != Role::Factory {
            self.clear_the_yard(id, unit, &builder, commands);
        }
    }

    /// A mobile unit of ours in a standing factory's yard or exit lane is moved beside the factory for PAD_CLEAR.
    fn clear_the_yard(&mut self, id: UnitId, unit: usize, builder: &OwnUnit, commands: &mut Vec<Command>) {
        let blocking: Option<OwnUnit> = self.own.values().filter(|f| !f.being_built && f.id != id && self.role_of_def(f.def) == Some(Role::Factory)).find(|f| builder.pos.dist2d(f.pos) < FACTORY_YARD || exit_lane(f).contains(builder.pos)).cloned();
        if let Some(f) = blocking {
            let to = self.beside_factory(&f, id);
            commands.push(Command::Move { unit: id, to, queue: false });
            self.hands.orders.insert(id, Order { what: What::Assist { factory: f.id, until: self.frame + PAD_CLEAR }, given: self.frame, told: self.frame, arrived: None, step: None });
            self.note_order(id, &What::Assist { factory: f.id, until: self.frame + PAD_CLEAR }, "yard");
            *self.hands.ordered.entry((unit, "yard")).or_default() += 1;
        }
    }

    /// T-ENERGY: the energy the hands may commit over the affordability window: the stock above the reserve, the
    /// window's net income with the frames that finish inside it credited back, less what the orders still walking
    /// draw inside it (a unit's draw is what its frame takes over the window at its builder's power, or its whole
    /// cost if less). A unit or a helper is refused when its draw does not fit. The program's plan is feasible in
    /// fractions and spends energy to the edge; the hands' whole units overshot it into a stall that lasted minutes
    /// (terr25-lp-nullai-comet-1, game 00: two fusions, a cloakable one and an advanced one framed together from
    /// 10:08 to 11:59 beside nine upgrades, the energy at zero from 12:00 to 15:00, the income growth a third of
    /// the plan's). A producer is held to the stock and to the net only when that is positive, and against the
    /// pending producers alone: a fusion is what raises the net, and the window's balance must not refuse it
    /// (terr26: the first budget counted every walking order's whole draw inside the window and refused fusions
    /// and upgrades alike at a stock on its cap; the hands built twenty solars into that cap and banked the metal).
    fn energy_budget(&self, producer: bool) -> f64 {
        let units = &self.game().units;
        let e = &self.energy;
        let w = AFFORD_SECONDS;
        // In a stall there is one budget: a producer's net counts only its own kind's draw, and with the stock at
        // zero the consumers take the whole income, so ninety seconds of it read as free (terr39 game 03, 8:00:
        // the stock at zero, the usage the income, a producers' budget of 28,890, and the stall class itself put
        // six advanced solars of 4,000 energy on it; the stall ran from 8:00 to 10:00).
        let producer = producer && !self.in_stall();
        // The frames in the budget's own class: every frame for the consumers, the producers' and factories' own
        // for them. Their draw now, and what of it is credited back because the frame finishes inside the window.
        // Nothing is credited in a stall: there the income is the whole draw whatever the frames ask, a frame that
        // finishes frees nothing (the others speed up to take it), and a new frame only slows them all. Credited
        // at full speed, fifteen advanced solar frames read as 33,000 of budget with the stock at one and every
        // advanced solar fitted (terr37-lp-nullai-comet-1 game 00, 7:00-10:00); credited at the stall share of
        // their draw, twenty-two frames still read as 16,000 and eleven more were started (terr40 game 00, 8:00).
        // Not credited either: the production of energy frames about to stand. The budget reads the minute's income
        // and not the fusions a minute from standing, and once refused 370 orders in a minute as two stood (terr41
        // game 01, 12:00-13:00); credited at the frames' full-speed pace it let the stalls back in (terr43, terr44:
        // the metal at zero in those minutes slows the frames too). The frames' realized pace is the honest rate,
        // and the next step.
        let frames = self.frames_draw();
        let stalled = self.in_stall();
        let (drawing, finishing) = frames
            .iter()
            .filter(|f| !producer || f.own)
            .fold((0.0, 0.0), |(d, fin), f| (d + f.rate, fin + if stalled { 0.0 } else { (f.rate * w - f.left).max(0.0) }));
        let pending: f64 = self
            .hands
            .orders
            .iter()
            .filter_map(|(id, o)| match &o.what {
                What::Build { unit, frame: None, near, .. } if !producer || self.is_energy(*unit) => {
                    let builder = self.own.get(id)?;
                    let bi = self.unit_index(builder.def)?;
                    let def = &units.list[*unit];
                    let inside = (w - builder.pos.dist2d(*near) as f64 / units.list[bi].speed.max(1.0)).max(0.0);
                    Some(def.energy_cost.min(inside * units.list[bi].worker_time * def.energy_cost / def.build_time.max(1.0)))
                }
                _ => None,
            })
            .sum();
        // A producer is held to its own kind's draw, not the consumers' (terr27), but to that: five advanced fusions
        // framed together on a producers' budget that read no draw at all (terr31).
        let net = if producer { e.income as f64 - drawing } else { e.income as f64 - e.usage as f64 };
        (e.current as f64 - self.options.energy_reserve * e.storage as f64) + w * net + finishing - pending
    }

    /// Every frame under way: whether it is a producer's or a factory's, its energy draw a second at full speed
    /// with the power standing on it, and the energy left to pay on it.
    fn frames_draw(&self) -> Vec<FrameDraw> {
        let units = &self.game().units;
        self.own
            .values()
            .filter(|u| u.being_built)
            .filter_map(|u| {
                let i = self.unit_index(u.def)?;
                let def = &units.list[i];
                let rate = self.power_on(u.id) * def.energy_cost / def.build_time.max(1.0);
                let left = def.energy_cost * (1.0 - (u.health / u.max_health.max(1.0)).clamp(0.0, 1.0) as f64);
                Some(FrameDraw { own: self.is_energy(i) || def.role == Role::Factory, rate, left })
            })
            .collect()
    }

    /// The share of their draw the frames run at: one while the stock is above the stall line, else what the
    /// engine's usage is of the draw they ask for (the engine shares a stall over every build alike).
    fn stall_share(&self, frames: &[FrameDraw]) -> f64 {
        let asked: f64 = frames.iter().map(|f| f.rate).sum();
        if !self.in_stall() || asked <= 0.0 { 1.0 } else { (self.energy.usage as f64 / asked).min(1.0) }
    }

    /// The budget line for the log: the consumers' and the producers' budgets, the stall share, and the refusals
    /// by type since the last line.
    pub(crate) fn energy_line(&mut self) -> String {
        let units = &self.game().units;
        let refused: Vec<String> = self.hands.energy_refused_by.iter().map(|(u, n)| format!("{}:{n}", units.list[*u].name)).collect();
        self.hands.energy_refused_by.clear();
        let share = self.stall_share(&self.frames_draw());
        format!("budget {:.0}/{:.0} share {share:.2} refused {}", self.energy_budget(false), self.energy_budget(true), if refused.is_empty() { "-".to_string() } else { refused.join(" ") })
    }

    /// The build orders given since the last log line, by type, most first, with the rule that gave each (commit,
    /// class, stall, flows, energy, count, free: T-DISPATCH's order).
    pub(crate) fn ordered_line(&mut self) -> String {
        let units = &self.game().units;
        let mut by_type: BTreeMap<usize, (usize, Vec<String>)> = BTreeMap::new();
        for ((u, why), n) in &self.hands.ordered {
            let e = by_type.entry(*u).or_default();
            e.0 += n;
            e.1.push(format!("{why}:{n}"));
        }
        let mut ordered: Vec<(usize, usize, Vec<String>)> = by_type.into_iter().map(|(u, (n, ways))| (u, n, ways)).collect();
        ordered.sort_by(|a, b| b.1.cmp(&a.1));
        let line = if ordered.is_empty() { "-".to_string() } else { ordered.iter().map(|(u, n, ways)| format!("{}:{n}({})", units.list[*u].name, ways.join(" "))).collect::<Vec<_>>().join(" ") };
        self.hands.ordered.clear();
        line
    }

    /// The energy a frame of `u` draws over the window with `builder`'s power on it, or its whole cost if less.
    fn energy_draw(&self, u: usize, builder: usize) -> f64 {
        let units = &self.game().units;
        let def = &units.list[u];
        def.energy_cost.min(AFFORD_SECONDS * units.list[builder].worker_time * def.energy_cost / def.build_time.max(1.0))
    }

    /// Whether `builder` may start or help a `u` for the energy: a cheap one always (ENERGY_FREE), else by the
    /// budget, a factory on the producers' (the program takes a committed factory as standing, T-COMMIT, and its
    /// frame waited on the consumers' budget under one builder from 5:18 to 10:00 while ten built advanced solars
    /// beside it, terr28-lp-nullai-comet-1 game 03; exempt from both it drained the energy to zero, terr29).
    fn energy_fits(&self, u: usize, builder: usize) -> bool {
        let def = &self.game().units.list[u];
        def.energy_cost <= ENERGY_FREE || self.energy_draw(u, builder) <= self.energy_budget(self.is_energy(u) || def.role == Role::Factory)
    }

    /// A point beside a factory, out of its yard and off its exit lane: across the lane at NANO_BEHIND, the side
    /// by the builder's id. Across the lane the factory faces, not the line to his start: since T-SITES turns a
    /// plant whose lane toward him is blocked, a plant facing north with him to the east sent the commander to a
    /// point inside its own lane, and it stepped aside there every ten seconds from 1:51 past 3:21 without once
    /// guarding (terr108 game 00, constructors 28 s apart against 13-29).
    fn beside_factory(&self, factory: &OwnUnit, builder: UnitId) -> Vec3 {
        let (dx, dz) = FACING_DIR[factory.facing.rem_euclid(4) as usize];
        let side = if builder.0 % 2 == 0 { 1.0 } else { -1.0 };
        Vec3 { x: factory.pos.x - dz * NANO_BEHIND * side, y: factory.pos.y, z: factory.pos.z + dx * NANO_BEHIND * side }
    }

    /// The energy stock under the stall line (ENERGY_STALL of the storage).
    fn in_stall(&self) -> bool {
        (self.energy.current as f64) < ENERGY_STALL * self.energy.storage as f64
    }

    /// The energy frame that finishes soonest with `builder`'s power added to what is on it.
    fn energy_frame_to_finish(&self, builder: usize) -> Option<UnitId> {
        let units = &self.game().units;
        let mine = units.list[builder].worker_time;
        self.own
            .values()
            .filter(|u| u.being_built && (builder != self.game().commander || self.commander_ground(u.pos)))
            .filter_map(|u| {
                let i = self.unit_index(u.def)?;
                self.is_energy(i).then_some(())?;
                let left = units.list[i].build_time * (1.0 - (u.health / u.max_health.max(1.0)).clamp(0.0, 1.0) as f64);
                Some((u.id, left / (self.power_on(u.id) + mine)))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id)
    }

    /// The ring of the program's a point falls in (the nearest, with no program).
    fn ring_of_pos(&self, at: Vec3) -> usize {
        self.hands.targets.as_ref().map_or(0, |t| t.ring_of(at.dist2d(self.home) as f64))
    }

    /// Build-power-seconds standing on the frames of `u` since the plan's start `t0`, the lead ahead counted: the
    /// orders building them, the help on them, and the guards on a factory making one, each at its builder's
    /// power from when it arrived. The live commitment of a Frames column; with `by`, the builders of that type
    /// only (the helpers' own commitment, for a type that cannot start `u`).
    pub(crate) fn frames_power(&self, u: usize, t0: f64, by: Option<usize>) -> f64 {
        let units = &self.game().units;
        let now = self.frame as f64 / SECOND as f64;
        let frame_unit = |frame: &UnitId| self.own.get(frame).and_then(|f| self.unit_index(f.def));
        let factory_making_u = |factory: &UnitId| matches!(self.hands.orders.get(factory).map(|o| &o.what), Some(What::Build { unit, .. }) if *unit == u);
        self.hands
            .orders
            .iter()
            .filter(|(_, o)| match &o.what {
                What::Build { unit, .. } => *unit == u,
                What::Help { frame } => frame_unit(frame) == Some(u),
                What::Assist { factory, .. } => factory_making_u(factory),
                What::Walk { .. } => false,
            })
            .filter_map(|(id, o)| {
                let kind = self.unit_index(self.own.get(id)?.def)?;
                if by.is_some_and(|b| b != kind) {
                    return None;
                }
                let w = units.list[kind].worker_time;
                let since = (o.arrived.unwrap_or(o.given) as f64 / SECOND as f64).max(t0);
                Some(w * ((now - since).max(0.0) + LEAD))
            })
            .sum::<f64>()
    }

    /// The frame of `u` that finishes soonest with `builder`'s power added to what is on it, counted from `from`:
    /// the walk first, then the work left on arrival at the power then on it. A frame that finishes before the
    /// builder gets there is passed over (terr99: chosen by the finish alone, the help on the moho frames went to the
    /// one nearest done, 2,000-3,800 elmos off; it finished within 3.5 s, the order ended with it and the next
    /// nearest done was given, 50 frames a constructor in five minutes and none reached).
    fn frame_to_finish(&self, u: usize, builder: usize, from: Vec3) -> Option<UnitId> {
        let units = &self.game().units;
        let mine = units.list[builder].worker_time;
        let speed = units.list[builder].speed.max(1.0);
        self.own
            .values()
            .filter(|f| f.being_built && self.unit_index(f.def) == Some(u) && (builder != self.game().commander || self.commander_ground(f.pos)))
            .filter_map(|f| {
                let left = units.list[u].build_time * (1.0 - (f.health / f.max_health.max(1.0)).clamp(0.0, 1.0) as f64);
                let on = self.power_on(f.id);
                let walk = from.dist2d(f.pos) as f64 / speed;
                let done_alone = if on > 0.0 { left / on } else { f64::INFINITY };
                (walk < done_alone).then(|| (f.id, walk + (left - on * walk) / (on + mine)))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id)
    }

    /// Builders on a frame: the one building it and the helpers.
    fn helpers_on(&self, frame: UnitId) -> usize {
        self.hands.orders.values().filter(|o| matches!(&o.what, What::Help { frame: f } | What::Build { frame: Some(f), .. } if *f == frame)).count()
    }

    /// Build power on a frame: its builder's and its helpers'.
    fn power_on(&self, frame: UnitId) -> f64 {
        let units = &self.game().units;
        let builders: f64 = self
            .hands
            .orders
            .iter()
            .filter(|(_, o)| matches!(&o.what, What::Help { frame: f } | What::Build { frame: Some(f), .. } if *f == frame))
            .filter_map(|(id, _)| self.unit_index(self.own.get(id)?.def))
            .map(|b| units.list[b].worker_time)
            .sum();
        let turrets: f64 = self.hands.nano_frame.iter().filter(|(_, (f, _))| *f == frame).filter_map(|(nano, _)| self.unit_index(self.own.get(nano)?.def)).map(|b| units.list[b].worker_time).sum();
        builders + turrets
    }

    /// A cheap energy building of the program's types this builder builds, the most energy for the metal: what a
    /// builder the energy budget refused everything builds.
    fn free_energy_building(&mut self, id: UnitId, unit: usize, role: Role, commands: &mut Vec<Command>) -> bool {
        let pick: Option<usize> = {
            let units = &self.game().units;
            let def = &units.list[unit];
            let targets = self.hands.targets.as_ref().expect("dispatching");
            targets
                .types
                .iter()
                .copied()
                .filter(|u| def.builds.contains(u) && self.is_energy(*u) && units.list[*u].energy_cost <= ENERGY_FREE && !self.hands.struck.contains(u) && self.affordable(*u))
                .max_by(|a, b| (units.list[*a].energy_make / units.list[*a].metal_cost.max(1.0)).total_cmp(&(units.list[*b].energy_make / units.list[*b].metal_cost.max(1.0))))
        };
        match pick {
            Some(u) if role != Role::Factory => self.order_one(id, unit, role, u, self.game().units.list[u].build_time, None, "free", commands),
            _ => false,
        }
    }

    /// The energy as a class: the program's planned energy income at the lead, less what stands, is framed or is
    /// ordered, realized by the best affordable energy building of the program's types this builder builds, while
    /// the energy stock is under half its storage and fewer than three energy orders stand without a frame. The
    /// program's cheapest energy was a fusion it built as a fraction from its second minute and drew energy from at
    /// once, while the engine's first energy is solars (terr17: zero energy from 1:00 to 5:00); since terr22 the
    /// program is offered affordable types only and its energy flows are followed like any other, and this is the
    /// backstop (terr19: read by every idle builder it put the first constructors on four advanced solars by 4:31;
    /// terr20: two orders by constructors alone left the energy at zero to 8:00). A whole unit's shortfall starts
    /// one (terr24: at half a unit, three solars where the plan wanted two).
    fn energy_class(&mut self, id: UnitId, unit: usize, role: Role, ahead: f64, why: &'static str, commands: &mut Vec<Command>) -> bool {
        let energy: Option<(usize, f64)> = {
            let units = &self.game().units;
            let def = &units.list[unit];
            let targets = self.hands.targets.as_ref().expect("dispatching");
            let rate = |u: usize| units.list[u].energy_make;
            let pending = self.hands.orders.values().filter(|o| matches!(&o.what, What::Build { unit, frame: None, .. } if self.is_energy(*unit))).count();
            let had: f64 = self.own.values().filter_map(|o| self.unit_index(o.def)).filter(|u| self.is_energy(*u)).map(rate).sum::<f64>()
                + self.hands.orders.values().filter_map(|o| match &o.what { What::Build { unit, frame: None, .. } if self.is_energy(*unit) => Some(rate(*unit)), _ => None }).sum::<f64>();
            let short = if pending < ENERGY_PENDING && (self.energy.current as f64) < ENERGY_LOW * self.energy.storage as f64 { targets.energy_income_at(ahead) - units.list[self.game().commander].energy_make - had } else { 0.0 };
            // Of the program's own types (the roster leaves out what stands only on a geothermal vent).
            targets
                .types
                .iter()
                .copied()
                .filter(|u| def.builds.contains(u) && self.is_energy(*u) && !self.hands.struck.contains(u) && self.affordable(*u) && self.energy_fits(*u, unit))
                .max_by(|a, b| (rate(*a) / units.list[*a].metal_cost.max(1.0)).total_cmp(&(rate(*b) / units.list[*b].metal_cost.max(1.0))))
                .filter(|u| short >= rate(*u))
                .map(|u| (u, units.list[u].build_time))
        };
        match energy {
            Some((u, cost)) if role != Role::Factory => self.order_one(id, unit, role, u, cost, None, why, commands),
            _ => false,
        }
    }

    /// One unit of `u` ordered from this builder (T-SITES places it) and charged to its type's commitment; false
    /// when the hands pass it over (not affordable, no site, the cap, a struck type).
    fn order_one(&mut self, id: UnitId, unit: usize, role: Role, u: usize, cost: f64, ring: Option<usize>, why: &'static str, commands: &mut Vec<Command>) -> bool {
        if !self.affordable(u) || self.hands.type_refused.get(&u).is_some_and(|until| self.frame < *until) {
            return false;
        }
        if !self.energy_fits(u, unit) {
            self.hands.energy_refused += 1;
            *self.hands.energy_refused_by.entry(u).or_default() += 1;
            return false;
        }
        let attempt = self.hands.attempts.get(&id).copied().unwrap_or(0);
        let Some((command, what)) = self.give(id, unit, role, Step::build(u), attempt, ring) else { return false };
        commands.push(command);
        self.note_order(id, &what, why);
        self.hands.orders.insert(id, Order { what, given: self.frame, told: self.frame, arrived: None, step: None });
        *self.hands.committed.entry((u, ring.map_or(Class::Whole, Class::Ring))).or_default() += cost;
        *self.hands.ordered.entry((u, why)).or_default() += 1;
        true
    }

    /// Whether the opening's radius holds three spots or more: his opening, with the solars before the plant.
    fn opening_three(&self) -> bool {
        self.hello.metal_spots.iter().filter(|s| s.dist2d(self.home) < self.options.opening_spots).count() >= 3
    }

    /// The program's best energy building this builder builds, by energy a second per metal: the opening's solar.
    fn opening_energy(&self, unit: usize) -> Option<usize> {
        let units = &self.game().units;
        let def = &units.list[unit];
        let targets = self.hands.targets.as_ref().expect("dispatching");
        let worth = |u: usize| units.list[u].energy_make / units.list[u].metal_cost.max(1.0);
        targets
            .types
            .iter()
            .copied()
            .filter(|u| def.builds.contains(u) && self.is_energy(*u) && !self.hands.struck.contains(u) && self.affordable(*u) && self.energy_fits(*u, unit))
            .max_by(|a, b| worth(*a).total_cmp(&worth(*b)))
    }

    /// Whether the stock and `AFFORD_SECONDS` of income cover a unit's metal cost.
    fn affordable(&self, u: usize) -> bool {
        self.game().units.list[u].metal_cost <= self.metal.current as f64 + self.metal.income as f64 * AFFORD_SECONDS
    }

    /// An energy building: makes energy, extracts and converts nothing (the commander is not one).
    pub(crate) fn is_energy(&self, u: usize) -> bool {
        let unit = &self.game().units.list[u];
        unit.role == Role::Eco && unit.energy_make > 0.0 && unit.extracts_metal == 0.0 && unit.conv_capacity == 0.0
    }

    /// The standing factory nearest `from` that makes `u`.
    fn factory_making(&self, u: usize, from: Vec3) -> Option<UnitId> {
        let units = &self.game().units;
        self.own
            .values()
            .filter(|f| !f.being_built && self.unit_index(f.def).is_some_and(|fi| units.list[fi].role == Role::Factory && units.list[fi].builds.contains(&u)))
            .min_by(|a, b| a.pos.dist2d(from).total_cmp(&b.pos.dist2d(from)))
            .map(|f| f.id)
    }

    /// Per unit type in the program: its target at `t` less what stands, is framed, or is ordered.
    fn shortfalls(&self, t: f64) -> Vec<(usize, f64)> {
        let Some(targets) = self.hands.targets.as_ref() else { return Vec::new() };
        let mut have: BTreeMap<usize, f64> = BTreeMap::new();
        for u in self.own.values() {
            if let Some(i) = self.unit_index(u.def) {
                *have.entry(i).or_default() += 1.0;
            }
        }
        for order in self.hands.orders.values() {
            if let What::Build { unit, frame: None, .. } = &order.what {
                *have.entry(*unit).or_default() += 1.0;
            }
        }
        targets.types.iter().map(|u| (*u, targets.count_at(*u, t) - have.get(u).copied().unwrap_or(0.0))).collect()
    }

    fn review(&mut self, id: UnitId, order: &Order, commands: &mut Vec<Command>) -> Review {
        let Some(builder) = self.own.get(&id) else { return Review::Done };
        let age = self.frame - order.given;
        let since_told = self.frame - order.told;
        match &order.what {
            What::Build { frame: Some(frame), .. } => match self.own.get(frame) {
                Some(u) if u.being_built => {
                    // The builder let go of a frame that still stands (an idle flag with a frame beside it): put it
                    // back on it.
                    if builder.idle && since_told > GRACE {
                        commands.push(Command::Repair { unit: id, target: *frame, queue: false });
                        self.hands.orders.get_mut(&id).expect("reviewed").told = self.frame;
                    }
                    Review::Keep
                }
                _ => Review::Done,
            },
            What::Build { frame: None, near, .. } => {
                // Held: standing within reach of the site this long with no frame begun, from the first frame the
                // builder stood that near; a builder still walking (an extractor 4,600 elmos out, terr3-nullai-comet-1
                // game 00) is given the walk, up to three times the limit.
                let arrived = order.arrived.or_else(|| (builder.pos.dist2d(*near) < HELD_NEAR).then_some(self.frame));
                if arrived != order.arrived && let Some(o) = self.hands.orders.get_mut(&id) {
                    o.arrived = arrived;
                }
                if arrived.is_some_and(|a| self.frame - a > HELD) || age > 3 * HELD {
                    Review::Failed("held")
                } else if builder.idle && since_told > GRACE {
                    Review::Failed("dropped")
                } else {
                    Review::Keep
                }
            }
            What::Assist { factory, until } => {
                if self.frame >= *until || !self.own.contains_key(factory) {
                    Review::Done
                } else {
                    Review::Keep
                }
            }
            What::Help { frame } => match self.own.get(frame) {
                Some(u) if u.being_built => {
                    if builder.idle && since_told > GRACE {
                        commands.push(Command::Repair { unit: id, target: *frame, queue: false });
                        self.hands.orders.get_mut(&id).expect("reviewed").told = self.frame;
                    }
                    Review::Keep
                }
                _ => Review::Done,
            },
            // Over in the farm's core, or after a step of the walk, so the flows and the count are offered on the way.
            What::Walk { to } => {
                if builder.pos.dist2d(*to) <= FARM_CORE || age >= WALK_STEP { Review::Done } else { Review::Keep }
            }
        }
    }

    /// An order that came to nothing: logged, and its step given again once (at another site) before it is skipped.
    pub(crate) fn failed(&mut self, id: UnitId, why: &str) {
        let Some(order) = self.hands.orders.remove(&id) else { return };
        self.hands.dropped += 1;
        let what = match &order.what {
            What::Build { def, near, .. } => {
                // What stands on the site and what of his is near: the held extractor orders (terr13-lp-nullai-comet-1:
                // 21 in one game, two minutes of a constructor each, at spots all over the map).
                let name = |d: UnitDefId| self.def(d).map_or("?", |d| d.name.as_str());
                let on_site: Vec<String> = self.own.values().filter(|u| u.id != id && u.pos.dist2d(*near) < ON_SITE).map(|u| format!("{}{}", name(u.def), if u.being_built { "(frame)" } else if u.idle { "(idle)" } else { "" })).collect();
                let his: Vec<String> = self.enemies.iter().filter(|e| e.pos.dist2d(*near) < 5.0 * ON_SITE).map(|e| e.def.map_or("?", name).to_string()).collect();
                let builder = self.own.get(&id).map_or("gone".to_string(), |b| format!("{:.0} off{}", b.pos.dist2d(*near), if b.idle { ", idle" } else { "" }));
                format!("{} at ({:.1},{:.1}); builder {builder}; on the site: {}; his: {}", name(*def), near.x, near.z, if on_site.is_empty() { "-".to_string() } else { on_site.join(" ") }, if his.is_empty() { "-".to_string() } else { his.join(" ") })
            }
            What::Assist { .. } => "assist".to_string(),
            What::Help { .. } => "help".to_string(),
            What::Walk { .. } => "walk".to_string(),
        };
        self.log(format_args!("dropped: unit {} {what} after {:.1} s: {why}", id.0, (self.frame - order.given) as f32 / SECOND as f32));
        if let What::Build { unit, near, .. } = &order.what
            && self.game().units.list[*unit].extracts_metal > 0.0
            && let Some(spot) = self.hello.metal_spots.iter().position(|s| s.dist2d(*near) < SPOT_TAKEN)
        {
            self.hands.refused.insert(spot, self.frame + SPOT_REFUSED);
        }
        // A farm slot or a factory site the host found no site at is the site's fault, not the type's (the bot chose
        // the ground, and the engine sees features it does not): dead for a while, no strike.
        let farm = |me: &Self, unit: usize| { let kind = &me.game().units.list[unit]; kind.speed == 0.0 && kind.extracts_metal == 0.0 };
        if why == "no site" && let What::Build { unit, near, .. } = &order.what && farm(self, *unit) {
            let until = self.frame + FARM_DEAD;
            self.hands.farm_dead.push((*near, until));
        } else if why == "no site" && let What::Build { unit, .. } = &order.what {
            let frame = self.frame;
            let strikes = self.hands.no_site.entry(*unit).or_default();
            strikes.retain(|f| frame - *f < TYPE_REFUSED);
            strikes.push(frame);
            if strikes.len() >= NO_SITE_STRIKES {
                self.hands.type_refused.insert(*unit, frame + TYPE_REFUSED);
            }
        }
        let counted = self.frame >= FIRST_ORDERS;
        // Only an order the engine dropped or rejected says the type cannot be built; a site the host could not find,
        // or a builder held mid-walk, says nothing of the type (terr8-lp-nullai-comet-1 struck converters and
        // construction turrets on site failures round a crowded base); and never a factory.
        if counted && (why == "dropped" || why == "rejected") && let What::Build { unit, .. } = &order.what && self.game().units.list[*unit].role != Role::Factory {
            let failures = self.hands.failures.entry(*unit).or_default();
            *failures += 1;
            let failures = *failures;
            let successes = self.hands.successes.get(unit).copied().unwrap_or(0);
            if failures >= STRIKE && failures > successes && self.hands.struck.insert(*unit) {
                let name = self.game().units.list[*unit].name.clone();
                self.log(format_args!("struck: {name} after {failures} orders came to nothing and {successes} frames begun"));
                if let Some(planner) = self.planner.as_mut() {
                    let gone = |i: &Item| matches!(i, Item::Build(u) if *u == *unit);
                    planner.palette.mobile.retain(|i| !gone(i));
                    planner.palette.factory.retain(|i| !gone(i));
                }
            }
        }
        let attempts = self.hands.attempts.entry(id).or_default();
        if counted {
            *attempts += 1;
        }
        if *attempts < ATTEMPTS {
            if let (Some(step), Some((_, next))) = (order.step, self.hands.queues.get_mut(&id)) {
                *next = step;
            }
        } else {
            *attempts = 0;
            self.hands.skipped += 1;
        }
    }

    fn give_next(&mut self, id: UnitId, role: Role, commands: &mut Vec<Command>) {
        let Some(&(queue, mut next)) = self.hands.queues.get(&id) else { return };
        let Some(unit) = self.own.get(&id).and_then(|u| self.unit_index(u.def)) else { return };
        loop {
            let step = if queue == NO_QUEUE { None } else { self.hands.plan.queue(queue).get(next).copied() };
            let (step, from_queue) = match step {
                Some(step) => (step, true),
                // The simulator's default for a constructor whose queue has run out: the cheapest extractor it can
                // build, on the nearest free spot.
                None if role == Role::Builder => match self.game().units.extractor(unit) {
                    Some(extractor) => (Step::build(extractor), false),
                    None => return,
                },
                None => return,
            };
            if from_queue {
                next += 1;
                self.hands.queues.insert(id, (queue, next));
            }
            let attempt = self.hands.attempts.get(&id).copied().unwrap_or(0);
            if let Some((command, what)) = self.give(id, unit, role, step, attempt, None) {
                commands.push(command);
                self.note_order(id, &what, "queue");
                self.hands.orders.insert(id, Order { what, given: self.frame, told: self.frame, arrived: None, step: from_queue.then_some(next - 1) });
                return;
            }
            if !from_queue {
                return;
            }
        }
    }

    /// One step as an order, placed as the simulator places it (T-SITES); none when the step is passed over, as the
    /// simulator passes it over (no free spot, a unit this builder cannot build, a factory beyond the plan's queues).
    fn give(&mut self, id: UnitId, builder_unit: usize, role: Role, step: Step, attempt: u32, ring: Option<usize>) -> Option<(Command, What)> {
        let builder = self.own.get(&id)?.clone();
        let game = self.game();
        let units = &game.units;
        match step.item {
            Item::Assist => {
                if role == Role::Factory {
                    return None;
                }
                let factory = self.first_factory()?;
                Some((Command::Guard { unit: id, target: factory }, What::Assist { factory, until: self.frame + ASSIST_CHUNK }))
            }
            Item::Help => {
                if role == Role::Factory {
                    return None;
                }
                let frame = self.dearest_frame(builder_unit, builder.pos, (units.list[builder_unit].speed * LEAD) as f32)?;
                Some((Command::Repair { unit: id, target: frame, queue: false }, What::Help { frame }))
            }
            Item::Build(unit) => {
                if !units.list[builder_unit].builds.contains(&unit) || self.hands.struck.contains(&unit) {
                    return None;
                }
                let def = self.hello.unit_defs[unit].id;
                let target = &units.list[unit];
                if role == Role::Factory {
                    return Some((Command::Build { unit: id, def, site: None, queue: false }, What::Build { unit, def, near: builder.pos, frame: None }));
                }
                let site = if target.role == Role::Factory {
                    // The simulator's rule for the anneal's plan: no more factories than it has queues for. With
                    // the program's targets the cap is the program's own (terr6-lp-nullai-comet-1: the empty plan's
                    // zero queues refused every lab, and the commander played alone for twenty minutes).
                    let cap = if self.hands.targets.is_some() { self.options.factories } else { self.hands.plan.factories.len() };
                    if self.factories_planned() >= cap {
                        return None;
                    }
                    self.factory_site(builder.pos, builder_unit, unit, attempt)?
                } else if self.is_upgrade(unit) {
                    // Over the standing basic extractor nearest the builder with nothing ordered over it.
                    let standing = self.basic_to_upgrade(builder.pos, id, role, ring)?;
                    // Logged to the hundredth: the game's "Deny Invalid Mex Orders" rule allows an extractor over
                    // ours only at its exact position, and every Cortex order was dropped in terr1-nullai-comet-1.
                    self.log(format_args!("upgrade: {} over unit {} at ({:.2},{:.2}) by unit {}", target.name, standing.id.0, standing.pos.x, standing.pos.z, id.0));
                    BuildSite { near: standing.pos, search_radius: 0.0, min_dist: 0, placements: Vec::new(), exact: false }
                } else if target.extracts_metal > 0.0 {
                    let leash = (role == Role::Commander).then_some((self.home, self.options.com_leash));
                    let spot = self.free_spot(builder.pos, step.site, leash, ring)?;
                    BuildSite { near: self.hello.metal_spots[spot], search_radius: 0.0, min_dist: 0, placements: Vec::new(), exact: false }
                } else {
                    // T-FARM: turrets and the economy's buildings on the farm's lattice (before it, a turret went
                    // behind the nearest factory and an economy building beside its builder or on the simulator's
                    // spiral; the turrets then helped a factory each and the frames of the late game stood one
                    // builder apiece, terr22-53; terr54's sunflower is in the constants' note).
                    self.farm_site(id, unit)?
                };
                let near = site.near;
                Some((Command::Build { unit: id, def, site: Some(site), queue: false }, What::Build { unit, def, near, frame: None }))
            }
        }
    }

    /// T-SITES: the walking field from home of the class a factory of type `unit` is sited for: the strictest
    /// movement class among its products (a vehicle plant's heavy tanks climb the least), or its builder's when its
    /// products fly. Made once a type; None without terrain.
    fn factory_ground(&mut self, unit: usize, builder_unit: usize) -> Option<Arc<Field>> {
        if let Some(field) = self.hands.factory_ground.get(&unit) {
            return field.clone();
        }
        let units = &self.game().units;
        let terrain = &self.hello.terrain;
        let passable = units.list[unit]
            .builds
            .iter()
            .filter_map(|p| units.list[*p].move_class)
            .map(|class| terrain::passable(terrain, class))
            .min_by_key(|passable| passable.iter().filter(|ok| **ok).count())
            .or_else(|| units.list[builder_unit].move_class.map(|class| terrain::passable(terrain, class)));
        let field = passable.and_then(|passable| Field::from(terrain, &passable, self.home)).map(Arc::new);
        self.hands.factory_ground.insert(unit, field.clone());
        field
    }

    /// T-SITES: a factory's site. Round the point FACTORY_OUT from home toward his start, then a quarter turn
    /// further for each factory standing or ordered and each dropped order, the lattice point nearest it within
    /// FACTORY_SEARCH (the builder's walk added to that ranked an advanced plant 885 toward a far constructor on Great
    /// Divide, onto ground the engine refused, and the plant came at 10:05 or never, terr123) whose footprint the
    /// ground takes (as a farm slot's), not on a site the host found no site at lately (`farm_dead`: the engine sees
    /// features the bot does not), that its products reach from home on
    /// foot, FACTORY_GAP clear of every footprint and out of every yard, lane, mobile unit and the farm's core, with
    /// a facing whose exit lane covers no building and no metal spot and is ground its products reach its whole
    /// length; the facings toward his start first, the four turns tried before giving up. Before, the host was asked
    /// for the engine's nearest site to the point, one facing: on All That Glitters a vehicle plant faced a rock its
    /// constructors never left the pad for, and an advanced plant went to a plateau 340 above the base whose builder
    /// stood 235 off for fifteen minutes (terr105). None, logged, when no point serves.
    fn factory_site(&mut self, from: Vec3, builder_unit: usize, unit: usize, attempt: u32) -> Option<BuildSite> {
        let frame = self.frame;
        self.hands.farm_dead.retain(|(_, until)| frame < *until);
        let ground = self.factory_ground(unit, builder_unit);
        let origin = self.farm_origin();
        let core = Lane { from: origin, to: origin, half_width: FARM_CORE };
        let fp = self.hello.unit_defs[unit].footprint;
        let half = fp.0.max(fp.1) as f32 * 4.0;
        // The engine's build grid: a footprint of 2 mod 4 squares centres on an odd multiple of 8.
        let snap = |v: f32, squares: i32| if squares & 2 != 0 { (v / 16.0).floor() * 16.0 + 8.0 } else { ((v + 8.0) / 16.0).floor() * 16.0 };
        let (taken, lanes, mobile) = self.site_blockers();
        let spots = &self.hello.metal_spots;
        let (dx, dz) = (self.enemy_home.x - self.home.x, self.enemy_home.z - self.home.z);
        let length = dx.hypot(dz).max(1.0);
        let (ux, uz) = (dx / length, dz / length);
        let mut facings = [0, 1, 2, 3];
        facings.sort_by(|a, b| {
            let along = |f: &i32| FACING_DIR[*f as usize].0 * ux + FACING_DIR[*f as usize].1 * uz;
            along(b).total_cmp(&along(a))
        });
        let lane_serves = |p: Vec3, facing: i32| {
            let lane = lane_at(p, facing);
            if spots.iter().any(|s| lane.contains(*s)) || taken.iter().any(|(q, h)| Lane { half_width: LANE_HALF + h, ..lane }.contains(*q)) {
                return false;
            }
            let (fx, fz) = FACING_DIR[facing as usize];
            let steps = ((LANE_LENGTH - half) / LANE_STEP).floor() as i32;
            ground.as_ref().is_none_or(|g| {
                (0..=steps).all(|k| {
                    let t = half + LANE_STEP * k as f32;
                    [-LANE_SIDE, 0.0, LANE_SIDE].iter().all(|s| g.reaches(Vec3 { x: p.x + fx * t - fz * s, y: p.y, z: p.z + fz * t + fx * s }))
                })
            })
        };
        let (mut off_ground, mut unreached, mut crowded, mut no_lane) = (0, 0, 0, 0);
        let n = (FACTORY_SEARCH / FACTORY_GRID).ceil() as i32;
        for turn in 0..4 {
            let (rx, rz) = match (self.factories_planned() as u32 + attempt + turn) % 4 {
                0 => (ux, uz),
                1 => (-uz, ux),
                2 => (uz, -ux),
                _ => (-ux, -uz),
            };
            let want = Vec3 { x: self.home.x + rx * FACTORY_OUT, y: self.home.y, z: self.home.z + rz * FACTORY_OUT };
            let mut points: Vec<Vec3> = (-n..=n)
                .flat_map(|i| (-n..=n).map(move |j| (i, j)))
                .map(|(i, j)| Vec3 { x: snap(want.x + FACTORY_GRID * i as f32, fp.0), y: want.y, z: snap(want.z + FACTORY_GRID * j as f32, fp.1) })
                .filter(|p| p.dist2d(want) <= FACTORY_SEARCH)
                .collect();
            points.sort_by(|a, b| a.dist2d(want).total_cmp(&b.dist2d(want)));
            for p in points {
                if !self.farm_ground(p, half) {
                    off_ground += 1;
                    continue;
                }
                if ground.as_ref().is_some_and(|g| !g.reaches(p)) {
                    unreached += 1;
                    continue;
                }
                let clear = !core.contains(p)
                    && !self.dead_ground(p)
                    && !self.hands.farm_dead.iter().any(|(d, _)| d.dist2d(p) < half)
                    && !lanes.iter().any(|l| l.contains(p))
                    && !taken.iter().any(|(q, h)| (q.x - p.x).abs().max((q.z - p.z).abs()) < h + half + FACTORY_GAP)
                    && !mobile.iter().any(|q| q.dist2d(p) < half + FARM_MOBILE);
                if !clear {
                    crowded += 1;
                    continue;
                }
                let Some(facing) = facings.iter().copied().find(|f| lane_serves(p, *f)) else {
                    no_lane += 1;
                    continue;
                };
                self.log(format_args!(
                    "factory: {} at ({:.0},{:.0}) facing {facing}, {:.0} from the wanted point at turn {turn}, {:.0} from the builder, {:.0} from home; passed over {off_ground} off the ground, {unreached} its products cannot reach, {crowded} crowded, {no_lane} with no lane",
                    self.game().units.list[unit].name, p.x, p.z, p.dist2d(want), p.dist2d(from), p.dist2d(self.home)
                ));
                let mut keep_out = lanes.clone();
                keep_out.push(core);
                return Some(BuildSite { near: p, search_radius: FARM_SEARCH, min_dist: 0, placements: vec![Placement { facing, keep_out }], exact: true });
            }
        }
        self.log(format_args!(
            "factory: no site for {} within {FACTORY_SEARCH:.0} of the four turns: {off_ground} off the ground, {unreached} its products cannot reach, {crowded} crowded, {no_lane} with no lane",
            self.game().units.list[unit].name
        ));
        // The search covers the ground round home for every builder alike: it is not asked again for the window.
        self.hands.type_refused.insert(unit, self.frame + TYPE_REFUSED);
        None
    }

    /// The commander's ground (T-SITES): as far from home as a plant of ours may stand, or the farm's core. Its cheap
    /// buildings and the frames it helps stay on it: placed beside it outside the core a wind generator at a time, it
    /// walked from 1,270 to 3,500-3,800 of home on Great Divide and built beside his commander from 15:00 (terr104,
    /// the commanders' duel in two games of four), and 13-70 such orders a game took it past 1,000 on the windy maps
    /// (terr104-107; 0-4 on Comet).
    fn commander_ground(&self, p: Vec3) -> bool {
        p.dist2d(self.home) <= FACTORY_OUT + FACTORY_SEARCH || self.hands.farm_origin.is_some_and(|o| p.dist2d(o) <= FARM_CORE)
    }

    /// The farm's origin (T-FARM), chosen once: of the points FARM_OFFSETS from home at sixteen bearings, the one
    /// whose core holds the most building slots the ground can take (off the base's own 400), discounted by how far
    /// it lies toward his start (by half, on the line to him), snapped to the build grid.
    pub(crate) fn farm_origin(&mut self) -> Vec3 {
        if let Some(origin) = self.hands.farm_origin {
            return origin;
        }
        let (ex, ez) = (self.enemy_home.x - self.home.x, self.enemy_home.z - self.home.z);
        let length = ex.hypot(ez).max(1.0);
        let half = FARM_SLOT / 2.0 - FARM_GAP;
        let mut best: Option<(f32, Vec3, usize, f32, usize)> = None;
        for bearing in 0..16 {
            let angle = std::f32::consts::TAU * bearing as f32 / 16.0;
            let toward = (angle.cos() * ex + angle.sin() * ez) / length;
            for offset in FARM_OFFSETS {
                let snap = |v: f32| (v / 16.0).round() * 16.0;
                let candidate = Vec3 { x: snap(self.home.x + offset * angle.cos()), y: self.home.y, z: snap(self.home.z + offset * angle.sin()) };
                let slots = farm_slots(candidate, candidate, FARM_CORE, false).into_iter().filter(|p| p.dist2d(self.home) > 400.0 && self.farm_ground(*p, half)).count();
                let score = slots as f32 * (1.0 - 0.5 * toward.max(0.0));
                if best.is_none_or(|b| score > b.0) {
                    best = Some((score, candidate, bearing, offset, slots));
                }
            }
        }
        let (score, origin, bearing, offset, slots) = best.expect("sixteen bearings");
        let his = ez.atan2(ex).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU * 16.0;
        self.log(format_args!("farm: origin ({:.0},{:.0}), {offset:.0} from home at bearing {bearing}/16 (his start at {his:.1}/16), {slots} building slots of ground in the core, score {score:.0}", origin.x, origin.z));
        self.hands.farm_origin = Some(origin);
        origin
    }

    /// Ground a footprint of half-size `half` centred at `p` can take: on the map, no slope over FARM_SLOPE and no
    /// water under it, no metal spot in it (an extractor or its upgrade goes there).
    fn farm_ground(&self, p: Vec3, half: f32) -> bool {
        let terrain = &self.hello.terrain;
        let (width, height) = (self.hello.map.width, self.hello.map.height);
        if p.x - half < 32.0 || p.z - half < 32.0 || p.x + half > width - 32.0 || p.z + half > height - 32.0 {
            return false;
        }
        let cell = terrain.cell.max(1.0);
        let span = |v: f32| ((v - half) / cell).floor() as i64..=((v + half - 1.0) / cell).floor() as i64;
        for j in span(p.z) {
            for i in span(p.x) {
                if i < 0 || j < 0 || i >= i64::from(terrain.width) || j >= i64::from(terrain.height) {
                    return false;
                }
                let at = j as usize * terrain.width as usize + i as usize;
                if terrain.slopes.get(at).is_none_or(|s| *s > FARM_SLOPE) || terrain.heights.get(at).is_none_or(|h| *h < 0) {
                    return false;
                }
            }
        }
        !self.hello.metal_spots.iter().any(|s| s.dist2d(p) < SPOT_TAKEN + half)
    }

    /// What a site must clear (T-FARM): the footprints standing or pending (position, half-width), the factories'
    /// yards and exit lanes, and the mobile units' positions.
    fn site_blockers(&self) -> (Vec<(Vec3, f32)>, Vec<Lane>, Vec<Vec3>) {
        let units = &self.game().units;
        let footprint = |fp: (i32, i32)| fp.0.max(fp.1) as f32 * 4.0;
        let mut taken: Vec<(Vec3, f32)> = self.own.values().filter_map(|u| self.def(u.def).filter(|d| d.speed == 0.0).map(|d| (u.pos, footprint(d.footprint)))).collect();
        taken.extend(self.hands.orders.values().filter_map(|o| match &o.what {
            What::Build { unit, near, frame: None, .. } if units.list[*unit].speed == 0.0 => Some((*near, footprint(self.hello.unit_defs[*unit].footprint))),
            _ => None,
        }));
        let mobile: Vec<Vec3> = self.own.values().filter(|u| self.def(u.def).is_some_and(|d| d.speed > 0.0)).map(|u| u.pos).collect();
        (taken, self.keep_out(false), mobile)
    }

    /// Whether a footprint of half-width `half` can stand at `p` (T-FARM): ground it can take, not dead ground (T-SITES),
    /// not a dead slot, not in a yard or lane, FARM_GAP clear of every footprint, FARM_MOBILE clear of every mobile unit.
    fn site_free(&self, p: Vec3, half: f32, taken: &[(Vec3, f32)], lanes: &[Lane], mobile: &[Vec3]) -> bool {
        self.farm_ground(p, half)
            && !self.dead_ground(p)
            && !self.hands.farm_dead.iter().any(|(d, _)| d.dist2d(p) < FARM_GAP)
            && !lanes.iter().any(|l| l.contains(p))
            && !taken.iter().any(|(q, h)| (q.x - p.x).abs().max((q.z - p.z).abs()) < h + half + FARM_GAP)
            && !mobile.iter().any(|q| q.dist2d(p) < half + FARM_MOBILE)
    }

    /// T-PLANT: each plant standing, framed or ordered (the program counts a committed plant as standing) with the
    /// construction turrets within PLANT_RING of its centre and the further turrets its ring could take, packed as
    /// the hands pack them (the free 16-grid point nearest the centre, then the next clear of it, until none is
    /// free): the ring's capacity, the program's bound on the turrets' assist on that plant's products
    /// (`LpOptions::plant_rings`; his 29 within 400 of his advanced plant).
    pub(crate) fn plant_rings(&self) -> Vec<(usize, Vec3, usize, usize)> {
        let units = &self.game().units;
        let Some(turret) = (0..units.list.len()).find(|&i| units.list[i].role == Role::Nano) else { return Vec::new() };
        let fp = self.hello.unit_defs[turret].footprint;
        let half = fp.0.max(fp.1) as f32 * 4.0;
        let (taken, lanes, mobile) = self.site_blockers();
        let mut plants: Vec<(usize, Vec3)> = self.own.values().filter_map(|u| self.unit_index(u.def).filter(|i| units.list[*i].role == Role::Factory).map(|i| (i, u.pos))).collect();
        plants.extend(self.hands.orders.values().filter_map(|o| match &o.what {
            What::Build { unit, near, frame: None, .. } if units.list[*unit].role == Role::Factory => Some((*unit, *near)),
            _ => None,
        }));
        let turrets: Vec<Vec3> = self.own.values().filter(|u| self.role_of_def(u.def) == Some(Role::Nano)).map(|u| u.pos).collect();
        let grid = |v: f32| (v / 16.0).floor() * 16.0 + 8.0;
        let n = (PLANT_RING / 16.0).ceil() as i32;
        plants
            .into_iter()
            .map(|(unit, centre)| {
                let standing = turrets.iter().filter(|t| t.dist2d(centre) < PLANT_RING).count();
                let candidates: Vec<Vec3> = (-n..=n)
                    .flat_map(|i| (-n..=n).map(move |j| (i, j)))
                    .map(|(i, j)| Vec3 { x: grid(centre.x) + 16.0 * i as f32, y: centre.y, z: grid(centre.z) + 16.0 * j as f32 })
                    .filter(|p| p.dist2d(centre) < PLANT_RING)
                    .collect();
                let mut packed = taken.clone();
                let mut free = 0;
                while let Some(p) = candidates.iter().copied().filter(|p| self.site_free(*p, half, &packed, &lanes, &mobile)).min_by(|a, b| a.dist2d(centre).total_cmp(&b.dist2d(centre))) {
                    packed.push((p, half));
                    free += 1;
                }
                (unit, centre, standing, free)
            })
            .collect()
    }

    /// The farm's slot for `unit` (T-FARM): a turret's the free turret slot of the core nearest home (past the
    /// core, the nearest outside it; first, while the program's factory assist asks for turrets at a plant, the free
    /// grid point within PLANT_RING of that plant nearest its centre, T-PLANT), a big building's the free building slot with the most turrets (standing or
    /// framed) in reach and nearest home among those, a cheap building's the free building slot nearest its
    /// `builder` outside the core. Free: ground the footprint
    /// can take, not in a factory's yard or lane, not a dead slot, and FARM_GAP clear of every standing building,
    /// frame and pending site by the footprints. None when the lattice within FARM_EXTENT has no such slot.
    fn farm_site(&mut self, builder: UnitId, unit: usize) -> Option<BuildSite> {
        let origin = self.farm_origin();
        let frame = self.frame;
        self.hands.farm_dead.retain(|(_, until)| frame < *until);
        let kind = &self.game().units.list[unit];
        let (turret, big) = (kind.role == Role::Nano, kind.build_time >= FARM_BIG);
        let footprint = |fp: (i32, i32)| fp.0.max(fp.1) as f32 * 4.0;
        let half = footprint(self.hello.unit_defs[unit].footprint);
        let (taken, lanes, mobile) = self.site_blockers();
        let free = |p: Vec3| self.site_free(p, half, &taken, &lanes, &mobile);
        let home = self.home;
        let nearer = |a: Vec3, b: Vec3, to: Vec3| a.dist2d(to).total_cmp(&b.dist2d(to));
        // The core fills from home's side (terr55-56: filled from its centre, 800 out, every turret and frame cost
        // its builder the walk there, 29-37 s a base order against terr53's, and 10:00 fell 20 in all eight games).
        let slot = if turret {
            // T-PLANT: round the factory the program's turret assist asks for, while it asks for more turrets than
            // stand or are ordered within reach of a factory; else the farm's.
            let plant = self.plant_turrets_wanted().1.and_then(|centre| {
                let grid = |v: f32| (v / 16.0).floor() * 16.0 + 8.0;
                let n = (PLANT_RING / 16.0).ceil() as i32;
                let candidates: Vec<Vec3> = (-n..=n)
                    .flat_map(|i| (-n..=n).map(move |j| (i, j)))
                    .map(|(i, j)| Vec3 { x: grid(centre.x) + 16.0 * i as f32, y: centre.y, z: grid(centre.z) + 16.0 * j as f32 })
                    .filter(|p| p.dist2d(centre) < PLANT_RING)
                    .collect();
                let slot = candidates.iter().copied().filter(|p| free(*p)).min_by(|a, b| nearer(*a, *b, centre).then_with(|| nearer(*a, *b, home)));
                match slot {
                    Some(p) => self.log(format_args!("plant: turret slot ({:.0},{:.0}), {:.0} from the plant at ({:.0},{:.0})", p.x, p.z, p.dist2d(centre), centre.x, centre.z)),
                    None => {
                        let ground = candidates.iter().filter(|p| !self.farm_ground(**p, half)).count();
                        let laned = candidates.iter().filter(|p| lanes.iter().any(|l| l.contains(**p))).count();
                        let held = candidates.iter().filter(|p| taken.iter().any(|(q, h)| (q.x - p.x).abs().max((q.z - p.z).abs()) < h + half + FARM_GAP)).count();
                        let crowded = candidates.iter().filter(|p| mobile.iter().any(|q| q.dist2d(**p) < half + FARM_MOBILE)).count();
                        let dead = candidates.iter().filter(|p| self.hands.farm_dead.iter().any(|(d, _)| d.dist2d(**p) < FARM_GAP)).count();
                        self.log(format_args!("plant: no turret slot within {PLANT_RING:.0} of ({:.0},{:.0}): {} points, {ground} off the ground, {laned} in a yard or lane, {held} by footprints, {crowded} by mobile units, {dead} dead", centre.x, centre.z, candidates.len()));
                    }
                }
                slot
            });
            plant.or_else(|| farm_slots(origin, origin, FARM_EXTENT, true).into_iter().filter(|p| free(*p)).min_by(|a, b| (a.dist2d(origin) > FARM_CORE, a.dist2d(home)).partial_cmp(&(b.dist2d(origin) > FARM_CORE, b.dist2d(home))).unwrap()))
        } else if big {
            let turrets: Vec<Vec3> = self.own.values().filter(|u| self.role_of_def(u.def) == Some(Role::Nano)).map(|u| u.pos).collect();
            let reach = |p: Vec3| turrets.iter().filter(|t| t.dist2d(p) < NANO_REACH).count();
            farm_slots(origin, origin, FARM_EXTENT, false).into_iter().filter(|p| free(*p)).max_by(|a, b| reach(*a).cmp(&reach(*b)).then_with(|| nearer(*b, *a, home)))
        } else {
            // Beside its builder, as the simulator charges it: the slots nearest home were the pad-keeper's ground
            // and the commander's, and the solars walked 500-1,000 out round them (terr55-56). Not in the turrets'
            // reach outside the core (terr63-64): the turrets fill the core from home's side, so the ground they
            // reach outside it is the base's own, full of the early solars, the lanes and the spots, and no
            // converter found such a slot in two batches; a converter beside its builder costs it 6 s and 29 s of
            // building. A 48-elmo footprint takes half a slot.
            let from = self.own.get(&builder).map_or(home, |u| u.pos);
            let commander = self.own.get(&builder).and_then(|u| self.role_of_def(u.def)) == Some(Role::Commander);
            let positions = |p: Vec3| if half <= 24.0 { vec![Vec3 { x: p.x - FARM_PAIR, ..p }, Vec3 { x: p.x + FARM_PAIR, ..p }] } else { vec![p] };
            farm_slots(origin, from, FARM_EXTENT, false)
                .into_iter()
                .filter(|p| p.dist2d(origin) > FARM_CORE && (!commander || self.commander_ground(*p)))
                .flat_map(positions)
                .filter(|p| free(*p))
                .min_by(|a, b| nearer(*a, *b, from))
        }?;
        Some(BuildSite { near: slot, search_radius: FARM_SEARCH, min_dist: 0, placements: vec![Placement { facing: self.facing_enemy(), keep_out: lanes }], exact: true })
    }

    /// T-PLANT: the program's turret assist over the next PLANT_LEAD seconds (its Assist flows of the turret type) as
    /// turrets of the type's power, and the factory those turrets should stand round while that runs half a turret
    /// ahead of the turrets standing, framed or ordered within reach of a factory: the factory standing or framed
    /// (the program counts a committed plant as standing, T-COMMIT, and its turrets are placed from its frame)
    /// nearest home among those making the product the program puts the most turret power on.
    pub(crate) fn plant_turrets_wanted(&self) -> (f64, Option<Vec3>) {
        let Some(targets) = self.hands.targets.as_ref() else { return (0.0, None) };
        let units = &self.game().units;
        let until = self.frame as f64 / SECOND as f64 + PLANT_LEAD;
        let span = (until - targets.t0).max(1.0);
        let mut by_product: BTreeMap<usize, f64> = BTreeMap::new();
        let mut seen: BTreeSet<(usize, usize, Class)> = BTreeSet::new();
        for f in targets.flows.iter().filter(|f| f.kind == Kind::Assist && units.list[f.builder].role == Role::Nano) {
            if seen.insert((f.builder, f.unit, f.class)) {
                *by_product.entry(f.unit).or_default() += targets.planned_power_class(f.builder, f.unit, f.class, until) / (units.list[f.builder].worker_time * span);
            }
        }
        let wanted: f64 = by_product.values().sum();
        let plants: Vec<&OwnUnit> = self.own.values().filter(|u| self.role_of_def(u.def) == Some(Role::Factory)).collect();
        let at_a_plant = |p: Vec3| plants.iter().any(|f| f.pos.dist2d(p) < NANO_REACH);
        let standing = self.own.values().filter(|u| self.role_of_def(u.def) == Some(Role::Nano) && at_a_plant(u.pos)).count();
        let ordered = self.hands.orders.values().filter(|o| matches!(&o.what, What::Build { unit, near, frame: None, .. } if units.list[*unit].role == Role::Nano && at_a_plant(*near))).count();
        if wanted < (standing + ordered) as f64 + START_AT {
            return (wanted, None);
        }
        let Some((&product, _)) = by_product.iter().max_by(|a, b| a.1.total_cmp(b.1)) else { return (wanted, None) };
        let plant = plants.iter().filter(|f| self.unit_index(f.def).is_some_and(|fi| units.list[fi].builds.contains(&product))).min_by(|a, b| a.pos.dist2d(self.home).total_cmp(&b.pos.dist2d(self.home))).map(|f| f.pos);
        (wanted, plant)
    }

    /// The standing basic extractor nearest `from` with nothing ordered over it (not the builder itself; the
    /// commander's within its leash; in ring `ring` when given): what an upgrade goes over.
    fn basic_to_upgrade(&self, from: Vec3, id: UnitId, role: Role, ring: Option<usize>) -> Option<OwnUnit> {
        let units = &self.game().units;
        let ordered: Vec<Vec3> = self.hands.orders.values().filter_map(|o| match &o.what { What::Build { unit, near, .. } if self.is_upgrade(*unit) => Some(*near), _ => None }).collect();
        self.own
            .values()
            .filter(|u| !u.being_built && u.id != id)
            .filter(|u| role != Role::Commander || u.pos.dist2d(self.home) < self.options.com_leash)
            .filter(|u| self.unit_index(u.def).is_some_and(|i| units.list[i].extracts_metal > 0.0 && !self.is_upgrade(i)))
            .filter(|u| ring.is_none_or(|k| self.ring_of_pos(u.pos) == k))
            .filter(|u| !ordered.iter().any(|o| o.dist2d(u.pos) < SPOT_TAKEN))
            .min_by(|a, b| a.pos.dist2d(from).total_cmp(&b.pos.dist2d(from)))
            .cloned()
    }

    /// T-CHAIN at the far end: a constructor out among the spots takes the nearest free spot, or the nearest basic
    /// extractor it can upgrade, whichever is nearer, when it is nearer than home and the program's count of the
    /// type at the horizon runs past what stands or is ordered; a turret, a guard or a frame at the base can wait
    /// for a builder at the base. terr70's order lines (the standing code, minutes 4-15): constructors beyond 2,000
    /// of home were given 17-30 guard chunks a game (a 77-90 s walk for a 20 s chunk), 8-13 turrets on the farm
    /// (98-127 s), and 3-10 frames to help at the base; 31-46 turn-arounds a game and 55-74 constructor-minutes
    /// of walking toward home.
    fn chain_on(&mut self, id: UnitId, unit: usize, role: Role, builder: &OwnUnit, commands: &mut Vec<Command>) -> bool {
        let Some(targets) = self.hands.targets.as_ref() else { return false };
        let end = targets.t0 + targets.starts[targets.buckets];
        let shortfalls = self.shortfalls(end);
        let wants = |u: usize| shortfalls.iter().any(|(v, short)| *v == u && *short > START_AT);
        let home = builder.pos.dist2d(self.home);
        let (extractor, upgrade) = {
            let units = &self.game().units;
            (units.extractor(unit).filter(|u| wants(*u)), units.list[unit].builds.iter().copied().find(|u| self.is_upgrade(*u)).filter(|u| wants(*u)))
        };
        let mut picks: Vec<(f32, usize)> = Vec::new();
        let leash = (role == Role::Commander).then_some((self.home, self.options.com_leash));
        if let Some(u) = extractor && let Some(spot) = self.free_spot(builder.pos, None, leash, None) && self.hello.metal_spots[spot].dist2d(builder.pos) < home {
            picks.push((self.hello.metal_spots[spot].dist2d(builder.pos), u));
        }
        if let Some(u) = upgrade && let Some(basic) = self.basic_to_upgrade(builder.pos, id, role, None) && basic.pos.dist2d(builder.pos) < home {
            picks.push((basic.pos.dist2d(builder.pos), u));
        }
        picks.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, u) in picks {
            let cost = self.game().units.list[u].build_time;
            if self.order_one(id, unit, role, u, cost, None, "chain", commands) {
                return true;
            }
        }
        false
    }

    /// A spot refused lately, with an extractor standing on it, or with one ordered to it.
    pub(crate) fn spot_taken(&self, i: usize) -> bool {
        let at = self.hello.metal_spots[i];
        self.hands.refused.get(&i).is_some_and(|until| self.frame < *until)
            || self.own.values().any(|u| self.unit_index(u.def).is_some_and(|x| self.game().units.list[x].extracts_metal > 0.0) && u.pos.dist2d(at) < SPOT_TAKEN)
            || self.hands.orders.values().any(|o| matches!(&o.what, What::Build { unit, near, .. } if self.game().units.list[*unit].extracts_metal > 0.0 && near.dist2d(at) < SPOT_TAKEN))
    }

    /// The free spot of the scenario's list nearest `from`; with `named`, the free spot within 100 of it or none;
    /// with `leash`, within that radius of that point.
    fn free_spot(&self, from: Vec3, named: Option<(f64, f64)>, leash: Option<(Vec3, f32)>, ring: Option<usize>) -> Option<usize> {
        let spots = &self.hello.metal_spots;
        let near_named = |i: usize| named.is_none_or(|(x, z)| spots[i].dist2d(Vec3 { x: x as f32, y: 0.0, z: z as f32 }) < SPOT_TAKEN);
        let leashed = |i: usize| leash.is_none_or(|(centre, radius)| spots[i].dist2d(centre) < radius);
        let in_ring = |i: usize| ring.is_none_or(|k| self.ring_of_pos(spots[i]) == k);
        // Nearest, with no heading kept: a turn penalty on the distance (the spot's distance over again for the
        // share of a turn back from the line out of home through the builder) moved neither the chains' step nor
        // their interval in terr59 (673-1,008 elmos and 42-59 s against terr57's 791-1,071 and 46-56).
        self.spots.iter().copied().filter(|i| !self.spot_taken(*i) && near_named(*i) && leashed(*i) && in_ring(*i)).min_by(|a, b| spots[*a].dist2d(from).total_cmp(&spots[*b].dist2d(from)))
    }

    /// Factories standing or ordered, so a plan cannot start more than it has queues for (the simulator's rule).
    fn factories_planned(&self) -> usize {
        let standing = self.own.values().filter(|u| self.role_of_def(u.def) == Some(Role::Factory)).count();
        let ordered = self.hands.orders.values().filter(|o| matches!(&o.what, What::Build { unit, frame: None, .. } if self.game().units.list[*unit].role == Role::Factory)).count();
        standing + ordered
    }

    fn first_factory(&self) -> Option<UnitId> {
        self.own.values().filter(|u| !u.being_built && self.role_of_def(u.def) == Some(Role::Factory)).map(|u| u.id).min()
    }

    fn nearest_factory(&self, from: Vec3) -> Option<&OwnUnit> {
        self.own.values().filter(|u| !u.being_built && self.role_of_def(u.def) == Some(Role::Factory)).min_by(|a, b| a.pos.dist2d(from).total_cmp(&b.pos.dist2d(from)))
    }

    /// The building frame within `reach` of `from` with the most metal still to build per builder on it, one more
    /// counted (a factory's pad is not a frame), among those the energy budget allows `builder` on: the help spreads over the frames rather
    /// than piling on the dearest (terr22-lp-nullai-comet-1: 48 constructors on one frame at 20:00 while five
    /// fusions stood with a builder each, and the program's power on fusions, committed to those frames, never
    /// cleared for new ones).
    fn dearest_frame(&self, builder: usize, from: Vec3, reach: f32) -> Option<UnitId> {
        let helpers = |frame: UnitId| self.hands.orders.values().filter(|o| matches!(&o.what, What::Help { frame: f } | What::Build { frame: Some(f), .. } if *f == frame)).count() as f32;
        self.own
            .values()
            .filter(|u| u.being_built && u.pos.dist2d(from) <= reach && (builder != self.game().commander || self.commander_ground(u.pos)))
            .filter_map(|u| {
                let d = self.def(u.def)?;
                let i = self.unit_index(u.def)?;
                (d.speed == 0.0 && self.energy_fits(i, builder)).then(|| (u.id, d.metal_cost * (1.0 - u.health / u.max_health.max(1.0)) / (1.0 + helpers(u.id))))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id)
    }

    /// The engine named a frame begun by one of our builders: the builder's order has its frame, and the engine
    /// log the order's age and the builder's distance (the walk calibration).
    pub(crate) fn frame_begun(&mut self, unit: UnitId, builder: UnitId) {
        let Some(frame) = self.own.get(&unit).cloned() else { return };
        let begun_unit = self.unit_index(frame.def);
        let Some(order) = self.hands.orders.get_mut(&builder) else { return };
        let What::Build { def, frame: slot, .. } = &mut order.what else { return };
        if *def != frame.def || slot.is_some() {
            return;
        }
        *slot = Some(unit);
        let given = order.given;
        // A frame begun clears the builder's failures: the attempt count is per run of failures, and left standing it
        // sent every economy building of a builder that had once failed to the base spiral, from wherever it stood
        // (terr14-lp-nullai-comet-1: 30 of 128 economy drops with the builder more than 1,500 off, one 5,400).
        self.hands.attempts.remove(&builder);
        if let Some(u) = begun_unit {
            self.hands.begun_log.entry(builder).or_default().push((self.frame, u));
            *self.hands.successes.entry(u).or_default() += 1;
        }
        self.hands.begun += 1;
        let age = (self.frame - given) as f32 / SECOND as f32;
        let distance = self.own.get(&builder).map_or(0.0, |b| b.pos.dist2d(frame.pos));
        let name = self.def(frame.def).map_or("?", |d| d.name.as_str());
        self.log(format_args!("begun: {name} by unit {} at {distance:.0}, {age:.1} s after its order, site ({:.0},{:.0})", builder.0, frame.pos.x, frame.pos.z));
    }

    /// The host placed a builder's site: the order's position is where the frame will stand (the hold clock and the
    /// dropped line read it), and other sites keep off it until the frame stands.
    pub(crate) fn sited(&mut self, builder: UnitId, at: Vec3) {
        if let Some(order) = self.hands.orders.get_mut(&builder)
            && let What::Build { frame: None, near, .. } = &mut order.what
        {
            *near = at;
        }
    }

    /// A unit of ours is gone: a builder's orders and queue with it, and an order whose frame it was.
    pub(crate) fn unit_lost(&mut self, unit: UnitId) {
        self.hands.orders.remove(&unit);
        self.hands.queues.remove(&unit);
        let holders: Vec<UnitId> = self.hands.orders.iter().filter(|(_, o)| matches!(&o.what, What::Build { frame: Some(f), .. } | What::Help { frame: f } if *f == unit)).map(|(id, _)| *id).collect();
        for holder in holders {
            self.failed(holder, "frame lost");
        }
    }

    /// Construction turrets feed the nearest factory within their reach (the calibrated replays' behaviour; the
    /// simulator's `nanos_help_frames` is off to match).
    pub(crate) fn run_nanos(&mut self, commands: &mut Vec<Command>) {
        let nanos: Vec<(UnitId, Vec3)> = self.own.values().filter(|u| !u.being_built && u.idle && self.role_of_def(u.def) == Some(Role::Nano)).map(|u| (u.id, u.pos)).collect();
        let plant_assist = self.plant_turrets_wanted().0 > START_AT;
        for (nano, at) in nanos {
            if self.hands.nano_told.get(&nano).is_some_and(|told| self.frame - told < GRACE) {
                continue;
            }
            // T-PLANT: while the program asks turret assist at a plant, a turret within reach of a factory at work is
            // the factory's.
            if plant_assist
                && let Some(factory) = self.own.values().filter(|f| !f.being_built && !f.idle && self.role_of_def(f.def) == Some(Role::Factory) && f.pos.dist2d(at) < NANO_REACH).min_by(|a, b| a.pos.dist2d(at).total_cmp(&b.pos.dist2d(at))).map(|f| f.id)
            {
                self.hands.nano_told.insert(nano, self.frame);
                self.hands.nano_frame.remove(&nano);
                commands.push(Command::Guard { unit: nano, target: factory });
                continue;
            }
            // With the program's targets a turret helps the dearest building frame in its reach, as the program
            // prices its power (bounded by the builders' own); else, and with none in reach, the nearest factory.
            let frame = self.hands.targets.as_ref().and_then(|_| {
                self.own
                    .values()
                    .filter(|u| u.being_built && u.pos.dist2d(at) < NANO_REACH && self.def(u.def).is_some_and(|d| d.speed == 0.0))
                    .max_by(|a, b| (self.def(a.def).map_or(0.0, |d| d.metal_cost) * (1.0 - a.health / a.max_health.max(1.0))).total_cmp(&(self.def(b.def).map_or(0.0, |d| d.metal_cost) * (1.0 - b.health / b.max_health.max(1.0)))))
                    .map(|u| u.id)
            });
            if let Some(frame) = frame {
                let now = self.frame;
                self.hands.nano_told.insert(nano, now);
                self.hands.nano_frame.entry(nano).and_modify(|e| if e.0 != frame { *e = (frame, now) }).or_insert((frame, now));
                commands.push(Command::Repair { unit: nano, target: frame, queue: false });
                continue;
            }
            let Some(factory) = self.nearest_factory(at).filter(|f| f.pos.dist2d(at) < NANO_REACH).map(|f| f.id) else { continue };
            self.hands.nano_told.insert(nano, self.frame);
            self.hands.nano_frame.remove(&nano);
            commands.push(Command::Guard { unit: nano, target: factory });
        }
    }

    /// The engine's facing (0 south +z, 1 east +x, 2 north -z, 3 west -x) that looks most toward his start.
    fn facing_enemy(&self) -> i32 {
        let (dx, dz) = (self.enemy_home.x - self.home.x, self.enemy_home.z - self.home.z);
        match (dx.abs() > dz.abs(), dx > 0.0, dz > 0.0) {
            (true, true, _) => 1,
            (true, false, _) => 3,
            (false, _, true) => 0,
            (false, _, false) => 2,
        }
    }

    /// Our factories' exit lanes and yards, the metal spots (an extractor or its upgrade needs the ground), and
    /// with `pending` the building sites placed and not yet begun: nothing is built in them (a lane of no length is
    /// a disc). The farm's slots keep their own distance from the pending sites, by the footprints (T-FARM).
    fn keep_out(&self, pending: bool) -> Vec<Lane> {
        let units = &self.game().units;
        let pending = self.hands.orders.values().filter(|_| pending).filter_map(|o| match &o.what {
            What::Build { unit, near, frame: None, .. } if units.list[*unit].speed == 0.0 => Some(Lane { from: *near, to: *near, half_width: PENDING_SITE }),
            _ => None,
        });
        let spots = self.hello.metal_spots.iter().map(|s| Lane { from: *s, to: *s, half_width: SPOT_TAKEN });
        self.own
            .values()
            .filter(|u| self.role_of_def(u.def) == Some(Role::Factory))
            .flat_map(|plant| [exit_lane(plant), Lane { from: plant.pos, to: plant.pos, half_width: FACTORY_YARD }])
            .chain(pending)
            .chain(spots)
            .collect()
    }

    /// One line per order given to a builder (the chain study, 2026-10-25: which orders walk a far constructor home):
    /// what, how far the builder stands from its target and from home, and the rule that gave it.
    fn note_order(&self, id: UnitId, what: &What, rule: &str) {
        let name = |d: UnitDefId| self.def(d).map_or("?", |d| d.name.as_str());
        let builder = self.own.get(&id);
        let (text, target) = match what {
            What::Build { def, near, .. } => (format!("build {} at ({:.0},{:.0})", name(*def), near.x, near.z), Some(*near)),
            What::Assist { factory, until } => {
                let verb = if *until - self.frame <= PAD_CLEAR { "pad-clear by" } else { "assist" };
                (format!("{verb} unit {} ({})", factory.0, self.own.get(factory).map_or("?", |f| name(f.def))), self.own.get(factory).map(|f| f.pos))
            }
            What::Help { frame } => (format!("help unit {} ({})", frame.0, self.own.get(frame).map_or("?", |f| name(f.def))), self.own.get(frame).map(|f| f.pos)),
            What::Walk { to } => (format!("walk home to ({:.0},{:.0})", to.x, to.z), Some(*to)),
        };
        let (off, home) = builder.map_or((0.0, 0.0), |b| (target.map_or(0.0, |t| b.pos.dist2d(t)), b.pos.dist2d(self.home)));
        self.log(format_args!("order: unit {} {}: {text}, {off:.0} off, {home:.0} from home ({rule})", id.0, builder.map_or("?", |b| name(b.def))));
    }

    /// The builders' orders counted by kind, for the jobs line.
    pub(crate) fn jobs_line(&self) -> String {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for (id, order) in &self.hands.orders {
            let role = self.own.get(id).and_then(|u| self.role_of_def(u.def));
            let kind = match role {
                Some(Role::Commander) => "com",
                Some(Role::Factory) => "fac",
                _ => "con",
            };
            let word = match &order.what {
                What::Build { def, frame, .. } => format!("{}{}", self.def(*def).map_or("?", |d| d.name.as_str()), if frame.is_some() { "" } else { "(walking)" }),
                What::Assist { .. } => "assist".to_string(),
                What::Help { frame } => format!("help:{}", self.own.get(frame).and_then(|f| self.def(f.def)).map_or("?", |d| d.name.as_str())),
                What::Walk { .. } => "walk".to_string(),
            };
            *counts.entry(format!("{kind}:{word}")).or_default() += 1;
        }
        counts.iter().map(|(k, n)| if *n > 1 { format!("{k} x{n}") } else { k.clone() }).collect::<Vec<_>>().join(" ")
    }
}
