//! Template for a heuristic bot: the whole bot inside the library the engine loads (`ai_host::InProcess`), with no
//! bot process, no socket and no model. Lobby name `WRTemplate`. Copy the directory to start a bot.
//!
//! Heuristic bots live under `bots/` and decide in code. The flagship's rule that no code decides a live game
//! without Jev or the player (`docs/code-decisions.md`) governs `crates/bot`, not this directory; see `bots/README.md`.
//!
//! What it plays is one rule, enough to show the path from snapshot to order: an idle builder builds an extractor
//! on the nearest metal spot it has not been sent to; on a metal map (no spots published; `MapInfo::metal_map`), on
//! the nearest free tile of metal (`layout::Coverage`, the claims of its own extractors over the metal map).

use std::collections::{BTreeMap, BTreeSet};

use bot_protocol::{Bot, BuildSite, Command, Commands, Hello, Tick, UnitDefId, UnitDefInfo, UnitId, Vec3};

/// Frames a builder is left alone after an order: the engine's idle flag clears a few frames after the order lands.
const ORDER_GRACE_FRAMES: i32 = 60;

struct Template {
    hello: Hello,
    /// Metal spots (indices into `hello.metal_spots`) a builder has been sent to.
    sent: BTreeSet<usize>,
    /// On a metal map, the metal under the ground with the claims of our extractors over it, rebuilt every tick;
    /// `None` on a spot map.
    coverage: Option<layout::Coverage>,
    /// On a metal map, the tiles a builder has been sent to.
    tiles_sent: Vec<Vec3>,
    /// The frame of each builder's last order.
    ordered: BTreeMap<UnitId, i32>,
}

impl Template {
    fn def(&self, id: UnitDefId) -> Option<&UnitDefInfo> {
        self.hello.unit_defs.iter().find(|d| d.id == id)
    }

    /// The cheapest extractor the builder can build.
    fn extractor_of(&self, builder: &UnitDefInfo) -> Option<UnitDefId> {
        builder
            .build_options
            .iter()
            .filter_map(|id| self.def(*id))
            .filter(|d| d.extracts_metal > 0.0)
            .min_by(|a, b| a.metal_cost.total_cmp(&b.metal_cost))
            .map(|d| d.id)
    }
}

impl Bot for Template {
    fn start(hello: Hello) -> Self {
        let coverage = hello.map.metal_map.then(|| layout::Coverage::new(&hello.terrain, hello.map.metal_scale, hello.map.extractor_radius)).flatten();
        Template { hello, sent: BTreeSet::new(), coverage, tiles_sent: Vec::new(), ordered: BTreeMap::new() }
    }

    fn tick(&mut self, tick: Tick) -> Commands {
        let mut commands = Vec::new();
        // On a metal map, our extractors' claims as they stand this tick.
        if let Some(coverage) = &mut self.coverage {
            coverage.clear_claims();
            for u in &tick.snapshot.own_units {
                if let Some(d) = self.hello.unit_defs.iter().find(|d| d.id == u.def).filter(|d| d.extracts_metal > 0.0) {
                    coverage.claim(u.pos, d.extracts_metal);
                }
            }
        }
        for unit in tick.snapshot.own_units.iter().filter(|u| u.idle && !u.being_built) {
            if self.ordered.get(&unit.id).is_some_and(|at| tick.frame - at < ORDER_GRACE_FRAMES) {
                continue;
            }
            let Some(extractor) = self.def(unit.def).and_then(|d| self.extractor_of(d)) else { continue };
            if let Some(coverage) = &self.coverage {
                // The nearest free tile of metal, not one already sent to; the tile is the site itself.
                let def = self.def(extractor).expect("the extractor is a unit type");
                let footprint = layout::Footprint { x: def.footprint.0, z: def.footprint.1 };
                let sent = &self.tiles_sent;
                let ok = |_: Vec3, rect: &layout::Rect| !sent.iter().any(|t| footprint.rect(*t).overlaps(rect));
                let Some((tile, _)) = coverage.best_site(footprint, unit.pos, unit.pos, f32::MAX, def.extracts_metal, &ok) else { continue };
                self.tiles_sent.push(tile);
                self.ordered.insert(unit.id, tick.frame);
                let site = BuildSite { near: tile, search_radius: 0.0, min_dist: 0, placements: Vec::new(), exact: true };
                commands.push(Command::Build { unit: unit.id, def: extractor, site: Some(site), queue: false });
                continue;
            }
            let nearest = (0..self.hello.metal_spots.len())
                .filter(|i| !self.sent.contains(i))
                .min_by(|a, b| unit.pos.dist2d(self.hello.metal_spots[*a]).total_cmp(&unit.pos.dist2d(self.hello.metal_spots[*b])));
            let Some(spot) = nearest else { continue };
            self.sent.insert(spot);
            self.ordered.insert(unit.id, tick.frame);
            let site = BuildSite { near: self.hello.metal_spots[spot], search_radius: 0.0, min_dist: 0, placements: Vec::new(), exact: false };
            commands.push(Command::Build { unit: unit.id, def: extractor, site: Some(site), queue: false });
        }
        Commands(commands)
    }
}

ai_host::export!(ai_host::InProcess<Template>);
