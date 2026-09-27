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

/// A weapon as the engine names it, with its range: what hit us when the shooter is out of sight.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Weapon {
    pub name: String,
    pub range: f32,
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
    /// Where each ally team may start, for the ally teams whose box the start script gives.
    pub start_boxes: Vec<StartBox>,
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
    /// The seat's lobby colour (`rgbcolor` of its `[TEAMn]` section, 0-1 each), how people in the game name a
    /// seat; `None` from a shim older than 2026-09-27 or a script without one.
    #[serde(default)]
    pub color: Option<[f32; 3]>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub metal_storage: f32,
    pub energy_storage: f32,
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
    UnitDestroyed { unit: UnitId, attacker: Option<UnitId> },
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
    EnemyDestroyed { enemy: UnitId },
    /// A [`Command::Build`] was dropped because the shim found no legal site.
    BuildSiteNotFound { unit: UnitId, def: UnitDefId },
    /// The engine rejected a command.
    CommandRejected { unit: UnitId, code: i32 },
    /// A unit became ours from another team (a transfer between seats, a capture).
    UnitGiven { unit: UnitId, from_team: i32, to_team: i32 },
    /// A unit of ours went to another team.
    UnitTaken { unit: UnitId, from_team: i32, to_team: i32 },
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
}

/// The shim resolves this to the closest legal build position, since only it can query the map.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildSite {
    pub near: Vec3,
    pub search_radius: f32,
    /// Minimum gap to other buildings, in build squares.
    pub min_dist: i32,
    /// Ground no site may have its centre on: our factories' exit lanes (docs/design/2026-09-22-yard-and-reclaim.md).
    pub keep_out: Vec<Lane>,
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
