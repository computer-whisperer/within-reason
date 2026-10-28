//! Safe wrapper over the engine's callback table, exposing only what the host needs.
//!
//! Every method must be called on the engine thread, from inside one of the library's exports.

use std::ffi::{CStr, CString, c_int, c_void};

use bot_protocol::{
    Grid, Sensors,
    Controller,
    AllyUnit, Blast, BuildSite, Command, Converter, EnemyUnit, Hello, Lane, MapInfo, MoveClass, MoveKind, OwnUnit, Resource, Snapshot, Terrain,
    FeatureId, StartBox, TeamInfo, UnitDefId, UnitDefInfo, UnitId, Vec3, Wreck,
};
use recoil_ai_sys as sys;

/// Heightmap squares to elmos.
const SQUARE_SIZE: f32 = 8.0;
const MAX_UNITS: usize = 32_000;
/// Features with less metal than this are not worth telling the bot about (trees, pebbles).
const WRECK_MIN_METAL: f32 = 25.0;
const MAX_WRECKS: usize = 150;
/// Base buildings stay this far from any metal spot, leaving room for the extractor and a path to it.
const SPOT_KEEPOUT: f32 = 100.0;

pub struct Engine {
    ai_id: c_int,
    callback: *const sys::SSkirmishAICallback,
    metal: c_int,
    energy: c_int,
    id_buf: Vec<c_int>,
    /// Metal spots, kept so base buildings stay off them.
    metal_spots: Vec<Vec3>,
    /// Per feature type: the metal a whole one holds, or `None` when it cannot be reclaimed.
    feature_metal: std::collections::HashMap<c_int, Option<f32>>,
    /// Per unit type: whether it stockpiles rounds (a silo, an anti-nuke), so the snapshot asks the engine for a
    /// stockpile count only where there is one.
    stockpilers: std::collections::HashMap<c_int, bool>,
}

/// The one engine call that must be made with the instance table unlocked: the cheat that creates a unit, during
/// which the engine delivers the unit's creation events to every AI.
#[derive(Clone, Copy)]
pub struct Spawner {
    ai_id: c_int,
    callback: *const sys::SSkirmishAICallback,
}

/// Calls a callback-table entry, which always takes the AI id first.
macro_rules! call {
    ($engine:expr, $name:ident ( $($arg:expr),* )) => {
        unsafe { ((*$engine.callback).$name.expect(stringify!($name)))($engine.ai_id $(, $arg)*) }
    };
}

impl Engine {
    /// The colour a team is drawn in on the machine this AI runs on (the host's): the engine's own, which the
    /// game's AutoColorPicker gadget sets over the lobby's `rgbcolor` (`luarules/gadgets/game_autocolors.lua`:
    /// blues and greens for the first ally team, reds and yellows for the second, whatever the lobby said). The
    /// user, human-22: the colours the player names do not match the ones in the game. Falls back to the
    /// script's when the engine has none. A client with "simple team colors" on sees its own three colours, which
    /// nothing here can know.
    fn shown_color(&self, team: i32, scripted: Option<[f32; 3]>) -> Option<[f32; 3]> {
        let mut rgb = [0i16; 3];
        if unsafe { (*self.callback).Game_getTeamColor }.is_some() {
            call!(self, Game_getTeamColor(team, rgb.as_mut_ptr()));
        }
        let shown = rgb.iter().any(|c| *c != 0).then(|| rgb.map(|c| (c as f32 / 255.0).clamp(0.0, 1.0)));
        if shown != scripted {
            eprintln!("[shim] team {team} colour: the script's {scripted:?}, the engine's {shown:?}");
        }
        shown.or(scripted)
    }
}

impl Spawner {
    /// A finished unit of type `def` at `at` for this AI's team, by the synced console command `/give` sent as the
    /// AI's text: the engine takes an AI's text that starts with a slash as a command (`CAICallback::SendTextMsg`,
    /// `CGame::ProcessCommandText`) and wraps a synced one as a `CommandMessage` over the net, so the spawn is in the
    /// demo and a replay re-simulates it. Until 2026-10-24 this was the AI cheat interface's `GiveMeNewUnit`, which
    /// loads the unit on the host's simulation directly, so a demo of a duel showed two commanders standing and
    /// desync warnings (docs/harness/pitfalls.md). `/give` needs the game's cheat flag, which [`Engine::console`]'s
    /// `/cheat 1` sets from the host's first update; the flag comes back from the server a frame on, and the unsynced
    /// give executor refuses locally until it has (`UnsyncedActionExecutor.h`), so the host holds spawns until then
    /// (`lib.rs` `CHEAT_LAG`). The unit appears when the command comes back from the server, not within this call, so
    /// `Err` says only that the engine refused the text. A single unit is placed at the position itself, facing south
    /// (`CUnitLoader::GiveUnits`).
    pub fn give(self, def: UnitDefId, at: Vec3) -> Result<(), i32> {
        let team = call!(self, SkirmishAI_getTeamId());
        let name_ptr = call!(self, UnitDef_getName(def.0));
        let name = if name_ptr.is_null() { String::new() } else { unsafe { CStr::from_ptr(name_ptr) }.to_string_lossy().into_owned() };
        if name.is_empty() {
            return Err(-1);
        }
        console(self.ai_id, self.callback, &format!("/give 1 {name} {team} @{:.1},0,{:.1}", at.x, at.z))
    }
}

/// A console command sent as this AI's text (`CAICallback::SendTextMsg` takes a line that starts with a slash as a
/// command and nothing else from an AI). A synced one travels as a `CommandMessage` over the net and is recorded.
fn console(ai_id: c_int, callback: *const sys::SSkirmishAICallback, line: &str) -> Result<(), i32> {
    let text = CString::new(line).unwrap_or_default();
    let mut command = sys::SSendTextMessageCommand { text: text.as_ptr(), zone: 0 };
    let code = unsafe {
        ((*callback).Engine_handleCommand.expect("Engine_handleCommand"))(ai_id, sys::COMMAND_TO_ID_ENGINE, -1, sys::COMMAND_SEND_TEXT_MESSAGE as c_int, std::ptr::from_mut(&mut command).cast::<c_void>())
    };
    if code == 0 { Ok(()) } else { Err(code) }
}

impl Engine {
    pub fn spawner(&self) -> Spawner {
        Spawner { ai_id: self.ai_id, callback: self.callback }
    }

    /// A console command as this AI's text (see [`console`]); `/cheat 1` is what the spawner needs.
    pub fn console(&self, line: &str) -> Result<(), i32> {
        console(self.ai_id, self.callback, line)
    }

    /// # Safety
    /// `callback` must be the table the engine passed to `init` for `ai_id`, and stay valid
    /// until `release`.
    pub unsafe fn new(ai_id: c_int, callback: *const sys::SSkirmishAICallback) -> Self {
        let mut engine = Engine { ai_id, callback, metal: -1, energy: -1, id_buf: vec![0; MAX_UNITS], metal_spots: Vec::new(), feature_metal: Default::default(), stockpilers: Default::default() };
        engine.metal = call!(engine, getResourceByName(c"Metal".as_ptr()));
        engine.energy = call!(engine, getResourceByName(c"Energy".as_ptr()));
        engine
    }

    pub fn hello(&mut self, frame: i32, tick_frames: i32) -> Hello {
        let map = MapInfo {
            name: self.string(call!(self, Map_getName())),
            width: call!(self, Map_getWidth()) as f32 * SQUARE_SIZE,
            height: call!(self, Map_getHeight()) as f32 * SQUARE_SIZE,
            wind_min: call!(self, Map_getMinWind()),
            wind_max: call!(self, Map_getMaxWind()),
            tidal: call!(self, Map_getTidalStrength()),
            extractor_radius: call!(self, Map_getExtractorRadius(self.metal)),
        };
        let def_count = call!(self, getUnitDefs(std::ptr::null_mut(), 0));
        let mut def_ids = vec![0; def_count.max(0) as usize];
        call!(self, getUnitDefs(def_ids.as_mut_ptr(), def_count));
        let unit_defs = def_ids.into_iter().map(|id| self.unit_def(id)).collect();

        let float_count = call!(self, Map_getResourceMapSpotsPositions(self.metal, std::ptr::null_mut(), 0));
        let mut floats = vec![0f32; float_count.max(0) as usize];
        call!(self, Map_getResourceMapSpotsPositions(self.metal, floats.as_mut_ptr(), float_count));
        let engine_spots: Vec<Vec3> = floats.chunks_exact(3).map(|s| Vec3 { x: s[0], y: s[1], z: s[2] }).collect();
        // The game's own spot list, when the game publishes one (BAR's api_resource_spot_finder sets the game rules
        // params `mex_count`, `mex_x<i>`, `mex_z<i>`; the replay widget reads the same): the positions extractors
        // snap to, and the numbers the replay cards use, so the brief, the picture and the cards name the same spots.
        // The engine's list, snapped to the patch centroids below, has matched it index for index on Comet Catcher
        // since 2026-09-23; it stays the fallback for a game without the params, and a difference is logged.
        let game_spots: Vec<Vec3> = {
            let count = call!(self, Game_getRulesParamFloat(c"mex_count".as_ptr(), 0.0)) as usize;
            (1..=count)
                .map_while(|i| {
                    let (kx, kz) = (CString::new(format!("mex_x{i}")).unwrap_or_default(), CString::new(format!("mex_z{i}")).unwrap_or_default());
                    let (x, z) = (call!(self, Game_getRulesParamFloat(kx.as_ptr(), f32::NAN)), call!(self, Game_getRulesParamFloat(kz.as_ptr(), f32::NAN)));
                    (!x.is_nan() && !z.is_nan()).then(|| {
                        let y = engine_spots.iter().min_by(|a, b| a.dist2d(Vec3 { x, y: 0.0, z }).total_cmp(&b.dist2d(Vec3 { x, y: 0.0, z }))).map_or(0.0, |s| s.y);
                        Vec3 { x, y, z }
                    })
                })
                .collect()
        };
        let from_game = !game_spots.is_empty();
        let mut metal_spots: Vec<Vec3> = if from_game { game_spots } else { engine_spots.clone() };
        // The raw metal map: one value per 2x2 heightmap squares (16 elmos), index 0 top left. Each square with metal
        // belongs to the nearest spot within a patch's reach of it.
        const METAL_SQUARE: f32 = 2.0 * SQUARE_SIZE;
        const PATCH_REACH: f32 = 200.0;
        let (half_w, half_h) = ((call!(self, Map_getWidth()) / 2) as usize, (call!(self, Map_getHeight()) / 2) as usize);
        let raw_count = call!(self, Map_getResourceMapRaw(self.metal, std::ptr::null_mut(), 0));
        let mut raw = vec![0i16; raw_count.max(0) as usize];
        call!(self, Map_getResourceMapRaw(self.metal, raw.as_mut_ptr(), raw_count));
        let mut metal_spot_squares: Vec<Vec<(f32, f32)>> = vec![Vec::new(); metal_spots.len()];
        // (sum of x times metal, sum of z times metal, sum of metal) per spot, for the centroid below.
        let mut weighted: Vec<(f64, f64, f64)> = vec![(0.0, 0.0, 0.0); metal_spots.len()];
        for (index, value) in raw.iter().enumerate() {
            if *value <= 0 || half_w == 0 {
                continue;
            }
            let (x, z) = (((index % half_w) as f32 + 0.5) * METAL_SQUARE, ((index / half_w) as f32 + 0.5) * METAL_SQUARE);
            if index / half_w >= half_h {
                break;
            }
            let nearest = metal_spots.iter().enumerate().min_by(|a, b| {
                let da = (a.1.x - x).hypot(a.1.z - z);
                let db = (b.1.x - x).hypot(b.1.z - z);
                da.total_cmp(&db)
            });
            if let Some((i, spot)) = nearest
                && (spot.x - x).hypot(spot.z - z) < PATCH_REACH
            {
                metal_spot_squares[i].push((x, z));
                let w = *value as f64;
                weighted[i] = (weighted[i].0 + x as f64 * w, weighted[i].1 + z as f64 * w, weighted[i].2 + w);
            }
        }
        // The engine's spot positions are not where the game puts extractors: on Comet Catcher they sit 82 elmos
        // (+40, +72) from every extractor BARb and people build (human-9, the escalate-7 truth file), and the site
        // asked at the corner spot was refused. The game and BARb snap to the metal-weighted centre of the patch,
        // so that is the spot we publish; the squares stay for placing anywhere the extractor radius allows.
        if !from_game {
            for (spot, (sx, sz, sw)) in metal_spots.iter_mut().zip(&weighted) {
                if *sw > 0.0 {
                    spot.x = (sx / sw) as f32;
                    spot.z = (sz / sw) as f32;
                }
            }
        }
        // A spot's amount is its patch's worth: the squares' metal times the map's scale (`maxMetal`), what an
        // extractor standing where it covers the whole patch draws when multiplied by its `extractsMetal`. The
        // engine's own figure (the y of its spot list) is the same sum round its spot point, which is off the patch
        // and can miss squares of it.
        let scale = f64::from(call!(self, Map_getMaxResource(self.metal)));
        for (spot, (_, _, sw)) in metal_spots.iter_mut().zip(&weighted) {
            if *sw > 0.0 {
                spot.y = (sw * scale) as f32;
            }
        }
        let differing = metal_spots.iter().filter(|g| engine_spots.iter().all(|e| e.dist2d(**g) > 130.0)).count();
        eprintln!(
            "[wreason ai={}] metal spots: {} from {} (the engine's list has {}, {differing} of ours nowhere within 130 of one of its)",
            self.ai_id,
            metal_spots.len(),
            if from_game { "the game's rules params (mex_count, mex_x<i>, mex_z<i>)" } else { "the engine's resource map, snapped to the patch centroids" },
            engine_spots.len()
        );
        self.metal_spots = metal_spots.clone();

        let script = self.string(call!(self, Game_getSetupScript()));
        let controllers = crate::script::controllers(&script);
        let scripted_teams = crate::script::teams(&script);
        let colors = crate::script::colors(&script);
        let start_pos_type = crate::script::start_pos_type(&script);
        let starts = crate::script::start_positions(&script);
        let teams = (0..call!(self, Game_getTeams()))
            .map(|team| TeamInfo {
                team,
                ally_team: call!(self, Game_getTeamAllyTeam(team)),
                side: self.string(call!(self, Game_getTeamSide(team))),
                start_pos: starts.iter().find(|(t, _, _)| *t == team).map(|(_, x, z)| Vec3 { x: *x, y: 0.0, z: *z }),
                color: self.shown_color(team, colors.iter().find(|(t, _)| *t == team).map(|(_, c)| *c)),
                income_multiplier: unsafe { (*self.callback).Game_getTeamIncomeMultiplier }.is_some().then(|| call!(self, Game_getTeamIncomeMultiplier(team))),
                controller: if !scripted_teams.is_empty() && !scripted_teams.contains(&team) {
                    Controller::Gaia
                } else {
                    controllers.iter().find(|(t, _)| *t == team).map_or(Controller::Unknown, |(_, c)| c.clone())
                },
            })
            .collect();
        let game_id = {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            script.hash(&mut hasher);
            hasher.finish()
        };
        let start_boxes = crate::script::start_rects(&script)
            .into_iter()
            .map(|(ally_team, [left, top, right, bottom])| StartBox {
                ally_team,
                left: left * map.width,
                top: top * map.height,
                right: right * map.width,
                bottom: bottom * map.height,
            })
            .collect();

        Hello {
            ai_id: self.ai_id,
            team: call!(self, SkirmishAI_getTeamId()),
            ally_team: call!(self, Game_getMyAllyTeam()),
            game_id,
            teams,
            start_boxes,
            start_pos_type,
            script,
            frame,
            tick_frames,
            map,
            unit_defs,
            metal_spots,
            metal_spot_squares,
            terrain: self.terrain(),
        }
    }

    fn move_class(&self, id: c_int) -> Option<MoveClass> {
        if !call!(self, UnitDef_isMoveDataAvailable(id)) {
            return None;
        }
        // The engine's speed-mod classes, in its own order.
        let kind = match call!(self, UnitDef_MoveData_getSpeedModClass(id)) {
            0 => MoveKind::Tank,
            1 => MoveKind::Bot,
            2 => MoveKind::Hover,
            _ => MoveKind::Ship,
        };
        Some(MoveClass {
            kind,
            max_slope: call!(self, UnitDef_MoveData_getMaxSlope(id)),
            depth: call!(self, UnitDef_MoveData_getDepth(id)),
            slope_mod: call!(self, UnitDef_MoveData_getSlopeMod(id)),
        })
    }

    /// Heights and slopes at the slope map's resolution (two height squares to a cell).
    fn terrain(&self) -> Terrain {
        let (squares_x, squares_z) = (call!(self, Map_getWidth()) as usize, call!(self, Map_getHeight()) as usize);
        let (width, height) = (squares_x / 2, squares_z / 2);
        let mut heights = vec![0f32; squares_x * squares_z];
        let mut slopes = vec![0f32; width * height];
        let got_heights = call!(self, Map_getHeightMap(heights.as_mut_ptr(), heights.len() as c_int)) as usize;
        let got_slopes = call!(self, Map_getSlopeMap(slopes.as_mut_ptr(), slopes.len() as c_int)) as usize;
        if got_heights != heights.len() || got_slopes != slopes.len() {
            return Terrain::default();
        }
        let cell_height = |x: usize, z: usize| {
            let at = |dx: usize, dz: usize| heights[(2 * z + dz) * squares_x + 2 * x + dx];
            ((at(0, 0) + at(1, 0) + at(0, 1) + at(1, 1)) / 4.0).round().clamp(i16::MIN as f32, i16::MAX as f32) as i16
        };
        // The raw metal map has the slope map's resolution (one value per 2x2 height squares); the viewer draws the
        // patches from it, beside the spots the bot publishes.
        let raw_count = call!(self, Map_getResourceMapRaw(self.metal, std::ptr::null_mut(), 0));
        let mut raw = vec![0i16; raw_count.max(0) as usize];
        call!(self, Map_getResourceMapRaw(self.metal, raw.as_mut_ptr(), raw_count));
        let metal = if raw.len() == width * height { raw.iter().map(|v| (*v).clamp(0, 255) as u8).collect() } else { Vec::new() };
        Terrain {
            cell: 2.0 * SQUARE_SIZE,
            width: width as u32,
            height: height as u32,
            heights: (0..height).flat_map(|z| (0..width).map(move |x| (x, z))).map(|(x, z)| cell_height(x, z)).collect(),
            slopes: slopes.iter().map(|s| (s * 255.0).round().clamp(0.0, 255.0) as u8).collect(),
            metal,
        }
    }

    fn unit_def(&mut self, id: c_int) -> UnitDefInfo {
        let option_count = call!(self, UnitDef_getBuildOptions(id, std::ptr::null_mut(), 0));
        let mut options = vec![0; option_count.max(0) as usize];
        call!(self, UnitDef_getBuildOptions(id, options.as_mut_ptr(), option_count));
        let (reach, reload, blast_radius) = self.longest_weapon(id);
        let water_weapons = self.water_weapons(id);
        let stockpile = self.stockpile(id);
        UnitDefInfo {
            id: UnitDefId(id),
            name: self.string(call!(self, UnitDef_getName(id))),
            metal_cost: call!(self, UnitDef_getCost(id, self.metal)),
            energy_cost: call!(self, UnitDef_getCost(id, self.energy)),
            speed: call!(self, UnitDef_getSpeed(id)),
            build_speed: call!(self, UnitDef_getBuildSpeed(id)),
            build_time: call!(self, UnitDef_getBuildTime(id)),
            build_distance: call!(self, UnitDef_getBuildDistance(id)),
            extracts_metal: call!(self, UnitDef_getExtractsResource(id, self.metal)),
            metal_make: call!(self, UnitDef_getResourceMake(id, self.metal)),
            energy_make: call!(self, UnitDef_getResourceMake(id, self.energy)),
            energy_upkeep: call!(self, UnitDef_getUpkeep(id, self.energy)),
            wind_cap: call!(self, UnitDef_getWindResourceGenerator(id, self.energy)),
            tidal_make: call!(self, UnitDef_getTidalResourceGenerator(id, self.energy)),
            metal_storage: call!(self, UnitDef_getStorage(id, self.metal)),
            energy_storage: call!(self, UnitDef_getStorage(id, self.energy)),
            radar_range: call!(self, UnitDef_getRadarRadius(id)) as f32,
            sonar_range: call!(self, UnitDef_getSonarRadius(id)) as f32,
            hits_submerged: water_weapons.0,
            water_only: water_weapons.1,
            converter: self.converter(id),
            weapon_count: call!(self, UnitDef_getWeaponMounts(id)),
            build_options: options.into_iter().map(UnitDefId).collect(),
            move_class: self.move_class(id),
            footprint: (call!(self, UnitDef_getXSize(id)), call!(self, UnitDef_getZSize(id))),
            death_blast: self.blast(call!(self, UnitDef_getDeathExplosion(id))),
            self_destruct_blast: self.blast(call!(self, UnitDef_getSelfDExplosion(id))),
            self_destruct_seconds: call!(self, UnitDef_getSelfDCountdown(id)) as f32,
            reach,
            reload,
            blast_radius,
            stockpile_seconds: stockpile.0,
            stockpile_energy: stockpile.1,
            coverage: self.coverage(id),
            sight: call!(self, UnitDef_getLosRadius(id)),
            jam_range: call!(self, UnitDef_getJammerRadius(id)) as f32,
            stealth: call!(self, UnitDef_isStealth(id)),
            cloak: self.cloak(id),
            shield: self.shield(id),
            intercepted_by: self.intercepted_by(id),
            health: call!(self, UnitDef_getHealth(id)),
            reclaim_speed: call!(self, UnitDef_getReclaimSpeed(id)),
            reclaimable: call!(self, UnitDef_isReclaimable(id)),
            stun_taken: self.custom_number(id, "paralyzemultiplier").unwrap_or(1.0),
            stun: self.stun(id),
        }
    }

    /// The type's stun weapon: of its paralyzer weapons, the one with the largest stun a shot.
    fn stun(&self, id: c_int) -> Option<bot_protocol::Stun> {
        let mounts = call!(self, UnitDef_getWeaponMounts(id)).max(0);
        let mut best: Option<bot_protocol::Stun> = None;
        for mount in 0..mounts {
            let weapon = call!(self, UnitDef_WeaponMount_getWeaponDef(id, mount));
            if weapon < 0 || !call!(self, WeaponDef_isParalyzer(weapon)) {
                continue;
            }
            let count = call!(self, WeaponDef_Damage_getTypes(weapon, std::ptr::null_mut(), 0)).max(0);
            let mut damages = vec![0f32; count as usize];
            call!(self, WeaponDef_Damage_getTypes(weapon, damages.as_mut_ptr(), count));
            let per_shot = damages.into_iter().fold(0f32, f32::max);
            if best.is_none_or(|b| per_shot > b.per_shot) {
                best = Some(bot_protocol::Stun {
                    per_shot,
                    reach: call!(self, WeaponDef_getRange(weapon)),
                    reload: call!(self, WeaponDef_getReload(weapon)),
                    seconds: call!(self, WeaponDef_Damage_getParalyzeDamageTime(weapon)) as f32,
                });
            }
        }
        best
    }

    /// Seconds the unit stays stunned (`bot_protocol::STUN_DECLINE`); 0 when it is not, or when the engine will not say
    /// (an enemy out of sight).
    fn stunned(&self, id: c_int) -> f32 {
        if !call!(self, Unit_isParalyzed(id)) {
            return 0.0;
        }
        let (stun, health) = (call!(self, Unit_getParalyzeDamage(id)), call!(self, Unit_getMaxHealth(id)));
        if health <= 0.0 { 0.0 } else { (bot_protocol::STUN_DECLINE * (stun / health - 1.0)).max(0.0) }
    }

    /// How the type cloaks, for one that can; None for the rest.
    fn cloak(&self, id: c_int) -> Option<bot_protocol::Cloak> {
        if !call!(self, UnitDef_isAbleToCloak(id)) {
            return None;
        }
        Some(bot_protocol::Cloak {
            cost: call!(self, UnitDef_getCloakCost(id)),
            decloak_distance: call!(self, UnitDef_getDecloakDistance(id)),
            decloak_on_fire: call!(self, UnitDef_isDecloakOnFire(id)),
            starts_cloaked: call!(self, UnitDef_isStartCloaked(id)),
        })
    }

    /// The type's shield, for one that carries one; None for the rest.
    fn shield(&self, id: c_int) -> Option<bot_protocol::Shield> {
        let weapon = call!(self, UnitDef_getShieldDef(id));
        if weapon < 0 {
            return None;
        }
        Some(bot_protocol::Shield {
            radius: call!(self, WeaponDef_Shield_getRadius(weapon)),
            power: call!(self, WeaponDef_Shield_getPower(weapon)),
            intercepts: call!(self, WeaponDef_Shield_getInterceptType(weapon)).max(0) as u32,
            exterior: call!(self, WeaponDef_isExteriorShield(weapon)),
        })
    }

    /// Which shields stop the type's shots: the OR of its ordinary weapons' `interceptedByShieldType` (shield weapons
    /// and manual-fire ones left out).
    fn intercepted_by(&self, id: c_int) -> u32 {
        let mounts = call!(self, UnitDef_getWeaponMounts(id)).max(0);
        let mut mask = 0u32;
        for mount in 0..mounts {
            let weapon = call!(self, UnitDef_WeaponMount_getWeaponDef(id, mount));
            if weapon < 0 || call!(self, WeaponDef_isShield(weapon)) || call!(self, WeaponDef_isManualFire(weapon)) {
                continue;
            }
            mask |= call!(self, WeaponDef_getInterceptedByShieldType(weapon)).max(0) as u32;
        }
        mask
    }

    /// Our side's sight, radar and air-sight coverage this frame, as the engine computes them over the terrain: one
    /// grid each at the mod's mip level (BAR: sight 64 elmos a square, radar 32, air 128). None when a read fails.
    pub fn sensors(&self) -> Option<Sensors> {
        let (squares_x, squares_z) = (call!(self, Map_getWidth()), call!(self, Map_getHeight()));
        let (los_mip, radar_mip, air_mip) = (call!(self, Mod_getLosMipLevel()), call!(self, Mod_getRadarMipLevel()), call!(self, Mod_getAirMipLevel()));
        let grid = |mip: c_int, read: &dyn Fn(*mut c_int, c_int) -> c_int| -> Option<Grid> {
            let mip = mip.clamp(0, 30);
            let (cols, rows) = ((squares_x >> mip).max(1) as u32, (squares_z >> mip).max(1) as u32);
            let mut values = vec![0 as c_int; (cols * rows) as usize];
            let got = read(values.as_mut_ptr(), values.len() as c_int);
            if got != values.len() as c_int {
                return None;
            }
            Some(Grid::pack(cols, rows, SQUARE_SIZE * (1 << mip) as f32, values.iter().map(|v| *v > 0)))
        };
        let los = grid(los_mip, &|buf, n| call!(self, Map_getLosMap(buf, n)))?;
        let radar = grid(radar_mip, &|buf, n| call!(self, Map_getRadarMap(buf, n)))?;
        let air = grid(air_mip, &|buf, n| call!(self, Map_getAirLosMap(buf, n)))?;
        Some(Sensors { los, radar, air })
    }

    /// Seconds a round of the type's stockpiled weapon takes and the energy it costs; (0, 0) for a type that stockpiles
    /// nothing. The engine keeps `stockpileTime` in frames (`WeaponDef.cpp`: the file's seconds times `GAME_SPEED`).
    fn stockpile(&self, id: c_int) -> (f32, f32) {
        let weapon = call!(self, UnitDef_getStockpileDef(id));
        if weapon < 0 {
            return (0.0, 0.0);
        }
        (call!(self, WeaponDef_getStockpileTime(weapon)) / 30.0, call!(self, WeaponDef_getCost(weapon, self.energy)))
    }

    /// The coverage of the type's interceptor weapon (an anti-nuke takes a missile aimed within it); 0 without one.
    fn coverage(&self, id: c_int) -> f32 {
        let mounts = call!(self, UnitDef_getWeaponMounts(id)).max(0);
        let mut coverage = 0.0f32;
        for mount in 0..mounts {
            let weapon = call!(self, UnitDef_WeaponMount_getWeaponDef(id, mount));
            if weapon >= 0 && call!(self, WeaponDef_getInterceptor(weapon)) != 0 {
                coverage = coverage.max(call!(self, WeaponDef_getCoverageRange(weapon)));
            }
        }
        coverage
    }

    /// Whether the type stockpiles rounds, remembered per type.
    fn stockpiles(&mut self, def: c_int) -> bool {
        if let Some(&yes) = self.stockpilers.get(&def) {
            return yes;
        }
        let yes = call!(self, UnitDef_getStockpileDef(def)) >= 0;
        self.stockpilers.insert(def, yes);
        yes
    }

    /// Whether any weapon of the type is a water weapon (a torpedo, a depth charge: what reaches a submerged
    /// target), and whether every weapon is (it hits nothing out of the water).
    fn water_weapons(&mut self, id: c_int) -> (bool, bool) {
        let mounts = call!(self, UnitDef_getWeaponMounts(id)).max(0);
        let (mut any, mut all) = (false, mounts > 0);
        for mount in 0..mounts {
            let weapon = call!(self, UnitDef_WeaponMount_getWeaponDef(id, mount));
            if weapon < 0 {
                continue;
            }
            let water = call!(self, WeaponDef_isWaterWeapon(weapon)) || call!(self, WeaponDef_isSubMissile(weapon));
            any |= water;
            all &= water;
        }
        (any, all)
    }

    /// The range and reload of a type's longest ordinary weapon, and the largest area of effect (a radius) among its
    /// ordinary weapons: manual-fire weapons (the D-gun) left out.
    fn longest_weapon(&mut self, id: c_int) -> (f32, f32, f32) {
        let mounts = call!(self, UnitDef_getWeaponMounts(id)).max(0);
        let mut best = (0.0f32, 0.0f32);
        let mut area = 0.0f32;
        for mount in 0..mounts {
            let weapon = call!(self, UnitDef_WeaponMount_getWeaponDef(id, mount));
            if weapon < 0 || call!(self, WeaponDef_isManualFire(weapon)) {
                continue;
            }
            area = area.max(call!(self, WeaponDef_getAreaOfEffect(weapon)));
            let range = call!(self, WeaponDef_getRange(weapon));
            if range > best.0 {
                best = (range, call!(self, WeaponDef_getReload(weapon)));
            }
        }
        (best.0, best.1, area)
    }

    /// An explosion weapon's reach and its largest damage figure; None for no weapon or one that hurts nothing.
    fn blast(&self, weapon: c_int) -> Option<Blast> {
        if weapon < 0 {
            return None;
        }
        let count = call!(self, WeaponDef_Damage_getTypes(weapon, std::ptr::null_mut(), 0)).max(0);
        let mut damages = vec![0f32; count as usize];
        call!(self, WeaponDef_Damage_getTypes(weapon, damages.as_mut_ptr(), count));
        let damage = damages.into_iter().fold(0f32, f32::max);
        let radius = call!(self, WeaponDef_getAreaOfEffect(weapon));
        (damage > 0.0 && radius > 0.0).then_some(Blast { radius, damage })
    }

    fn converter(&self, id: c_int) -> Option<Converter> {
        Some(Converter { capacity: self.custom_number(id, "energyconv_capacity")?, efficiency: self.custom_number(id, "energyconv_efficiency")? })
    }

    /// One of the type's custom parameters (the unit file's `customparams`), as a number.
    fn custom_number(&self, id: c_int, name: &str) -> Option<f32> {
        let count = call!(self, UnitDef_getCustomParams(id, std::ptr::null_mut(), std::ptr::null_mut())).max(0) as usize;
        let (mut keys, mut values) = (vec![std::ptr::null(); count], vec![std::ptr::null(); count]);
        call!(self, UnitDef_getCustomParams(id, keys.as_mut_ptr(), values.as_mut_ptr()));
        let at = keys.iter().position(|k| self.string(*k) == name)?;
        self.string(values[at]).parse::<f32>().ok()
    }

    pub fn snapshot(&mut self) -> Snapshot {
        let max = self.id_buf.len() as c_int;
        let own_count = call!(self, getTeamUnits(self.id_buf.as_mut_ptr(), max)).max(0) as usize;
        let ids: Vec<c_int> = self.id_buf[..own_count].to_vec();
        let own_units = ids
            .into_iter()
            .map(|id| {
                let def = call!(self, Unit_getDef(id));
                let (stockpile, stockpile_queued) = if self.stockpiles(def) {
                    (call!(self, Unit_getStockpile(id)), call!(self, Unit_getStockpileQueued(id)))
                } else {
                    (0, 0)
                };
                OwnUnit {
                    id: UnitId(id),
                    def: UnitDefId(def),
                    pos: self.unit_pos(id),
                    vel: self.unit_vel(id),
                    health: call!(self, Unit_getHealth(id)),
                    max_health: call!(self, Unit_getMaxHealth(id)),
                    being_built: call!(self, Unit_isBeingBuilt(id)),
                    idle: call!(self, Unit_getCurrentCommands(id)) == 0,
                    reload_frame: self.reload_frame(id),
                    facing: call!(self, Unit_getBuildingFacing(id)),
                    stockpile,
                    stockpile_queued,
                    // The gadget's unit rules param: unset (a unit without the flag, or never toggled) reads as high.
                    priority_low: call!(self, Unit_getRulesParamFloat(id, c"builderPriority".as_ptr(), 1.0)) < 0.5,
                    build_progress: call!(self, Unit_getBuildProgress(id)).clamp(0.0, 1.0),
                    stunned: self.stunned(id),
                }
            })
            .collect();

        let my_team = call!(self, SkirmishAI_getTeamId());
        let friendly_count = call!(self, getFriendlyUnits(self.id_buf.as_mut_ptr(), max)).max(0) as usize;
        let allies = self.id_buf[..friendly_count]
            .iter()
            .filter_map(|&id| {
                let team = call!(self, Unit_getTeam(id));
                (team != my_team).then(|| AllyUnit {
                    id: UnitId(id),
                    def: UnitDefId(call!(self, Unit_getDef(id))),
                    pos: self.unit_pos(id),
                    team,
                    being_built: call!(self, Unit_isBeingBuilt(id)),
                })
            })
            .collect();

        let enemy_count = call!(self, getEnemyUnitsInRadarAndLos(self.id_buf.as_mut_ptr(), max)).max(0) as usize;
        let enemies = self.id_buf[..enemy_count]
            .iter()
            .map(|&id| {
                let def = call!(self, Unit_getDef(id));
                EnemyUnit {
                    id: UnitId(id),
                    def: (def >= 0).then_some(UnitDefId(def)),
                    pos: self.unit_pos(id),
                    vel: self.unit_vel(id),
                    health: call!(self, Unit_getHealth(id)),
                    team: Some(call!(self, Unit_getTeam(id))).filter(|team| *team >= 0),
                    being_built: def >= 0 && call!(self, Unit_isBeingBuilt(id)),
                    stunned: if def >= 0 { self.stunned(id) } else { 0.0 },
                }
            })
            .collect();

        Snapshot { metal: self.resource(self.metal), energy: self.resource(self.energy), wind: call!(self, Map_getCurWind()), own_units, allies, enemies, wrecks: None, sensors: None }
    }

    /// The reclaimable features in sight that hold metal worth a walk, the richest first.
    pub fn wrecks(&mut self) -> Vec<Wreck> {
        let max = self.id_buf.len() as c_int;
        let count = call!(self, getFeatures(self.id_buf.as_mut_ptr(), max)).max(0) as usize;
        let ids: Vec<c_int> = self.id_buf[..count].to_vec();
        let mut wrecks = Vec::new();
        for id in ids {
            let def = call!(self, Feature_getDef(id));
            if def < 0 {
                continue;
            }
            let whole = match self.feature_metal.get(&def) {
                Some(known) => *known,
                None => {
                    let metal = call!(self, FeatureDef_isReclaimable(def)).then(|| call!(self, FeatureDef_getContainedResource(def, self.metal)));
                    self.feature_metal.insert(def, metal);
                    metal
                }
            };
            let Some(metal) = whole.map(|whole| whole * call!(self, Feature_getReclaimLeft(id))).filter(|metal| *metal >= WRECK_MIN_METAL) else { continue };
            let mut pos = [0f32; 3];
            call!(self, Feature_getPosition(id, pos.as_mut_ptr()));
            let raises = call!(self, Feature_getResurrectDef(id));
            wrecks.push(Wreck { id: FeatureId(id), pos: Vec3 { x: pos[0], y: pos[1], z: pos[2] }, metal, resurrects_into: (raises >= 0).then_some(UnitDefId(raises)) });
        }
        wrecks.sort_by(|a, b| b.metal.total_cmp(&a.metal));
        wrecks.truncate(MAX_WRECKS);
        wrecks
    }

    fn resource(&self, id: c_int) -> Resource {
        Resource {
            current: call!(self, Economy_getCurrent(id)),
            income: call!(self, Economy_getIncome(id)),
            usage: call!(self, Economy_getUsage(id)),
            storage: call!(self, Economy_getStorage(id)),
        }
    }

    fn unit_pos(&self, unit: c_int) -> Vec3 {
        let mut pos = [0f32; 3];
        call!(self, Unit_getPos(unit, pos.as_mut_ptr()));
        Vec3 { x: pos[0], y: pos[1], z: pos[2] }
    }

    fn unit_vel(&self, unit: c_int) -> Vec3 {
        let mut vel = [0f32; 3];
        call!(self, Unit_getVel(unit, vel.as_mut_ptr()));
        Vec3 { x: vel[0], y: vel[1], z: vel[2] }
    }

    /// The frame the unit's first weapon can next fire (`Weapon::reloadStatus`); 0 for a unit without weapons.
    fn reload_frame(&self, unit: c_int) -> i32 {
        if call!(self, Unit_getWeapons(unit)) <= 0 {
            return 0;
        }
        let weapon = call!(self, Unit_getWeapon(unit, 0));
        if weapon < 0 {
            return 0;
        }
        call!(self, Unit_Weapon_getReloadFrame(unit, weapon))
    }

    /// A legal position for `def` near `site`, if the map has one, and the index of the placement it was found
    /// for (None when the site carries no placements).
    ///
    /// An extractor goes exactly on its spot or nowhere: the engine's site search refuses a spot holding a wreck,
    /// though building there is allowed (the builder reclaims the wreck first). Anything else keeps off the
    /// metal spots, or the base's own generators bury the nearest extractor sites.
    pub fn find_build_site(&self, def: UnitDefId, site: &BuildSite) -> Option<(Vec3, Option<usize>)> {
        if call!(self, UnitDef_getExtractsResource(def.0, self.metal)) > 0.0 {
            let near = self.snap_to_build_grid(def, site.near);
            let mut at = [near.x, near.y, near.z];
            return call!(self, Map_isPossibleToBuildAt(def.0, at.as_mut_ptr(), sys::UNIT_COMMAND_BUILD_NO_FACING))
                .then_some((near, None));
        }
        if site.placements.is_empty() {
            return self.site_facing(def, site, sys::UNIT_COMMAND_BUILD_NO_FACING, &[]).map(|pos| (pos, None));
        }
        // The placements in the bot's order: the first facing with a site wins.
        site.placements.iter().enumerate().find_map(|(i, p)| self.site_facing(def, site, p.facing, &p.keep_out).map(|pos| (pos, Some(i))))
    }

    /// Where the engine would stand a building of `def` ordered at `pos` (`CGameHelper::Pos2BuildPos`: a 16-elmo grid,
    /// offset by 8 on an axis whose footprint size in squares is 2 mod 4). The AI interface reports a standing unit
    /// at its model's centre, which for Cortex's extractor is 0.01 off the grid; a tier-2 extractor ordered over it at
    /// that position is denied by the game's "Deny Invalid Mex Orders" rule, which takes the order's position against
    /// the unit's to the last digit when the unit counts as ours (terr2-nullai-comet-1: every tier-2 extractor order
    /// of seat 0 as Cortex dropped by the engine, Armada's at .00 begun; K-mech-tier-2-extractor-stands-on-ours).
    fn snap_to_build_grid(&self, def: UnitDefId, pos: Vec3) -> Vec3 {
        let snap = |v: f32, size: c_int| if size & 2 != 0 { (v / 16.0).floor() * 16.0 + 8.0 } else { ((v + 8.0) / 16.0).floor() * 16.0 };
        let (xs, zs) = (call!(self, UnitDef_getXSize(def.0)), call!(self, UnitDef_getZSize(def.0)));
        Vec3 { x: snap(pos.x, xs), y: pos.y, z: snap(pos.z, zs) }
    }

    /// The closest site to `site.near` for one facing, off the metal spots and out of `keep_out` (our factories'
    /// exit lanes, or the yard fills with our own solar collectors: hands-2-bulldogs, five Bulls stood in the plant's
    /// exit for five to seven minutes behind a solar 165 elmos off its front).
    fn site_facing(&self, def: UnitDefId, site: &BuildSite, facing: c_int, keep_out: &[Lane]) -> Option<Vec3> {
        let on_a_spot = |pos: Vec3| self.metal_spots.iter().any(|s| s.dist2d(pos) < SPOT_KEEPOUT) || keep_out.iter().any(|lane| lane.contains(pos));
        // The search returns the closest site to its centre, so walk the centre outwards until the answer is clear
        // (an exact site: the first ring only).
        let rings = [0.0, 1.0, 2.0, 3.0].map(|r| r * 2.0 * SPOT_KEEPOUT);
        for (ring, radius) in rings.into_iter().enumerate().take(if site.exact { 1 } else { 4 }) {
            let directions = if ring == 0 { 1 } else { 8 };
            for step in 0..directions {
                let angle = step as f32 * std::f32::consts::TAU / directions as f32;
                let centre = Vec3 { x: site.near.x + radius * angle.cos(), z: site.near.z + radius * angle.sin(), ..site.near };
                if let Some(found) = self.closest_build_site(def, centre, site, facing) && !on_a_spot(found) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn closest_build_site(&self, def: UnitDefId, centre: Vec3, site: &BuildSite, facing: c_int) -> Option<Vec3> {
        let mut near = [centre.x, centre.y, centre.z];
        let mut found = [0f32; 3];
        call!(self, Map_findClosestBuildSite(def.0, near.as_mut_ptr(), site.search_radius, site.min_dist, facing, found.as_mut_ptr()));
        // The engine reports failure as x == -1.
        (found[0] >= 0.0).then_some(Vec3 { x: found[0], y: found[1], z: found[2] })
    }

    /// Everything the enemy owns, by type, with its economy where the engine tells: a study aid, never the brain's
    /// input. Cheat access is switched on for the length of this call only, so the bot's own view stays fair.
    pub fn enemy_census(&mut self) -> String {
        call!(self, Cheats_setEnabled(true));
        let max = self.id_buf.len() as c_int;
        let count = call!(self, getEnemyUnits(self.id_buf.as_mut_ptr(), max)).max(0) as usize;
        let census = self.census(count);
        call!(self, Cheats_setEnabled(false));
        census
    }

    /// Soldiers' metal value and finished extractors for both sides (our whole ally team, every enemy), `(ours, theirs)`: what the arena's referee judges
    /// a settled game by. Like the census, read with cheat access for the length of the call and never sent to the bot.
    pub fn balance(&mut self) -> ((f32, u32), (f32, u32)) {
        let max = self.id_buf.len() as c_int;
        let own = call!(self, getFriendlyUnits(self.id_buf.as_mut_ptr(), max)).max(0) as usize;
        let ours = self.strength(own);
        call!(self, Cheats_setEnabled(true));
        let enemy = call!(self, getEnemyUnits(self.id_buf.as_mut_ptr(), max)).max(0) as usize;
        let theirs = self.strength(enemy);
        call!(self, Cheats_setEnabled(false));
        (ours, theirs)
    }

    /// (metal value of finished soldiers, finished extractors) among the first `count` ids in the id buffer.
    fn strength(&self, count: usize) -> (f32, u32) {
        let (mut army, mut extractors) = (0.0, 0);
        for &id in &self.id_buf[..count] {
            let def = call!(self, Unit_getDef(id));
            if def < 0 || call!(self, Unit_isBeingBuilt(id)) {
                continue;
            }
            if call!(self, UnitDef_getExtractsResource(def, self.metal)) > 0.0 {
                extractors += 1;
            }
            let soldier = call!(self, UnitDef_getSpeed(def)) > 0.0
                && call!(self, UnitDef_getWeaponMounts(def)) > 0
                && call!(self, UnitDef_getBuildSpeed(def)) == 0.0;
            if soldier {
                army += call!(self, UnitDef_getCost(def, self.metal));
            }
        }
        (army, extractors)
    }

    /// Every enemy unit as `[id, "name", x, z, health percent, being built]`: ground truth for post-game analysis,
    /// never the brain's input. Cheat access is on for the length of this call only.
    pub fn enemy_truth(&mut self) -> String {
        call!(self, Cheats_setEnabled(true));
        let max = self.id_buf.len() as c_int;
        let count = call!(self, getEnemyUnits(self.id_buf.as_mut_ptr(), max)).max(0) as usize;
        let mut rows = Vec::with_capacity(count);
        for &id in &self.id_buf[..count] {
            let def = call!(self, Unit_getDef(id));
            if def < 0 {
                continue;
            }
            let name = self.string(call!(self, UnitDef_getName(def)));
            let pos = self.unit_pos(id);
            let (health, max_health) = (call!(self, Unit_getHealth(id)), call!(self, Unit_getMaxHealth(id)));
            let percent = if max_health > 0.0 { (health / max_health * 100.0).round() as i32 } else { 0 };
            rows.push(format!("[{id},\"{name}\",{:.0},{:.0},{percent},{}]", pos.x, pos.z, call!(self, Unit_isBeingBuilt(id)) as i32));
        }
        call!(self, Cheats_setEnabled(false));
        format!("[{}]", rows.join(","))
    }

    /// What our allies' overflow put into our stock over the engine's last second, metal and energy.
    pub fn received(&self) -> (f32, f32) {
        (call!(self, Economy_getReceived(self.metal)), call!(self, Economy_getReceived(self.energy)))
    }

    /// Metal income a second of our ally team's teams and of every other team's, summed (cheat access for theirs,
    /// as the truth's units): the comparison of the two sides' economies, which the bot never sees.
    pub fn metal_incomes(&mut self) -> (f32, f32) {
        let ally_team = call!(self, Game_getMyAllyTeam());
        call!(self, Cheats_setEnabled(true));
        let (mut ours, mut theirs) = (0.0, 0.0);
        for team in 0..call!(self, Game_getTeams()) {
            let income = call!(self, Game_getTeamResourceIncome(team, self.metal)).max(0.0);
            if call!(self, Game_getTeamAllyTeam(team)) == ally_team {
                ours += income;
            } else {
                theirs += income;
            }
        }
        call!(self, Cheats_setEnabled(false));
        (ours, theirs)
    }

    /// Our own units in the census format, for setting beside the enemy's.
    pub fn own_census(&mut self) -> String {
        let max = self.id_buf.len() as c_int;
        let count = call!(self, getTeamUnits(self.id_buf.as_mut_ptr(), max)).max(0) as usize;
        self.census(count)
    }

    /// `name x count @ mean position` for the first `count` ids in the id buffer.
    fn census(&self, count: usize) -> String {
        let mut by_name: std::collections::BTreeMap<String, (u32, f32, f32)> = Default::default();
        for &id in &self.id_buf[..count] {
            let def = call!(self, Unit_getDef(id));
            if def < 0 {
                continue;
            }
            let name = self.string(call!(self, UnitDef_getName(def)));
            let pos = self.unit_pos(id);
            let entry = by_name.entry(name).or_default();
            entry.0 += 1;
            entry.1 += pos.x;
            entry.2 += pos.z;
        }
        by_name
            .into_iter()
            .map(|(name, (n, x, z))| format!("{name}x{n}@{:.0},{:.0}", x / n as f32, z / n as f32))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// What stands within `radius` of `pos`, for diagnosing a build site the engine refused.
    pub fn describe_site(&mut self, def: UnitDefId, pos: Vec3, radius: f32) -> String {
        let mut at = [pos.x, pos.y, pos.z];
        let max = self.id_buf.len() as c_int;
        let possible = call!(self, Map_isPossibleToBuildAt(def.0, at.as_mut_ptr(), sys::UNIT_COMMAND_BUILD_NO_FACING));
        let mut out = format!("possible_at_exact={possible}");
        let n = call!(self, getFriendlyUnitsIn(at.as_mut_ptr(), radius, false, self.id_buf.as_mut_ptr(), max)).max(0) as usize;
        for &id in &self.id_buf[..n] {
            let def = call!(self, Unit_getDef(id));
            let name = self.string(call!(self, UnitDef_getName(def)));
            out += &format!(" friendly:{name}");
        }
        let n = call!(self, getEnemyUnitsIn(at.as_mut_ptr(), radius, false, self.id_buf.as_mut_ptr(), max)).max(0) as usize;
        for &id in &self.id_buf[..n] {
            let def = call!(self, Unit_getDef(id));
            let name = if def >= 0 { self.string(call!(self, UnitDef_getName(def))) } else { "?".into() };
            out += &format!(" enemy:{name}");
        }
        let n = call!(self, getFeaturesIn(at.as_mut_ptr(), radius, false, self.id_buf.as_mut_ptr(), max)).max(0) as usize;
        for &id in &self.id_buf[..n] {
            let def = call!(self, Feature_getDef(id));
            let name = self.string(call!(self, FeatureDef_getName(def)));
            out += &format!(" feature:{name}");
        }
        out
    }

    /// Issues a command; `Err` carries the engine's non-zero result code.
    /// A command position with the map's height at it. The bot's points carry whatever height they were made
    /// with (0 for a route cell or a ring round a builder, a neighbouring spot's for a spot from the game's list),
    /// and the engine keeps a command's parameters as given: it corrects the height when it executes the order
    /// (`Pos2BuildPos`, the path), but the replay draws the queued ghost or destination at the height sent, in
    /// the sky or under the ground (the user, 2026-09-27: "to-be-started mex ghosts and unit movement destinations
    /// appear as either high in the sky or under the terrain").
    /// The ground's height at a place now, craters and all ([`Command::Probe`]).
    pub fn elevation(&self, x: f32, z: f32) -> f32 {
        call!(self, Map_getElevationAt(x, z))
    }

    fn on_the_ground(&self, at: Vec3) -> [f32; 3] {
        [at.x, call!(self, Map_getElevationAt(at.x, at.z)), at.z]
    }

    /// [`Command::Build`] must already have a concrete position in `site.near`.
    pub fn issue(&self, command: &Command) -> Result<(), i32> {
        let options = |queue: bool| if queue { sys::UNIT_COMMAND_OPTION_SHIFT_KEY as i16 } else { 0 };
        const NO_GROUP: c_int = -1;
        const NO_TIMEOUT: c_int = c_int::MAX;
        const META_KEY: i16 = 1 << 2;
/// BAR's `CMD_PRIORITY` (`modules/customcommands.lua`), the builder-priority gadget's command id.
const BUILDER_PRIORITY_COMMAND: c_int = 34571;
        match *command {
            Command::Build { unit, def, ref site, queue } => {
                let mut pos = site.as_ref().map_or([0.0; 3], |s| self.on_the_ground(s.near));
                self.handle(sys::COMMAND_UNIT_BUILD, &mut sys::SBuildUnitCommand {
                    unitId: unit.0,
                    groupId: NO_GROUP,
                    // For factories the shift bit means "build five" (FactoryCAI.cpp), not "append".
                    options: options(queue && site.is_some()),
                    timeOut: NO_TIMEOUT,
                    toBuildUnitDefId: def.0,
                    buildPos_posF3: pos.as_mut_ptr(),
                    // The facing the site was found for (`find_build_site` keeps that placement alone).
                    facing: site.as_ref().and_then(|s| s.facing()).unwrap_or(sys::UNIT_COMMAND_BUILD_NO_FACING),
                })
            }
            Command::Move { unit, to, queue } => {
                let mut pos = self.on_the_ground(to);
                self.handle(sys::COMMAND_UNIT_MOVE, &mut sys::SMoveUnitCommand {
                    unitId: unit.0,
                    groupId: NO_GROUP,
                    options: options(queue),
                    timeOut: NO_TIMEOUT,
                    toPos_posF3: pos.as_mut_ptr(),
                })
            }
            Command::Fight { unit, to, queue } => {
                let mut pos = self.on_the_ground(to);
                self.handle(sys::COMMAND_UNIT_FIGHT, &mut sys::SFightUnitCommand {
                    unitId: unit.0,
                    groupId: NO_GROUP,
                    options: options(queue),
                    timeOut: NO_TIMEOUT,
                    toPos_posF3: pos.as_mut_ptr(),
                })
            }
            Command::FightReclaimingEnemies { unit, to, queue } => {
                let mut pos = self.on_the_ground(to);
                self.handle(sys::COMMAND_UNIT_FIGHT, &mut sys::SFightUnitCommand {
                    unitId: unit.0,
                    groupId: NO_GROUP,
                    // The engine's META bit (1 << 2, `Command.h`) has no name in the AI interface; the options pass
                    // through to the command whole (`AISCommands.cpp`).
                    options: options(queue) | sys::UNIT_COMMAND_OPTION_CONTROL_KEY as i16 | META_KEY,
                    timeOut: NO_TIMEOUT,
                    toPos_posF3: pos.as_mut_ptr(),
                })
            }
            Command::Attack { unit, target, queue } => self.handle(sys::COMMAND_UNIT_ATTACK, &mut sys::SAttackUnitCommand {
                unitId: unit.0,
                groupId: NO_GROUP,
                options: options(queue),
                timeOut: NO_TIMEOUT,
                toAttackUnitId: target.0,
            }),
            Command::Say { ref text } => {
                // The engine takes an AI's text only as a slash command (`CGame::ProcessCommandText`); `/say` is chat
                // from the AI's host player. `{name}` becomes the name the game gave this AI (`ai_namer.lua` puts it
                // in the rules parameter `ainame_<team>`), so the line says which of the random names is ours.
                let team = call!(self, SkirmishAI_getTeamId());
                let key = CString::new(format!("ainame_{team}")).unwrap_or_default();
                let none = CString::default();
                let name = self.string(call!(self, Game_getRulesParamString(key.as_ptr(), none.as_ptr())));
                let line = CString::new(format!("/say {}", text.replace("{name}", if name.is_empty() { "an AI" } else { &name }))).unwrap_or_default();
                self.handle(sys::COMMAND_SEND_TEXT_MESSAGE, &mut sys::SSendTextMessageCommand { text: line.as_ptr(), zone: 0 })
            }
            Command::Stop { unit } => self.handle(sys::COMMAND_UNIT_STOP, &mut sys::SStopUnitCommand {
                unitId: unit.0,
                groupId: NO_GROUP,
                options: 0,
                timeOut: NO_TIMEOUT,
            }),
            Command::MoveState { unit, state } => self.handle(sys::COMMAND_UNIT_SET_MOVE_STATE, &mut sys::SSetMoveStateUnitCommand {
                unitId: unit.0,
                groupId: NO_GROUP,
                options: 0,
                timeOut: NO_TIMEOUT,
                moveState: state,
            }),
            Command::FireState { unit, state } => self.handle(sys::COMMAND_UNIT_SET_FIRE_STATE, &mut sys::SSetFireStateUnitCommand {
                unitId: unit.0,
                groupId: NO_GROUP,
                options: 0,
                timeOut: NO_TIMEOUT,
                fireState: state,
            }),
            Command::SetRepeat { unit, repeat } => {
                self.handle(sys::COMMAND_UNIT_SET_REPEAT, &mut sys::SSetRepeatUnitCommand {
                    unitId: unit.0,
                    groupId: NO_GROUP,
                    options: 0,
                    timeOut: NO_TIMEOUT,
                    repeat,
                })
            }
           Command::Guard { unit, target } => self.handle(sys::COMMAND_UNIT_GUARD, &mut sys::SGuardUnitCommand {
                unitId: unit.0,
                groupId: NO_GROUP,
                options: 0,
                timeOut: NO_TIMEOUT,
                toGuardUnitId: target.0,
            }),
            Command::ReclaimFeature { unit, feature, queue } => self.handle(sys::COMMAND_UNIT_RECLAIM_FEATURE, &mut sys::SReclaimFeatureUnitCommand {
                unitId: unit.0,
                groupId: NO_GROUP,
                options: options(queue),
                timeOut: NO_TIMEOUT,
                toReclaimFeatureId: feature.0,
            }),
            Command::ReclaimUnit { unit, target, queue } => self.handle(sys::COMMAND_UNIT_RECLAIM_UNIT, &mut sys::SReclaimUnitUnitCommand {
                unitId: unit.0,
                groupId: NO_GROUP,
                options: options(queue),
                timeOut: NO_TIMEOUT,
                toReclaimUnitId: target.0,
            }),
            Command::Resurrect { unit, feature, queue } => self.handle(sys::COMMAND_UNIT_RESURRECT, &mut sys::SResurrectUnitCommand {
                unitId: unit.0,
                groupId: NO_GROUP,
                options: options(queue),
                timeOut: NO_TIMEOUT,
                // The engine's own command wants a feature as its id plus the unit limit. The AI interface adds that for
                // reclaiming a feature and forgets it for resurrecting one (AISCommands.cpp, COMMAND_UNIT_RESURRECT against
                // COMMAND_UNIT_RECLAIM_FEATURE; BuilderCAI::ExecuteResurrect subtracts it): a bare id is taken for a unit
                // and nothing is raised (rec-1: 57 orders, no unit).
                toResurrectFeatureId: call!(self, Unit_getMax()) + feature.0,
            }),
            Command::Repair { unit, target, queue } => self.handle(sys::COMMAND_UNIT_REPAIR, &mut sys::SRepairUnitCommand {
                unitId: unit.0,
                groupId: NO_GROUP,
                options: options(queue),
                timeOut: NO_TIMEOUT,
                toRepairUnitId: target.0,
            }),
            // Creating a unit makes the engine call back into `handleEvent` before it returns, so it cannot be
            // issued from inside an export that holds the instance table; see `Spawner`.
            Command::SendResources { metal, energy, to_team } => {
                let mut result = Ok(());
                for (resource, amount) in [(self.metal, metal), (self.energy, energy)] {
                    if amount > 0.0 && resource >= 0 {
                        let mut c = sys::SSendResourcesCommand { resourceId: resource, amount, receivingTeamId: to_team, ret_isExecuted: false };
                        let r = self.handle(sys::COMMAND_SEND_RESOURCES, &mut c);
                        if r.is_err() || !c.ret_isExecuted {
                            result = Err(r.err().unwrap_or(-2));
                        }
                    }
                }
                result
            }
            Command::GiveResource { metal, energy } => {
                // Cheat access for this one command, as `Spawner::give` has it; the engine ignores it outside a
                // locally hosted single-player game.
                let mut result = Ok(());
                call!(self, Cheats_setEnabled(true));
                for (resource, amount) in [(self.metal, metal), (self.energy, energy)] {
                    if amount > 0.0 && resource >= 0 {
                        let mut c = sys::SGiveMeResourceCheatCommand { resourceId: resource, amount };
                        if let Err(code) = self.handle(sys::COMMAND_CHEATS_GIVE_ME_RESOURCE, &mut c) {
                            result = Err(code);
                        }
                    }
                }
                call!(self, Cheats_setEnabled(false));
                result
            }
            Command::SendUnits { ref units, to_team } => {
                let mut ids: Vec<c_int> = units.iter().map(|u| u.0).collect();
                let mut c = sys::SSendUnitsCommand { unitIds: ids.as_mut_ptr(), unitIds_size: ids.len() as c_int, receivingTeamId: to_team, ret_sentUnits: 0 };
                let r = self.handle(sys::COMMAND_SEND_UNITS, &mut c);
                if r.is_ok() && c.ret_sentUnits as usize != ids.len() { Err(-3) } else { r }
            }
            Command::DGun { unit, target } => self.handle(sys::COMMAND_UNIT_D_GUN, &mut sys::SDGunUnitCommand {
                unitId: unit.0,
                groupId: NO_GROUP,
                options: 0,
                timeOut: NO_TIMEOUT,
                toAttackUnitId: target.0,
            }),
            // The engine's manual-fire command is allowed only to a type with `canManualFire` (the commander, for its
            // D-gun; `CCommandAI::AllowedCommand`), and a type without it fires its command-fire weapon on a plain
            // attack at the ground (`CWeapon::TestTarget` skips the manual-fire check for it): BAR's silo is the
            // latter, so the missile goes by an attack at a place with no radius (K-rules-the-silo-fires-on-an-attack-at-the-ground).
            Command::FireAt { unit, at } => {
                let mut pos = self.on_the_ground(at);
                let def = call!(self, Unit_getDef(unit.0));
                if call!(self, UnitDef_canManualFire(def)) {
                    self.handle(sys::COMMAND_UNIT_D_GUN_POS, &mut sys::SDGunPosUnitCommand {
                        unitId: unit.0,
                        groupId: NO_GROUP,
                        options: 0,
                        timeOut: NO_TIMEOUT,
                        pos_posF3: pos.as_mut_ptr(),
                    })
                } else {
                    self.handle(sys::COMMAND_UNIT_ATTACK_AREA, &mut sys::SAttackAreaUnitCommand {
                        unitId: unit.0,
                        groupId: NO_GROUP,
                        options: 0,
                        timeOut: NO_TIMEOUT,
                        toAttackPos_posF3: pos.as_mut_ptr(),
                        radius: 0.0,
                    })
                }
            }
            // The game's stockpile gadget reads each order as one step of the kept count, up with no option and down
            // with the right-mouse one (shift and control would make it five or twenty); one order a step.
            Command::Stockpile { unit, change } => {
                let option = if change < 0 { sys::UNIT_COMMAND_OPTION_RIGHT_MOUSE_KEY as i16 } else { 0 };
                let mut result = Ok(());
                for _ in 0..change.unsigned_abs().min(100) {
                    let r = self.handle(sys::COMMAND_UNIT_STOCKPILE, &mut sys::SStockpileUnitCommand {
                        unitId: unit.0,
                        groupId: NO_GROUP,
                        options: option,
                        timeOut: NO_TIMEOUT,
                    });
                    if r.is_err() {
                        result = r;
                    }
                }
                result
            }
            // BAR's builder-priority gadget takes its command with one parameter (0 low, 1 high) and consumes it in
            // `AllowCommand`: the flag is set, nothing is queued (K-mech-builder-priority-is-a-builder-flag).
            Command::Priority { unit, low } => {
                let mut params = [if low { 0.0f32 } else { 1.0 }];
                self.handle(sys::COMMAND_UNIT_CUSTOM, &mut sys::SCustomUnitCommand {
                    unitId: unit.0,
                    groupId: NO_GROUP,
                    options: 0,
                    timeOut: NO_TIMEOUT,
                    cmdId: BUILDER_PRIORITY_COMMAND,
                    params: params.as_mut_ptr(),
                    params_size: 1,
                })
            }
            // The shim answers these itself (`lib.rs` `apply`): nothing reaches the engine.
            Command::GiveUnit { .. } | Command::Probe { .. } => Err(-1),
            Command::SelfDestruct { unit } => {
                self.handle(sys::COMMAND_UNIT_SELF_DESTROY, &mut sys::SSelfDestroyUnitCommand {
                    unitId: unit.0,
                    groupId: NO_GROUP,
                    options: 0,
                    timeOut: NO_TIMEOUT,
                })
            }
        }
    }

    fn handle<T>(&self, topic: sys::CommandTopic, data: &mut T) -> Result<(), i32> {
        const NO_COMMAND_ID: c_int = -1;
        let code = call!(self, Engine_handleCommand(
            sys::COMMAND_TO_ID_ENGINE, NO_COMMAND_ID, topic as c_int, std::ptr::from_mut(data).cast::<c_void>()
        ));
        if code == 0 { Ok(()) } else { Err(code) }
    }

    /// The weapon behind a damage event, by the engine's weapon definition id; `None` for -1.
    /// How a unit died, from the kill event's weapon id: a weapon def when 0 or more, else one of the engine's own
    /// causes negated (`CSolidObject::DamageType`: debris 1, self-destruct 10, reclaimed 12, construction decay 19).
    pub fn death(&self, id: std::ffi::c_int) -> bot_protocol::Death {
        use bot_protocol::Death;
        if id >= 0 {
            return self.weapon(id).map_or(Death::Unknown, Death::Weapon);
        }
        match -id {
            1 => Death::Debris,
            10 => Death::SelfDestruct,
            12 => Death::Reclaimed,
            19 => Death::Decayed,
            code => Death::Other(code),
        }
    }

    pub fn weapon(&self, def: std::ffi::c_int) -> Option<bot_protocol::Weapon> {
        if def < 0 {
            return None;
        }
        let name = self.string(call!(self, WeaponDef_getName(def)));
        let range = call!(self, WeaponDef_getRange(def));
        Some(bot_protocol::Weapon { name, range })
    }

    fn string(&self, ptr: *const std::ffi::c_char) -> String {
        if ptr.is_null() {
            return String::new();
        }
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned()
    }
}
