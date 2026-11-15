//! The army: one declared group as an object that decides (`docs/design/2026-11-15-army-control-loop.md` §5).
//!
//! The army owns every decision: the facing, how many companies there are and who is in which (a re-form), each
//! company's block at the place (`layout.rs`), which company goes to which block, each company's own way there, and
//! who has which slot (the deal). A decision is a function of the order and the units at the moment it is taken and
//! stands until it has become impossible or its dwell has passed (§5.11). Between decisions a company is a frame (its
//! block of slots) at a distance `f` along its way: `f` is the only state that moves, it never decreases, one law
//! advances it (§5.6) and the enemy can only stop it (§5.9). Nothing writes a frame's way, end, facing or deal from
//! inside the tick, and no company reads another's state.
//!
//! What the loop must hold (§4): it converges under one order; a disturbance moves only what it touches; the response
//! is continuous in the order; the report is of the units; the state is small; decisions dwell; something always
//! moves; nothing is held back from a fight.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use bot_protocol::{UnitId, Vec3};

use crate::form::{self, Pitch};
use crate::ground::{BLOCK_COST, Ground};
use crate::group::{
    BLOCK_MARGIN, Cause, CompanyState, DEAL_DWELL, DEAL_GAIN, DEAL_GAIN_EACH, DWELL, FOLLOW_KEEP, FORM_AHEAD, GroupKind, GroupOrder, GroupState, MOVING, PARTY_SWITCH, Pacing, REFACE_DEG, RELAX_CLEAR,
    RELAX_STEP, REPLAN_MOVED, ROUTE_MOVED, WAY_GAIN, WAY_GAIN_ELMOS, Walker, debug_on, relax_on,
};
use crate::layout::{self, Block, ROW, Want};

/// The approach a facing is read from: the last stretch of a way planned from the army's centre to the place.
const APPROACH: f32 = 400.0;

/// The group a follower follows, as the lane read it at the last tick: its blocks at its place, the place and the
/// facing they were laid for, where its members are, and whether it has arrived.
#[derive(Clone, Debug)]
pub struct Ward {
    pub blocks: Vec<Block>,
    pub place: Vec3,
    pub facing: (f32, f32),
    pub centre: Vec3,
    pub arrived: bool,
}

/// What the lane hands an army each tick: the members as they are, the ground, the enemy, and the other groups'
/// decisions.
pub struct Ctx<'a> {
    pub frame: i32,
    pub walkers: &'a [Walker],
    /// The spacing the blast radii about ask for, and the shares of a hull a pitch keeps (`form::pitch_for`).
    pub spacing: f32,
    pub hull_across: f32,
    pub hull_deep: f32,
    /// The group's ground, and a number that changes when it has been rebuilt.
    pub ground: Option<&'a Arc<Ground>>,
    pub ground_version: u64,
    /// Armed enemies in sight (none for a walk), and everything hostile: those and his buildings seen or remembered.
    pub armed: &'a [(UnitId, Vec3)],
    pub hostile: &'a [Vec3],
    /// An attack's unit where it is seen or remembered, and the members of the order's party in sight.
    pub target: Option<Vec3>,
    pub party: &'a [(UnitId, Vec3)],
    /// The blocks the other groups hold.
    pub held: &'a [Block],
    pub ward: Option<&'a Ward>,
    pub pacing: &'a Pacing,
    /// The halt's margins: inside its reach by this much to halt, within the reach plus this much to stay halted.
    pub stand_inside: f32,
    pub stand_keep: f32,
}

/// A company: its frame, its block at the place, its way there and how far along it stands, and who has which slot.
#[derive(Clone, Debug)]
pub struct Company {
    pub frame: Block,
    pub stand: Block,
    way: Vec<Vec3>,
    way_from: Vec3,
    /// The way's length up to each of its points, and to each of the host's route points on it.
    cum: Vec<f32>,
    host_at: Vec<f32>,
    f: f32,
    /// The tight stretches of the way: where each begins and its forming-up ground, along the way.
    gaps: Vec<(f32, f32)>,
    /// A stretch being walked in ranks (something hostile was near when the frame came to it) ends here.
    in_ranks_to: f32,
    no_way: bool,
    /// Each member's slot: the one book by unit.
    pub slot_of: HashMap<UnitId, usize>,
    /// The slots open to members: the first `open` of the block.
    pub open: usize,
    pub halted: bool,
    /// Waiting beyond a tight stretch for its body to come through.
    pub passing: bool,
    stand_whole: bool,
    dealt_at: i32,
    pace_full: f32,
    /// This tick's slots, on the ground, and the points the members are sent to.
    slots: HashMap<UnitId, Vec3>,
    goals: HashMap<UnitId, Vec3>,
}

fn dot(a: (f32, f32), b: (f32, f32)) -> f32 {
    a.0 * b.0 + a.1 * b.1
}

fn sub(a: Vec3, b: Vec3) -> (f32, f32) {
    (a.x - b.x, a.z - b.z)
}

fn length_of(from: Vec3, way: &[Vec3]) -> f32 {
    let mut at = from;
    let mut total = 0.0;
    for p in way {
        total += at.dist2d(*p);
        at = *p;
    }
    total
}

/// Whether a walker is of the pace type: the slowest, within a per cent.
fn pace_type(w: &Walker, slowest: f32) -> bool {
    slowest.is_finite() && w.speed <= slowest * 1.01
}

/// The way from `from` to `to` over the ground, priced with the blocks that stand by decision (other groups' and
/// this army's other companies'): a block is walked round when the way round is about a block's width longer, and
/// through otherwise; never forbidden. `[to]` with no ground, `None` when the ground gives no way. (The design's
/// §5.4 also priced a band about the ways already planned for the army's other companies. Not built: on open ground
/// the way is straightened to a line whatever the band, and where the ground has two corridors a band sends the
/// companies down different ones, an army arriving from two sides.)
fn plan(ground: Option<&Arc<Ground>>, from: Vec3, to: Vec3, priced: &[Block]) -> Option<Vec<Vec3>> {
    let Some(g) = ground else { return Some(vec![to]) };
    let blocks: Vec<&Block> = priced.iter().filter(|b| !b.contains(to, BLOCK_MARGIN) && !b.contains(from, BLOCK_MARGIN)).collect();
    if blocks.is_empty() {
        return g.way_found(from, to);
    }
    let mut costs = g.costs().to_vec();
    for b in &blocks {
        let (across, along) = b.cell();
        for (i, c) in g.cells_within(b.centre, across.hypot(along) / 2.0 + BLOCK_MARGIN) {
            if b.contains(c, BLOCK_MARGIN) && costs[i] != terrain::IMPASSABLE {
                costs[i] = costs[i].saturating_add(BLOCK_COST);
            }
        }
    }
    g.way_over(&costs, from, to, &|_, _| true)
}

impl Company {
    fn new(frame: Block, open: usize, frame_no: i32) -> Company {
        Company {
            frame,
            stand: frame,
            way: Vec::new(),
            way_from: frame.centre,
            cum: Vec::new(),
            host_at: Vec::new(),
            f: 0.0,
            gaps: Vec::new(),
            in_ranks_to: 0.0,
            no_way: false,
            slot_of: HashMap::new(),
            open,
            halted: false,
            passing: false,
            stand_whole: true,
            dealt_at: frame_no,
            pace_full: 0.0,
            slots: HashMap::new(),
            goals: HashMap::new(),
        }
    }

    /// The way's length.
    fn length(&self) -> f32 {
        self.cum.last().copied().unwrap_or(0.0)
    }

    /// The way's point `f` along it and its direction there (the facing with no way to speak of).
    fn point(&self, f: f32) -> (Vec3, (f32, f32)) {
        let mut from = self.way_from;
        let mut walked = 0.0f32;
        let mut direction = self.frame.facing;
        for (k, &to) in self.way.iter().enumerate() {
            let leg = self.cum[k] - walked;
            if leg >= 1.0 {
                direction = form::heading(from, to);
                if f < self.cum[k] - 0.5 {
                    let t = ((f - walked) / leg).clamp(0.0, 1.0);
                    return (Vec3 { x: from.x + (to.x - from.x) * t, y: to.y, z: from.z + (to.z - from.z) * t }, direction);
                }
            }
            walked = self.cum[k];
            from = to;
        }
        (from, direction)
    }

    fn at_end(&self) -> bool {
        self.f >= self.length() - 0.5
    }

    /// The frame put where its body is (§5.5): the median by coordinate of where each member's position says the
    /// frame's centre would be.
    fn seat(&mut self, by_id: &HashMap<UnitId, &Walker>) {
        let centre = self.frame.centre;
        let implied = self.slot_of.iter().filter_map(|(id, i)| {
            let (w, s) = (by_id.get(id)?, self.frame.slot(*i));
            Some(Vec3 { x: w.pos.x - (s.x - centre.x), y: 0.0, z: w.pos.z - (s.z - centre.z) })
        });
        if let Some(at) = form::median(implied) {
            self.frame.centre = Vec3 { y: centre.y, ..at };
        }
    }

    /// The deal afresh (§5.7): the members to the open slots by the least total squared travel from where they
    /// stand. Returns the total before and after.
    fn best_deal(&self, by_id: &HashMap<UnitId, &Walker>) -> (Vec<(UnitId, usize)>, f32, f32) {
        let members: Vec<(UnitId, Vec3, usize)> = self.slot_of.iter().filter_map(|(id, i)| Some((*id, by_id.get(id)?.pos, *i))).collect();
        let slots: Vec<Vec3> = (0..self.open.max(members.len())).map(|i| self.frame.slot(i)).collect();
        let units: Vec<Vec3> = members.iter().map(|m| m.1).collect();
        let deal = form::assign_some(&units, &slots);
        let square = |p: Vec3, s: Vec3| p.dist2d(s).powi(2);
        let before: f32 = members.iter().map(|m| square(m.1, self.frame.slot(m.2))).sum();
        let after: f32 = members.iter().zip(&deal).map(|(m, s)| square(m.1, slots[*s])).sum();
        (members.iter().zip(deal).map(|(m, s)| (m.0, s)).collect(), before, after)
    }

    fn deal(&mut self, by_id: &HashMap<UnitId, &Walker>, frame_no: i32) {
        let (deal, ..) = self.best_deal(by_id);
        for (id, slot) in deal {
            self.slot_of.insert(id, slot);
        }
        self.dealt_at = frame_no;
    }

    /// The free slot nearest `pos`: an open one nobody has, or the next one to open while the cap allows.
    fn free_slot(&self, pos: Vec3, cap: usize) -> Option<(f32, usize)> {
        let taken: HashSet<usize> = self.slot_of.values().copied().collect();
        let opens = cap == 0 || self.open < cap;
        (0..self.open + usize::from(opens)).filter(|i| !taken.contains(i)).map(|i| (self.frame.slot(i).dist2d(pos), i)).min_by(|a, b| a.0.total_cmp(&b.0))
    }

    /// The way from where the frame is through the host's points to the block (§5.4), each point but the last passed
    /// by the block's offset from the place.
    fn set_way(&mut self, route: &[Vec3], place: Vec3, ground: Option<&Arc<Ground>>, priced: &[Block]) {
        let offset = sub(self.stand.centre, place);
        self.way_from = self.frame.centre;
        self.way.clear();
        self.cum.clear();
        self.host_at.clear();
        self.no_way = false;
        let mut at = self.way_from;
        let mut walked = 0.0f32;
        let last = route.len().saturating_sub(1);
        for (i, p) in route.iter().enumerate() {
            let to = if i == last { self.stand.centre } else { Vec3 { x: p.x + offset.0, y: p.y, z: p.z + offset.1 } };
            let mut leg = plan(ground, at, to, priced).unwrap_or_else(|| {
                self.no_way = true;
                vec![to]
            });
            // The planner ends on a cell's centre: the leg ends on the point itself.
            if let Some(end) = leg.last_mut() {
                *end = to;
            }
            for q in leg {
                walked += at.dist2d(q);
                self.way.push(q);
                self.cum.push(walked);
                at = q;
            }
            self.host_at.push(walked);
        }
        if route.is_empty() {
            walked += at.dist2d(self.stand.centre);
            self.way.push(self.stand.centre);
            self.cum.push(walked);
            self.host_at.push(walked);
        }
        self.f = 0.0;
        self.in_ranks_to = 0.0;
        self.halted = false;
        self.passing = false;
    }

    /// The tight stretches of the way (§5.4): where fewer than the share of the block's open slots stand on ground
    /// joined to the frame's centre, sampled every `RELAX_STEP`; samples nearer than a block's depth are one stretch.
    /// Each stretch's forming-up ground is the first place beyond it where the block stands whole and clear of the
    /// grounds already taken (`forming`), the way's end for a stretch that reaches it.
    fn find_gaps(&mut self, ground: Option<&Arc<Ground>>, share: f32, forming: &mut Vec<Block>) {
        self.gaps.clear();
        let Some(g) = ground else { return };
        let total = self.length();
        if !relax_on() || self.no_way || total < 1.0 || self.open == 0 {
            return;
        }
        let need = ((share * self.open as f32).ceil() as usize).max(1);
        let last = (total / RELAX_STEP).ceil() as usize;
        let at = |i: usize| (i as f32 * RELAX_STEP).min(total);
        let block_at = |s: f32| self.frame.at(self.point(s).0);
        let tight: Vec<bool> = (0..=last).map(|i| block_at(at(i)).standing(g, self.open) < need).collect();
        let depth = self.frame.cell().1;
        let joined = ((depth + 96.0).max(192.0) / RELAX_STEP).ceil() as usize;
        let mut runs: Vec<(usize, usize)> = Vec::new();
        for i in (0..=last).filter(|i| tight[*i]) {
            match runs.last_mut() {
                Some((_, end)) if i - *end <= joined => *end = i,
                _ => runs.push((i, i)),
            }
        }
        let mut gaps = Vec::new();
        for (a, b) in runs {
            let clear = b + (depth / RELAX_STEP).ceil() as usize;
            let limit = (clear + (FORM_AHEAD / RELAX_STEP) as usize).min(last);
            let form_at = if b >= last || clear >= last {
                total
            } else {
                (clear..=limit)
                    .find(|i| {
                        let block = block_at(at(*i));
                        block.standing(g, self.open) == self.open && !forming.iter().any(|f| f.overlaps(&block, BLOCK_MARGIN))
                    })
                    .map_or(if limit >= last { total } else { at(limit) }, at)
            };
            forming.push(block_at(form_at));
            gaps.push((at(a), form_at));
        }
        self.gaps = gaps;
    }

    /// One tick of the frame (§5.5, §5.6, §5.9): the hold from the members' lags, a jump over a tight stretch, the
    /// step limited by the enemy and a ward, then every member's slot on the ground and the point it is sent to.
    fn tick(&mut self, name: &str, index: usize, kind: GroupKind, by_id: &HashMap<UnitId, &Walker>, ctx: &Ctx, dt: f32) {
        let pacing = ctx.pacing;
        let ground = ctx.ground;
        let members: Vec<(&Walker, usize)> = self.slot_of.iter().filter_map(|(id, i)| Some((*by_id.get(id)?, *i))).collect();
        let slowest = members.iter().map(|(w, _)| w.speed).filter(|s| *s > 0.0).fold(f32::INFINITY, f32::min);
        self.pace_full = if slowest.is_finite() { slowest * pacing.fraction } else { 0.0 };
        let total = self.length();
        let (_, travel) = self.point(self.f);
        // The hold (§5.6): the lag of the member `share` of the way up the list of live members; a member that
        // stands out of its slot, or is under a reflex, is not live and holds nothing.
        let stop_at = pacing.exception_at * (self.frame.pitch.across / form::HULL_SPACING).max(1.0);
        let mut lags: Vec<f32> = (members.iter())
            .filter_map(|(w, i)| {
                let slot = self.frame.slot(*i);
                let live = !w.reflex && (w.pos.dist2d(slot) <= pacing.formed_within || w.moving >= MOVING);
                live.then(|| dot(sub(slot, w.pos), travel))
            })
            .collect();
        lags.sort_by(f32::total_cmp);
        let hold = if lags.is_empty() {
            1.0
        } else {
            let k = ((pacing.share * lags.len() as f32).ceil() as usize).clamp(1, lags.len()) - 1;
            ((stop_at - lags[k]) / (stop_at - pacing.drag_from).max(1.0)).clamp(0.0, 1.0)
        };
        if self.passing && hold >= 1.0 {
            self.passing = false;
        }
        let mut step = 0.0f32;
        if self.at_end() {
            self.frame.centre = self.stand.centre;
            self.halted = false;
        } else {
            // A tight stretch is a gap in the frame's progress (§5.5): the frame is set on the forming-up ground and
            // the hold keeps it there until its body is through. A group that fights passes relaxed only with
            // nothing hostile near the stretch; else it walks the stretch in ranks.
            let gap = self.gaps.iter().copied().find(|(from, form_at)| self.f >= from - RELAX_STEP && self.f < form_at - 0.5);
            let mut jumped = false;
            if let Some((_, form_at)) = gap
                && self.f >= self.in_ranks_to
            {
                let clear = kind == GroupKind::Go || {
                    let mut s = self.f;
                    let mut near = false;
                    while s <= form_at && !near {
                        let p = self.point(s).0;
                        near = ctx.hostile.iter().any(|h| h.dist2d(p) < RELAX_CLEAR);
                        s += 2.0 * RELAX_STEP;
                    }
                    !near
                };
                if clear {
                    if debug_on() {
                        let to = self.point(form_at).0;
                        eprintln!("[group {name}.{index} f={}] PASS of {}: from ({:.0},{:.0}) to form at ({:.0},{:.0}), {:.0} along a way of {total:.0}", ctx.frame, members.len(), self.frame.centre.x, self.frame.centre.z, to.x, to.z, form_at);
                    }
                    self.f = form_at.min(total);
                    self.passing = true;
                    jumped = true;
                } else {
                    self.in_ranks_to = form_at;
                }
            }
            if !jumped {
                step = self.pace_full * hold * dt;
                if kind != GroupKind::Go && !ctx.armed.is_empty() {
                    let nearest = |p: Vec3| ctx.armed.iter().map(|(_, e)| e.dist2d(p)).fold(f32::INFINITY, f32::min);
                    // The halt (§5.9): every slot whose member is armed has an armed enemy within that member's
                    // reach. Read on the slots, so no straggler decides it.
                    let keep = if self.halted { ctx.stand_keep } else { -ctx.stand_inside };
                    let armed: Vec<(&Walker, Vec3)> = members.iter().filter(|(w, _)| w.reach > 0.0).map(|(w, i)| (*w, self.frame.slot(*i))).collect();
                    self.halted = !armed.is_empty() && armed.iter().all(|(w, slot)| nearest(*slot) < w.reach + keep);
                    if self.halted {
                        step = 0.0;
                    } else if step > 0.0 {
                        // The front's guard: no step that brings a slot nearer an armed enemy than the stop.
                        let shortest = armed.iter().map(|(w, _)| w.reach).fold(f32::INFINITY, f32::min);
                        if shortest.is_finite() {
                            let stop = shortest * pacing.close_to;
                            let moved = (travel.0 * step, travel.1 * step);
                            let blocked = members.iter().any(|(_, i)| {
                                let now = self.frame.slot(*i);
                                let next = Vec3 { x: now.x + moved.0, y: now.y, z: now.z + moved.1 };
                                ctx.armed.iter().any(|(_, e)| e.dist2d(next) < stop && e.dist2d(next) < e.dist2d(now))
                            });
                            if blocked {
                                step = 0.0;
                            }
                        }
                    }
                } else {
                    self.halted = false;
                }
                // A follower stays behind its ward while the ward marches.
                if let Some(ward) = ctx.ward
                    && !ward.arrived
                {
                    let ahead = dot(sub(ward.centre, self.frame.centre), travel);
                    if ahead > 0.0 {
                        step = step.min((ahead - FOLLOW_KEEP).max(0.0));
                    }
                }
                self.f = (self.f + step).min(total);
            }
            self.frame.centre = if self.at_end() { self.stand.centre } else { self.point(self.f).0 };
        }
        // The slots on the ground, and where each member is sent: the slowest type its slot carried ahead along the
        // travel by the lead times the pace's share (nothing while the frame stands), the others their slot.
        let share = if self.pace_full > 0.0 && dt > 0.0 { (step / dt / self.pace_full).clamp(0.0, 1.0) } else { 0.0 };
        let left = total - self.f;
        let (_, travel) = self.point(self.f);
        self.slots.clear();
        self.goals.clear();
        for (w, i) in &members {
            let nominal = self.frame.slot(*i);
            let slot = ground.and_then(|g| g.snap(nominal)).unwrap_or(nominal);
            let lead = if share > 0.0 && pace_type(w, slowest) && pacing.lead > 0.0 { pacing.lead * share * (left / pacing.lead).clamp(0.0, 1.0) } else { 0.0 };
            let goal = if lead > 0.0 {
                let carried = Vec3 { x: slot.x + travel.0 * lead, y: slot.y, z: slot.z + travel.1 * lead };
                match ground {
                    Some(g) => g.snap(carried).unwrap_or(slot),
                    None => carried,
                }
            } else {
                slot
            };
            self.slots.insert(w.id, slot);
            self.goals.insert(w.id, goal);
        }
    }
}

/// One declared group's army.
#[derive(Default)]
pub struct Army {
    pub companies: Vec<Company>,
    /// The order the decisions in force were taken for, and the place its layout is laid at.
    order: Option<GroupOrder>,
    place: Option<Vec3>,
    /// The way the ranks face, and the frame it was last changed.
    facing: Option<(f32, f32)>,
    faced_at: Option<i32>,
    /// The cap the companies were made under, and the frame they were last re-formed.
    cap: Option<usize>,
    reformed_at: i32,
    /// The member of the order's party the place is on.
    party_target: Option<UnitId>,
    ground_version: u64,
    /// The rows behind the place this army's first row stands (a follower's).
    first_row: usize,
    /// A decision is due at the next tick whatever the order: a change of facing whose dwell has passed.
    turn_due: Option<i32>,
    last_frame: Option<i32>,
    pub state: Option<GroupState>,
}

impl Army {
    /// The blocks this army holds at its place.
    pub fn blocks(&self) -> Vec<Block> {
        self.companies.iter().map(|c| c.stand).collect()
    }

    /// What a follower needs of this army.
    pub fn as_ward(&self) -> Option<Ward> {
        let state = self.state.as_ref()?;
        Some(Ward { blocks: self.blocks(), place: self.place?, facing: self.facing?, centre: state.centre, arrived: state.arrived })
    }

    fn pitch_of(hulls: impl Iterator<Item = f32>, ctx: &Ctx) -> Pitch {
        let mut hulls: Vec<f32> = hulls.collect();
        hulls.sort_by(|a, b| b.total_cmp(a));
        form::pitch_for(ctx.spacing, form::widest_pair(&hulls), ctx.hull_across, ctx.hull_deep)
    }

    /// One tick: the members taken in, the decisions that are due, every company's frame, the report. Returns the
    /// point each member is sent to.
    pub fn step(&mut self, name: &str, members: &[UnitId], order: &GroupOrder, ctx: &Ctx) -> HashMap<UnitId, Vec3> {
        let by_id: HashMap<UnitId, &Walker> = ctx.walkers.iter().map(|w| (w.id, w)).collect();
        let present: Vec<UnitId> = members.iter().copied().filter(|id| by_id.contains_key(id)).collect();
        if present.is_empty() {
            self.state = None;
            return HashMap::new();
        }
        let dt = self.last_frame.map_or(0.0, |f| (ctx.frame - f).max(0) as f32 / 30.0);
        self.last_frame = Some(ctx.frame);
        // Who left: its slot is free and nobody moves (§5.7). A company of nobody is gone.
        let set: HashSet<UnitId> = present.iter().copied().collect();
        for c in &mut self.companies {
            c.slot_of.retain(|id, _| set.contains(id));
        }
        self.companies.retain(|c| !c.slot_of.is_empty());
        // The re-form (§5.7): founded, the cap changed, or a company's worth of slots stands free.
        let (count, cap) = (self.companies.len(), order.cap);
        let spare = cap > 0 && count > 1 && present.len() <= (count - 1) * cap && ctx.frame - self.reformed_at >= DWELL && !self.companies.iter().any(|c| c.halted);
        if self.companies.is_empty() || self.cap != Some(cap) || spare {
            self.reform(name, &present, &by_id, order, ctx);
        } else {
            self.take_in(name, &present, &by_id, order, ctx);
        }
        // The order (§5.8).
        let place = self.place_of(order, &present, &by_id, ctx);
        let turn_due = self.turn_due.is_some_and(|at| ctx.frame >= at);
        if self.is_new(order, place) || turn_due {
            self.decide(name, order, place, &by_id, ctx);
        } else if let Some(mine) = &mut self.order {
            // The same place: the kind and the party may have changed, and move nobody.
            mine.kind = order.kind;
            mine.party = order.party.clone();
        }
        // The ground changed (§5.4): a way stands unless it is blocked or beaten, a block unless it no longer stands.
        if ctx.ground_version != self.ground_version {
            self.ground_version = ctx.ground_version;
            self.ground_changed(name, ctx);
        }
        let kind = order.kind;
        let mut goals = HashMap::new();
        for (i, c) in self.companies.iter_mut().enumerate() {
            c.tick(name, i, kind, &by_id, ctx, dt);
            goals.extend(c.goals.iter().map(|(id, p)| (*id, *p)));
        }
        // A deal on gain (§5.7), once a second a company: taken when it lowers the total squared travel by a quarter.
        for (i, c) in self.companies.iter_mut().enumerate() {
            if (ctx.frame / 3 + i as i32) % 10 != 0 || ctx.frame - c.dealt_at < DEAL_DWELL || c.passing {
                continue;
            }
            let (deal, before, after) = c.best_deal(&by_id);
            if before - after >= DEAL_GAIN * before && before - after >= DEAL_GAIN_EACH * DEAL_GAIN_EACH * deal.len() as f32 {
                if debug_on() {
                    let moved = deal.iter().filter(|(id, s)| c.slot_of.get(id) != Some(s)).count();
                    eprintln!("[group {name}.{i} f={}] DEAL on gain: {moved} of {} take another slot, the total squared travel {before:.0} to {after:.0}", ctx.frame, deal.len());
                }
                c.slot_of = deal.into_iter().collect();
                c.dealt_at = ctx.frame;
            }
        }
        self.state = Some(self.report(order, &present, &by_id, ctx));
        goals
    }

    /// `present` as `count` companies facing `facing` (`reform`): the body halved into compact parts, each part's
    /// frame seated on its members and they dealt to its slots by the least travel; and the total squared travel
    /// of that deal.
    fn companies_for(present: &[UnitId], by_id: &HashMap<UnitId, &Walker>, count: usize, cap: usize, facing: (f32, f32), ctx: &Ctx) -> (Vec<Company>, f32) {
        let mut parts: Vec<Vec<UnitId>> = Vec::new();
        split(present.to_vec(), count, cap, facing, by_id, &mut parts);
        let mut companies: Vec<Company> = Vec::new();
        let mut total = 0.0f32;
        for part in &parts {
            let pitch = Self::pitch_of(part.iter().map(|id| by_id[id].hull), ctx);
            let middle = form::median(part.iter().map(|id| by_id[id].pos)).unwrap_or_default();
            let frame = Block { centre: middle, facing, pitch, ranks: Block::ranks_for(cap, part.len()) };
            let mut company = Company::new(frame, part.len(), ctx.frame);
            let slots: Vec<Vec3> = (0..part.len()).map(|i| frame.slot(i)).collect();
            let units: Vec<Vec3> = part.iter().map(|id| by_id[id].pos).collect();
            for (id, slot) in part.iter().zip(form::assign(&units, &slots)) {
                company.slot_of.insert(*id, slot);
            }
            company.seat(by_id);
            total += part.iter().map(|id| by_id[id].pos.dist2d(company.frame.slot(company.slot_of[id])).powi(2)).sum::<f32>();
            companies.push(company);
        }
        (companies, total)
    }

    /// The companies made afresh (§5.7): the fewest that hold the members, as even as the cap allows, each a compact
    /// part of the body as it stands (the body halved across its longer side until the parts are companies), its
    /// frame seated on its own members and they dealt to its slots by the least travel. Nobody walks to a lattice:
    /// where the companies stand apart is the layout's at the place (the first build laid a row of blocks at the
    /// body, and a column of ninety-six walked 700 sideways to line up before it set out). The order is to be
    /// decided again.
    fn reform(&mut self, name: &str, present: &[UnitId], by_id: &HashMap<UnitId, &Walker>, order: &GroupOrder, ctx: &Ctx) {
        let (n, cap) = (present.len(), order.cap);
        let count = if cap == 0 { 1 } else { n.div_ceil(cap) };
        let centre = form::median(present.iter().map(|id| by_id[id].pos)).unwrap_or_default();
        // The facing in force; with none (founded, or made again), the approach to the order's place; with the place
        // too near for one, the facing the body stands in: the one in which its members are nearest slots of their
        // own (§4 P5: an army made again where it stands reads its facing from its units and moves nobody).
        let facing = self.facing.or_else(|| order.route.last().and_then(|p| Self::approach(centre, *p, ctx.ground))).unwrap_or_else(|| {
            let cost = |degrees: f32| Self::companies_for(present, by_id, count, cap, (degrees.to_radians().cos(), degrees.to_radians().sin()), ctx).1;
            let mut best = (0..16).map(|k| k as f32 * 22.5).map(|d| (cost(d), d)).min_by(|a, b| a.0.total_cmp(&b.0)).unwrap_or((0.0, 0.0));
            let mut step = 11.25f32;
            while step >= 1.0 {
                for d in [best.1 - step, best.1 + step] {
                    let c = cost(d);
                    if c < best.0 {
                        best = (c, d);
                    }
                }
                step /= 2.0;
            }
            (best.1.to_radians().cos(), best.1.to_radians().sin())
        });
        let (companies, _) = Self::companies_for(present, by_id, count, cap, facing, ctx);
        if debug_on() {
            eprintln!(
                "[group {name} f={}] RE-FORM: {n} members in {count} companies at ({:.0},{:.0}) facing ({:.2},{:.2}): {}",
                ctx.frame,
                centre.x,
                centre.z,
                facing.0,
                facing.1,
                companies.iter().map(|c| format!("{} at ({:.0},{:.0})", c.slot_of.len(), c.frame.centre.x, c.frame.centre.z)).collect::<Vec<_>>().join(", ")
            );
        }
        self.companies = companies;
        self.facing = Some(facing);
        self.cap = Some(cap);
        self.reformed_at = ctx.frame;
        self.order = None;
    }

    /// The newcomers (§5.7): each takes the free slot nearest it and nobody else moves; with no slot free a block is
    /// added beside the layout and a company founded on the newcomers.
    fn take_in(&mut self, name: &str, present: &[UnitId], by_id: &HashMap<UnitId, &Walker>, order: &GroupOrder, ctx: &Ctx) {
        let have: HashSet<UnitId> = self.companies.iter().flat_map(|c| c.slot_of.keys().copied()).collect();
        let cap = order.cap;
        for id in present.iter().filter(|id| !have.contains(id)) {
            let pos = by_id[id].pos;
            let free = (self.companies.iter().enumerate()).filter_map(|(k, c)| c.free_slot(pos, cap).map(|(d, i)| (d, k, i))).min_by(|a, b| a.0.total_cmp(&b.0));
            if let Some((_, k, i)) = free {
                let c = &mut self.companies[k];
                c.slot_of.insert(*id, i);
                if i >= c.open {
                    c.open = i + 1;
                    // With no cap the block grows by a rank when its last one is full.
                    while c.open > c.frame.capacity() {
                        c.frame.ranks += 1;
                        c.stand.ranks += 1;
                    }
                }
                continue;
            }
            // Every slot taken: one more block beside the layout, its frame where the newcomer stands.
            let facing = self.facing.unwrap_or((1.0, 0.0));
            let want = Want { ranks: Block::ranks_for(cap, 1), pitch: Self::pitch_of(std::iter::once(by_id[id].hull), ctx), open: 1 };
            let taken: Vec<Block> = self.blocks().into_iter().chain(ctx.held.iter().copied()).collect();
            let stand = match self.place {
                Some(place) => layout::lay_more(place, facing, want, self.companies.len(), self.first_row, ctx.ground.map(|g| g.as_ref()), &taken),
                None => Block { centre: pos, facing, pitch: want.pitch, ranks: want.ranks },
            };
            let mut company = Company::new(stand.at(pos), 1, ctx.frame);
            company.stand = stand;
            company.slot_of.insert(*id, 0);
            company.seat(by_id);
            let place = self.place.unwrap_or(stand.centre);
            company.set_way(&[place], place, ctx.ground, &taken);
            let mut forming: Vec<Block> = Vec::new();
            company.find_gaps(ctx.ground, ctx.pacing.share, &mut forming);
            if debug_on() {
                eprintln!("[group {name} f={}] FOUNDED a company on a newcomer at ({:.0},{:.0}); now {}", ctx.frame, pos.x, pos.z, self.companies.len() + 1);
            }
            self.companies.push(company);
        }
    }

    /// The place the order names now: a follower's is its ward's, an attack's its target where seen, a party's the
    /// member nearest the army (kept until it is gone or another is nearer by `PARTY_SWITCH`), else the route's end.
    fn place_of(&mut self, order: &GroupOrder, present: &[UnitId], by_id: &HashMap<UnitId, &Walker>, ctx: &Ctx) -> Option<Vec3> {
        if let Some(ward) = ctx.ward {
            return Some(ward.place);
        }
        if let GroupKind::Attack(_) = order.kind
            && let Some(at) = ctx.target
        {
            return Some(at);
        }
        if !order.party.is_empty() && !ctx.party.is_empty() {
            let centre = form::median(present.iter().map(|id| by_id[id].pos)).unwrap_or_default();
            let nearest = ctx.party.iter().min_by(|a, b| a.1.dist2d(centre).total_cmp(&b.1.dist2d(centre))).copied();
            let current = self.party_target.and_then(|id| ctx.party.iter().find(|p| p.0 == id).copied());
            let target = match (current, nearest) {
                (Some(c), Some(n)) if n.1.dist2d(centre) + PARTY_SWITCH < c.1.dist2d(centre) => Some(n),
                (Some(c), _) => Some(c),
                (None, n) => n,
            };
            self.party_target = target.map(|t| t.0);
            if let Some((_, at)) = target {
                return Some(at);
            }
        }
        order.route.last().copied()
    }

    /// Whether the order asks for new decisions (§5.8): none in force; another route shape, march or ward; or a place
    /// that moved (an attack's target by `REPLAN_MOVED`, else by `ROUTE_MOVED`). A hold whose point lies inside the
    /// layout in force names no new place.
    fn is_new(&self, order: &GroupOrder, place: Option<Vec3>) -> bool {
        let (Some(mine), Some(at)) = (&self.order, self.place) else { return true };
        let Some(place) = place else { return false };
        if order.march != mine.march || order.ward != mine.ward {
            return true;
        }
        let follows = matches!(order.kind, GroupKind::Attack(_)) || !order.party.is_empty() || order.ward.is_some();
        if !follows && (order.route.len() != mine.route.len() || order.route.iter().zip(&mine.route).any(|(a, b)| a.dist2d(*b) > ROUTE_MOVED)) {
            return !(order.kind == GroupKind::Hold && self.companies.iter().any(|c| c.stand.contains(place, BLOCK_MARGIN)));
        }
        follows && place.dist2d(at) > REPLAN_MOVED
    }

    /// The approach to a place from the army's centre: the direction of the last `APPROACH` of a way planned there;
    /// `None` for a place nearer than that.
    fn approach(centre: Vec3, place: Vec3, ground: Option<&Arc<Ground>>) -> Option<(f32, f32)> {
        let way = ground.and_then(|g| g.way_found(centre, place)).unwrap_or_else(|| vec![place]);
        if length_of(centre, &way) < APPROACH {
            return None;
        }
        let points: Vec<Vec3> = std::iter::once(centre).chain(way).collect();
        let end = *points.last()?;
        let mut left = APPROACH;
        for pair in points.windows(2).rev() {
            let leg = pair[0].dist2d(pair[1]);
            if leg >= left {
                let t = left / leg;
                let from = Vec3 { x: pair[1].x + (pair[0].x - pair[1].x) * t, y: 0.0, z: pair[1].z + (pair[0].z - pair[1].z) * t };
                return Some(form::heading(from, end));
            }
            left -= leg;
        }
        Some(form::heading(centre, end))
    }

    /// The decisions an order takes (§5.8): every frame seated on its body, the facing (kept unless the approach is
    /// across it and its dwell has passed), each company's pitch, the layout at the place, the pairing of companies
    /// with blocks by the least total travel, each company's way, the tight stretches and their forming-up grounds.
    fn decide(&mut self, name: &str, order: &GroupOrder, place: Option<Vec3>, by_id: &HashMap<UnitId, &Walker>, ctx: &Ctx) {
        let centre = form::median(self.companies.iter().flat_map(|c| c.slot_of.keys()).filter_map(|id| by_id.get(id)).map(|w| w.pos)).unwrap_or_default();
        let place = place.or(self.place).unwrap_or(centre);
        for c in &mut self.companies {
            c.seat(by_id);
        }
        // The facing.
        self.turn_due = None;
        let old = self.facing.unwrap_or((1.0, 0.0));
        let mut facing = old;
        if let Some(approach) = Self::approach(centre, place, ctx.ground) {
            let across = dot(old, approach).abs() < REFACE_DEG.to_radians().cos();
            match self.faced_at {
                None => {
                    facing = approach;
                    self.faced_at = Some(ctx.frame);
                }
                Some(at) if across && ctx.frame - at >= DWELL => {
                    facing = approach;
                    self.faced_at = Some(ctx.frame);
                }
                Some(at) if across => self.turn_due = Some(at + DWELL),
                Some(_) => {}
            }
        }
        let turned = dot(facing, old) < 0.999;
        self.facing = Some(facing);
        for c in &mut self.companies {
            let pitch = Self::pitch_of(c.slot_of.keys().filter_map(|id| by_id.get(id)).map(|w| w.hull), ctx);
            c.frame.pitch = pitch;
            if turned {
                // The ranks turn: the frame on the body's centre, the deal afresh where the members stand.
                c.frame.facing = facing;
                if let Some(at) = form::median(c.slot_of.keys().filter_map(|id| by_id.get(id)).map(|w| w.pos)) {
                    c.frame.centre = at;
                }
                c.deal(by_id, ctx.frame);
            }
        }
        // The layout, a follower's in the rows behind its ward's.
        self.first_row = ctx.ward.map_or(0, |w| w.blocks.len().div_ceil(ROW));
        let wants: Vec<Want> = self.companies.iter().map(|c| Want { ranks: c.frame.ranks, pitch: c.frame.pitch, open: c.open }).collect();
        let blocks = layout::lay(place, facing, &wants, self.first_row, ctx.ground.map(|g| g.as_ref()), ctx.held);
        // The pairing: the least total travel from where the frames are (at most a handful of companies; past six,
        // each the nearest block left).
        let count = self.companies.len();
        let travel = |perm: &[usize]| -> f32 { perm.iter().enumerate().map(|(c, b)| self.companies[c].frame.centre.dist2d(blocks[*b].centre)).sum() };
        let mut pairing: Vec<usize> = (0..count).collect();
        if count <= 6 {
            let mut best = travel(&pairing);
            let mut perm = pairing.clone();
            permute(&mut perm, 0, &mut |p| {
                let t = travel(p);
                if t < best - 0.5 {
                    best = t;
                    pairing = p.to_vec();
                }
            });
        } else {
            let mut left: Vec<usize> = (0..count).collect();
            for c in 0..count {
                let at = self.companies[c].frame.centre;
                let k = (0..left.len()).min_by(|a, b| at.dist2d(blocks[left[*a]].centre).total_cmp(&at.dist2d(blocks[left[*b]].centre))).unwrap_or(0);
                pairing[c] = left.remove(k);
            }
        }
        // The ways: each priced with the blocks that stand.
        let route: Vec<Vec3> = if matches!(order.kind, GroupKind::Attack(_)) || !order.party.is_empty() || ctx.ward.is_some() || order.route.is_empty() { vec![place] } else { order.route.clone() };
        for c in 0..count {
            let stand = blocks[pairing[c]];
            let priced: Vec<Block> = (blocks.iter().enumerate()).filter(|(b, _)| *b != pairing[c]).map(|(_, b)| *b).chain(ctx.held.iter().copied()).collect();
            let company = &mut self.companies[c];
            company.stand = Block { ranks: company.frame.ranks, ..stand };
            company.stand_whole = ctx.ground.is_none_or(|g| company.stand.standing(g, company.open) == company.open);
            company.set_way(&route, place, ctx.ground, &priced);
            if !order.march {
                company.f = company.length();
            }
        }
        self.find_all_gaps(ctx);
        if debug_on() {
            eprintln!(
                "[group {name} f={}] ORDER to ({:.0},{:.0}) facing ({:.2},{:.2}){}: {}",
                ctx.frame,
                place.x,
                place.z,
                facing.0,
                facing.1,
                if turned { " (turned)" } else { "" },
                (self.companies.iter().enumerate()).map(|(i, c)| format!("{i}: {} to ({:.0},{:.0}), {:.0} of way, {} tight", c.slot_of.len(), c.stand.centre.x, c.stand.centre.z, c.length(), c.gaps.len())).collect::<Vec<_>>().join("; ")
            );
        }
        self.order = Some(order.clone());
        self.place = Some(place);
    }

    /// Every company's tight stretches, the forming-up grounds laid one beyond the other where companies share a
    /// stretch: the company nearest its first stretch takes the furthest ground (§5.4).
    fn find_all_gaps(&mut self, ctx: &Ctx) {
        let mut forming: Vec<Block> = Vec::new();
        for c in &mut self.companies {
            let mut none = Vec::new();
            c.find_gaps(ctx.ground, ctx.pacing.share, &mut none);
        }
        let mut order: Vec<usize> = (0..self.companies.len()).collect();
        order.sort_by(|a, b| {
            let first = |c: &Company| c.gaps.first().map_or(f32::INFINITY, |g| g.0 - c.f);
            first(&self.companies[*b]).total_cmp(&first(&self.companies[*a])).then(a.cmp(b))
        });
        // The furthest from its stretch first: it takes the ground nearest the stretch, and those before it go on.
        for i in order {
            self.companies[i].find_gaps(ctx.ground, ctx.pacing.share, &mut forming);
        }
    }

    /// The ground was rebuilt (§5.4). A block that stood whole and no longer does is laid again, the nearest place
    /// where it stands; a way with a structure on it is planned again; a fresh plan shorter than the way left by a
    /// fifth and 200 elmos is taken. Anything else stands. The tight stretches are found again on the ways.
    fn ground_changed(&mut self, name: &str, ctx: &Ctx) {
        let Some(g) = ctx.ground else { return };
        let Some(place) = self.place else { return };
        for i in 0..self.companies.len() {
            let others: Vec<Block> = (self.companies.iter().enumerate()).filter(|(k, _)| *k != i).map(|(_, c)| c.stand).chain(ctx.held.iter().copied()).collect();
            let c = &mut self.companies[i];
            let mut again = false;
            if c.stand_whole && c.stand.standing(g, c.open) < c.open {
                let moved = layout::settle(c.stand, c.open, Some(g.as_ref()), &others);
                c.stand_whole = moved.standing(g, c.open) == c.open;
                if moved.centre.dist2d(c.stand.centre) > 1.0 {
                    if debug_on() {
                        eprintln!("[group {name}.{i} f={}] BLOCK laid again: ({:.0},{:.0}) to ({:.0},{:.0})", ctx.frame, c.stand.centre.x, c.stand.centre.z, moved.centre.x, moved.centre.z);
                    }
                    c.stand = moved;
                    again = true;
                }
            }
            if c.at_end() && !again {
                continue;
            }
            // The way left: blocked by a structure, or beaten by a fresh plan.
            let left = c.length() - c.f;
            let mut s = c.f;
            let mut blocked = false;
            while s <= c.length() && !blocked {
                blocked = !g.passable_at(c.point(s).0);
                s += g.cell();
            }
            let beaten = !blocked && !again && plan(ctx.ground, c.frame.centre, c.stand.centre, &others).is_some_and(|fresh| {
                let fresh = length_of(c.frame.centre, &fresh);
                fresh <= left * (1.0 - WAY_GAIN) && left - fresh >= WAY_GAIN_ELMOS
            });
            if blocked || again || beaten {
                if debug_on() {
                    eprintln!("[group {name}.{i} f={}] WAY planned again ({}), {left:.0} was left", ctx.frame, if blocked { "a structure on it" } else if again { "its block moved" } else { "a shorter one" });
                }
                let was_passing = c.passing;
                c.set_way(&[place], place, ctx.ground, &others);
                c.passing = was_passing;
            }
        }
        // The stretches of the ways as they stand, kept from where each frame is.
        let at: Vec<f32> = self.companies.iter().map(|c| c.f).collect();
        self.find_all_gaps(ctx);
        for (c, f) in self.companies.iter_mut().zip(at) {
            c.gaps.retain(|g| g.1 > f + 0.5);
        }
    }

    /// The report (§5.10), from the units, the order and the blocks.
    fn report(&self, order: &GroupOrder, present: &[UnitId], by_id: &HashMap<UnitId, &Walker>, ctx: &Ctx) -> GroupState {
        let pacing = ctx.pacing;
        let centre = form::median(present.iter().map(|id| by_id[id].pos)).unwrap_or_default();
        let mut formed = Vec::new();
        let mut exceptions = Vec::new();
        let mut to_go: Vec<f32> = Vec::new();
        let mut companies = Vec::new();
        let mut at_place = 0usize;
        let mut passing: Option<usize> = None;
        for c in &self.companies {
            let left = c.length() - c.f;
            let (mut c_formed, mut c_at) = (0usize, 0usize);
            for id in c.slot_of.keys() {
                let Some(w) = by_id.get(id) else { continue };
                let off = c.slots.get(id).map_or(0.0, |s| s.dist2d(w.pos));
                to_go.push(left + off);
                if c.stand.contains(w.pos, pacing.formed_within) {
                    c_at += 1;
                }
                if off <= pacing.formed_within && !w.reflex {
                    formed.push(*id);
                    c_formed += 1;
                } else {
                    let cause = if w.reflex { Cause::Reflex } else if w.moving >= MOVING { Cause::Coming } else { Cause::Standing };
                    exceptions.push((*id, cause, if w.speed > 0.0 { off / w.speed } else { f32::INFINITY }));
                }
            }
            at_place += c_at;
            if c.passing {
                passing = Some(passing.unwrap_or(0) + c_formed);
            }
            let middle = form::median(c.slot_of.keys().filter_map(|id| by_id.get(id)).map(|w| w.pos)).unwrap_or(c.frame.centre);
            companies.push(CompanyState { centre: middle, block: c.stand.centre, members: c.slot_of.len(), formed: c_formed, at_place: c_at, to_go: left, halted: c.halted, passing: c.passing });
        }
        to_go.sort_by(f32::total_cmp);
        let mut speeds: Vec<f32> = present.iter().map(|id| by_id[id].moving).collect();
        speeds.sort_by(f32::total_cmp);
        // Contact, of the units: who has an armed enemy in reach, and the nearest such enemy to the group.
        let mut in_reach = 0usize;
        let mut engaged: Option<(f32, UnitId)> = None;
        if order.kind != GroupKind::Go {
            for id in present {
                let w = by_id[id];
                if w.reach <= 0.0 {
                    continue;
                }
                let nearest = ctx.armed.iter().map(|(e, p)| (p.dist2d(w.pos), *e)).min_by(|a, b| a.0.total_cmp(&b.0));
                if let Some((d, e)) = nearest.filter(|(d, _)| *d < w.reach + ctx.stand_keep) {
                    in_reach += 1;
                    let from_centre = ctx.armed.iter().find(|(x, _)| *x == e).map_or(d, |(_, p)| p.dist2d(centre));
                    if engaged.is_none_or(|(best, _)| from_centre < best) {
                        engaged = Some((from_centre, e));
                    }
                }
            }
        }
        let members = present.len();
        let needed = ((pacing.share * members as f32).ceil() as usize).max(1);
        let legs_left = (self.companies.iter()).map(|c| c.host_at.iter().rev().skip(1).filter(|at| **at > c.f + 0.5).count()).max().unwrap_or(0);
        GroupState {
            place: self.place,
            centre,
            facing: self.facing.unwrap_or((1.0, 0.0)),
            members,
            at_place,
            arrived: self.place.is_some() && at_place >= needed,
            to_go: if self.companies.iter().any(|c| c.no_way) { None } else { to_go.get(to_go.len() / 2).copied() },
            speed: speeds.get(speeds.len() / 2).copied().unwrap_or(0.0),
            full_pace: Some(self.companies.iter().map(|c| c.pace_full).filter(|p| *p > 0.0).fold(f32::INFINITY, f32::min)).filter(|p| p.is_finite()).unwrap_or(0.0),
            formed,
            exceptions,
            engaged: engaged.map(|(_, e)| e),
            in_reach,
            halted: self.companies.iter().filter(|c| c.halted).count(),
            legs_left,
            passing,
            companies,
        }
    }

    /// A member's slot this tick, on the ground (the tests' instrument).
    #[cfg(test)]
    pub(crate) fn slot(&self, id: UnitId) -> Option<Vec3> {
        self.companies.iter().find_map(|c| c.slots.get(&id).copied())
    }
}

/// `units` in `count` compact parts of at most `cap` each, as even as that allows: halved across the longer side of
/// their extent (along the facing or across it) until each part is one company.
fn split(mut units: Vec<UnitId>, count: usize, cap: usize, facing: (f32, f32), by_id: &HashMap<UnitId, &Walker>, out: &mut Vec<Vec<UnitId>>) {
    if count <= 1 || units.len() <= 1 {
        out.push(units);
        return;
    }
    let along = |id: &UnitId| by_id[id].pos.x * facing.0 + by_id[id].pos.z * facing.1;
    let across = |id: &UnitId| -by_id[id].pos.x * facing.1 + by_id[id].pos.z * facing.0;
    let extent = |f: &dyn Fn(&UnitId) -> f32| {
        let (lo, hi) = units.iter().map(f).fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| (lo.min(v), hi.max(v)));
        hi - lo
    };
    if extent(&along) > extent(&across) {
        units.sort_by(|a, b| along(b).total_cmp(&along(a)).then(a.0.cmp(&b.0)));
    } else {
        units.sort_by(|a, b| across(a).total_cmp(&across(b)).then(a.0.cmp(&b.0)));
    }
    let (first, n) = (count / 2, units.len());
    let second = count - first;
    // The first part's share, kept within what each side's companies can hold.
    let wanted = (n * first + count / 2) / count;
    let take = if cap == 0 { wanted } else { wanted.clamp(n.saturating_sub(second * cap), (first * cap).min(n)) };
    let rest = units.split_off(take);
    split(units, first, cap, facing, by_id, out);
    split(rest, second, cap, facing, by_id, out);
}

/// Every order of `items[at..]`, in place.
fn permute(items: &mut Vec<usize>, at: usize, visit: &mut dyn FnMut(&[usize])) {
    if at + 1 >= items.len() {
        visit(items);
        return;
    }
    for i in at..items.len() {
        items.swap(at, i);
        permute(items, at + 1, visit);
        items.swap(at, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    /// A plant of the simplest kind: every unit walks straight at the point it was last sent to at its speed, ten
    /// ticks a second; a unit in `stuck` never moves.
    struct Sim {
        units: Vec<Walker>,
        army: Army,
        frame: i32,
        goals: HashMap<UnitId, Vec3>,
        stuck: HashSet<UnitId>,
        armed: Vec<(UnitId, Vec3)>,
        held: Vec<Block>,
    }

    impl Sim {
        /// `n` units in ranks of twelve about `(x, z)`, 64 apart, at 60 a second with a reach of 200.
        fn new(n: usize, x: f32, z: f32) -> Sim {
            let units = (0..n)
                .map(|i| Walker { id: UnitId(i as i32 + 1), pos: at(x - (i / 12) as f32 * 64.0, z + (i % 12) as f32 * 64.0 - 352.0), speed: 60.0, moving: 0.0, reach: 200.0, hull: 32.0, reflex: false })
                .collect();
            Sim { units, army: Army::default(), frame: 0, goals: HashMap::new(), stuck: HashSet::new(), armed: Vec::new(), held: Vec::new() }
        }

        fn ids(&self) -> Vec<UnitId> {
            self.units.iter().map(|w| w.id).collect()
        }

        fn tick(&mut self, order: &GroupOrder) {
            self.frame += 3;
            let pacing = Pacing::default();
            let ctx = Ctx {
                frame: self.frame,
                walkers: &self.units,
                spacing: form::HULL_SPACING,
                hull_across: form::HULL_ACROSS,
                hull_deep: form::HULL_DEEP,
                ground: None,
                ground_version: 0,
                armed: &self.armed,
                hostile: &[],
                target: None,
                party: &[],
                held: &self.held,
                ward: None,
                pacing: &pacing,
                stand_inside: 20.0,
                stand_keep: 40.0,
            };
            let members = self.ids();
            self.goals = self.army.step("a", &members, order, &ctx);
            for w in &mut self.units {
                let Some(goal) = self.goals.get(&w.id) else { continue };
                let d = w.pos.dist2d(*goal);
                let step = (w.speed * 0.1).min(d);
                if self.stuck.contains(&w.id) || d < 0.5 {
                    w.moving = 0.0;
                    continue;
                }
                w.pos = Vec3 { x: w.pos.x + (goal.x - w.pos.x) / d * step, y: 0.0, z: w.pos.z + (goal.z - w.pos.z) / d * step };
                w.moving = step / 0.1;
            }
        }

        fn run(&mut self, order: &GroupOrder, seconds: f32) {
            for _ in 0..(seconds * 10.0) as usize {
                self.tick(order);
            }
        }

        fn deal(&self) -> Vec<(UnitId, usize, usize)> {
            let mut deal: Vec<(UnitId, usize, usize)> = self.army.companies.iter().enumerate().flat_map(|(k, c)| c.slot_of.iter().map(move |(id, s)| (*id, k, *s))).collect();
            deal.sort_by_key(|d| (d.0.0, d.1, d.2));
            deal
        }

        fn state(&self) -> &GroupState {
            self.army.state.as_ref().expect("a state")
        }
    }

    fn go(x: f32, z: f32) -> GroupOrder {
        GroupOrder::new(GroupKind::Go, vec![at(x, z)]).with_cap(24)
    }

    /// P1: ninety-six told to one point stand in four blocks about it in about their travel time, every member in
    /// its slot, the blocks the layout's own for the place; after that no goal moves.
    #[test]
    fn an_army_converges_on_the_layout_of_its_order() {
        let mut sim = Sim::new(96, 0.0, 0.0);
        let order = go(3000.0, 0.0);
        sim.run(&order, 75.0);
        let state = sim.state().clone();
        assert!(state.arrived && state.at_place == 96 && state.formed.len() == 96, "{} at the place, {} formed", state.at_place, state.formed.len());
        assert_eq!(sim.army.companies.len(), 4);
        let wants: Vec<Want> = sim.army.companies.iter().map(|c| Want { ranks: 4, pitch: c.frame.pitch, open: c.open }).collect();
        let mut laid: Vec<(i32, i32)> = layout::lay(at(3000.0, 0.0), state.facing, &wants, 0, None, &[]).iter().map(|b| (b.centre.x.round() as i32, b.centre.z.round() as i32)).collect();
        let mut stands: Vec<(i32, i32)> = sim.army.companies.iter().map(|c| (c.stand.centre.x.round() as i32, c.stand.centre.z.round() as i32)).collect();
        laid.sort();
        stands.sort();
        assert_eq!(laid, stands, "the blocks are a function of the order");
        for w in &sim.units {
            assert!(w.pos.dist2d(sim.army.slot(w.id).unwrap()) < 2.0);
        }
        let goals = sim.goals.clone();
        sim.run(&order, 10.0);
        assert_eq!(goals, sim.goals, "at rest nothing moves");
    }

    /// P2: at rest a newcomer takes a free slot and a death frees one, and no other member's goal moves; on the march
    /// neither changes another member's slot.
    #[test]
    fn a_member_coming_or_going_moves_nobody_else() {
        let mut sim = Sim::new(40, 0.0, 0.0);
        let order = go(2000.0, 0.0);
        sim.run(&order, 12.0);
        let deal = sim.deal();
        sim.units.retain(|w| w.id != UnitId(7));
        sim.units.push(Walker { id: UnitId(500), pos: at(300.0, 500.0), speed: 60.0, moving: 0.0, reach: 200.0, hull: 32.0, reflex: false });
        sim.tick(&order);
        let after = sim.deal();
        assert!(deal.iter().filter(|d| d.0 != UnitId(7)).all(|d| after.contains(d)), "the others keep their slots on the march");
        sim.run(&order, 60.0);
        assert!(sim.state().arrived);
        let goals = sim.goals.clone();
        sim.units.retain(|w| w.id != UnitId(9));
        sim.units.push(Walker { id: UnitId(501), pos: at(1500.0, 300.0), speed: 60.0, moving: 0.0, reach: 200.0, hull: 32.0, reflex: false });
        sim.tick(&order);
        for (id, goal) in &goals {
            if *id != UnitId(9) {
                assert_eq!(sim.goals.get(id), Some(goal), "unit {id:?} at rest");
            }
        }
        assert!(sim.goals.contains_key(&UnitId(501)));
    }

    /// P3: the same order again changes nothing; a place moved by a hundred moves every goal by a hundred and no
    /// slot; two places alternated every second change no deal.
    #[test]
    fn the_response_is_continuous_in_the_order() {
        let mut sim = Sim::new(48, 0.0, 0.0);
        sim.run(&go(2000.0, 0.0), 60.0);
        assert!(sim.state().arrived);
        let (goals, deal) = (sim.goals.clone(), sim.deal());
        sim.run(&go(2000.0, 20.0), 3.0);
        assert_eq!(goals, sim.goals, "a place within the tolerance is the same place");
        sim.run(&go(2000.0, 100.0), 30.0);
        assert_eq!(deal, sim.deal(), "a moved place deals nobody anew");
        for (id, goal) in &goals {
            let now = sim.goals[id];
            assert!((now.x - goal.x).abs() < 1.0 && (now.z - goal.z - 100.0).abs() < 1.0, "{goal:?} to {now:?}");
        }
        for second in 0..20 {
            sim.run(&go(2000.0, if second % 2 == 0 { 400.0 } else { -200.0 }), 1.0);
        }
        assert_eq!(deal, sim.deal(), "alternating orders deal nobody anew");
    }

    /// P5: a fresh army on the same units and order at rest sends nobody further than a slot's width.
    #[test]
    fn an_army_made_again_at_rest_moves_nobody_far() {
        // Along an axis of the map and at an angle to it: the facing is read from the units, whatever it was.
        for (x, z) in [(2500.0, 0.0), (2300.0, 1400.0)] {
            let mut sim = Sim::new(72, 0.0, 0.0);
            let order = go(x, z);
            sim.run(&order, 80.0);
            assert!(sim.state().arrived);
            let goals = sim.goals.clone();
            sim.army = Army::default();
            sim.tick(&order);
            let worst = goals.iter().map(|(id, g)| g.dist2d(sim.goals[id])).fold(0.0, f32::max);
            assert!(worst <= form::HULL_SPACING, "to ({x},{z}): a goal moved {worst:.0}");
        }
    }

    /// P7: members that stand hold nothing. A third of one company never moves; the rest of the army arrives and the
    /// report names the ones standing.
    #[test]
    fn members_that_stand_hold_nobody() {
        let mut sim = Sim::new(48, 0.0, 0.0);
        let order = go(2500.0, 0.0);
        sim.tick(&order);
        let first: Vec<UnitId> = sim.army.companies[0].slot_of.keys().copied().take(8).collect();
        sim.stuck.extend(first.iter().copied());
        sim.run(&order, 75.0);
        let state = sim.state();
        assert_eq!(state.at_place, 40, "the forty that can walk stand at the place");
        assert!(state.arrived, "forty of forty-eight is arrival");
        let standing: HashSet<UnitId> = state.exceptions.iter().filter(|e| e.1 == Cause::Standing).map(|e| e.0).collect();
        assert_eq!(standing, first.into_iter().collect());
    }

    /// The hold (§5.6): the frame waits for a company whose members are walking in from behind, and does not leave
    /// them; one member far behind holds nothing.
    #[test]
    fn the_frame_waits_for_its_body_and_not_for_its_last() {
        // One far behind: the rest march at full pace.
        let mut sim = Sim::new(24, 0.0, 0.0);
        sim.units[5].pos = at(-1500.0, 0.0);
        sim.run(&go(4000.0, 0.0), 20.0);
        let centre = sim.state().centre;
        assert!(centre.x > 900.0, "twenty seconds at sixty: the body is at {:.0}", centre.x);
        // Half far behind: the frame waits, and the half in front does not walk away from them.
        let mut sim = Sim::new(24, 0.0, 0.0);
        for w in sim.units.iter_mut().take(12) {
            w.pos.x -= 1200.0;
        }
        sim.run(&go(4000.0, 0.0), 10.0);
        let front = sim.units.iter().skip(12).map(|w| w.pos.x).fold(f32::NEG_INFINITY, f32::max);
        assert!(front < 500.0, "the front waited: {front:.0}");
        sim.run(&go(4000.0, 0.0), 100.0);
        assert!(sim.state().arrived && sim.state().formed.len() == 24);
    }

    /// §5.9: a company that fights stops while every slot of it has an armed enemy in reach, walks on to its block
    /// when the enemy is gone, and never puts a slot nearer an enemy than the front's stop. A walk stops for nothing.
    #[test]
    fn the_enemy_stops_a_frame_and_moves_no_goal() {
        let fight = GroupOrder::new(GroupKind::FightTo, vec![at(3000.0, 0.0)]).with_cap(24);
        let mut sim = Sim::new(24, 0.0, 0.0);
        sim.armed = vec![(UnitId(900), at(1500.0, 0.0))];
        sim.run(&fight, 60.0);
        let state = sim.state().clone();
        assert!(!state.arrived && state.place == Some(at(3000.0, 0.0)), "stopped short, the place its own");
        let nearest = sim.units.iter().map(|w| w.pos.dist2d(at(1500.0, 0.0))).fold(f32::INFINITY, f32::min);
        assert!(nearest >= 99.0, "no nearer than half the reach: {nearest:.0}");
        assert!(state.in_reach > 0 && state.engaged == Some(UnitId(900)));
        sim.armed.clear();
        sim.run(&fight, 60.0);
        assert!(sim.state().arrived, "the enemy gone, it walks on to its place");
        let mut walk = Sim::new(24, 0.0, 0.0);
        walk.armed = vec![(UnitId(900), at(1500.0, 0.0))];
        walk.run(&go(3000.0, 0.0), 70.0);
        assert!(walk.state().arrived, "a walk stops for nothing");
    }

    /// §5.2: a second group's layout is laid round the blocks the first holds.
    #[test]
    fn a_layout_is_laid_round_the_blocks_another_group_holds() {
        let mut first = Sim::new(24, 0.0, 0.0);
        first.run(&go(2000.0, 0.0), 50.0);
        let mut second = Sim::new(24, 0.0, 800.0);
        second.held = first.army.blocks();
        second.run(&go(2000.0, 0.0), 60.0);
        assert!(second.state().arrived);
        for a in second.army.blocks() {
            for b in first.army.blocks() {
                assert!(!a.overlaps(&b, BLOCK_MARGIN));
            }
        }
    }
}
