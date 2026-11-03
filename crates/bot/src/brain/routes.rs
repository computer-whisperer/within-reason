//! Walking distances for the brain's geometry: what is reachable, what is ours, which way is forward.
//! Everything falls back to straight lines when the engine sent no terrain.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};

use bot_protocol::{MoveClass, MoveKind, UnitDefId, Vec3};

use super::Brain;
use super::roster::Kit;
use terrain::Field;

/// The enemy-side field is rebuilt when our estimate of where an enemy lives has moved this far.
const ENEMY_MOVED: f32 = 600.0;

/// The place fields' cache, in bytes (the metal-maps design §1.2: route costs bounded by use, not by the count of
/// places). A field is four bytes a terrain cell: 786 KB on Comet, 2.4 MB on Full Metal Plate; this holds every
/// spot of Comet for three classes, and eighty places on the largest maps, the least recently asked evicted.
const FIELD_CACHE_BYTES: usize = 192 << 20;

/// How a unit gets about (docs/design/2026-09-22-domains.md, decision 8): aircraft go straight, everything else
/// by the fields of its own movement class. The soldiers' default (the lab's raider's class) serves the picture's
/// "our half", the passages and the sketch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Walker {
    Air,
    Class(ClassKey),
}

/// A movement class as a key: its kind, its slope limit and its depth, rounded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClassKey {
    kind: MoveKind,
    max_slope: i32,
    depth: i32,
}

impl ClassKey {
    fn of(class: MoveClass) -> ClassKey {
        ClassKey { kind: class.kind, max_slope: (class.max_slope * 255.0).round() as i32, depth: (class.depth * 10.0).round() as i32 }
    }
}

pub struct Routes {
    /// The soldiers' walker (the lab's raider's class): the default for every question without a unit.
    soldiers: Walker,
    from_home: Field,
    /// Where each movement class we field can stand, and its distances from home: for snapping and reachability
    /// by class. Built at the survey for every class among the units the commander reaches.
    classes: HashMap<Walker, (layout::Passable, Field)>,
    /// Distance from the nearest live enemy base.
    from_enemy: Option<Field>,
    /// The narrow places between home and the enemy's bases: of the two fields, so computed when `from_enemy` is
    /// (every picture asked for them, and computing them was 9% of the bot's time in player-51's profile).
    passages: Vec<terrain::Passage>,
    enemy_origins: Vec<Vec3>,
    passable: Vec<bool>,
    /// A field per place asked for, per class, in effective elmos (slopes priced by the engine's law), so that
    /// "nearest" means travel time from wherever a unit stands (the user, 2026-09-20: the commander walked the
    /// cliffs behind the Quicksilver base to spots a straight line called near). Computed on demand by a worker
    /// thread the first time a place is asked for (10 ms a field) and kept while the cache has room: a place with
    /// no field yet answers by the straight line this tick and has its field the next. On Comet the cache holds
    /// every spot within seconds of the first menu, as the per-spot fields did; on a metal map it never holds ten
    /// thousand (the metal-maps design §1.2).
    fields: Arc<Mutex<FieldCache>>,
    /// Asks the worker for a place's field; dropping it ends the worker.
    ask: mpsc::Sender<(Walker, Vec3)>,
}

/// The place fields by walker and place (the terrain cell the place lies in), least recently used first out.
struct FieldCache {
    entries: HashMap<(Walker, (i32, i32)), (Field, u64)>,
    /// Asked of the worker and not yet landed: asked once.
    pending: std::collections::HashSet<(Walker, (i32, i32))>,
    clock: u64,
    bytes: usize,
    /// The cache's room in bytes (`FIELD_CACHE_BYTES`; smaller in tests).
    budget: usize,
    cell: f32,
}

impl FieldCache {
    fn key(&self, walker: Walker, at: Vec3) -> (Walker, (i32, i32)) {
        (walker, ((at.x / self.cell) as i32, (at.z / self.cell) as i32))
    }

    /// The field for the place when the cache holds it (touched), else None and an ask recorded.
    fn get(&mut self, walker: Walker, at: Vec3) -> Result<&Field, bool> {
        let key = self.key(walker, at);
        self.clock += 1;
        let clock = self.clock;
        match self.entries.get_mut(&key) {
            Some((field, used)) => {
                *used = clock;
                Ok(field)
            }
            None => Err(self.pending.insert(key)),
        }
    }

    fn insert(&mut self, walker: Walker, at: Vec3, field: Option<Field>) {
        let key = self.key(walker, at);
        self.pending.remove(&key);
        let Some(field) = field else { return };
        self.bytes += field.bytes();
        self.clock += 1;
        self.entries.insert(key, (field, self.clock));
        while self.bytes > self.budget && self.entries.len() > 1 {
            let oldest = *self.entries.iter().min_by_key(|(_, (_, used))| *used).map(|(k, _)| k).expect("entries");
            if let Some((field, _)) = self.entries.remove(&oldest) {
                self.bytes -= field.bytes();
            }
        }
    }
}

impl Brain {
    /// Fields for our soldiers' movement class (the lab's raider stands for all of them).
    pub(super) fn survey(&mut self, kit: &Kit) {
        let terrain = &self.world.hello.terrain;
        let Some(class) = self.world.def(kit.raider).and_then(|d| d.move_class) else { return };
        let passable = terrain::passable(terrain, class);
        // A start in the water (SailAway 2: a seat's commander at height -62) is no ground the raiders stand on: the
        // field then runs from the nearest shore cell, and every distance is from there.
        let origin = if Field::from(terrain, &passable, self.home).is_some() { self.home } else { nearest_passable(terrain, &passable, self.home, 2500.0).unwrap_or(self.home) };
        if origin != self.home {
            eprintln!("[ai {}] our start ({:.0}, {:.0}) is not ground our soldiers stand on; distances run from the nearest shore at ({:.0}, {:.0})", self.ai(), self.home.x, self.home.z, origin.x, origin.z);
        }
        let Some(from_home) = Field::from(terrain, &passable, origin) else {
            eprintln!("[ai {}] no terrain data; distances are straight lines", self.ai());
            return;
        };
        // H-MAP-ENEMY-START: the mirror image of our start is only where the enemy would be on a symmetric map. On
        // Quicksilver it is a beach across the water from the real base, 700 elmos off, and armies sent "to the enemy
        // start" stood there looking at the sea (the spot we can walk to nearest it is 140 from the real start).
        if self.enabled("H-MAP-ENEMY-START") {
            self.snap_guesses_to_metal(|spot| from_home.distance(spot).is_some());
        }
        let terrain = &self.world.hello.terrain;
        let enemy_origins = self.live_enemy_bases();
        let from_enemy = Field::from_many(terrain, &passable, &enemy_origins);
        let spots = &self.world.hello.metal_spots;
        let reachable = spots.iter().filter(|s| from_home.distance(**s).is_some()).count();
        let ours = spots.iter().filter(|s| is_ours(&from_home, from_enemy.as_ref(), **s)).count();
        eprintln!(
            "[ai {}] terrain: {} of {} metal spots reachable on foot, {} nearer to us than to the enemy; enemy start {:.0} away on foot, {:.0} in a straight line",
            self.ai(), reachable, spots.len(), ours,
            from_home.distance(self.enemy_base(self.home)).unwrap_or(f32::INFINITY), self.home.dist2d(self.enemy_base(self.home))
        );
        let cut_off: Vec<String> =
            spots.iter().filter(|s| from_home.distance(**s).is_none()).map(|s| format!("({:.0}, {:.0})", s.x, s.z)).collect();
        eprintln!("[ai {}] terrain: spots we cannot walk to: {}", self.ai(), cut_off.join(" "));
        // Every movement class among the units the commander reaches by build lists, the raider's included.
        let mut distinct: Vec<(Walker, MoveClass)> = Vec::new();
        for id in self.world.reachable_from(kit.commander) {
            if let Some(mc) = self.world.def(id).filter(|d| d.speed > 0.0).and_then(|d| d.move_class) {
                let key = Walker::Class(ClassKey::of(mc));
                if !distinct.iter().any(|(k, _)| *k == key) {
                    distinct.push((key, mc));
                }
            }
        }
        // Each class's field starts from the nearest ground that class stands on: seeded at our start itself, a ship's
        // field from a start on land was never made, the ship fell back to the soldiers' reach, and the construction
        // ships of game 12 on SailAway 2 were offered no spot all game (K-hands-a-ships-field-from-a-land-start).
        let classes: HashMap<Walker, (layout::Passable, Field)> = distinct
            .iter()
            .filter_map(|(key, mc)| {
                let passable: layout::Passable = terrain::passable(terrain, *mc).into();
                let origin = nearest_passable(terrain, &passable, self.home, 2500.0)?;
                Field::from(terrain, &passable, origin).map(|f| (*key, (passable, f)))
            })
            .collect();
        eprintln!("[ai {}] terrain: fields for {} movement classes", self.ai(), classes.len());
        // The place fields' worker: it prices each class's ground once and answers asks in the order they come.
        let fields = Arc::new(Mutex::new(FieldCache { entries: HashMap::new(), pending: Default::default(), clock: 0, bytes: 0, budget: FIELD_CACHE_BYTES, cell: terrain.cell.max(1.0) }));
        let (ask, asks) = mpsc::channel::<(Walker, Vec3)>();
        {
            let terrain = terrain.clone();
            let all_costs: HashMap<Walker, Vec<u32>> = distinct.iter().map(|(key, mc)| (*key, terrain::costs(&terrain, *mc))).collect();
            let fields = Arc::clone(&fields);
            std::thread::spawn(move || {
                while let Ok((walker, at)) = asks.recv() {
                    let field = all_costs.get(&walker).and_then(|costs| Field::from_costs(&terrain, costs, &[at]));
                    fields.lock().expect("the field cache").insert(walker, at, field);
                }
            });
        }
        self.routes = Some(Routes {
            soldiers: Walker::Class(ClassKey::of(class)),
            passages: from_enemy.as_ref().map(|enemy| terrain::passages(&from_home, enemy)).unwrap_or_default(),
            from_home, classes, from_enemy, enemy_origins, passable, fields, ask,
        });
        for p in self.passages() {
            eprintln!("[ai {}] terrain: passage at {} ({:.0}, {:.0}), {:.0} wide, {:.0} % of the way to the enemy", self.ai(), self.world.grid(p.at), p.at.x, p.at.z, p.width, p.along * 100.0);
        }
        if let Some(sketch) = self.terrain_sketch() {
            for row in sketch["rows"].as_array().into_iter().flatten() {
                eprintln!("[ai {}] terrain: {}", self.ai(), row.as_str().unwrap_or_default());
            }
        }
    }

    /// Call when an enemy base may have moved, died or come back.
    pub(super) fn resurvey_enemy(&mut self) {
        let origins = self.live_enemy_bases();
        let terrain = &self.world.hello.terrain;
        if let Some(routes) = &mut self.routes
            && (routes.enemy_origins.len() != origins.len() || routes.enemy_origins.iter().zip(&origins).any(|(a, b)| a.dist2d(*b) > ENEMY_MOVED))
        {
            routes.from_enemy = Field::from_many(terrain, &routes.passable, &origins);
            routes.passages = routes.from_enemy.as_ref().map(|enemy| terrain::passages(&routes.from_home, enemy)).unwrap_or_default();
            routes.enemy_origins = origins;
            // The map the player reads names his starts, the passages and whose spots are whose by walk.
            self.publish_map();
        }
    }

    /// How a unit of this type gets about: straight for aircraft, else by its own movement class; a building or an
    /// unknown type as our soldiers do.
    pub(super) fn walker_of(&self, def: UnitDefId) -> Walker {
        match self.world.def(def) {
            Some(d) if d.speed > 0.0 && d.move_class.is_none() => Walker::Air,
            Some(d) if d.move_class.is_some() => Walker::Class(ClassKey::of(d.move_class.expect("checked"))),
            _ => self.soldiers_walker(),
        }
    }

    fn soldiers_walker(&self) -> Walker {
        self.routes.as_ref().map_or(Walker::Air, |r| r.soldiers)
    }

    /// The walker of a group: its first member's.
    pub(super) fn group_walker(&self, group: &super::pianist::Group, own: &[bot_protocol::OwnUnit]) -> Walker {
        group.members.iter().find_map(|id| own.iter().find(|u| u.id == *id)).map_or(self.soldiers_walker(), |u| self.walker_of(u.def))
    }

    /// Where a unit of this walker is sent for `pos`: the point itself for aircraft, else the ground its class can
    /// reach nearest it.
    pub(super) fn snap_for(&self, walker: Walker, pos: Vec3) -> Vec3 {
        match walker {
            Walker::Air => pos,
            Walker::Class(_) => match self.routes.as_ref().and_then(|r| r.classes.get(&walker)) {
                Some((_, from_home)) => from_home.snap(pos).unwrap_or(pos),
                None => self.snap_to_reachable(pos),
            },
        }
    }

    /// Whether a unit of this walker can get from home to `pos`; true when we cannot tell.
    pub(super) fn reachable_for(&self, walker: Walker, pos: Vec3) -> bool {
        match walker {
            Walker::Air => true,
            Walker::Class(_) => match self.routes.as_ref().and_then(|r| r.classes.get(&walker)) {
                Some((_, from_home)) => from_home.distance(pos).is_some(),
                None => self.reachable_on_foot(pos),
            },
        }
    }

    /// Effective elmos (elmos at full speed, slopes priced) from `from` to the place `at` for a class; the straight
    /// line for aircraft, until the place's field is computed (asked of the worker now, there by the next tick) or
    /// where the place cannot be walked to.
    pub(super) fn walk_to_as(&self, walker: Walker, at: Vec3, from: Vec3) -> f32 {
        let straight = at.dist2d(from);
        let (Walker::Class(_), Some(routes)) = (walker, self.routes.as_ref()) else { return straight };
        match routes.fields.lock().expect("the field cache").get(walker, at) {
            Ok(field) => field.distance(from).unwrap_or(straight),
            Err(asked) => {
                if asked {
                    let _ = routes.ask.send((walker, at));
                }
                straight
            }
        }
    }

    /// Seconds a unit of this type takes from `from` to the place (its speed over the effective elmos).
    pub(super) fn seconds_to(&self, def: UnitDefId, at: Vec3, from: Vec3) -> f32 {
        let speed = self.world.def(def).map_or(1.0, |d| d.speed.max(1.0));
        self.walk_to_as(self.walker_of(def), at, from) / speed
    }

    /// The place fields held now and their bytes: the record's and the log's measure of the cache.
    pub(super) fn field_cache_size(&self) -> (usize, usize) {
        self.routes.as_ref().map_or((0, 0), |r| {
            let cache = r.fields.lock().expect("the field cache");
            (cache.entries.len(), cache.bytes)
        })
    }

    /// Whether our soldiers can walk from home to (next to) `pos`. True when we cannot tell.
    pub(super) fn reachable_on_foot(&self, pos: Vec3) -> bool {
        self.routes.as_ref().is_none_or(|r| r.from_home.distance(pos).is_some())
    }

    /// Walking distance from home, else the straight line.
    pub(super) fn walk_from_home(&self, pos: Vec3) -> f32 {
        self.routes.as_ref().and_then(|r| r.from_home.distance(pos)).unwrap_or_else(|| pos.dist2d(self.home))
    }

    /// A metal spot we can walk to that is nearer to us than to the enemy, on foot.
    pub(super) fn spot_is_ours(&self, spot: Vec3) -> bool {
        match &self.routes {
            Some(routes) => is_ours(&routes.from_home, routes.from_enemy.as_ref(), spot),
            None => spot.dist2d(self.home) < spot.dist2d(self.enemy_base(spot)),
        }
    }

    /// The point `along` elmos from home on the way to `goal`, on foot.
    pub(super) fn on_the_way_to(&self, goal: Vec3, along: f32) -> Option<Vec3> {
        self.routes.as_ref()?.from_home.towards(goal, along)
    }

    /// The map in text for the language model, with a legend; `None` without terrain data.
    pub(super) fn terrain_sketch(&self) -> Option<serde_json::Value> {
        const SIZE: usize = 32;
        let routes = self.routes.as_ref()?;
        let rows = terrain::sketch(&self.world.hello.terrain, &routes.passable, &routes.from_home, SIZE);
        // The grid's columns lettered over the picture and its rows numbered down the side, each where its cell
        // begins, so the cells can be read off it (a 32-wide sketch of eight columns has four characters a cell).
        let grid = self.world.cells;
        let mut header = vec![b' '; 3 + SIZE];
        for column in 0..grid.columns {
            header[3 + (column as usize * SIZE / grid.columns as usize)] = b'A' + column;
        }
        let mut lines = vec![String::from_utf8(header).expect("ascii")];
        lines.extend(rows.iter().enumerate().map(|(i, row)| {
            let label = (0..grid.rows).find(|r| *r as usize * rows.len() / grid.rows as usize == i).map_or("   ".to_string(), |r| format!("{:<3}", r + 1));
            format!("{label}{row}")
        }));
        Some(serde_json::json!({
            "legend": format!("north is up; {}x{} characters per grid cell, the cells' letters and numbers at the edges. ~ water, # cliff or slope our bots cannot cross, x ground we cannot walk to from our start, . o O walkable ground (low, middle, high)", SIZE / grid.columns as usize, rows.len() / grid.rows as usize),
            "rows": lines,
        }))
    }

    /// The narrow places on the ways between home and the nearest enemy bases, the narrowest first.
    pub(super) fn passages(&self) -> Vec<terrain::Passage> {
        self.routes.as_ref().map(|r| r.passages.clone()).unwrap_or_default()
    }

    /// Whether any builder class of ours reaches `pos`: an amphibious or hover constructor or a construction ship
    /// gets to an underwater spot our soldiers cannot walk to (Cape Violet, 2026-09-27: 28 of 72 spots under water,
    /// none of them a place, the Beavers' lists skipped as "not free").
    pub(super) fn reachable_by_any_class(&self, pos: Vec3) -> bool {
        self.reachable_on_foot(pos) || self.routes.as_ref().is_some_and(|r| r.classes.values().any(|(_, field)| field.distance(pos).is_some()))
    }

    /// Where the units a factory of this type makes can stand: the passable grid of the strictest movement class
    /// among its build options (a vehicle plant's tanks climb less than a lab's bots), else the soldiers'.
    /// Where our soldiers' class stands: the layout layer's ground for every plant's street. None without terrain.
    pub(super) fn routes_passable(&self) -> Option<&[bool]> {
        self.routes.as_ref().map(|r| r.passable.as_slice())
    }

    pub(super) fn passable_for_factory(&self, def_id: UnitDefId) -> Option<layout::Passable> {
        let routes = self.routes.as_ref()?;
        let strictest = self
            .world
            .def(def_id)?
            .build_options
            .iter()
            .filter_map(|id| self.world.def(*id).filter(|d| d.speed > 0.0).and_then(|d| d.move_class))
            .filter_map(|mc| routes.classes.get(&Walker::Class(ClassKey::of(mc))).map(|(passable, _)| passable))
            .min_by_key(|passable| passable.iter().filter(|p| **p).count());
        Some(strictest.cloned().unwrap_or_else(|| routes.passable.clone().into()))
    }

    /// The reachable ground nearest `pos`; `pos` itself when we cannot tell.
    pub(super) fn snap_to_reachable(&self, pos: Vec3) -> Vec3 {
        self.routes.as_ref().and_then(|r| r.from_home.snap(pos)).unwrap_or(pos)
    }
}

fn is_ours(from_home: &Field, from_enemy: Option<&Field>, spot: Vec3) -> bool {
    let Some(ours) = from_home.distance(spot) else { return false };
    from_enemy.and_then(|f| f.distance(spot)).is_none_or(|theirs| ours < theirs)
}

/// The centre of the passable cell nearest `pos` within `radius`, if any: where a field starts for a home that is
/// itself in the water.
fn nearest_passable(terrain: &bot_protocol::Terrain, passable: &[bool], pos: Vec3, radius: f32) -> Option<Vec3> {
    if terrain.cell <= 0.0 || passable.is_empty() {
        return None;
    }
    let cells = (radius / terrain.cell).ceil() as i64;
    let (cx, cz) = ((pos.x / terrain.cell) as i64, (pos.z / terrain.cell) as i64);
    let mut best: Option<(f32, Vec3)> = None;
    for row in (cz - cells).max(0)..=(cz + cells).min(i64::from(terrain.height) - 1) {
        for col in (cx - cells).max(0)..=(cx + cells).min(i64::from(terrain.width) - 1) {
            if !passable[(row * i64::from(terrain.width) + col) as usize] {
                continue;
            }
            let at = Vec3 { x: (col as f32 + 0.5) * terrain.cell, y: 0.0, z: (row as f32 + 0.5) * terrain.cell };
            let d = at.dist2d(pos);
            if d <= radius && best.is_none_or(|(b, _)| d < b) {
                best = Some((d, at));
            }
        }
    }
    best.map(|(_, at)| at)
}

#[cfg(test)]
mod cache_tests {
    use super::*;
    use bot_protocol::Terrain;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    /// A place is asked of the worker once and answered by the straight line until its field lands; the cache
    /// holds its budget, the least recently asked place leaving first.
    #[test]
    fn a_place_is_asked_once_and_the_oldest_leaves_when_the_cache_is_full() {
        let terrain = Terrain { cell: 16.0, width: 8, height: 8, heights: vec![10; 64], slopes: vec![0; 64], metal: Vec::new() };
        let costs = vec![10u32; 64];
        let field = |p: Vec3| Field::from_costs(&terrain, &costs, &[p]).expect("a field");
        let walker = Walker::Class(ClassKey { kind: MoveKind::Tank, max_slope: 10, depth: 0 });
        // Room for two fields of 64 cells.
        let mut cache = FieldCache { entries: HashMap::new(), pending: Default::default(), clock: 0, bytes: 0, budget: 2 * 64 * 4, cell: 16.0 };
        assert_eq!(cache.get(walker, at(8.0, 8.0)).err(), Some(true), "the first ask goes to the worker");
        assert_eq!(cache.get(walker, at(9.0, 9.0)).err(), Some(false), "the same cell, asked already");
        cache.insert(walker, at(8.0, 8.0), Some(field(at(8.0, 8.0))));
        assert!(cache.pending.is_empty());
        assert_eq!(cache.get(walker, at(8.0, 8.0)).ok().and_then(|f| f.distance(at(8.0, 120.0))).map(|d| d.round()), Some(112.0));
        cache.insert(walker, at(40.0, 40.0), Some(field(at(40.0, 40.0))));
        // The first place is touched, then a third lands: the second, least recently used, leaves.
        assert!(cache.get(walker, at(8.0, 8.0)).is_ok());
        cache.insert(walker, at(72.0, 72.0), Some(field(at(72.0, 72.0))));
        assert_eq!(cache.entries.len(), 2);
        assert!(cache.get(walker, at(8.0, 8.0)).is_ok() && cache.get(walker, at(72.0, 72.0)).is_ok());
        assert_eq!(cache.get(walker, at(40.0, 40.0)).err(), Some(true), "evicted, asked again");
        assert_eq!(cache.bytes, 2 * 64 * 4);
        // A place no field reaches (off every class's ground) lands as nothing and is not pending any more.
        cache.insert(walker, at(40.0, 40.0), None);
        assert!(cache.pending.is_empty() && cache.entries.len() == 2);
    }
}
