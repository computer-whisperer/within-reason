//! A failing site search saved whole, for replay off the game (`site_replay`): the terrain, the world as the bot
//! built it with every walk set, the request, and the game's answer with its time and trace beside them. The bot
//! writes one for each of a game's first failures (`crates/bot` `economy.rs`, `place_in_world`); the replayer reads
//! it back, runs `choose_traced` over the same world and prints the trace. Site-probe-1 (2026-11-05) left open why a
//! search that finds a site beside the commander on flat ground fails in the game with the same geometry: the
//! game's world (its ground, its walk sets, what stands) could not be rebuilt by hand.

use std::io;
use std::path::Path;

use bot_protocol::{Terrain, UnitDefId, Vec3};
use serde::{Deserialize, Serialize};

use crate::block::Lattice;
use crate::choose::{Frame, Request, Reservation, Standing, Worker, World};
use crate::street::{Builder, Plant};
use crate::{Ground, Passable};

#[derive(Serialize, Deserialize)]
pub struct Dump {
    /// Who asked for what, and when: "armsolar for commander at 4:12".
    pub label: String,
    /// The game's answer.
    pub why: String,
    /// How long the game's search took.
    pub ms: f32,
    /// The game's trace of the same search (`choose_traced`, run once more for the dump).
    pub trace: Vec<String>,
    pub terrain: Terrain,
    /// The map's extent in elmos.
    pub extent: (f32, f32),
    /// The world's `open`: the soldiers' walk set, one flag a terrain cell; None where everything was open.
    #[serde(with = "walk_bits")]
    pub open: Option<Passable>,
    pub spots: Vec<Vec3>,
    pub spot_keep_out: f32,
    pub standing: Vec<Standing>,
    pub plants: Vec<Plant>,
    pub builders: Vec<Builder>,
    pub workers: Vec<Worker>,
    pub frames: Vec<Frame>,
    pub farm_frame: Option<crate::Footprint>,
    pub reservations: Vec<Reservation>,
    pub lattice: Option<Lattice>,
    pub refused: Vec<(UnitDefId, Vec3)>,
    pub home: Vec3,
    pub toward: Vec3,
    pub request: Request,
}

impl Dump {
    /// The world and request as the bot had them. `open` is the walk set behind the world's `open` closure (the
    /// closure itself cannot be read back), None when the world was open everywhere.
    #[allow(clippy::too_many_arguments)]
    pub fn of(world: &World, req: &Request, terrain: &Terrain, open: Option<&[bool]>, label: String, why: String, ms: f32, trace: Vec<String>) -> Dump {
        Dump {
            label,
            why,
            ms,
            trace,
            terrain: terrain.clone(),
            extent: world.ground.extent(),
            open: open.map(Passable::from),
            spots: world.spots.to_vec(),
            spot_keep_out: world.spot_keep_out,
            standing: world.standing.clone(),
            plants: world.plants.clone(),
            builders: world.builders.clone(),
            workers: world.workers.clone(),
            frames: world.frames.clone(),
            farm_frame: world.farm_frame,
            reservations: world.reservations.clone(),
            lattice: world.lattice,
            refused: world.refused.clone(),
            home: world.home,
            toward: world.toward,
            request: req.clone(),
        }
    }

    pub fn write(&self, path: &Path) -> io::Result<()> {
        let bytes = postcard::to_allocvec(self).map_err(io::Error::other)?;
        std::fs::write(path, bytes)
    }

    pub fn read(path: &Path) -> io::Result<Dump> {
        let bytes = std::fs::read(path)?;
        postcard::from_bytes(&bytes).map_err(io::Error::other)
    }

    /// The world again, over this dump's terrain and walk sets.
    pub fn world(&self) -> World<'_> {
        let ground = Ground::new(&self.terrain, self.extent.0, self.extent.1);
        let open: Box<dyn Fn(Vec3) -> bool + '_> = match &self.open {
            Some(walk) => Box::new(move |p: Vec3| Ground::new(&self.terrain, self.extent.0, self.extent.1).walks(walk, p.x, p.z)),
            None => Box::new(|_| true),
        };
        World {
            ground,
            spots: &self.spots,
            spot_keep_out: self.spot_keep_out,
            standing: self.standing.clone(),
            plants: self.plants.clone(),
            builders: self.builders.clone(),
            workers: self.workers.clone(),
            frames: self.frames.clone(),
            farm_frame: self.farm_frame,
            reservations: self.reservations.clone(),
            lattice: self.lattice,
            refused: self.refused.clone(),
            home: self.home,
            toward: self.toward,
            open,
            walks: Default::default(),
            standing_near: Default::default(),
            spots_near: Default::default(),
            streets: Default::default(), blasts_near: Default::default(),
            farm: Default::default(), fixed: Default::default(), walk_sums: Default::default(),
        }
    }
}

/// A slow pass of the builder menus saved whole: the world of the pass and every site question it asked of the
/// layer, in the order asked (`crates/bot` `menu.rs`, `fill_builder_menus`; `layout-pass-<ai>-<n>.bin`). `site_bench`
/// asks them again over the same world, so a change to the search is judged on the game's own questions. The dump's
/// own `request` is the pass's first and its `ms` the pass's time.
#[derive(Serialize, Deserialize)]
pub struct PassDump {
    pub dump: Dump,
    pub requests: Vec<Request>,
}

impl PassDump {
    pub fn write(&self, path: &Path) -> io::Result<()> {
        let bytes = postcard::to_allocvec(self).map_err(io::Error::other)?;
        std::fs::write(path, bytes)
    }

    /// The pass as it was in the game: there every question for one kind of plant holds the same walk set (the route
    /// survey's, shared), and the world's memo of fixed parts knows a kind by it; read back, each question has its
    /// own copy, so equal sets are made one again.
    pub fn read(path: &Path) -> io::Result<PassDump> {
        let bytes = std::fs::read(path)?;
        let mut pass: PassDump = postcard::from_bytes(&bytes).map_err(io::Error::other)?;
        let mut sets: Vec<Passable> = Vec::new();
        let mut shared = |set: &mut Option<Passable>| {
            if let Some(own) = set {
                match sets.iter().find(|known| known[..] == own[..]) {
                    Some(known) => *own = known.clone(),
                    None => sets.push(own.clone()),
                }
            }
        };
        shared(&mut pass.dump.request.facts.product_passable);
        for request in pass.requests.iter_mut() {
            shared(&mut request.facts.product_passable);
            shared(&mut request.builder.product_passable);
        }
        Ok(pass)
    }
}

/// A walk set (`Option<Passable>`) as its length and one bit a cell: a 1,024-cell-square map's million flags in
/// 128 KB, where a byte a flag would be a megabyte for each of a dump's plants and builders.
pub mod walk_bits {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::Passable;

    pub fn serialize<S: Serializer>(walk: &Option<Passable>, s: S) -> Result<S::Ok, S::Error> {
        let packed: Option<(usize, Vec<u8>)> = walk.as_ref().map(|w| {
            let mut bytes = vec![0u8; w.len().div_ceil(8)];
            for (i, flag) in w.iter().enumerate() {
                if *flag {
                    bytes[i / 8] |= 1 << (i % 8);
                }
            }
            (w.len(), bytes)
        });
        packed.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Passable>, D::Error> {
        let packed: Option<(usize, Vec<u8>)> = Deserialize::deserialize(d)?;
        Ok(packed.map(|(n, bytes)| (0..n).map(|i| bytes.get(i / 8).is_some_and(|b| b & (1 << (i % 8)) != 0)).collect::<Vec<bool>>().into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::choose::Purpose;
    use crate::{Facts, Footprint, NO_DEPTH_FLOOR};

    /// A dump reads back as the world it was made of: the same search gives the same answer, walk sets included.
    #[test]
    fn a_dump_reads_back_and_the_search_answers_the_same() {
        let (w, h) = (64u32, 64u32);
        let terrain = Terrain { cell: 16.0, width: w, height: h, heights: vec![50; (w * h) as usize], slopes: vec![0; (w * h) as usize], metal: Vec::new() };
        // Vehicles stand west of x 512 only.
        let vehicles: Passable = (0..w * h).map(|k| ((k % w) as f32 * 16.0 + 8.0) < 512.0).collect::<Vec<bool>>().into();
        let soldiers: Vec<bool> = vec![true; (w * h) as usize];
        let facts = |def: i32, side: i32| Facts {
            def: UnitDefId(def),
            footprint: Footprint { x: side, z: side },
            build_distance: 128.0,
            build_speed: 0.0,
            build_time: 100.0,
            metal: 100.0,
            health: 100.0,
            blast: None,
            height_dif: 10.7,
            min_depth: NO_DEPTH_FLOOR,
            max_depth: 0.0,
            floater: false,
            extractor: false,
            factory: false,
            building: true,
            product_half: 0.0,
            product_passable: None,
        };
        let at = |x: f32, z: f32| Vec3 { x, y: 0.0, z };
        let world = World {
            ground: Ground::new(&terrain, 1024.0, 1024.0),
            spots: &[at(300.0, 300.0)],
            spot_keep_out: 100.0,
            standing: vec![Standing { rect: Footprint { x: 10, z: 10 }.rect(at(400.0, 400.0)), def: Some(UnitDefId(1)), kind: "armsolar".into(), ours: true, bank: true, metal: 150.0, health: 300.0, blast: None, name: "solar_1".into() }],
            plants: Vec::new(),
            builders: vec![Builder { at: at(480.0, 400.0), half: 8.0, passable: Some(vehicles.clone()) }],
            workers: Vec::new(),
            frames: Vec::new(),
            farm_frame: None,
            reservations: Vec::new(),
            lattice: None,
            refused: Vec::new(),
            home: at(400.0, 400.0),
            toward: at(900.0, 900.0),
            open: Box::new(|_| true),
            walks: Default::default(),
            standing_near: Default::default(),
            spots_near: Default::default(),
            streets: Default::default(), blasts_near: Default::default(),
            farm: Default::default(), fixed: Default::default(), walk_sums: Default::default(),
        };
        let req = Request { facts: facts(1, 10), purpose: Purpose::Generator { cap: Some(8) }, centre: at(480.0, 400.0), radius: 300.0, builder_at: at(480.0, 400.0), builder: facts(2, 2) };
        let answer = crate::choose(&world, &req);
        let dump = Dump::of(&world, &req, &terrain, Some(&soldiers), "a solar for the commander".into(), answer.clone().err().unwrap_or_default(), 1.0, Vec::new());
        let dir = std::env::temp_dir().join(format!("layout-dump-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("layout-fail-0-0.bin");
        dump.write(&path).unwrap();
        let back = Dump::read(&path).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(back.builders[0].passable.as_deref(), Some(&vehicles[..]));
        assert_eq!(back.open.as_deref().map(<[bool]>::len), Some(soldiers.len()));
        let again = crate::choose(&back.world(), &back.request);
        assert_eq!(again.as_ref().map(|s| (s.at, s.facing)).map_err(String::as_str), answer.as_ref().map(|s| (s.at, s.facing)).map_err(String::as_str));
    }
}
