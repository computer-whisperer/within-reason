//! Where an army stands (`docs/design/2026-11-15-army-control-loop.md` §5.2): a block is one company's rectangle of
//! slots, and a layout is one block a company laid on a lattice about a place, each where its slots stand on the
//! ground. Pure functions of the place, the facing, the companies and the ground: the same inputs give the same
//! layout whoever arrives first and wherever anyone is.

use bot_protocol::Vec3;

use crate::form::{self, Pitch};
use crate::ground::Ground;
use crate::group::BLOCK_MARGIN;

/// Blocks abreast in a row of a layout; further blocks stand in rows behind.
pub const ROW: usize = 4;
/// `ROW`, or the bench's arm (`WITHIN_REASON_ROW=2`: rows of two, an army of four in a square).
pub fn row() -> usize {
    static ROW_ARM: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *ROW_ARM.get_or_init(|| std::env::var("WITHIN_REASON_ROW").ok().and_then(|v| v.parse().ok()).filter(|n| *n > 0).unwrap_or(ROW))
}
/// How far from its lattice cell a block's place is looked for when it does not stand there, and the step.
const SETTLE_REACH: f32 = 512.0;
const SETTLE_STEP: f32 = 32.0;

/// A company's rectangle of slots: ranks of `form::FILES` across `facing`, `ranks` of them, about `centre`. A slot's
/// place depends on the block alone, never on how many of its slots are taken, so a member joining or leaving moves
/// no other member's slot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Block {
    pub centre: Vec3,
    pub facing: (f32, f32),
    pub pitch: Pitch,
    pub ranks: usize,
}

impl Block {
    /// The ranks a company of `members` under `cap` stands in: the cap's when there is one, so the block is the same
    /// with a slot free; the members' with no cap.
    pub fn ranks_for(cap: usize, members: usize) -> usize {
        (if cap > 0 { cap } else { members }).max(1).div_ceil(form::FILES)
    }

    pub fn capacity(&self) -> usize {
        self.ranks * form::FILES
    }

    /// Half the block's extent across its facing and along it, slot centre to slot centre.
    fn half(&self) -> (f32, f32) {
        ((form::FILES as f32 - 1.0) / 2.0 * self.pitch.across + self.pitch.across / 4.0, (self.ranks as f32 - 1.0) / 2.0 * self.pitch.deep)
    }

    /// The lattice cell a block of this shape takes: its width and depth with the margin between two blocks.
    pub fn cell(&self) -> (f32, f32) {
        let (across, along) = self.half();
        (2.0 * across + self.pitch.across + BLOCK_MARGIN, 2.0 * along + self.pitch.deep)
    }

    /// Slot `index`'s place: ranks from the front, within a rank from the centre outward (so the slots first taken
    /// are the block's middle), every other rank shifted half a spacing (a rear member looks between two in front).
    pub fn slot(&self, index: usize) -> Vec3 {
        let (rank, k) = (index / form::FILES, index % form::FILES);
        let mid = (form::FILES as f32 - 1.0) / 2.0;
        // The files in order of their distance from the middle, the left of two equals first.
        let mut files: [usize; form::FILES] = std::array::from_fn(|j| j);
        files.sort_by(|a, b| (*a as f32 - mid).abs().total_cmp(&(*b as f32 - mid).abs()).then(a.cmp(b)));
        let shift = if rank % 2 == 1 { 0.25 } else { -0.25 };
        let lateral = (files[k] as f32 - mid + shift) * self.pitch.across;
        let forward = ((self.ranks as f32 - 1.0) / 2.0 - rank as f32) * self.pitch.deep;
        let (h, left) = (self.facing, (-self.facing.1, self.facing.0));
        Vec3 { x: self.centre.x + h.0 * forward + left.0 * lateral, y: self.centre.y, z: self.centre.z + h.1 * forward + left.1 * lateral }
    }

    /// The same block about another centre.
    pub fn at(&self, centre: Vec3) -> Block {
        Block { centre, ..*self }
    }

    /// Whether `p` lies in the block's rectangle grown by `margin`.
    pub fn contains(&self, p: Vec3, margin: f32) -> bool {
        let (dx, dz) = (p.x - self.centre.x, p.z - self.centre.z);
        let (across, along) = self.half();
        (dx * self.facing.0 + dz * self.facing.1).abs() <= along + margin && (-dx * self.facing.1 + dz * self.facing.0).abs() <= across + margin
    }

    /// Whether two blocks stand nearer than `margin`: by their rectangles when they face one axis, by their circles
    /// otherwise.
    pub fn overlaps(&self, other: &Block, margin: f32) -> bool {
        let (dx, dz) = (other.centre.x - self.centre.x, other.centre.z - self.centre.z);
        let (a, b) = (self.half(), other.half());
        let parallel = (self.facing.0 * other.facing.0 + self.facing.1 * other.facing.1).abs();
        if parallel >= 0.95 {
            (dx * self.facing.0 + dz * self.facing.1).abs() < a.1 + b.1 + margin && (-dx * self.facing.1 + dz * self.facing.0).abs() < a.0 + b.0 + margin
        } else {
            dx.hypot(dz) < a.0.hypot(a.1) + b.0.hypot(b.1) + margin
        }
    }

    /// How many of the block's first `open` slots stand on ground joined to its centre: the slot's cell is ground and
    /// so is every cell on the line from it to the centre (a slot behind a wall does not stand though its cell is
    /// ground).
    pub fn standing(&self, ground: &Ground, open: usize) -> usize {
        (0..open).filter(|i| ground.clear_line(self.slot(*i), self.centre, 0.0)).count()
    }
}

/// What a company asks of a layout: its block's shape and how many of its slots are open.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Want {
    pub ranks: usize,
    pub pitch: Pitch,
    pub open: usize,
}

/// The lattice cell of block `index` of `count` about `place` facing `facing`, in cells of `cell` (across, along):
/// rows of `ROW` abreast, each row centred on the place, the first row `first_row` rows behind it.
fn ideal(place: Vec3, facing: (f32, f32), cell: (f32, f32), index: usize, count: usize, first_row: usize) -> Vec3 {
    let (row, column) = (index / row(), index % row());
    let in_row = (count - row * self::row()).min(self::row());
    let lateral = (column as f32 - (in_row as f32 - 1.0) / 2.0) * cell.0;
    let back = (first_row + row) as f32 * cell.1;
    let left = (-facing.1, facing.0);
    Vec3 { x: place.x - facing.0 * back + left.0 * lateral, y: place.y, z: place.z - facing.1 * back + left.1 * lateral }
}

/// The place nearest `block`'s where it stands whole (its first `open` slots on ground joined to its centre) and
/// clear of `taken`; where none within reach does, the clear place where the most slots stand; the block's own
/// when nothing is clear. Without a ground only the others are kept off.
pub fn settle(block: Block, open: usize, ground: Option<&Ground>, taken: &[Block]) -> Block {
    let clear = |b: &Block| !taken.iter().any(|t| t.overlaps(b, BLOCK_MARGIN));
    let stands = |b: &Block| ground.map_or(open, |g| b.standing(g, open));
    if clear(&block) && stands(&block) == open {
        return block;
    }
    let reach = (SETTLE_REACH / SETTLE_STEP) as i32;
    let mut steps: Vec<(i32, i32, i32)> = (-reach..=reach).flat_map(|dz| (-reach..=reach).map(move |dx| (dx * dx + dz * dz, dx, dz))).filter(|s| s.0 > 0).collect();
    steps.sort_unstable();
    let mut best: Option<(usize, Block)> = clear(&block).then(|| (stands(&block), block));
    for (_, dx, dz) in steps {
        let b = block.at(Vec3 { x: block.centre.x + dx as f32 * SETTLE_STEP, y: block.centre.y, z: block.centre.z + dz as f32 * SETTLE_STEP });
        if !clear(&b) {
            continue;
        }
        let n = stands(&b);
        if n == open {
            return b;
        }
        if best.is_none_or(|(most, _)| n > most) {
            best = Some((n, b));
        }
    }
    best.map_or(block, |(_, b)| b)
}

/// The army's layout at `place` (§5.2): one block a company, in the order given, each settled from its lattice cell
/// clear of the blocks before it and of `held` (the blocks other groups hold). `first_row` is how many rows behind
/// the place the first row stands (a follower's, behind its ward's rows).
pub fn lay(place: Vec3, facing: (f32, f32), wants: &[Want], first_row: usize, ground: Option<&Ground>, held: &[Block]) -> Vec<Block> {
    let shape = |w: &Want| Block { centre: place, facing, pitch: w.pitch, ranks: w.ranks };
    let cell = wants.iter().map(|w| shape(w).cell()).fold((0.0f32, 0.0f32), |a, c| (a.0.max(c.0), a.1.max(c.1)));
    let mut taken: Vec<Block> = held.to_vec();
    let mut out = Vec::with_capacity(wants.len());
    for (i, w) in wants.iter().enumerate() {
        let block = settle(shape(w).at(ideal(place, facing, cell, i, wants.len(), first_row)), w.open, ground, &taken);
        taken.push(block);
        out.push(block);
    }
    out
}

/// One more block beside a layout already laid (§5.7: a company founded on joiners when every slot is taken): the
/// next lattice cell, settled clear of `taken`. No block of the layout moves.
pub fn lay_more(place: Vec3, facing: (f32, f32), want: Want, index: usize, first_row: usize, ground: Option<&Ground>, taken: &[Block]) -> Block {
    let shape = Block { centre: place, facing, pitch: want.pitch, ranks: want.ranks };
    settle(shape.at(ideal(place, facing, shape.cell(), index, index + 1, first_row)), want.open, ground, taken)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    fn want(open: usize) -> Want {
        Want { ranks: 4, pitch: form::HULL_PITCH, open }
    }

    /// A block's slots are distinct, at least a spacing apart, inside its rectangle, and centred on its centre; a
    /// slot's place does not depend on how many are open.
    #[test]
    fn a_blocks_slots_stand_about_its_centre() {
        let block = Block { centre: at(1000.0, 1000.0), facing: (1.0, 0.0), pitch: form::HULL_PITCH, ranks: 4 };
        let slots: Vec<Vec3> = (0..block.capacity()).map(|i| block.slot(i)).collect();
        for (i, a) in slots.iter().enumerate() {
            assert!(block.contains(*a, 1.0), "slot {i} at {a:?}");
            for b in &slots[i + 1..] {
                assert!(a.dist2d(*b) >= form::HULL_SPACING - 0.5, "{a:?} {b:?}");
            }
        }
        let mean = form::centroid(slots.iter().copied()).unwrap();
        assert!(mean.dist2d(block.centre) < 1.0, "{mean:?}");
        // The first rank is the front, and its first two slots are its middle.
        assert!(slots[0].x > slots[6].x && (slots[0].z - 1000.0).abs() < form::HULL_SPACING && (slots[1].z - 1000.0).abs() < form::HULL_SPACING);
    }

    /// Four companies are laid in one row centred on the place, clear of each other; the layout is the same again
    /// from the same inputs, and a fifth stands behind.
    #[test]
    fn a_layout_is_a_row_about_the_place_and_a_function_of_its_inputs() {
        let wants = [want(24), want(24), want(24), want(20)];
        let blocks = lay(at(5000.0, 5000.0), (1.0, 0.0), &wants, 0, None, &[]);
        assert_eq!(blocks, lay(at(5000.0, 5000.0), (1.0, 0.0), &wants, 0, None, &[]));
        for (i, a) in blocks.iter().enumerate() {
            assert!((a.centre.x - 5000.0).abs() < 1.0, "one row: {a:?}");
            for b in &blocks[i + 1..] {
                assert!(!a.overlaps(b, BLOCK_MARGIN));
            }
        }
        let mean = form::centroid(blocks.iter().map(|b| b.centre)).unwrap();
        assert!(mean.dist2d(at(5000.0, 5000.0)) < 1.0, "centred on the place: {mean:?}");
        let five = lay(at(5000.0, 5000.0), (1.0, 0.0), &[want(24); 5], 0, None, &[]);
        assert!(five[4].centre.x < 5000.0 - 300.0, "the fifth behind: {:?}", five[4]);
    }

    /// A layout is laid round the blocks another group holds, and one more block is laid beside it without moving it.
    #[test]
    fn a_layout_keeps_off_held_blocks_and_grows_by_one() {
        let held = lay(at(5000.0, 5000.0), (1.0, 0.0), &[want(24)], 0, None, &[]);
        let mine = lay(at(5000.0, 5000.0), (1.0, 0.0), &[want(24), want(24)], 0, None, &held);
        for b in &mine {
            assert!(!b.overlaps(&held[0], BLOCK_MARGIN), "{b:?}");
        }
        let all: Vec<Block> = held.iter().chain(&mine).copied().collect();
        let more = lay_more(at(5000.0, 5000.0), (1.0, 0.0), want(24), 2, 0, None, &all);
        for b in &all {
            assert!(!b.overlaps(&more, BLOCK_MARGIN), "{more:?} on {b:?}");
        }
    }
}
