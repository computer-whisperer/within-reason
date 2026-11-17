//! A whole game in memory, as the capture widget's raw stream tells it (`tape.raw`, written by
//! `run/replay-template/LuaUI/Widgets/tape_dump.lua`): every unit's life with its samples, every projectile, the
//! features, the ground and its changes. The packer cuts this into the tape; the checker asks it what the tape should
//! say at a frame.
//!
//! One rule gives the state at a frame, here and in the tape: everything stamped with that frame or an earlier one.

use crate::format::{Error, NEVER, R, Result, err};
use serde_json::{Value, json};
use std::collections::HashMap;

/// Where a unit stands and how it is turned: whole quarters of an elmo (the header's `position_scale`), and its
/// heading, pitch and roll in 4096ths of a turn, each from -2048 to 2047 (`attitude` in tape_dump.lua makes them of
/// the unit's front and up directions; `read::attitude` turns them back).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pose {
    pub pos: [i32; 3],
    pub rot: [i16; 3],
}

/// The angles' scale: this many to a turn (0.35 degrees). The codec's differences wrap at it, so it is the format's,
/// not a tape's.
pub const ANGLE_TURN: i32 = 1024;

/// The raw stream's version this packer reads (`tape_dump.lua`'s header), and no other: a stream of an older widget is
/// captured again from its demo.
pub const RAW_VERSION: u32 = 6;

/// An angle (or a difference of two) brought into -512..=511.
pub fn wrap(angle: i32) -> i32 {
    (angle + ANGLE_TURN / 2).rem_euclid(ANGLE_TURN) - ANGLE_TURN / 2
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vital {
    pub health: u32,
    /// Build progress in thousandths: 1000 is finished.
    pub build: u16,
    pub paralyze: u32,
    /// 1 cloaked.
    pub flags: u8,
}

/// What a unit is doing: the head of its order queue.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Activity {
    /// The engine's command id; negative is a build of that unit type; 0 none.
    pub cmd: i32,
    /// How the order points: 0 nowhere, 1 at the unit or feature id `a` (the engine's), 3 at the place a, b, c (elmos).
    pub how: u8,
    pub a: i32,
    pub b: i32,
    pub c: i32,
    /// The engine id of the unit it is building, 0 none.
    pub builds: u32,
    pub queue: u16,
}

/// A keyframe of one of a unit's pieces (raw version 4): its turn against its parent, the script's own angles about x,
/// y and z in the header's `piece_angle_scale`ths of a radian (unwound: a spin keeps counting), and its place against
/// its parent (the offset plus the script's move) in `piece_place_scale`ths of an elmo. Between two keyframes of a
/// piece every number runs straight; before its first the piece is at rest, after its last it stays.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PieceKey {
    pub frame: u32,
    /// The piece's number in its model, from 1 (`GetUnitPieceList`'s order).
    pub piece: u16,
    pub angles: [i32; 3],
    pub place: [i16; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    pub engine_id: u32,
    pub def: u32,
    pub team: u8,
    pub created: u32,
    pub finished: u32,
    pub destroyed: u32,
    /// The rows (1-based, 0 none) of its killer and its builder.
    pub killer: u32,
    pub builder: u32,
    pub mobile: bool,
    pub max_health: f32,
    pub birth: Pose,
    pub birth_vital: Vital,
    /// Where it died (its birth place while it lives).
    pub death: [i32; 3],
    /// Teams it was given to: (frame, team).
    pub given: Vec<(u32, u8)>,
    pub samples: Vec<(u32, Pose)>,
    pub vitals: Vec<(u32, Vital)>,
    pub activity: Vec<(u32, Activity)>,
    /// (frame, [the ally teams that have it in sight, those that have it on radar]: bit `a` for ally team a).
    pub sight: Vec<(u32, [u16; 2])>,
    /// What the waste audit reads, each a row when it changed: (frame, [a builder's build power in
    /// use in hundredths of its build speed, a factory's line]); (frame, [experience, full health now]); and (frame,
    /// the damage it took since the last such row).
    pub work: Vec<(u32, [u16; 2])>,
    pub rank: Vec<(u32, [f32; 2])>,
    pub hurt: Vec<(u32, f32)>,
    /// Its pieces' keyframes, by piece and then frame (none in a stream captured without them).
    pub keys: Vec<PieceKey>,
}

pub const PATH_FORMULA: u8 = 0;
pub const PATH_BEAM: u8 = 1;
pub const PATH_SAMPLED: u8 = 2;

pub const AIM_NONE: u8 = 0;
pub const AIM_GROUND: u8 = 1;
pub const AIM_UNIT: u8 = 2;
pub const AIM_FEATURE: u8 = 3;
pub const AIM_PROJECTILE: u8 = 4;

#[derive(Debug, Clone, PartialEq)]
pub struct Projectile {
    pub id: u32,
    /// The weapon type; -1 a piece of a dying unit, -2 neither.
    pub wdef: i32,
    /// The owner's row, 1-based; 0 none.
    pub owner: u32,
    pub team: i32,
    /// First seen at `begin`, gone at `end`: in flight for the frames from `begin` to `end - 1`.
    pub begin: u32,
    pub end: u32,
    pub pos: [f32; 3],
    /// Elmos a frame, and elmos a frame a frame.
    pub vel: [f32; 3],
    pub gravity: f32,
    /// How a reader finds its place: by the formula from the start, as a beam from the start to the target, or from
    /// the samples.
    pub path: u8,
    pub aim: u8,
    /// The target: for a unit its row (1-based) in the first, for the ground the place.
    pub target: [f32; 3],
    pub samples: Vec<(u32, [f32; 3])>,
}

impl Projectile {
    /// Where the formula puts it `t` frames after its start: each frame the engine adds gravity to the speed, then
    /// the speed to the place (measured against the engine's own places, docs/harness/tape-format.md).
    pub fn formula(&self, t: f32) -> [f32; 3] {
        [self.pos[0] + self.vel[0] * t, self.pos[1] + self.vel[1] * t + self.gravity * t * (t + 1.0) * 0.5, self.pos[2] + self.vel[2] * t]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Burst {
    pub frame: u32,
    /// 0 an explosion, 1 muzzle fire, as the game reported them.
    pub kind: u8,
    pub pos: [f32; 3],
    pub wdef: i32,
    pub owner: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub frame: u32,
    /// The unit's row, 1-based; 0 when the order named a unit the tape never met.
    pub unit: u32,
    pub cmd: i32,
    pub shift: bool,
    pub params: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Feature {
    pub id: u32,
    pub def: u32,
    pub team: i32,
    pub pos: [f32; 3],
    pub front: [f32; 3],
    pub up: [f32; 3],
    pub metal: f32,
    pub energy: f32,
    pub created: u32,
    pub gone: u32,
}

/// A rectangle of the height map: `w` by `h` corners from column `x`, row `z`, row by row.
#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    pub frame: u32,
    pub x: u32,
    pub z: u32,
    pub w: u32,
    pub h: u32,
    pub heights: Vec<f32>,
}

#[derive(Debug, Default)]
pub struct Game {
    pub head: Value,
    /// Each team's ally team, by team.
    pub ally_of: Vec<u8>,
    pub unit_defs: Vec<Value>,
    pub weapon_defs: Vec<Value>,
    pub feature_defs: Vec<Value>,
    /// The ground at the start: (columns, rows, heights).
    pub ground: (u32, u32, Vec<f32>),
    pub patches: Vec<Patch>,
    pub units: Vec<Unit>,
    pub features: Vec<Feature>,
    pub projectiles: Vec<Projectile>,
    pub bursts: Vec<Burst>,
    pub orders: Vec<Order>,
    /// Each second's resources: (frame, [[team, metal, store, pull, income, spend, energy, store, pull, income, spend]]),
    /// from raw version 3 with metal's excess, sent and received and energy's after them.
    pub resources: Vec<(u32, Value)>,
    /// The wind's strength at each of those seconds (raw version 6), beside `resources`.
    pub wind: Vec<f32>,
    /// The engine's statistics of each team as it added an entry (raw version 3): (frame, [[team, metal used,
    /// produced, excess, energy used, produced, excess, damage dealt, received, units made, lost, killed]]).
    pub team_stats: Vec<(u32, Value)>,
    /// A feature's metal as it changed (a wreck reclaimed in part; raw version 3): (frame, its index in `features`,
    /// whole metal).
    pub feature_metal: Vec<(u32, u32, u32)>,
    /// Chat and the end, as the widget wrote them.
    pub events: Vec<Value>,
    pub last_frame: u32,
    pub winners: Value,
    pub stats: Value,
    /// The files a viewer draws with: the path in the engine's archives (lower case) -> the copy's name in the cache.
    pub assets: serde_json::Map<String, Value>,
    /// The map's own files among them (its `.smf` and `.smt` paths).
    pub map_files: Value,
    /// What the formula's places missed the engine's by, per weapon kind: (rows checked, the worst miss in elmos).
    pub path_check: HashMap<String, (u32, f32)>,
    /// Records of kinds this reader does not know, passed over: kind -> count.
    pub unknown: std::collections::BTreeMap<char, u32>,
}

fn num(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn triple(v: &Value, key: &str) -> [f32; 3] {
    let a = v.get(key).and_then(Value::as_array);
    let at = |i: usize| a.and_then(|a| a.get(i)).and_then(Value::as_f64).unwrap_or(0.0) as f32;
    [at(0), at(1), at(2)]
}

/// A row's place and attitude, which the widget wrote in the tape's own whole numbers.
fn pose_of(v: &Value) -> Pose {
    let (p, r) = (triple(v, "p"), triple(v, "rot"));
    Pose { pos: [p[0] as i32, p[1] as i32, p[2] as i32], rot: [r[0] as i16, r[1] as i16, r[2] as i16] }
}

fn u32s(r: &mut R, n: usize) -> Result<Vec<u32>> {
    (0..n).map(|_| r.u32()).collect()
}

fn f32s(r: &mut R, n: usize) -> Result<Vec<f32>> {
    (0..n).map(|_| r.f32()).collect()
}

impl Game {
    /// The frames a slice of the tape covers and the frames between unit samples, as the header says them.
    pub fn sample_frames(&self) -> u32 {
        num(&self.head, "sample_frames").max(1.0) as u32
    }

    pub fn position_scale(&self) -> f32 {
        num(&self.head, "position_scale").max(1.0) as f32
    }

    /// Whether a raw stream being written holds its first record, the header row, whole. The widget's writes reach the
    /// disk a buffer at a time, and a header longer than one (a 16-team game's teams and metal spots, 6 kB on All That
    /// Glitters) stands on disk cut short while the widget reads and hashes the map's files: a stream read then is not begun,
    /// where `ingest` would call it one without a header.
    pub fn header_written(raw: &[u8]) -> bool {
        raw.len() >= 5 && raw.len() - 5 >= u32::from_le_bytes([raw[1], raw[2], raw[3], raw[4]]) as usize
    }

    /// Reads a raw stream whole.
    pub fn ingest(raw: &[u8]) -> Result<Game> {
        let mut game = Game::default();
        let mut r = R::new(raw);
        // The engine's unit id -> the row of the unit that holds it now; the same for features and projectiles.
        let mut unit_at: HashMap<u32, usize> = HashMap::new();
        // The engine's unit id -> the row of the last unit that held it, alive or dead: a killer that died first (a
        // shell landing after its gun fell) is still named.
        let mut last_at: HashMap<u32, usize> = HashMap::new();
        let mut feature_at: HashMap<u32, usize> = HashMap::new();
        let mut projectile_at: HashMap<u32, usize> = HashMap::new();
        let mut mobile_def: HashMap<u32, bool> = HashMap::new();
        let mut weapon_kind: HashMap<i32, String> = HashMap::new();
        let row_of = |unit_at: &HashMap<u32, usize>, id: f64| if id > 0.0 { unit_at.get(&(id as u32)).map_or(0, |r| *r as u32 + 1) } else { 0 };
        while r.left() > 0 {
            // A stream still being written, or one whose engine was killed, can end inside a record: what stands
            // before it is the game so far.
            if r.left() < 5 {
                break;
            }
            let kind = r.u8()?;
            let len = r.u32()? as usize;
            if r.left() < len {
                break;
            }
            let mut body = R::new(r.take(len)?);
            match kind {
                b'J' => {
                    let row: Value = serde_json::from_slice(body.take(len)?).map_err(|e| Error(format!("a JSON row of the raw stream does not parse: {e}")))?;
                    let frame = num(&row, "f") as u32;
                    game.last_frame = game.last_frame.max(frame);
                    match row.get("t").and_then(Value::as_str).unwrap_or("") {
                        "header" => {
                            let version = num(&row, "version") as u32;
                            if version != RAW_VERSION {
                                return err(format!("the raw stream is of version {version}; this packer reads version {RAW_VERSION} (a stream of another widget: capture the game again)"));
                            }
                            for t in row.get("teams").and_then(Value::as_array).into_iter().flatten() {
                                let team = num(t, "team") as usize;
                                if game.ally_of.len() <= team {
                                    game.ally_of.resize(team + 1, 0);
                                }
                                game.ally_of[team] = num(t, "ally") as u8;
                            }
                            game.head = row;
                        }
                        "unitdef" => {
                            mobile_def.insert(num(&row, "id") as u32, num(&row, "mobile") > 0.0);
                            game.unit_defs.push(row);
                        }
                        "weapondef" => {
                            weapon_kind.insert(num(&row, "id") as i32, row.get("kind").and_then(Value::as_str).unwrap_or("").to_string());
                            game.weapon_defs.push(row);
                        }
                        "featuredef" => game.feature_defs.push(row),
                        "asset" => {
                            if let (Some(path), Some(file)) = (row.get("path").and_then(Value::as_str), row.get("file")) {
                                game.assets.insert(path.to_string(), file.clone());
                            }
                        }
                        "map_files" => game.map_files = row.get("files").cloned().unwrap_or(Value::Null),
                        "res" => {
                            game.resources.push((frame, row.get("teams").cloned().unwrap_or(Value::Null)));
                            game.wind.push(num(&row, "wind") as f32);
                            if let Some(stats) = row.get("stats").filter(|s| s.as_array().is_some_and(|a| !a.is_empty())) {
                                game.team_stats.push((frame, stats.clone()));
                            }
                        }
                        "pieces" => {
                            // A unit type's pieces (raw version 4), kept with the type: names, parents, offsets.
                            let id = num(&row, "d");
                            if let Some(def) = game.unit_defs.iter_mut().find(|d| num(d, "id") == id) {
                                def["pieces"] = json!({ "names": row["names"], "parents": row["parents"], "offsets": row["offsets"] });
                            }
                        }
                        "fmetal" => {
                            for w in row.get("w").and_then(Value::as_array).into_iter().flatten() {
                                let pair = |i: usize| w.get(i).and_then(Value::as_f64).unwrap_or(0.0);
                                if let Some(&at) = feature_at.get(&(pair(0) as u32)) {
                                    game.feature_metal.push((frame, at as u32, pair(1).max(0.0) as u32));
                                }
                            }
                        }
                        "stats" => game.stats = row,
                        "over" => {
                            game.winners = row.get("winners").cloned().unwrap_or(Value::Null);
                            game.events.push(row);
                        }
                        "cmd" => {
                            let params = row.get("p").and_then(Value::as_array).map(|p| p.iter().map(|v| v.as_f64().unwrap_or(0.0) as i32).collect()).unwrap_or_default();
                            game.orders.push(Order { frame, unit: row_of(&unit_at, num(&row, "u")), cmd: num(&row, "c") as i32, shift: num(&row, "shift") > 0.0, params });
                        }
                        "ev" => match row.get("k").and_then(Value::as_str).unwrap_or("") {
                            "created" => {
                                let (id, def) = (num(&row, "u") as u32, num(&row, "d") as u32);
                                let birth = pose_of(&row);
                                let birth_vital = Vital { health: num(&row, "health").round() as u32, build: (num(&row, "build") * 1000.0).round() as u16, paralyze: 0, flags: 0 };
                                let builder = row_of(&unit_at, num(&row, "by"));
                                unit_at.insert(id, game.units.len());
                                last_at.insert(id, game.units.len());
                                game.units.push(Unit {
                                    engine_id: id, def, team: num(&row, "team") as u8, created: frame, finished: NEVER, destroyed: NEVER, killer: 0, builder,
                                    mobile: mobile_def.get(&def).copied().unwrap_or(true), max_health: num(&row, "max_health") as f32, birth, birth_vital,
                                    death: birth.pos, given: Vec::new(), samples: Vec::new(), vitals: Vec::new(), activity: Vec::new(), sight: Vec::new(),
                                    work: Vec::new(), rank: Vec::new(), hurt: Vec::new(), keys: Vec::new(),
                                });
                            }
                            "finished" => {
                                if let Some(&at) = unit_at.get(&(num(&row, "u") as u32)) {
                                    game.units[at].finished = frame;
                                }
                            }
                            "destroyed" => {
                                let id = num(&row, "u") as u32;
                                let by = num(&row, "by");
                                let killer = match row_of(&unit_at, by) {
                                    0 if by > 0.0 => last_at.get(&(by as u32)).map_or(0, |r| *r as u32 + 1),
                                    row => row,
                                };
                                if let Some(at) = unit_at.remove(&id) {
                                    let unit = &mut game.units[at];
                                    unit.destroyed = frame;
                                    unit.killer = killer;
                                    unit.death = pose_of(&row).pos;
                                    // Its vitals as it died: its build progress then (a frame killed unfinished), which
                                    // no sample may have caught.
                                    if let Some(build) = row.get("build").and_then(Value::as_f64) {
                                        let mut vital = unit.vital(frame);
                                        let build = (build * 1000.0).round() as u16;
                                        if vital.build != build {
                                            vital.build = build;
                                            unit.vitals.push((frame, vital));
                                        }
                                    }
                                }
                            }
                            "given" => {
                                if let Some(&at) = unit_at.get(&(num(&row, "u") as u32)) {
                                    game.units[at].given.push((frame, num(&row, "team") as u8));
                                }
                            }
                            "feature" => {
                                let id = num(&row, "id") as u32;
                                feature_at.insert(id, game.features.len());
                                game.features.push(Feature {
                                    id, def: num(&row, "d") as u32, team: num(&row, "team") as i32, pos: triple(&row, "p"), front: triple(&row, "front"), up: triple(&row, "up"),
                                    metal: num(&row, "metal") as f32, energy: num(&row, "energy") as f32, created: frame, gone: NEVER,
                                });
                            }
                            "feature_gone" => {
                                if let Some(at) = feature_at.remove(&(num(&row, "id") as u32)) {
                                    game.features[at].gone = frame;
                                }
                            }
                            _ => game.events.push(row),
                        },
                        _ => {}
                    }
                }
                b'U' => {
                    let (frame, n) = (body.u32()?, body.u32()? as usize);
                    game.last_frame = game.last_frame.max(frame);
                    let ids: Vec<u16> = (0..n).map(|_| body.u16()).collect::<Result<_>>()?;
                    let pos: Vec<i32> = (0..3 * n).map(|_| body.u32().map(|v| v as i32)).collect::<Result<_>>()?;
                    let rot: Vec<i16> = (0..3 * n).map(|_| body.u16().map(|v| v as i16)).collect::<Result<_>>()?;
                    for (i, id) in ids.iter().enumerate() {
                        let Some(&at) = unit_at.get(&(*id as u32)) else { continue };
                        let pose = Pose { pos: pos[i * 3..i * 3 + 3].try_into().unwrap(), rot: rot[i * 3..i * 3 + 3].try_into().unwrap() };
                        game.units[at].samples.push((frame, pose));
                    }
                }
                b'V' => {
                    let (frame, n) = (body.u32()?, body.u32()? as usize);
                    let rows = f32s(&mut body, n * 5)?;
                    for row in rows.chunks_exact(5) {
                        let Some(&at) = unit_at.get(&(row[0] as u32)) else { continue };
                        let vital = Vital { health: row[1].max(0.0) as u32, build: (row[2] * 1000.0).round() as u16, paralyze: row[3].max(0.0) as u32, flags: row[4] as u8 };
                        game.units[at].vitals.push((frame, vital));
                    }
                }
                b'O' => {
                    let (frame, n) = (body.u32()?, body.u32()? as usize);
                    let rows = f32s(&mut body, n * 8)?;
                    for row in rows.chunks_exact(8) {
                        let Some(&at) = unit_at.get(&(row[0] as u32)) else { continue };
                        let (builds, queue) = (row[6] as u32, row[7].min(65535.0) as u16);
                        let activity = Activity { cmd: row[1] as i32, how: row[2] as u8, a: row[3] as i32, b: row[4] as i32, c: row[5] as i32, builds, queue };
                        game.units[at].activity.push((frame, activity));
                    }
                }
                b'S' => {
                    let (frame, n) = (body.u32()?, body.u32()? as usize);
                    for _ in 0..n {
                        let (id, los, radar) = (body.u16()?, body.u16()?, body.u16()?);
                        let Some(&at) = unit_at.get(&(id as u32)) else { continue };
                        game.units[at].sight.push((frame, [los, radar]));
                    }
                }
                b'K' => {
                    let n = body.u32()? as usize;
                    let frames = u32s(&mut body, n)?;
                    let ids: Vec<u16> = (0..n).map(|_| body.u16()).collect::<Result<_>>()?;
                    let pieces: Vec<u16> = (0..n).map(|_| body.u16()).collect::<Result<_>>()?;
                    let angles: Vec<i32> = (0..3 * n).map(|_| body.u32().map(|v| v as i32)).collect::<Result<_>>()?;
                    let places: Vec<i16> = (0..3 * n).map(|_| body.u16().map(|v| v as i16)).collect::<Result<_>>()?;
                    for i in 0..n {
                        let Some(&at) = unit_at.get(&(ids[i] as u32)) else { continue };
                        game.units[at].keys.push(PieceKey { frame: frames[i], piece: pieces[i], angles: angles[i * 3..i * 3 + 3].try_into().unwrap(), place: places[i * 3..i * 3 + 3].try_into().unwrap() });
                    }
                }
                b'W' => {
                    let (frame, n) = (body.u32()?, body.u32()? as usize);
                    for _ in 0..n {
                        let (id, power, line) = (body.u16()?, body.u16()?, body.u16()?);
                        let Some(&at) = unit_at.get(&(id as u32)) else { continue };
                        game.units[at].work.push((frame, [power, line]));
                    }
                }
                b'R' => {
                    let (frame, n) = (body.u32()?, body.u32()? as usize);
                    let rows = f32s(&mut body, n * 3)?;
                    for row in rows.chunks_exact(3) {
                        let Some(&at) = unit_at.get(&(row[0] as u32)) else { continue };
                        game.units[at].rank.push((frame, [row[1], row[2]]));
                    }
                }
                b'H' => {
                    let (frame, n) = (body.u32()?, body.u32()? as usize);
                    let rows = f32s(&mut body, n * 2)?;
                    for row in rows.chunks_exact(2) {
                        let Some(&at) = unit_at.get(&(row[0] as u32)) else { continue };
                        game.units[at].hurt.push((frame, row[1]));
                    }
                }
                b'P' => {
                    let (frame, n) = (body.u32()?, body.u32()? as usize);
                    game.last_frame = game.last_frame.max(frame);
                    let rows = f32s(&mut body, n * 16)?;
                    for row in rows.chunks_exact(16) {
                        let (id, wdef) = (row[0] as u32, row[1] as i32);
                        let kind = weapon_kind.get(&wdef).map(String::as_str).unwrap_or("");
                        let path = match kind {
                            "BeamLaser" | "LightningCannon" => PATH_BEAM,
                            "Cannon" | "AircraftBomb" => PATH_FORMULA,
                            _ if wdef < 0 => PATH_FORMULA,
                            _ => PATH_SAMPLED,
                        };
                        let aim = row[12] as u8;
                        let target = if aim == AIM_UNIT { [row_of(&unit_at, row[13] as f64) as f32, 0.0, 0.0] } else { [row[13], row[14], row[15]] };
                        projectile_at.insert(id, game.projectiles.len());
                        game.projectiles.push(Projectile {
                            id, wdef, owner: row_of(&unit_at, row[2] as f64), team: row[3] as i32, begin: frame, end: NEVER, pos: [row[4], row[5], row[6]],
                            vel: [row[7], row[8], row[9]], gravity: row[10], path, aim, target, samples: Vec::new(),
                        });
                    }
                }
                b'Q' => {
                    let (frame, n) = (body.u32()?, body.u32()? as usize);
                    let rows = f32s(&mut body, n * 4)?;
                    for row in rows.chunks_exact(4) {
                        let Some(&at) = projectile_at.get(&(row[0] as u32)) else { continue };
                        game.projectiles[at].samples.push((frame, [row[1], row[2], row[3]]));
                    }
                }
                b'E' => {
                    let (frame, n) = (body.u32()?, body.u32()? as usize);
                    for id in u32s(&mut body, n)? {
                        if let Some(at) = projectile_at.remove(&id) {
                            game.projectiles[at].end = frame;
                        }
                    }
                }
                b'X' | b'B' => {
                    let (frame, n) = (body.u32()?, body.u32()? as usize);
                    let rows = f32s(&mut body, n * 5)?;
                    for row in rows.chunks_exact(5) {
                        game.bursts.push(Burst { frame, kind: (kind == b'B') as u8, pos: [row[0], row[1], row[2]], wdef: row[3] as i32, owner: row_of(&unit_at, row[4] as f64) });
                    }
                }
                b'G' => {
                    let (frame, x, z, w, h) = (body.u32()?, body.u32()?, body.u32()?, body.u32()?, body.u32()?);
                    let heights = f32s(&mut body, (w * h) as usize)?;
                    game.patches.push(Patch { frame, x, z, w, h, heights });
                }
                // A record of a later widget than this reader (an arena binary older than the widget it copies): its
                // length is known, so it is passed over and counted, and the rest packed.
                other => *game.unknown.entry(other as char).or_default() += 1,
            }
        }
        if game.head.is_null() {
            return err("the raw stream has no header row");
        }
        game.settle(&weapon_kind);
        Ok(game)
    }

    /// After the last row: the ground at the start put together from its first patches, the projectiles still in
    /// flight ended, each formula-path projectile checked against the engine's own places (and given its samples to
    /// fly by when the formula misses), the lists in frame order.
    fn settle(&mut self, weapon_kind: &HashMap<i32, String>) {
        for u in &mut self.units {
            u.keys.sort_by_key(|k| (k.piece, k.frame));
            u.keys.dedup_by_key(|k| (k.piece, k.frame));
        }
        let map = self.head.get("map").cloned().unwrap_or(Value::Null);
        let (columns, rows) = ((num(&map, "width") / 8.0) as u32 + 1, (num(&map, "height") / 8.0) as u32 + 1);
        let mut heights = vec![0.0f32; (columns * rows) as usize];
        let start: Vec<Patch> = self.patches.iter().filter(|p| p.frame == 0 && p.x == 0 && p.w == columns).cloned().collect();
        for patch in &start {
            let from = (patch.z * columns) as usize;
            let to = (from + patch.heights.len()).min(heights.len());
            heights[from..to].copy_from_slice(&patch.heights[..to - from]);
        }
        self.patches.retain(|p| !(p.frame == 0 && p.x == 0 && p.w == columns));
        self.ground = (columns, rows, heights);
        let end = self.last_frame + 1;
        for p in &mut self.projectiles {
            if p.end == NEVER {
                p.end = end;
            }
            if p.path == PATH_SAMPLED {
                continue;
            }
            if p.path == PATH_FORMULA && p.wdef >= 0 {
                // A piece of a dying unit bounces and is drawn by the formula all the same: it is debris.
                let kind = weapon_kind.get(&p.wdef).cloned().unwrap_or_default();
                let mut worst = 0.0f32;
                for (frame, at) in &p.samples {
                    let f = p.formula((*frame - p.begin) as f32);
                    worst = worst.max(((f[0] - at[0]).powi(2) + (f[1] - at[1]).powi(2) + (f[2] - at[2]).powi(2)).sqrt());
                }
                let entry = self.path_check.entry(kind).or_insert((0, 0.0));
                entry.0 += p.samples.len() as u32;
                entry.1 = entry.1.max(worst);
                if worst > 4.0 {
                    p.path = PATH_SAMPLED;
                    continue;
                }
            }
            p.samples.clear();
        }
        self.orders.sort_by_key(|o| o.frame);
        self.bursts.sort_by_key(|b| b.frame);
    }
}

/// The last entry of a list in frame order stamped at or before `frame`.
pub fn at_or_before<T>(list: &[(u32, T)], frame: u32) -> Option<&T> {
    let n = list.partition_point(|(f, _)| *f <= frame);
    if n == 0 { None } else { Some(&list[n - 1].1) }
}

impl Unit {
    pub fn alive(&self, frame: u32) -> bool {
        self.created <= frame && frame < self.destroyed
    }

    pub fn pose(&self, frame: u32) -> Pose {
        at_or_before(&self.samples, frame).copied().unwrap_or(self.birth)
    }

    pub fn vital(&self, frame: u32) -> Vital {
        at_or_before(&self.vitals, frame).copied().unwrap_or(self.birth_vital)
    }

    pub fn doing(&self, frame: u32) -> Activity {
        at_or_before(&self.activity, frame).copied().unwrap_or_default()
    }

    pub fn team_at(&self, frame: u32) -> u8 {
        at_or_before(&self.given, frame).copied().unwrap_or(self.team)
    }

    /// Which ally teams have it in sight and on radar at `frame`; before its first look, its own (`own`, the ally
    /// team of the team that made it) alone.
    pub fn sight(&self, frame: u32, own: u8) -> [u16; 2] {
        let mine = 1u16.checked_shl(own as u32).unwrap_or(0);
        at_or_before(&self.sight, frame).copied().unwrap_or([mine, mine])
    }

    /// One piece's keyframes (a run of `keys`).
    pub fn piece_keys(&self, piece: u16) -> &[PieceKey] {
        let from = self.keys.partition_point(|k| k.piece < piece);
        let to = self.keys.partition_point(|k| k.piece <= piece);
        &self.keys[from..to]
    }

    /// Its build power in use and a factory's line as last written at or before `frame` (none: nothing).
    pub fn work_at(&self, frame: u32) -> [u16; 2] {
        at_or_before(&self.work, frame).copied().unwrap_or_default()
    }

    /// Its experience and full health as last written at or before `frame` (none: no experience, the full health it
    /// was made with).
    pub fn rank_at(&self, frame: u32) -> [f32; 2] {
        at_or_before(&self.rank, frame).copied().unwrap_or([0.0, self.max_health])
    }
}

impl Game {
    /// Whether the stream has the units' moving pieces (not captured without them).
    pub fn has_pieces(&self) -> bool {
        num(&self.head, "pieces") > 0.0
    }

    /// Each unit type's pieces at rest: each one's offset from its parent in the tape's units of a place
    /// (`piece_place_scale`ths of an elmo), by the type's id.
    pub fn piece_rest(&self) -> HashMap<u32, Vec<[f64; 3]>> {
        let scale = num(&self.head, "piece_place_scale").max(1.0);
        self.unit_defs
            .iter()
            .filter_map(|d| {
                let offsets = d.get("pieces")?.get("offsets")?.as_array()?;
                let rest = offsets.iter().map(|o| [0, 1, 2].map(|k| (o.get(k).and_then(Value::as_f64).unwrap_or(0.0) * scale).round())).collect();
                Some((num(d, "id") as u32, rest))
            })
            .collect()
    }

    pub fn ally(&self, team: u8) -> u8 {
        self.ally_of.get(team as usize).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A header row longer than the cut: on disk while the widget's buffer holds its end.
    #[test]
    fn a_stream_cut_inside_its_header_is_not_begun() {
        let teams: Vec<Value> = (0..17).map(|t| serde_json::json!({ "team": t, "ally": t / 8, "name": "BARb stable", "color": [0.1, 0.2, 0.3], "start": [1000.5, 200.25, 3000.75] })).collect();
        let spots: Vec<Value> = (0..152).map(|i| serde_json::json!([i * 40, 192, 1836.8])).collect();
        let head = serde_json::json!({ "t": "header", "format": "within-reason-tape-raw", "version": RAW_VERSION, "teams": teams, "spots": spots, "map": { "width": 64, "height": 64 } });
        let body = serde_json::to_vec(&head).unwrap();
        assert!(body.len() > 4096, "the header is {} bytes", body.len());
        let mut raw = vec![b'J'];
        raw.extend_from_slice(&(body.len() as u32).to_le_bytes());
        raw.extend_from_slice(&body);
        for cut in [0, 1, 4, 5, 4096, raw.len() - 1] {
            assert!(!Game::header_written(&raw[..cut]), "cut at {cut}");
            assert!(Game::ingest(&raw[..cut]).is_err(), "cut at {cut}");
        }
        assert!(Game::header_written(&raw));
        assert_eq!(num(&Game::ingest(&raw).unwrap().head, "version") as u32, RAW_VERSION);
    }
}
