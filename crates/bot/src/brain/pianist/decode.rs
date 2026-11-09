//! The decode and its fuses (`docs/design/2026-10-01-hands-rebuild.md` §8, the `fuse` layer; the user, 2026-10-01:
//! "a decode of the opus's existing orders (not a second tool call) to fuse off jev questions that we can answer
//! once for the entire span"). Once a packet, and again for whatever is new under it (an actor, a place, a type),
//! one lean request asks Jev about the player's instructions alone: whether they name a place for an actor, have a
//! builder build a type, tell a group to join or follow another, forbid a group a detachment of a size, his
//! aircraft or a place. The fuse
//! rule: a fuse answers only a question about the instructions alone, never one about the picture, and only when
//! the decode's answer is clear. A move whose reading is under `OFF` is fused off until the next packet (it is not
//! asked and cannot be picked); the forbidden mark of a detachment or of an attack on aircraft at `ON` or over is
//! on, and under `OFF` off, without the second's own question; everything between is asked each second as in the
//! base. A group's walk to a place is marked at `AVOID` or over by the decode's reading alone. No tool call and no new
//! syntax: the decode reads the packet the player already wrote, and the player's report shows what it read.

use std::collections::{BTreeMap, BTreeSet};

use jev::Question;
use serde_json::json;

use super::menu::{Kind, Menu, Read};

/// A reading under this is clearly no, for the forbidden mark (off without asking).
pub(super) const OFF: f64 = 0.2;
/// A move whose reading is under this is fused off: the instructions do not give the actor this move, and it is
/// not asked. Raised from 0.2 on 2026-10-03 (the user: the cut stays, for pruning, and goes higher): player-56 at
/// 6:11, the reading for a body's walk back to one of our own extractors was 0.22, the move stayed on the menu and
/// was played. Over six games 62 of 995 place moves played had a reading of 0.2 to 0.3, and one under 0.2.
pub(super) const FUSE: f64 = 0.3;
/// A reading at this or over is clearly yes (the forbidden mark is on without asking).
pub(super) const ON: f64 = 0.8;
/// A group's walk to a place whose `Avoid` reading is at this or over carries the forbidden mark: the mark's own
/// bar (`compose::FORBIDDEN`), on the decode's reading and not on a question of the second's. The second's
/// question ("do they forbid this move for the group that would make it") does not separate a walk the packet
/// forbids from one it does not speak of: 0.40 to 0.89 for moves not spoken of over 90,638 moves of player-32
/// (K-jev-a-forbid-noul-on-every-arm-ranks-but-does-not-separate), and 0.51 and 0.66 for two walks the decode read
/// at 0.10 (walk-forbidden-smoke). `run/jev_avoid_ab.py` on 1794187275-apex_2v1 (198 readings of a place the
/// game's decode had read as a group's, `docs/studies/data/avoid-place-2026-11-08.jsonl`): 22 at this bar, the
/// eleven of a place a packet forbids the group by name (0.75 to 0.98), five of `home` under "never comes home
/// for raiders", three of a route's stops while the route's condition is unmet (0.70 to 0.72), and three of a
/// place named only as an edge ("never goes east of spot_64", 0.78 to 0.86), which the packet allows: the
/// reading's known fault. 160 of the 198 read under 0.3.
pub(super) const AVOID: f64 = 0.7;
/// The share of what the layer fuses that is asked anyway, for its audit.
const AUDIT_SHARE: f64 = 0.02;

/// What the decode has read of the packet in force: each reading by its key (`<actor>|place|<place>`,
/// `<actor>|build|<type>`, `<actor>|join|<group>`, `<actor>|follow|<ward>`, `<actor>|detach|<n>`, `<actor>|air`,
/// `<actor>|avoid|<place>`, `<group>|back`).
#[derive(Default)]
pub(crate) struct Decode {
    /// The packet the readings are of: another packet begins them again.
    pub packet: String,
    pub reads: BTreeMap<String, f64>,
}

fn key(actor: &str, read: &Read) -> String {
    match read {
        Read::Place(place) => format!("{actor}|place|{place}"),
        Read::Build(name, _) => format!("{actor}|build|{name}"),
        Read::Join(other) => format!("{actor}|join|{other}"),
        Read::TakeIn(other) => format!("{actor}|take_in|{other}"),
        Read::Follow(ward) => format!("{actor}|follow|{ward}"),
        Read::Detach(n) => format!("{actor}|detach|{n}"),
        Read::Air => format!("{actor}|air"),
        Read::Avoid(place) => format!("{actor}|avoid|{place}"),
    }
}

/// The decode's question for a reading. The place question is the form measured on two games (`run/jev_read_ab.py`,
/// the design note §9: 72% of answers under 0.2, a place in another actor's paragraph read as this actor's in 1
/// to 2%); a place the instructions keep the actor away from reads yes, and is then asked each second.
fn question(actor: &str, what: &str, read: &Read) -> Question {
    let text = match read {
        Read::Place(place) => format!("Read the player's `instructions` alone. Do they name {place} in anything they say to or about {actor} ({what}): a place it goes to, stands at, fights at, builds at, falls back to, or must keep away from?"),
        Read::Build(_, words) => format!("Read the player's `instructions` alone. Do they have {actor} ({what}) build a {words}, now or under some condition, or say anything else to it about that kind of building?"),
        // One way only (player-35, 23:25: under "group_S6 joins group_V4" the question's "or become one body with
        // it" read yes for both, and the army of 43 joined its own detachment of 8).
        Read::Join(other) => format!("Read the player's `instructions` alone. Do they tell {actor} ({what}) to join {other}, now or under some condition: {actor} to merge into {other} and take {other}'s name and course? No when it is {other} they tell to join {actor}."),
        Read::TakeIn(other) => format!("Read the player's `instructions` alone. Do they tell {other} to join {actor} ({what}), or {actor} to take {other} in, now or under some condition: {other} to merge into {actor} and take {actor}'s name and course? No when it is {actor} they tell to join {other}."),
        Read::Follow(ward) => format!("Read the player's `instructions` alone. Do they tell {actor} ({what}) to follow, escort or stay beside {ward}, now or under some condition?"),
        Read::Detach(n) => format!("Read the player's `instructions` alone. Do they forbid {actor} ({what}) to send a detachment of {n} of its soldiers after an enemy party?"),
        Read::Air => format!("Read the player's `instructions` alone. Do they forbid {actor} ({what}) to attack, chase or hunt his aircraft (planes, bombers, gunships, air scouts), or tell it to ignore them or leave them alone?"),
        // The `plain` form of `run/jev_avoid_ab.py` (the readings at `AVOID`). The two forms that tell Jev an edge
        // is not a prohibition read the forbidden places lower and everything else higher (under 0.2: 78 and 121
        // of 198 against this form's 142).
        Read::Avoid(place) => format!("Read the player's `instructions` alone. Do they forbid {actor} ({what}) to go to {place}, or tell it to keep away from it?"),
    };
    Question::noul(json!(text))
}

/// The question whether the instructions give a group a place to step back to, for the player's report.
fn back_question(actor: &str, what: &str) -> Question {
    Question::noul(json!(format!("Read the player's `instructions` alone. Do they say where {actor} ({what}) goes when it should not fight: a place it walks to, steps back to or falls back to?")))
}

/// The readings the menus need that the decode does not hold yet: one question each. `what` gives an actor's
/// words from the picture ("a handful: 3 Blitz (armflash)"). `newer` are the actors that appeared after the packet
/// landed and that it does not name: of them only whether a place is forbidden is read (`fuse`). A place's
/// `Avoid` reading is asked once the place is read as named for the actor (`Place` at `FUSE` or over: a walk to a
/// place the instructions do not name is fused off and needs no mark), and at once where the move carries no
/// `Place` reading (an attack on his buildings) or the actor is one of `newer`.
pub(super) fn needed(menus: &[Menu], decode: &Decode, newer: &BTreeSet<String>, what: &dyn Fn(&str) -> String) -> BTreeMap<String, Question> {
    let mut out = BTreeMap::new();
    for menu in menus {
        let new = newer.contains(&menu.name);
        let words = what(&menu.name);
        for m in menu.moves.iter().filter(|m| !m.far) {
            for read in &m.reads {
                if let Read::Avoid(place) = read {
                    let named = Read::Place(place.clone());
                    if !new && m.reads.contains(&named) && decode.reads.get(&key(&menu.name, &named)).is_none_or(|p| *p < FUSE) {
                        continue;
                    }
                } else if new {
                    continue;
                }
                let k = key(&menu.name, read);
                if !decode.reads.contains_key(&k) && !out.contains_key(&k) {
                    out.insert(k, question(&menu.name, &words, read));
                }
            }
        }
        if !new && matches!(menu.kind, Kind::Group(_)) {
            let k = format!("{}|back", menu.name);
            if !decode.reads.contains_key(&k) {
                out.insert(k, back_question(&menu.name, &words));
            }
        }
    }
    out
}

/// Puts the decode's clear readings on the menus: a move with a reading of what the instructions give its actor
/// under `FUSE` is fused off; a forbiddable move's mark is set where the reading of what they forbid it is clear;
/// a walk to a place read as forbidden the group at `AVOID` or over is marked. Of what it fuses, and of the
/// forbiddable moves it marks, a share of `AUDIT_SHARE` is asked anyway.
/// The extractor move (`extractor`, the faction's extractor by internal name) is never fused: the players write
/// "take free spots" and the question asks about a "Metal Extractor (armmex)", and the reading sat under the bar in
/// 11 of 25 paragraphs that give extractors while the light turret's and the solar's, which the players name, read
/// right (the impedance study §3.2b, 2026-10-16; the user: "stop fusing the extractor move, keep the reading for
/// the report"). The reading is still asked and kept, for the player's report.
/// An actor that appeared after the packet in force landed and that the packet does not name (`newer`) is not
/// fused: the packet could not have spoken of it, so its silence is no reading (player-35, 28:32-28:55: a detachment of eight was born with every
/// walk and its join back fused off, and six died while the audit rated those moves 0.55 to 0.68; 73 of the game's
/// 91 detachments were born with the join back fused). A place the instructions forbid is a reading of what they
/// say, not of their silence, and they say it of groups not yet born ("no group of ours ever goes ..."): a newer
/// group's walks are marked by it. Returns the questions not sent and the moves audited.
pub(super) fn fuse(menus: &mut [Menu], decode: &Decode, newer: &BTreeSet<String>, extractor: &str, draw: &mut dyn FnMut() -> f64) -> (usize, usize) {
    let (mut skipped, mut audited) = (0, 0);
    for menu in menus.iter_mut() {
        let new = newer.contains(&menu.name);
        let name = menu.name.clone();
        for m in menu.moves.iter_mut().skip(1).filter(|m| !m.far) {
            if m.reads.iter().any(|r| matches!(r, Read::Build(unit, _) if unit == extractor)) {
                continue;
            }
            let reading = |read: &Read| decode.reads.get(&key(&name, read)).copied();
            let mut audit = |m: &mut super::menu::Move| {
                if draw() < AUDIT_SHARE {
                    m.audit = true;
                    audited += 1;
                } else {
                    skipped += 1;
                }
            };
            // The fuse: the instructions clearly do not give the actor this move (its place, type, join or ward).
            if !new && m.reads.iter().filter(|r| !r.forbids()).filter_map(|r| reading(r)).any(|p| p < FUSE) {
                m.fused = true;
                audit(m);
                continue;
            }
            if m.forbiddable && !new {
                // The mark, never a fuse: the move itself stays a question. Forbidden by any of its readings (a
                // detachment after aircraft: the detachment's and the aircraft's).
                let Some(p) = m.reads.iter().filter(|r| r.forbids()).filter_map(|r| reading(r)).fold(None, |m: Option<f64>, p| Some(m.map_or(p, |m| m.max(p)))) else { continue };
                m.marked = if p >= ON { Some(true) } else if p < OFF { Some(false) } else { None };
                if m.marked.is_some() {
                    audit(m);
                }
            } else if m.reads.iter().filter(|r| matches!(r, Read::Avoid(_))).filter_map(|r| reading(r)).any(|p| p >= AVOID) {
                // A walk to a place the instructions forbid the group: marked by this reading alone, of a group
                // born after the packet as of any other.
                m.marked = Some(true);
            }
        }
    }
    (skipped, audited)
}

/// What the decode read of the packet for a builder, for the player's report: its extractor reading, which shuts
/// nothing (`fuse`), said so the player sees how its words read.
pub(super) fn builder_reads_words(decode: &Decode, actor: &str, extractor: &str) -> Option<String> {
    let p = decode.reads.get(&format!("{actor}|build|{extractor}"))?;
    Some(format!("extractors {p:.2} (how the packet reads for it; this one shuts no move)"))
}

/// What the decode reads for a group, for the player's report: the places at 0.5 or over, those of them it reads
/// as forbidden the group apart (at 0.5 or over), and whether the instructions say where it steps back to
/// ("spot_27, spot_18; kept away from spot_61; NO place to step back to").
pub(super) fn reads_words(decode: &Decode, actor: &str) -> Option<String> {
    let prefix = format!("{actor}|place|");
    let avoided = |place: &str| decode.reads.get(&format!("{actor}|avoid|{place}")).is_some_and(|p| *p >= 0.5);
    let named: Vec<&str> = decode.reads.iter().filter(|(k, p)| k.starts_with(&prefix) && **p >= 0.5).map(|(k, _)| &k[prefix.len()..]).collect();
    let (kept_from, places): (Vec<&str>, Vec<&str>) = named.into_iter().partition(|place| avoided(place));
    let back = decode.reads.get(&format!("{actor}|back")).copied();
    if places.is_empty() && kept_from.is_empty() && back.is_none() {
        return None;
    }
    let mut words = if places.is_empty() { "no place".to_string() } else { places.join(", ") };
    if !kept_from.is_empty() {
        words.push_str(&format!("; kept away from {}", kept_from.join(", ")));
    }
    if back.is_some_and(|p| p < 0.5) {
        words.push_str("; NO place to step back to");
    }
    Some(words)
}

#[cfg(test)]
mod tests {
    use super::super::menu::tests::{menu, mv};
    use super::super::menu::Order;
    use super::*;
    use bot_protocol::UnitId;

    fn group() -> Menu {
        let mut m = menu(
            "group_A",
            Kind::Group("A".into()),
            false,
            vec![mv("go_spot_30", Order::Go("spot_30".into()), None), mv("go_spot_62", Order::Go("spot_62".into()), None), mv("send_1_party_7", Order::Send(vec![UnitId(9)], "party_7".into()), Some("party_7")), mv("send_2_party_7", Order::Send(vec![UnitId(9), UnitId(8)], "party_7".into()), Some("party_7")), mv("attack_party_7", Order::Attack("party_7".into()), Some("party_7"))],
        );
        m.moves[1].reads = vec![Read::Place("spot_30".into())];
        m.moves[2].reads = vec![Read::Place("spot_62".into())];
        m.moves[3].reads = vec![Read::Detach(1)];
        m.moves[4].reads = vec![Read::Detach(2)];
        m
    }

    /// A group's walk carries the mark of a place the instructions forbid it (the user, 2026-11-08). The place is
    /// asked first; the forbidden reading only of a place read as named, of an attack on his buildings, and of
    /// every walk of a group born after the packet, which is never fused and is marked. The mark is the decode's
    /// alone: under the bar there is none, and nothing is left to the second's question.
    #[test]
    fn a_walk_to_a_place_the_instructions_forbid_is_marked() {
        let walks = || {
            let mut m = menu(
                "group_A",
                Kind::Group("A".into()),
                false,
                vec![mv("go_spot_61", Order::Go("spot_61".into()), None), mv("fight_to_spot_73", Order::FightTo("spot_73".into()), None), mv("go_spot_9", Order::Go("spot_9".into()), None), mv("fight_to_spot_47", Order::Raid("spot_47".into()), None)],
            );
            for (i, place) in [(1, "spot_61"), (2, "spot_73"), (3, "spot_9")] {
                m.moves[i].reads = vec![Read::Place(place.into()), Read::Avoid(place.into())];
            }
            m.moves[4].reads = vec![Read::Avoid("spot_47".into())];
            m
        };
        let what = |_: &str| "9 Incisor".to_string();
        let nobody = BTreeSet::new();
        let mut decode = Decode::default();
        let asks = needed(&[walks()], &decode, &nobody, &what);
        assert_eq!(asks.keys().cloned().collect::<Vec<_>>(), ["group_A|avoid|spot_47", "group_A|back", "group_A|place|spot_61", "group_A|place|spot_73", "group_A|place|spot_9"], "a place is read as named before it is read as forbidden; his buildings at once");
        let Question::Noul { instructions, .. } = &asks["group_A|avoid|spot_47"] else { panic!("a noul") };
        assert_eq!(instructions.as_str().unwrap(), "Read the player's `instructions` alone. Do they forbid group_A (9 Incisor) to go to spot_47, or tell it to keep away from it?");
        decode.reads = [("group_A|back", 0.9), ("group_A|avoid|spot_47", 0.5), ("group_A|place|spot_61", 0.97), ("group_A|place|spot_73", 0.99), ("group_A|place|spot_9", 0.04)].into_iter().map(|(k, p)| (k.to_string(), p)).collect();
        assert_eq!(needed(&[walks()], &decode, &nobody, &what).keys().cloned().collect::<Vec<_>>(), ["group_A|avoid|spot_61", "group_A|avoid|spot_73"], "not of the place the instructions do not name");
        decode.reads.insert("group_A|avoid|spot_61".into(), 0.97);
        decode.reads.insert("group_A|avoid|spot_73".into(), 0.05);
        let mut menus = vec![walks()];
        assert_eq!(fuse(&mut menus, &decode, &nobody, "armmex", &mut || 0.5), (1, 0));
        let m = &menus[0].moves;
        assert_eq!((m[1].fused, m[1].marked), (false, Some(true)), "named and forbidden: on the menu under its mark");
        assert_eq!((m[2].fused, m[2].marked), (false, None));
        assert_eq!((m[3].fused, m[3].marked), (true, None), "not named: fused as before, no mark asked");
        assert_eq!((m[4].fused, m[4].marked), (false, None), "under the bar: no mark");
        assert!(m.iter().all(|m| !m.forbiddable && !m.audit), "nothing of a walk is the second's question");
        assert_eq!(reads_words(&decode, "group_A").as_deref(), Some("spot_73; kept away from spot_61"));
        // A group born after the packet: nothing of what the packet gives it is read and nothing is fused; what
        // the packet forbids every group is read and marked.
        let newer: BTreeSet<String> = ["group_A".to_string()].into();
        let asks = needed(&[walks()], &Decode::default(), &newer, &what);
        assert_eq!(asks.keys().cloned().collect::<Vec<_>>(), ["group_A|avoid|spot_47", "group_A|avoid|spot_61", "group_A|avoid|spot_73", "group_A|avoid|spot_9"]);
        decode.reads.insert("group_A|avoid|spot_9".into(), 0.9);
        let mut menus = vec![walks()];
        assert_eq!(fuse(&mut menus, &decode, &newer, "armmex", &mut || 0.5), (0, 0));
        let m = &menus[0].moves;
        assert!(m.iter().all(|m| !m.fused));
        assert_eq!((m[1].marked, m[2].marked, m[3].marked, m[4].marked), (Some(true), None, Some(true), None));
    }

    /// The decode asks once what the menus need of the packet, and the fuse rule holds: a clear no fuses a move
    /// off, a clear reading sets a detachment's mark, the middle is left to the second's own question, and a move
    /// aimed at the picture (an attack on a party) is never the decode's.
    #[test]
    fn a_clear_reading_of_the_packet_fuses_a_move_and_the_middle_is_asked() {
        let mut decode = Decode::default();
        let menus = vec![group()];
        let what = |_: &str| "3 Blitz".to_string();
        let nobody = BTreeSet::new();
        let asks = needed(&menus, &decode, &nobody, &what);
        assert_eq!(asks.keys().cloned().collect::<Vec<_>>(), ["group_A|back", "group_A|detach|1", "group_A|detach|2", "group_A|place|spot_30", "group_A|place|spot_62"]);
        let Question::Noul { instructions, .. } = &asks["group_A|place|spot_30"] else { panic!("a noul") };
        assert!(instructions.as_str().unwrap().starts_with("Read the player's `instructions` alone. Do they name spot_30 in anything they say to or about group_A (3 Blitz)"));
        decode.reads = [("group_A|back", 0.9), ("group_A|detach|1", 0.05), ("group_A|detach|2", 0.9), ("group_A|place|spot_30", 0.85), ("group_A|place|spot_62", 0.03)].into_iter().map(|(k, p)| (k.to_string(), p)).collect();
        assert!(needed(&menus, &decode, &nobody, &what).is_empty(), "what is read is not asked again under the same packet");
        let mut menus = menus;
        assert_eq!(fuse(&mut menus, &decode, &nobody, "armmex", &mut || 0.5), (3, 0));
        let m = &menus[0].moves;
        assert!(!m[1].fused && m[2].fused && !m[5].fused);
        assert_eq!((m[3].marked, m[4].marked), (Some(false), Some(true)));
        assert_eq!(reads_words(&decode, "group_A").as_deref(), Some("spot_30"));
        // The middle is nobody's to fuse; a missing step-back place is said.
        decode.reads.insert("group_A|place|spot_62".into(), 0.4);
        decode.reads.insert("group_A|detach|2".into(), 0.5);
        decode.reads.insert("group_A|back".into(), 0.1);
        let mut menus = vec![group()];
        assert_eq!(fuse(&mut menus, &decode, &nobody, "armmex", &mut || 0.5), (1, 0));
        assert!(!menus[0].moves[2].fused && menus[0].moves[4].marked.is_none());
        assert_eq!(reads_words(&decode, "group_A").as_deref(), Some("spot_30; NO place to step back to"));
        // The audit: a fused move drawn under the share is asked anyway, and still not played.
        decode.reads.insert("group_A|place|spot_62".into(), 0.03);
        let mut menus = vec![group()];
        assert_eq!(fuse(&mut menus, &decode, &nobody, "armmex", &mut || 0.001), (0, 2));
        assert!(menus[0].moves[2].fused && menus[0].moves[2].audit && menus[0].moves[2].asked());
        // The extractor move is never fused, whatever its reading; another build type is.
        let mut menus = vec![group()];
        menus[0].moves[1].reads = vec![Read::Build("armmex".into(), "Metal Extractor (armmex)".into())];
        menus[0].moves[2].reads = vec![Read::Build("armllt".into(), "Sentry (armllt)".into())];
        decode.reads.insert("group_A|build|armmex".into(), 0.05);
        decode.reads.insert("group_A|build|armllt".into(), 0.05);
        assert_eq!(fuse(&mut menus, &decode, &nobody, "armmex", &mut || 0.5), (2, 0), "the turret fused and one detachment marked; the extractor move not");
        assert!(!menus[0].moves[1].fused && menus[0].moves[2].fused);
        assert_eq!(builder_reads_words(&decode, "group_A", "armmex").as_deref(), Some("extractors 0.05 (how the packet reads for it; this one shuts no move)"));
        decode.reads.insert("group_A|place|spot_62".into(), 0.03);
        // An actor that appeared after the packet landed is not read and not fused: the packet's silence about it
        // is no reading.
        // A party of aircraft: its whole-group attack carries the aircraft reading and the mark, and a detachment
        // after it is forbidden by either of its readings (the higher one).
        let mut menus = vec![group()];
        menus[0].moves[5].forbiddable = true;
        menus[0].moves[5].reads = vec![Read::Air];
        menus[0].moves[3].reads = vec![Read::Detach(1), Read::Air];
        let asks = needed(&menus, &decode, &nobody, &what);
        assert_eq!(asks.keys().cloned().collect::<Vec<_>>(), ["group_A|air"]);
        let Question::Noul { instructions, .. } = &asks["group_A|air"] else { panic!("a noul") };
        assert!(instructions.as_str().unwrap().contains("forbid group_A (3 Blitz) to attack, chase or hunt his aircraft"));
        decode.reads.insert("group_A|air".into(), 0.9);
        fuse(&mut menus, &decode, &nobody, "armmex", &mut || 0.5);
        assert_eq!((menus[0].moves[5].marked, menus[0].moves[3].marked), (Some(true), Some(true)), "the detachment of one reads 0.05 and the aircraft 0.9: forbidden");
        let newer: BTreeSet<String> = ["group_A".to_string()].into();
        let mut menus = vec![group()];
        assert!(needed(&menus, &Decode::default(), &newer, &what).is_empty());
        assert_eq!(fuse(&mut menus, &decode, &newer, "armmex", &mut || 0.5), (0, 0));
        assert!(menus[0].moves.iter().all(|m| !m.fused && m.marked.is_none()));
    }
}
