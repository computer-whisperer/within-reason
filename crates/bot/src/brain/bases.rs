//! Where the opponents live: one base per enemy seat (H-MAP-ENEMY-BASES).
//!
//! A base starts as a guess (its ally team's start box, else the mirror image of our start) and moves to the seat's
//! factories once one is seen. Every rule that used to ask for "the enemy start" asks for the live base nearest the
//! place in question; with two opponents a mean over both would be a point between them where nobody lives.

use std::collections::HashMap;

use bot_protocol::{Event, OwnUnit, Tick, UnitId, Vec3};

/// A factory with no team on it belongs to the base this near, else to the nearest.
const BASE_RADIUS: f32 = 1500.0;
/// A found base with no factory left and nothing remembered standing this near is dead.
const RAZED_RADIUS: f32 = 1200.0;
/// H-MAP-ENEMY-CLUSTER: a building of theirs seen this near an unfound base's guess, and nearer the guess than our
/// home, refines the guess to the metal-spot cluster nearest the building, spots chained within `CLUSTER_LINK` of
/// one another. (2,500 took a forward extractor of theirs in the contested middle for the base: pianist-player-5;
/// 1,200 took a building 1,105 off on hard for it and moved the guess to a lone empty spot: pianist-player-9. The
/// start's own extractors lie within 700 of it, K-scout-enemy-base-is-a-spot-cluster.)
const REFINE_RADIUS: f32 = 700.0;
const CLUSTER_LINK: f32 = 700.0;
/// H-MAP-ENEMY-GUESS-EMPTY: a refined guess with a unit of ours this near it, and no building of theirs remembered
/// within `EMPTY_RADIUS`, was wrong; it goes back to the start guess and that cluster is not guessed again.
const LOOKED_RADIUS: f32 = 500.0;
const EMPTY_RADIUS: f32 = 800.0;

pub struct EnemyBase {
    /// The seat, when the script named the teams; `None` for a lone mirror guess.
    pub team: Option<i32>,
    pub at: Vec3,
    /// A factory of this seat has been seen (it may be gone since).
    pub found: bool,
    /// Found, and then razed to the ground: no longer a target nor a claim on ground.
    pub dead: bool,
    /// The guess was moved to the spot cluster nearest the first building of theirs seen (H-MAP-ENEMY-CLUSTER).
    refined: bool,
    /// The start guess (box or mirror, snapped to metal), to go back to when a refined guess is found empty.
    guessed: Vec3,
    /// Refined guesses our units stood at and found empty (H-MAP-ENEMY-GUESS-EMPTY).
    rejected: Vec<Vec3>,
    factories: HashMap<UnitId, Vec3>,
}

impl super::Brain {
    /// First guesses, once our own start is known.
    pub(super) fn guess_enemy_bases(&mut self) {
        let hello = &self.world.hello;
        let mut bases: Vec<EnemyBase> = Vec::new();
        // Not the engine's Gaia team, which is on no side (it stood in this list as a phantom enemy base until 2026-09-23).
        let enemy_teams: Vec<(i32, i32)> = hello.teams.iter().filter(|t| t.ally_team != hello.ally_team && t.controller != bot_protocol::Controller::Gaia).map(|t| (t.team, t.ally_team)).collect();
        // The lobby's boxes are a guess only while they are honoured: our own start inside another team's box, or
        // outside our own, means the host placed the seats by hand and the boxes say nothing of where he is
        // (bluegecko-3v1-comet-catcher-10: our G1 seat stood in "his" top box; every rover and push went along the
        // top row, and his base in the south-west was never seen).
        let ours_in_theirs = hello.start_boxes.iter().any(|b| b.ally_team != hello.ally_team && b.contains(self.home));
        let ours_outside_ours = hello.start_boxes.iter().any(|b| b.ally_team == hello.ally_team && !b.contains(self.home));
        self.boxes_honoured = !(ours_in_theirs || ours_outside_ours);
        if !self.boxes_honoured {
            eprintln!("[ai {}] the lobby's start boxes are not honoured (our start {} in another team's box, {} outside ours): his start is the mirror until seen", hello.ai_id, if ours_in_theirs { "is" } else { "is not" }, if ours_outside_ours { "and" } else { "and not" });
        }
        // The lobby fixes the positions (`startpostype` 0, the map's; 3, chosen before the game): the script then
        // carries each team's start, his included, and the boxes are the map's defaults riding along.
        let fixed = matches!(hello.start_pos_type, Some(0) | Some(3));
        if fixed {
            self.boxes_honoured = false;
        }
        for (index, (team, ally_team)) in enemy_teams.iter().enumerate() {
            let sharing: Vec<i32> = enemy_teams.iter().filter(|(_, a)| a == ally_team).map(|(t, _)| *t).collect();
            let scripted = hello.teams.iter().find(|t| t.team == *team).and_then(|t| t.start_pos);
            if let Some(at) = scripted.filter(|_| fixed) {
                eprintln!("[ai {}] enemy team {team} starts at ({:.0}, {:.0}) by the script (startpostype {})", hello.ai_id, at.x, at.z, hello.start_pos_type.unwrap_or(-1));
                bases.push(EnemyBase { team: Some(*team), at, found: false, dead: false, refined: true, guessed: at, rejected: Vec::new(), factories: HashMap::new() });
                continue;
            }
            let at = match hello.start_boxes.iter().find(|b| b.ally_team == *ally_team).filter(|_| self.boxes_honoured) {
                // Seats sharing a box are spread along its longer side.
                Some(b) => {
                    let share = (sharing.iter().position(|t| t == team).unwrap_or(0) as f32 + 0.5) / sharing.len() as f32;
                    if b.right - b.left >= b.bottom - b.top {
                        Vec3 { x: b.left + (b.right - b.left) * share, ..b.centre() }
                    } else {
                        Vec3 { z: b.top + (b.bottom - b.top) * share, ..b.centre() }
                    }
                }
                None if index == 0 => self.world.mirrored(self.home),
                None => continue,
            };
            // A lobby's boxes can put an enemy beside us (human-1 second game: three ally teams, the first enemy's
            // box centre 579 from our start, so "enemy_base" was the ground next to home and the lab yard faced
            // spot_10). A guess nearer than a third of the map's shorter side is no guess: the mirror is.
            let too_near = hello.map.width.min(hello.map.height) / 3.0;
            let at = if at.dist2d(self.home) < too_near {
                eprintln!("[ai {}] enemy team {team}'s box guess {:.0} from our start is discarded for the mirror", hello.ai_id, at.dist2d(self.home));
                self.world.mirrored(self.home)
            } else {
                at
            };
            bases.push(EnemyBase { team: Some(*team), at, found: false, dead: false, refined: false, guessed: at, rejected: Vec::new(), factories: HashMap::new() });
        }
        if bases.is_empty() {
            let at = self.world.mirrored(self.home);
            bases.push(EnemyBase { team: None, at, found: false, dead: false, refined: false, guessed: at, rejected: Vec::new(), factories: HashMap::new() });
        }
        self.enemy_bases = bases;
    }

    /// H-MAP-ENEMY-START: a start is always beside metal, and a box centre or a mirror point may be a beach or a cliff
    /// top; each guess moves to the metal spot nearest it that we can walk to, never one near our own start (on
    /// SailAway 2, an islet start with three of 96 spots reachable on foot, two seats' guesses snapped to the spots
    /// beside their own homes, 90 and 310 away; the guess then stays where it was).
    pub(super) fn snap_guesses_to_metal(&mut self, reachable: impl Fn(Vec3) -> bool) {
        let spots = &self.world.hello.metal_spots;
        let too_near = self.world.hello.map.width.min(self.world.hello.map.height) / 3.0;
        let home = self.home;
        for base in self.enemy_bases.iter_mut().filter(|b| !b.found) {
            if let Some(spot) = spots.iter().filter(|s| reachable(**s) && s.dist2d(home) >= too_near).min_by(|a, b| a.dist2d(base.at).total_cmp(&b.dist2d(base.at))) {
                base.at = Vec3 { y: 0.0, ..*spot };
                base.guessed = base.at;
            }
        }
    }

    /// Moves each base to its seat's factories in sight or remembered, and calls a razed base dead.
    pub(super) fn track_enemy_bases(&mut self, tick: &Tick) {
        if !self.enabled("H-MAP-ENEMY-BASE") {
            return;
        }
        for event in &tick.events {
            if let Event::EnemyDestroyed { enemy } = event {
                self.enemy_bases.iter_mut().for_each(|b| { b.factories.remove(enemy); });
            }
        }
        for enemy in &tick.snapshot.enemies {
            let is_factory = enemy.def.and_then(|d| self.world.def(d)).is_some_and(|d| d.speed == 0.0 && !d.build_options.is_empty());
            if !is_factory || self.enemy_bases.iter().any(|b| b.factories.contains_key(&enemy.id)) {
                continue;
            }
            let by_team = enemy.team.and_then(|team| self.enemy_bases.iter().position(|b| b.team == Some(team)));
            let nearest = || {
                (0..self.enemy_bases.len()).min_by(|a, b| {
                    let key = |i: &usize| (self.enemy_bases[*i].dead, self.enemy_bases[*i].at.dist2d(enemy.pos));
                    key(a).0.cmp(&key(b).0).then(key(a).1.total_cmp(&key(b).1))
                })
            };
            let Some(index) = by_team.or_else(nearest) else { continue };
            // A teamless factory far from every base is a base of its own (a seat the script did not tell us of).
            if by_team.is_none() && self.enemy_bases[index].found && self.enemy_bases[index].at.dist2d(enemy.pos) > BASE_RADIUS {
                self.enemy_bases.push(EnemyBase { team: enemy.team, at: enemy.pos, found: true, dead: false, refined: true, guessed: enemy.pos, rejected: Vec::new(), factories: HashMap::from([(enemy.id, enemy.pos)]) });
                continue;
            }
            self.enemy_bases[index].factories.insert(enemy.id, enemy.pos);
        }
        // Factories we stood beside and could not see are gone (`forget_razed_buildings`).
        let remembered = &self.enemy_buildings;
        let mut moved = false;
        for base in &mut self.enemy_bases {
            base.factories.retain(|id, _| remembered.contains_key(id));
            if !base.factories.is_empty() {
                let n = base.factories.len() as f32;
                let at = base.factories.values().fold(Vec3::default(), |sum, p| Vec3 { x: sum.x + p.x / n, y: 0.0, z: sum.z + p.z / n });
                moved |= at.dist2d(base.at) > 1.0;
                (base.at, base.found, base.dead) = (at, true, false);
            } else if base.found {
                let standing = remembered.values().any(|(_, pos, _)| pos.dist2d(base.at) < RAZED_RADIUS);
                if base.dead == standing {
                    eprintln!("[ai {}] f={} enemy base at ({:.0}, {:.0}) is {}", self.world.hello.ai_id, tick.frame, base.at.x, base.at.z, if standing { "alive after all" } else { "razed" });
                    moved = true;
                }
                base.dead = !standing;
            }
        }
        moved |= self.reject_empty_guesses(&tick.snapshot.own_units, tick.frame);
        moved |= self.refine_guesses_from_sightings(tick.frame);
        if moved {
            self.resurvey_enemy();
        }
    }

    /// H-MAP-ENEMY-GUESS-EMPTY: a refined guess that a unit of ours stands beside with nothing of theirs remembered
    /// near it goes back to the start guess, and its cluster is not guessed again. (pianist-player-5: a forward
    /// extractor of theirs moved the guess to the middle of the map at 6:31; the ball stood on the guess at 8:52 and
    /// saw nothing, and the guess stayed there to the end while the player hunted south.) True when a guess moved.
    fn reject_empty_guesses(&mut self, own: &[OwnUnit], frame: i32) -> bool {
        let mut moved = false;
        let mut lines: Vec<String> = Vec::new();
        for base in self.enemy_bases.iter_mut().filter(|b| !b.found && !b.dead && b.refined) {
            let looked = own.iter().any(|u| !u.being_built && u.pos.dist2d(base.at) < LOOKED_RADIUS);
            let standing = self.enemy_buildings.values().any(|(_, pos, _)| pos.dist2d(base.at) < EMPTY_RADIUS);
            if !looked || standing {
                continue;
            }
            eprintln!(
                "[ai {}] f={frame} enemy base guess at ({:.0}, {:.0}) found empty: back to ({:.0}, {:.0})",
                self.world.hello.ai_id, base.at.x, base.at.z, base.guessed.x, base.guessed.z
            );
            lines.push(format!("{} the enemy base guess at {} was found empty by our units standing there; the guess is back at {}", super::pianist::clock(frame), self.world.grid(base.at), self.world.grid(base.guessed)));
            base.rejected.push(base.at);
            base.at = base.guessed;
            base.refined = false;
            moved = true;
        }
        if let Some(pianist) = &mut self.pianist {
            pianist.done.extend(lines);
        }
        moved
    }

    /// H-MAP-ENEMY-CLUSTER: the first building of theirs seen near an unfound base's guess moves the guess to the
    /// metal-spot cluster nearest that building, at the cluster's spot nearest its middle: the reasonable start spot.
    /// The user, from a Quicksilver game: first contact at 1:52 is enough for a high guess that the enemy's
    /// extractors are all round D7. True when a guess moved.
    fn refine_guesses_from_sightings(&mut self, frame: i32) -> bool {
        // Not under the player (docs/design/2026-09-22-enemy-evidence.md): finding the opponent is the player's
        // judgment and nothing of this guess reaches its picture; the guess only seeds the routing field.
        if self.pianist.is_some() {
            return false;
        }
        let spots = &self.world.hello.metal_spots;
        let mut moved = false;
        let home = self.home;
        for base in self.enemy_bases.iter_mut().filter(|b| !b.found && !b.dead && !b.refined) {
            let Some((_, building, _)) = self.enemy_buildings.values().filter(|(_, pos, _)| pos.dist2d(base.at) < REFINE_RADIUS && pos.dist2d(base.at) < pos.dist2d(home)).min_by(|a, b| a.1.dist2d(base.at).total_cmp(&b.1.dist2d(base.at))) else { continue };
            let Some(seed) = (0..spots.len()).min_by(|a, b| spots[*a].dist2d(*building).total_cmp(&spots[*b].dist2d(*building))) else { continue };
            let mut cluster = vec![seed];
            let mut grew = true;
            while grew {
                grew = false;
                for index in 0..spots.len() {
                    if !cluster.contains(&index) && cluster.iter().any(|c| spots[*c].dist2d(spots[index]) < CLUSTER_LINK) {
                        cluster.push(index);
                        grew = true;
                    }
                }
            }
            let n = cluster.len() as f32;
            let middle = cluster.iter().fold(Vec3::default(), |sum, i| Vec3 { x: sum.x + spots[*i].x / n, y: 0.0, z: sum.z + spots[*i].z / n });
            let at = cluster.iter().map(|i| Vec3 { y: 0.0, ..spots[*i] }).min_by(|a, b| a.dist2d(middle).total_cmp(&b.dist2d(middle))).unwrap();
            if base.rejected.iter().any(|r| r.dist2d(at) < 1.0) {
                continue;
            }
            eprintln!(
                "[ai {}] f={} enemy base guessed at ({:.0}, {:.0}) from a building at ({:.0}, {:.0}): a cluster of {} spots (was ({:.0}, {:.0}))",
                self.world.hello.ai_id, frame, at.x, at.z, building.x, building.z, cluster.len(), base.at.x, base.at.z
            );
            moved |= at.dist2d(base.at) > 1.0;
            base.at = at;
            base.refined = true;
        }
        moved
    }

    /// Bases that still count; all of them when every one is dead (something must be "the enemy's side").
    pub(super) fn live_enemy_bases(&self) -> Vec<Vec3> {
        let live: Vec<Vec3> = self.enemy_bases.iter().filter(|b| !b.dead).map(|b| b.at).collect();
        if live.is_empty() { self.enemy_bases.iter().map(|b| b.at).collect() } else { live }
    }

    /// The live enemy base nearest `from`, as the crow flies.
    pub(super) fn enemy_base(&self, from: Vec3) -> Vec3 {
        self.live_enemy_bases().into_iter().min_by(|a, b| a.dist2d(from).total_cmp(&b.dist2d(from))).unwrap_or_else(|| self.world.mirrored(self.home))
    }

    /// The found, live base nearest home, if any base has been found.
    pub(super) fn found_enemy_base(&self) -> Option<Vec3> {
        self.enemy_bases.iter().filter(|b| b.found && !b.dead).map(|b| b.at).min_by(|a, b| a.dist2d(self.home).total_cmp(&b.dist2d(self.home)))
    }
}
