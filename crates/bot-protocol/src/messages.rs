use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UnitId(pub i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UnitDefId(pub i32);

/// A map feature: a wreck, a rock, a tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FeatureId(pub i32);

/// Map position in elmos; `y` is height.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// A weapon as the engine names it, with its range: what hit us when the shooter is out of sight. The game names a
/// unit's weapons `<unit>_<weapon>` (`corstorm_cor_bot_rocket`); its explosions (a death blast, `fusionexplosion`)
/// carry no unit and no range (the engine's default, 10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Weapon {
    pub name: String,
    pub range: f32,
}

/// How a unit died, from the engine's kill event: the weapon of the hit that killed it (a gun's, a warhead's, an
/// explosion's such as a death blast), or one of the engine's own causes, which it passes as negative weapon ids
/// (`CSolidObject::DamageType`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum Death {
    /// The kill event carried no cause: a shim from before 2026-10-19.
    #[default]
    Unknown,
    Weapon(Weapon),
    /// Flying debris of a dying unit (a piece projectile's explosion).
    Debris,
    /// Taken apart by a constructor (the event's `attacker`).
    Reclaimed,
    /// An unfinished frame nobody worked on for long enough.
    Decayed,
    SelfDestruct,
    /// The engine's other causes by their `DamageType` number: collisions (2, 3), crushed (7), an aircraft's crash
    /// (8), a transport's or a factory's death (14, 15), a Lua kill (21 and beyond).
    Other(i32),
}

impl Vec3 {
    /// Horizontal distance; height is ignored.
    pub fn dist2d(self, other: Vec3) -> f32 {
        (self.x - other.x).hypot(self.z - other.z)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ToBot {
    Hello(Hello),
    Tick(Tick),
}

/// Static game data, sent once per connection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hello {
    pub ai_id: i32,
    pub team: i32,
    pub ally_team: i32,
    /// The same for every AI of one game and different from game to game (a hash of the start script): how the
    /// bot process tells which of its sessions play together.
    pub game_id: u64,
    /// Every team in the game, ours included.
    pub teams: Vec<TeamInfo>,
    /// Where each ally team may start, for the ally teams whose box the start script gives. The boxes mean
    /// something only under `start_pos_type` 2 (chosen in game): a map's default boxes ride along in every script.
    pub start_boxes: Vec<StartBox>,
    /// The script's `startpostype`: 0 fixed (the map's positions, each team's in its `[TEAMn]`), 1 random, 2 chosen
    /// in the game inside the boxes, 3 chosen before the game (each team's in its `[TEAMn]`); None when unsaid.
    #[serde(default)]
    pub start_pos_type: Option<i32>,
    /// The whole start script, for the record (`script.txt` beside it) and for reading what the lobby set.
    #[serde(default)]
    pub script: String,
    pub frame: i32,
    /// Frames between ticks (the sim runs 30 a second): the shim's setting, so that nothing in the bot assumes one.
    pub tick_frames: i32,
    pub map: MapInfo,
    pub unit_defs: Vec<UnitDefInfo>,
    pub metal_spots: Vec<Vec3>,
    /// Per metal spot (same order), the centres of the metal-map squares (16 elmos) that make up its patch, from the
    /// engine's raw metal map: the game allows an extractor wherever its extractor radius covers every square.
    pub metal_spot_squares: Vec<Vec<(f32, f32)>>,
    pub terrain: Terrain,
}

/// One seat: a team has one commander and one economy; teams on one ally team fight together.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamInfo {
    pub team: i32,
    pub ally_team: i32,
    /// Faction name as the start script gives it; may be empty.
    pub side: String,
    /// Who plays the seat, from the start script; `Unknown` from a shim older than 2026-09-23.
    #[serde(default)]
    pub controller: Controller,
    /// The seat's start as the script gives it (`startposx`/`startposz` of its `[TEAMn]`), when the lobby fixed
    /// the positions (`start_pos_type` 0 or 3): exact for every seat, ours and theirs (shims from 2026-09-28).
    #[serde(default)]
    pub start_pos: Option<Vec3>,
    /// The seat's lobby colour (`rgbcolor` of its `[TEAMn]` section, 0-1 each), how people in the game name a
    /// seat; `None` from a shim older than 2026-09-27 or a script without one.
    #[serde(default)]
    pub color: Option<[f32; 3]>,
    /// The factor on the metal and energy the seat's units make (the lobby's bonus: 1.5 at +50 %), as the engine
    /// holds it; `None` from a shim older than 2026-10-04.
    #[serde(default)]
    pub income_multiplier: Option<f32>,
}

/// Who plays a seat, read from the start script's `[PLAYERn]` and `[AIn]` sections (human-9: the player could not
/// tell it was a 2v1 against a person and BARb).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Controller {
    #[default]
    Unknown,
    /// The engine's neutral team (critters, wrecks): in the game's team list but in no script section, nobody's.
    Gaia,
    /// A person: the lobby name, and the lobby's skill rating when the script carries one (`skill=[43.19]`).
    Person { name: String, skill: Option<f32> },
    /// An AI: its name in the game, its short name (WReason, BARb), version, and its `profile` option when set.
    Ai { name: String, short_name: String, version: String, profile: Option<String> },
}

/// An ally team's start box, in elmos. The engine tells nobody where another team actually started.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct StartBox {
    pub ally_team: i32,
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl StartBox {
    pub fn centre(&self) -> Vec3 {
        Vec3 { x: (self.left + self.right) / 2.0, y: 0.0, z: (self.top + self.bottom) / 2.0 }
    }

    pub fn contains(&self, pos: Vec3) -> bool {
        pos.x >= self.left && pos.x <= self.right && pos.z >= self.top && pos.z <= self.bottom
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapInfo {
    pub name: String,
    /// Extent in elmos.
    pub width: f32,
    pub height: f32,
    /// Wind speed range; a wind generator produces the current wind speed in energy, up to its cap.
    pub wind_min: f32,
    pub wind_max: f32,
    /// The energy a tidal generator makes a second on this map (`Map_getTidalStrength`; from 2026-09-28).
    #[serde(default)]
    pub tidal: f32,
    /// How far from a metal spot's squares an extractor still draws from them (`Game.extractorRadius`): the game
    /// (cmd_mex_denier.lua) allows an extractor anywhere its radius covers the whole spot.
    pub extractor_radius: f32,
}

impl MapInfo {
    /// How far from a spot's centre an extractor still stands on that spot: the game denies a second extractor
    /// within the extractor radius of the spot (cmd_mex_denier.lua `mexExists`), and our own site chooser places
    /// one up to that radius off the centre toward the builder. Every test of "is this spot held" uses this;
    /// four games read spots we held as free with a 100-elmo test against extractors placed 100-113 off centre.
    pub fn spot_radius(&self) -> f32 {
        self.extractor_radius.max(60.0)
    }
}

/// The ground, at the engine's slope-map resolution. Rows run north to south (z), cells west to east (x).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Terrain {
    /// Edge of one cell in elmos.
    pub cell: f32,
    pub width: u32,
    pub height: u32,
    /// Ground height in elmos; water level is 0, so negative is under water.
    pub heights: Vec<i16>,
    /// The engine's slope value (1 - the ground normal's y; 0 is flat), scaled by 255. Comparable with
    /// [`MoveClass::max_slope`] after the same scaling.
    pub slopes: Vec<u8>,
    /// The raw metal map at the same cells (the engine's metal map has this resolution), each value clamped to
    /// 0..=255; empty from a shim that predates it.
    #[serde(default)]
    pub metal: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MoveKind {
    Tank,
    Bot,
    Hover,
    Ship,
}

/// How a mobile ground or sea unit moves; aircraft and buildings have none.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MoveClass {
    pub kind: MoveKind,
    /// Steepest ground it crosses, as the engine's slope value.
    pub max_slope: f32,
    /// Deepest water a land unit wades; for a ship, the shallowest it floats in.
    pub depth: f32,
    /// The engine slows a unit on a slope to `1 / (1 + slope * slope_mod)` of its speed (`GroundMoveMath.cpp`).
    pub slope_mod: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitDefInfo {
    pub id: UnitDefId,
    pub name: String,
    pub metal_cost: f32,
    pub energy_cost: f32,
    pub speed: f32,
    pub build_speed: f32,
    /// Work one unit of `build_speed` does on it for one second adds `1 / build_time` of it.
    pub build_time: f32,
    /// How far from itself a builder builds, measured to the target's edge.
    pub build_distance: f32,
    pub extracts_metal: f32,
    /// Made every second whatever happens.
    pub metal_make: f32,
    pub energy_make: f32,
    /// Energy used every second; negative for what produces this way (solar collectors).
    pub energy_upkeep: f32,
    /// A wind generator makes the current wind speed in energy up to this.
    pub wind_cap: f32,
    /// A tidal generator makes the map's tidal strength in energy; 0 for anything else (records from 2026-09-28 on).
    #[serde(default)]
    pub tidal_make: f32,
    pub metal_storage: f32,
    pub energy_storage: f32,
    /// Sonar coverage it gives, in elmos; 0 for none: what sees submerged units (records and shims from 2026-09-28).
    #[serde(default)]
    pub sonar_range: f32,
    /// Whether any of its weapons is a water weapon (a torpedo, a depth charge): the only kind that hits a submerged target.
    #[serde(default)]
    pub hits_submerged: bool,
    /// Whether every weapon it has is a water weapon: it hits nothing that is not in the water.
    #[serde(default)]
    pub water_only: bool,
    /// Radar coverage it gives, in elmos; 0 for a unit with none.
    pub radar_range: f32,
    /// For an energy converter (the game's `energyconv_*` custom parameters).
    pub converter: Option<Converter>,
    pub weapon_count: i32,
    pub build_options: Vec<UnitDefId>,
    pub move_class: Option<MoveClass>,
    /// The ground it stands on, in 8-elmo squares (x, z) before facing: a factory's exit lane runs off its front.
    pub footprint: (i32, i32),
    /// What its death does to what stands around it; None for a unit that dies quietly.
    pub death_blast: Option<Blast>,
    /// What its self-destruct does (the commander's is the game's largest); None when it has none.
    pub self_destruct_blast: Option<Blast>,
    /// Seconds from the self-destruct order to the blast.
    pub self_destruct_seconds: f32,
    /// The range of its longest ordinary weapon (the commander's D-gun and other manual-fire weapons left out); 0 unarmed.
    #[serde(default)]
    pub reach: f32,
    /// Seconds between that weapon's shots; 0 unarmed.
    #[serde(default)]
    pub reload: f32,
    /// The largest area of effect among its ordinary weapons (manual-fire left out), as the engine gives it
    /// (`WeaponDef_getAreaOfEffect`): a radius, half the unit file's `areaofeffect` (the Fatboy's file says 300, this
    /// is 150). What a shell landing on one unit also does to the units within it; the micro spaces a body by it
    /// (H-MICRO-FORM-SPACING). 0 unarmed, and in records and shims from before 2026-09-28.
    #[serde(default)]
    pub blast_radius: f32,
    /// Seconds a round of its stockpiled weapon takes (`WeaponDef_getStockpileTime`): 120 for Armada's silo, 180 for
    /// Cortex's, 90 for an anti-nuke; 0 for a unit that stockpiles nothing, and in records and shims from before
    /// 2026-10-09. A round advances at this rate whenever the frame's share of its cost can be paid, and no build
    /// power touches it (K-rules-a-stockpile-takes-no-build-power).
    #[serde(default)]
    pub stockpile_seconds: f32,
    /// Energy a round of its stockpiled weapon costs (125,000 for Armada's missile, 7,500 for an interceptor); 0 as above.
    /// Over `stockpile_seconds` it is the draw a stockpiling silo puts on the economy (1,042 a second).
    #[serde(default)]
    pub stockpile_energy: f32,
    /// An interceptor's coverage: a missile aimed within this of an anti-nuke is taken by it (2,000 in BAR); 0 for a unit
    /// with no interceptor, and in records and shims from before 2026-10-09.
    #[serde(default)]
    pub coverage: f32,
    /// Sight radius in elmos (`UnitDef_getLosRadius`): what it sees over flat ground; terrain cuts it, as the
    /// snapshot's `sensors` grids show. 0 in records and shims from before 2026-10-18, when the glossary's figure stands in.
    #[serde(default)]
    pub sight: f32,
    /// Radar jamming radius (`UnitDef_getJammerRadius`): everything of its side within it is hidden from our radar
    /// (BAR jams per side); 0 for a unit that jams nothing. Sight is not jammed.
    #[serde(default)]
    pub jam_range: f32,
    /// Invisible to radar whatever covers it (`UnitDef_isStealth`): only sight finds it.
    #[serde(default)]
    pub stealth: bool,
    /// How it cloaks, for a unit that can (`UnitDef_isAbleToCloak`): invisible to sight and radar until something of
    /// ours stands within `decloak_distance`, or it fires; None for the rest.
    #[serde(default)]
    pub cloak: Option<Cloak>,
    /// Its shield, for a unit that carries one (`UnitDef_getShieldDef`); None for the rest.
    #[serde(default)]
    pub shield: Option<Shield>,
    /// Which shields stop its shots: the OR of its ordinary weapons' `interceptedByShieldType` (the engine's default
    /// 1 for a cannon, 2 for a laser, 4 for a missile; BAR sets a starburst rocket to 1024). A shield takes a shot
    /// when `shield.intercepts & intercepted_by != 0`. 0 unarmed.
    #[serde(default)]
    pub intercepted_by: u32,
}

/// How a unit cloaks (the engine's cloak fields).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cloak {
    /// Energy a second it costs to stay cloaked standing still.
    pub cost: f32,
    /// An enemy within this of it decloaks it (BAR's `mincloakdistance`: 55 for a Pitbull).
    pub decloak_distance: f32,
    /// Whether firing decloaks it.
    pub decloak_on_fire: bool,
    /// Whether it is cloaked from birth.
    pub starts_cloaked: bool,
}

/// A unit's shield (the engine's `Shield` weapon).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shield {
    /// The bubble's radius in elmos.
    pub radius: f32,
    /// Damage it absorbs before it falls (it regenerates).
    pub power: f32,
    /// The bitmask of shot kinds it stops, against each weapon's `intercepted_by`.
    pub intercepts: u32,
    /// Whether it stops only shots from outside the bubble (a unit inside it shoots out and in freely).
    pub exterior: bool,
}

/// An explosion: full `damage` at its centre, falling to nothing at `radius` elmos.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Blast {
    pub radius: f32,
    pub damage: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Converter {
    /// Energy per second it can take.
    pub capacity: f32,
    /// Metal returned per energy.
    pub efficiency: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tick {
    pub frame: i32,
    /// Frames this tick was sent after the frame it was due at, for want of the bot's answer to the previous one
    /// (always 0 in lockstep). `frame - late` is the due frame, which the bot's own schedule keys on.
    pub late: i32,
    /// Everything that happened since the previous tick, in order.
    pub events: Vec<Event>,
    pub snapshot: Snapshot,
}

impl Tick {
    /// The frame this tick fell due at: what the bot's schedule keys on, so that a late tick is not a missed one.
    pub fn due(&self) -> i32 {
        self.frame - self.late
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Snapshot {
    pub metal: Resource,
    pub energy: Resource,
    /// The wind blowing now (the map's `wind_min` to `wind_max`): what a wind generator makes this second.
    pub wind: f32,
    pub own_units: Vec<OwnUnit>,
    /// Units of the other teams on our ally team: seen, never commanded.
    pub allies: Vec<AllyUnit>,
    /// Enemies currently in line of sight or radar.
    pub enemies: Vec<EnemyUnit>,
    /// Reclaimable features in sight worth the walk (wrecks mostly), the richest first, capped. `None` on the ticks the
    /// shim did not look (it looks every few seconds): what was sent last still stands.
    pub wrecks: Option<Vec<Wreck>>,
    /// Our side's sensor coverage this second, terrain-cut as the engine computes it; `None` on the ticks the shim
    /// did not read it (it reads once a second), and always from a shim before 2026-10-18.
    #[serde(default)]
    pub sensors: Option<Sensors>,
}

/// Where our ally team sees, hears radar and sees aircraft right now: the engine's own coverage grids, cut by terrain
/// for sight and radar (`Map_getLosMap`, `Map_getRadarMap`, `Map_getAirLosMap`). What enemy jammers hide is not in
/// them (BAR jams per side, so the engine's jammer map is our own); it follows from a seen jammer and its `jam_range`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Sensors {
    pub los: Grid,
    pub radar: Grid,
    pub air: Grid,
}

/// A coverage grid: one bit per square, row-major from the north-west (index 0 is x 0, z 0), `square` elmos a side.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Grid {
    pub cols: u32,
    pub rows: u32,
    pub square: f32,
    /// `cols * rows` bits, packed eight to a byte, the first square in the lowest bit of the first byte.
    pub bits: Vec<u8>,
}

impl Grid {
    /// Whether the square holding the point is covered; a point off the grid is not.
    pub fn at(&self, x: f32, z: f32) -> bool {
        if self.square <= 0.0 || x < 0.0 || z < 0.0 {
            return false;
        }
        let (col, row) = ((x / self.square) as u32, (z / self.square) as u32);
        if col >= self.cols || row >= self.rows {
            return false;
        }
        let index = (row * self.cols + col) as usize;
        self.bits.get(index / 8).is_some_and(|byte| byte & (1 << (index % 8)) != 0)
    }

    /// The share of the squares covered, 0 to 1.
    pub fn share(&self) -> f32 {
        let total = (self.cols * self.rows) as f32;
        if total <= 0.0 {
            return 0.0;
        }
        self.bits.iter().map(|b| b.count_ones()).sum::<u32>() as f32 / total
    }

    /// Packs `covered` (one entry per square, row-major) into a grid.
    pub fn pack(cols: u32, rows: u32, square: f32, covered: impl Iterator<Item = bool>) -> Grid {
        let mut bits = vec![0u8; ((cols * rows) as usize).div_ceil(8)];
        for (index, on) in covered.enumerate().take((cols * rows) as usize) {
            if on {
                bits[index / 8] |= 1 << (index % 8);
            }
        }
        Grid { cols, rows, square, bits }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wreck {
    pub id: FeatureId,
    pub pos: Vec3,
    /// Metal left in it.
    pub metal: f32,
    /// The unit a resurrection bot can raise from it.
    pub resurrects_into: Option<UnitDefId>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Resource {
    pub current: f32,
    pub income: f32,
    pub usage: f32,
    pub storage: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OwnUnit {
    pub id: UnitId,
    pub def: UnitDefId,
    pub pos: Vec3,
    /// Elmos a frame.
    pub vel: Vec3,
    pub health: f32,
    pub max_health: f32,
    pub being_built: bool,
    /// Command queue is empty.
    pub idle: bool,
    /// The frame at which its first weapon can next fire; 0 for a unit without one. For the control lane's kiting.
    pub reload_frame: i32,
    /// The engine's building facing: 0 south (+z), 1 east (+x), 2 north (-z), 3 west (-x). A factory's units leave
    /// through its front (`Factory.cpp` `SendToEmptySpot`). 0 for a mobile unit.
    pub facing: i32,
    /// Rounds ready in a stockpiler (a silo's missiles, an anti-nuke's interceptors); 0 for every other unit, and in
    /// shims from before 2026-10-09.
    #[serde(default)]
    pub stockpile: i32,
    /// Rounds the game is still stockpiling toward its kept count (`Command::Stockpile`); 0 as above.
    #[serde(default)]
    pub stockpile_queued: i32,
    /// The game's builder-priority flag is low (`Command::Priority`; the unit rules param `builderPriority`): false for
    /// a unit without the flag, and in shims from before 2026-10-13.
    #[serde(default)]
    pub priority_low: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AllyUnit {
    pub id: UnitId,
    pub def: UnitDefId,
    pub pos: Vec3,
    pub team: i32,
    pub being_built: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyUnit {
    pub id: UnitId,
    /// Unknown for radar-only contacts.
    pub def: Option<UnitDefId>,
    pub pos: Vec3,
    /// Elmos a frame, for where it will be next.
    pub vel: Vec3,
    pub health: f32,
    /// Whose it is, when the engine tells (it does for units in sight).
    pub team: Option<i32>,
    /// Still under construction (in sight only; a radar contact is never): it cannot shoot, and it dies to anything.
    #[serde(default)]
    pub being_built: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    UnitCreated { unit: UnitId, builder: Option<UnitId> },
    UnitFinished { unit: UnitId },
    UnitIdle { unit: UnitId },
    UnitMoveFailed { unit: UnitId },
    /// `from`: a unit vector from the hit unit toward the attacker, which the engine gives whether or not the
    /// attacker is in sight (only `attacker` is withheld then); `weapon`: what hit it. Both `None` for damage with
    /// no attacker (a collision, a crash).
    UnitDamaged { unit: UnitId, attacker: Option<UnitId>, damage: f32, #[serde(default)] from: Option<Vec3>, #[serde(default)] weapon: Option<Weapon> },
    /// `attacker`: the killer when the engine names one (one of his in sight, or one of ours for its fire or its
    /// reclaim; none for a missile strike or a death blast); `death`: how it died, from the kill event's weapon id.
    UnitDestroyed { unit: UnitId, attacker: Option<UnitId>, #[serde(default)] death: Death },
    /// A unit of ours hit an enemy (the engine sends this to the attacker's team only, and only while the enemy is
    /// in sight or on radar); `attacker` is ours, `None` when the engine names none.
    EnemyDamaged { enemy: UnitId, attacker: Option<UnitId>, damage: f32, #[serde(default)] weapon: Option<Weapon> },
    /// A unit of ours fired a weapon; with `EnemyDamaged` the record tells a soldier that shoots from one that stands
    /// in reach with its line of fire blocked by a friend (the engine refuses the shot: `avoidFriendly`).
    WeaponFired { unit: UnitId, #[serde(default)] weapon: Option<Weapon> },
    /// A chat line from a player in the game (the engine's message event), by player number.
    Chat { player: i32, text: String },
    EnemyEnterLos { enemy: UnitId },
    EnemyLeaveLos { enemy: UnitId },
    /// An enemy died in the team's sight (allies share sight, so every seat hears of it). `attacker` is the unit
    /// whose weapon killed it when that unit is allied with us (ours or an ally's), `None` when the engine names
    /// none or the killer is not allied: a kill is ours only when `attacker` is one of our own units
    /// (H-PLAYER-OWN-TRADE).
    EnemyDestroyed { enemy: UnitId, #[serde(default)] attacker: Option<UnitId> },
    /// A [`Command::Build`] was dropped because the shim found no legal site.
    BuildSiteNotFound { unit: UnitId, def: UnitDefId },
    /// A build order's site as the host placed it (the engine's closest free site to what the bot asked): where
    /// the frame will stand, so the bot keeps later sites out of it while nothing stands there yet.
    BuildSited { unit: UnitId, def: UnitDefId, at: Vec3 },
    /// The engine rejected a command.
    CommandRejected { unit: UnitId, code: i32 },
    /// A unit became ours from another team (a transfer between seats, a capture).
    UnitGiven { unit: UnitId, from_team: i32, to_team: i32 },
    /// A unit of ours went to another team.
    UnitTaken { unit: UnitId, from_team: i32, to_team: i32 },
    /// Metal and energy our allies' overflow put into our stock over the engine's last second (`Economy_getReceived`).
    /// The engine's income leaves them out: the overflow goes straight to the stock (`CTeam::SlowUpdate`), so a bot
    /// reading the income alone sees an ally's gift only as a rising stock. Sent once a second while either is above
    /// zero.
    Received { metal: f32, energy: f32 },
}

/// The bot's reply to a tick; also the credit that lets the shim send the next one.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Commands(pub Vec<Command>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    /// `site` is `None` for factories, which build in place and always append to their build
    /// queue; `queue` is ignored for them (the engine reads that option bit as "build five").
    Build { unit: UnitId, def: UnitDefId, site: Option<BuildSite>, queue: bool },
    Move { unit: UnitId, to: Vec3, queue: bool },
    Fight { unit: UnitId, to: Vec3, queue: bool },
    Stop { unit: UnitId },
    SetRepeat { unit: UnitId, repeat: bool },
    /// Follow and help `target`: a builder guarding a factory adds its build power to whatever the factory makes.
    Guard { unit: UnitId, target: UnitId },
    /// Take one feature apart for its metal.
    ReclaimFeature { unit: UnitId, feature: FeatureId, queue: bool },
    /// Take one unit of ours apart for its metal: the player's way to remove a building that stands in a factory's
    /// exit lane (docs/design/2026-09-22-yard-and-reclaim.md).
    ReclaimUnit { unit: UnitId, target: UnitId, queue: bool },
    /// Raise the unit a wreck was (resurrection bots only); it costs energy and time, no metal.
    Resurrect { unit: UnitId, feature: FeatureId, queue: bool },
    /// Restore `target`'s health (a builder's job; it also finishes a stalled construction).
    Repair { unit: UnitId, target: UnitId, queue: bool },
    /// Cheat: a finished unit of type `def` appears at `at`, owned by this AI's team. For the duel harness
    /// (`docs/harness/duels.md`); the engine honours it only in a game hosted locally with a single player.
    GiveUnit { def: UnitDefId, at: Vec3 },
    /// Cheat: this much metal and energy into the team's stores, honoured where `GiveUnit` is. For the drill bench's
    /// producing lab (`docs/design/2026-10-24-formation-drills.md`, D5).
    GiveResource { metal: f32, energy: f32 },
    /// Starts the unit's self-destruct countdown (a second order cancels it).
    SelfDestruct { unit: UnitId },
    /// Attack one unit of theirs: the turret first, deliberately, when a raid or an answer is priced against it.
    Attack { unit: UnitId, target: UnitId, queue: bool },
    /// The engine's move state: 0 hold position, 1 manoeuvre, 2 roam. An aircraft with an empty queue hunts targets
    /// within 1000 times its move state, so an air group holds by setting 0 (docs/design/2026-09-22-domains.md).
    MoveState { unit: UnitId, state: i32 },
    /// A line in the game chat, to everyone: the bot names itself at the start (the game gives AIs random names;
    /// `{name}` in the text becomes the one given to this AI).
    Say { text: String },
    /// The engine's fire state: 0 hold fire, 1 return fire, 2 fire at will. A unit holding fire still carries out an
    /// explicit `Attack`. The duel harness holds a scenario's units while it prepares them (`docs/harness/duels.md`).
    FireState { unit: UnitId, state: i32 },
    /// Give metal and energy to an allied team (the game caps it to what the receiver can hold; a tax option, when
    /// set, takes its share). Between seats of ours in a game with people (docs/design/2026-09-27-posing-changes.md §12b).
    SendResources { metal: f32, energy: f32, to_team: i32 },
    /// Give these units of ours to an allied team; the engine sends `UnitTaken` to us and `UnitGiven` to them.
    SendUnits { units: Vec<UnitId>, to_team: i32 },
    /// The commander's D-gun at one unit (a manual-fire weapon: never fired on its own).
    DGun { unit: UnitId, target: UnitId },
    /// A command-fire weapon at a place: a silo's missile at the ground under his buildings (BAR's nuke is
    /// `commandfire`: it never fires on its own, and a round sits in the silo until this order), or the commander's
    /// D-gun at the ground. The engine's attack at a position for a type without `canManualFire` (the silo: a
    /// command-fire weapon then fires on a plain attack), its manual-fire command for one with it (the commander).
    FireAt { unit: UnitId, at: Vec3 },
    /// Move a stockpiler's kept count by `change` rounds. The game fills every silo and anti-nuke to its unit file's
    /// limit on its own from the moment it stands (BAR's "Stockpile control" gadget: ten missiles for a silo, each
    /// drawing 1,042 energy a second for its two minutes), so there is nothing to order a round with; this order moves
    /// the count the gadget keeps the unit at, within 0 and that limit (K-rules-a-stockpile-fills-itself-to-its-limit).
    Stockpile { unit: UnitId, change: i32 },
    /// The game's builder-priority flag (BAR's `unit_builder_priority` gadget, `CMD_PRIORITY` 34571 with one parameter,
    /// 0 low, 1 high; K-mech-builder-priority-is-a-builder-flag): a low builder runs at full speed only out of the
    /// team's surplus and stands still otherwise; high is the game's default and changes nothing. On commanders,
    /// constructors, factories and construction turrets; the gadget consumes the order without queueing it, and the
    /// flag is read back as `OwnUnit::priority_low`.
    Priority { unit: UnitId, low: bool },
}

/// The shim resolves this to the closest legal build position, since only it can query the map.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildSite {
    pub near: Vec3,
    pub search_radius: f32,
    /// Minimum gap to other buildings, in build squares.
    pub min_dist: i32,
    /// The facings to try, in order, each with the ground no site may have its centre on for that facing. The shim
    /// keeps the first that has a site and builds with it; empty means the engine's default facing and no keep-out.
    /// A factory's exit lane runs off its front, so its facing decides where its units can leave (player-29: an
    /// Advanced Vehicle Plant faced the map's south edge, its lane off the map, and every unit it made stood stuck).
    pub placements: Vec<Placement>,
    /// The engine's closest site within `search_radius` of `near` or nothing: the shim does not walk the search's
    /// centre outwards when that fails or lands on a spot or in the keep-out (the territory bot's farm slots, which
    /// are the ground asked for or none: a building 200 elmos off its slot blocks two others', terr55). Off, the
    /// shim walks the centre out to 600 elmos for a site.
    #[serde(default)]
    pub exact: bool,
}

/// One way to stand a building: the engine's facing (0 south +z, 1 east +x, 2 north -z, 3 west -x) and the ground
/// a site with that facing must keep its centre off: our factories' exit lanes, and for a factory the strips from
/// which its own lane would cover a building of ours, unwalkable ground or the map's edge
/// (docs/design/2026-09-22-yard-and-reclaim.md).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Placement {
    pub facing: i32,
    pub keep_out: Vec<Lane>,
}

impl BuildSite {
    /// The facing the shim builds with: the placement it kept, else the engine's default.
    pub fn facing(&self) -> Option<i32> {
        self.placements.first().map(|p| p.facing)
    }
}

/// A strip of ground: within `half_width` of the segment `from`-`to`. A factory's exit lane runs from its centre out
/// through its front.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lane {
    pub from: Vec3,
    pub to: Vec3,
    pub half_width: f32,
}

impl Lane {
    /// Whether `p` lies on the strip (in the ground plane).
    pub fn contains(&self, p: Vec3) -> bool {
        let (dx, dz) = (self.to.x - self.from.x, self.to.z - self.from.z);
        let len2 = (dx * dx + dz * dz).max(1e-6);
        let t = (((p.x - self.from.x) * dx + (p.z - self.from.z) * dz) / len2).clamp(0.0, 1.0);
        let (cx, cz) = (self.from.x + t * dx, self.from.z + t * dz);
        (p.x - cx).hypot(p.z - cz) <= self.half_width
    }
}

#[cfg(test)]
mod lane_tests {
    use super::{Lane, Vec3};

    #[test]
    fn a_lane_is_a_strip_from_the_factory_out_through_its_front() {
        let at = |x: f32, z: f32| Vec3 { x, y: 0.0, z };
        // A plant at (2440, 3123) facing south (+z): hands-2's solar 165 elmos off its front sat in the lane.
        let lane = Lane { from: at(2440.0, 3123.0), to: at(2440.0, 3123.0 + 400.0), half_width: 100.0 };
        assert!(lane.contains(at(2456.0, 3288.0)));
        assert!(lane.contains(at(2440.0, 3523.0)), "the far end is on the strip");
        assert!(!lane.contains(at(2440.0, 3000.0)), "behind the factory is not");
        assert!(!lane.contains(at(2600.0, 3288.0)), "beside the lane is not");
        assert!(!lane.contains(at(2440.0, 3700.0)), "well past the end is not (the strip has round ends)");
    }
}

#[cfg(test)]
mod grid_tests {
    use super::Grid;

    /// A coverage grid packs one bit a square from the north-west and answers by position; a point off the grid is
    /// uncovered. BAR's sight grid on an 8192 map is 128 squares of 64 elmos.
    #[test]
    fn a_grid_packs_squares_and_answers_by_position() {
        let covered = (0..128 * 128).map(|i| i % 128 >= 64 && i / 128 < 2);
        let grid = Grid::pack(128, 128, 64.0, covered);
        assert_eq!(grid.bits.len(), 2048);
        assert!(grid.at(4100.0, 10.0), "the east half of the first row is covered");
        assert!(grid.at(8191.0, 127.0));
        assert!(!grid.at(4000.0, 10.0), "the west half is not");
        assert!(!grid.at(4100.0, 200.0), "the third row is not");
        assert!(!grid.at(-1.0, 10.0) && !grid.at(9000.0, 10.0), "off the grid is uncovered");
        assert!((grid.share() - 128.0 / 16384.0).abs() < 1e-6);
        assert!(!Grid::default().at(10.0, 10.0));
    }
}
