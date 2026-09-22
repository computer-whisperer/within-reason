//! Static game data from `Hello`, indexed for lookup.

use std::collections::HashMap;

use bot_protocol::{Hello, UnitDefId, UnitDefInfo, Vec3};

pub struct World {
    pub hello: Hello,
    defs: HashMap<UnitDefId, usize>,
    names: HashMap<String, UnitDefId>,
}

impl World {
    pub fn new(hello: Hello) -> Self {
        let defs = hello.unit_defs.iter().enumerate().map(|(i, d)| (d.id, i)).collect();
        let names = hello.unit_defs.iter().map(|d| (d.name.clone(), d.id)).collect();
        World { hello, defs, names }
    }

    pub fn def(&self, id: UnitDefId) -> Option<&UnitDefInfo> {
        self.defs.get(&id).map(|&i| &self.hello.unit_defs[i])
    }

    pub fn def_named(&self, name: &str) -> Option<UnitDefId> {
        self.names.get(name).copied()
    }

    /// Name of the cell containing `pos` on an 8x8 grid: columns A-H west to east, rows 1-8 north to south.
    pub fn grid(&self, pos: Vec3) -> String {
        let cell = |value: f32, extent: f32| ((value / extent * 8.0) as i32).clamp(0, 7) as u8;
        let (column, row) = (cell(pos.x, self.hello.map.width), cell(pos.z, self.hello.map.height));
        format!("{}{}", (b'A' + column) as char, row + 1)
    }

    /// The point opposite `pos` through the map centre: a first guess at where the enemy starts.
    pub fn mirrored(&self, pos: Vec3) -> Vec3 {
        Vec3 { x: self.hello.map.width - pos.x, y: 0.0, z: self.hello.map.height - pos.z }
    }

    // The role a definition plays, from the definition alone (docs/design/2026-09-22-full-roster.md, decision 9): a
    // gifted or captured unit of the other faction is played like our own.

    /// A commander: the mobile builder named `*com`.
    pub fn is_commander_def(&self, id: UnitDefId) -> bool {
        self.def(id).is_some_and(|d| d.name.ends_with("com") && d.speed > 0.0 && d.build_speed > 0.0)
    }

    /// A commander or a constructor of any kind: mobile, builds, has a build list.
    pub fn is_mobile_builder(&self, id: UnitDefId) -> bool {
        self.def(id).is_some_and(|d| d.speed > 0.0 && d.build_speed > 0.0 && !d.build_options.is_empty())
    }

    /// A constructor: a mobile builder that is not the commander.
    pub fn is_constructor_def(&self, id: UnitDefId) -> bool {
        self.is_mobile_builder(id) && !self.is_commander_def(id)
    }

    /// A factory of any kind: stands still and has a build list.
    pub fn is_factory_def(&self, id: UnitDefId) -> bool {
        self.def(id).is_some_and(|d| d.speed == 0.0 && !d.build_options.is_empty())
    }

    /// An extractor of any tier.
    pub fn is_extractor_def(&self, id: UnitDefId) -> bool {
        self.def(id).is_some_and(|d| d.extracts_metal > 0.0)
    }

    /// The faction an internal name belongs to, by its prefix.
    pub fn faction_of(name: &str) -> &'static str {
        match name {
            n if n.starts_with("arm") => "armada",
            n if n.starts_with("cor") => "cortex",
            n if n.starts_with("leg") => "legion",
            _ => "other",
        }
    }

    /// Every definition reachable from `root` by build lists, `root` first, the rest in the order found.
    pub fn reachable_from(&self, root: UnitDefId) -> Vec<UnitDefId> {
        let mut seen = vec![root];
        let mut i = 0;
        while i < seen.len() {
            if let Some(d) = self.def(seen[i]) {
                for b in &d.build_options {
                    if !seen.contains(b) {
                        seen.push(*b);
                    }
                }
            }
            i += 1;
        }
        seen
    }

    /// Whether ground under water lies within `radius` of `pos`: where a sea building can stand.
    pub fn water_within(&self, pos: Vec3, radius: f32) -> bool {
        let t = &self.hello.terrain;
        if t.cell <= 0.0 || t.heights.is_empty() {
            return false;
        }
        let cells = (radius / t.cell).ceil() as i64;
        let (cx, cz) = ((pos.x / t.cell) as i64, (pos.z / t.cell) as i64);
        for row in (cz - cells).max(0)..=(cz + cells).min(t.height as i64 - 1) {
            for col in (cx - cells).max(0)..=(cx + cells).min(t.width as i64 - 1) {
                let index = row as usize * t.width as usize + col as usize;
                if t.heights.get(index).is_some_and(|h| *h < 0) {
                    let (dx, dz) = ((col as f32 + 0.5) * t.cell - pos.x, (row as f32 + 0.5) * t.cell - pos.z);
                    if dx.hypot(dz) <= radius {
                        return true;
                    }
                }
            }
        }
        false
    }
}
