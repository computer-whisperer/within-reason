//! The decode and its fuses (`docs/design/2026-10-01-hands-rebuild.md` §8, the `fuse` layer; the user, 2026-10-01:
//! "a decode of the opus's existing orders (not a second tool call) to fuse off jev questions that we can answer
//! once for the entire span"). Once a packet, and again for whatever is new under it (an actor, a place, a type),
//! one lean request asks Jev about the player's instructions alone: whether they name a place for an actor, have a
//! builder build a type, tell a group to join or follow another, forbid a group a detachment of a size. The fuse
//! rule: a fuse answers only a question about the instructions alone, never one about the picture, and only when
//! the decode's answer is clear. A move whose reading is under `OFF` is fused off until the next packet (it is not
//! asked and cannot be picked); a detachment's forbidden mark at `ON` or over is on, and under `OFF` off, without
//! the second's own question; everything between is asked each second as in the base. No tool call and no new
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
/// The share of what the layer fuses that is asked anyway, for its audit.
const AUDIT_SHARE: f64 = 0.02;

/// What the decode has read of the packet in force: each reading by its key (`<actor>|place|<place>`,
/// `<actor>|build|<type>`, `<actor>|join|<group>`, `<actor>|follow|<ward>`, `<actor>|detach|<n>`,
/// `<group>|back`).
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
        Read::Follow(ward) => format!("{actor}|follow|{ward}"),
        Read::Detach(n) => format!("{actor}|detach|{n}"),
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
        Read::Follow(ward) => format!("Read the player's `instructions` alone. Do they tell {actor} ({what}) to follow, escort or stay beside {ward}, now or under some condition?"),
        Read::Detach(n) => format!("Read the player's `instructions` alone. Do they forbid {actor} ({what}) to send a detachment of {n} of its soldiers after an enemy party?"),
    };
    Question::noul(json!(text))
}

/// The question whether the instructions give a group a place to step back to, for the player's report.
fn back_question(actor: &str, what: &str) -> Question {
    Question::noul(json!(format!("Read the player's `instructions` alone. Do they say where {actor} ({what}) goes when it should not fight: a place it walks to, steps back to or falls back to?")))
}

/// The readings the menus need that the decode does not hold yet: one question each. `what` gives an actor's
/// words from the picture ("a handful: 3 Blitz (armflash)"). `newer` are the actors that appeared after the packet
/// landed and that it does not name: nothing is read for them (`fuse`).
pub(super) fn needed(menus: &[Menu], decode: &Decode, newer: &BTreeSet<String>, what: &dyn Fn(&str) -> String) -> BTreeMap<String, Question> {
    let mut out = BTreeMap::new();
    for menu in menus.iter().filter(|m| !newer.contains(&m.name)) {
        let words = what(&menu.name);
        for read in menu.moves.iter().flat_map(|m| &m.reads) {
            let k = key(&menu.name, read);
            if !decode.reads.contains_key(&k) && !out.contains_key(&k) {
                out.insert(k, question(&menu.name, &words, read));
            }
        }
        if matches!(menu.kind, Kind::Group(_)) {
            let k = format!("{}|back", menu.name);
            if !decode.reads.contains_key(&k) {
                out.insert(k, back_question(&menu.name, &words));
            }
        }
    }
    out
}

/// Puts the decode's clear readings on the menus: a move with a reading under `OFF` is fused off; a detachment's
/// mark is set where its reading is clear. Of what it fuses or marks, a share of `AUDIT_SHARE` is asked anyway.
/// An actor that appeared after the packet in force landed and that the packet does not name (`newer`) is not
/// fused: the packet could not have spoken of it, so its silence is no reading (player-35, 28:32-28:55: a detachment of eight was born with every
/// walk and its join back fused off, and six died while the audit rated those moves 0.55 to 0.68; 73 of the game's
/// 91 detachments were born with the join back fused). Returns the questions not sent and the moves audited.
pub(super) fn fuse(menus: &mut [Menu], decode: &Decode, newer: &BTreeSet<String>, draw: &mut dyn FnMut() -> f64) -> (usize, usize) {
    let (mut skipped, mut audited) = (0, 0);
    for menu in menus.iter_mut().filter(|m| !newer.contains(&m.name)) {
        let name = menu.name.clone();
        for m in menu.moves.iter_mut().skip(1) {
            let reading = |read: &Read| decode.reads.get(&key(&name, read)).copied();
            if m.detachment {
                // The mark, never a fuse: the detachment itself stays a question.
                let Some(p) = m.reads.iter().find_map(|r| reading(r)) else { continue };
                m.marked = if p >= ON { Some(true) } else if p < OFF { Some(false) } else { None };
                if m.marked.is_some() {
                    if draw() < AUDIT_SHARE {
                        m.audit = true;
                        audited += 1;
                    } else {
                        skipped += 1;
                    }
                }
                continue;
            }
            if m.reads.iter().filter_map(|r| reading(r)).any(|p| p < FUSE) {
                m.fused = true;
                if draw() < AUDIT_SHARE {
                    m.audit = true;
                    audited += 1;
                } else {
                    skipped += 1;
                }
            }
        }
    }
    (skipped, audited)
}

/// What the decode reads for a group, for the player's report: the places at 0.5 or over, and whether the
/// instructions say where it steps back to ("spot_27, spot_18; NO place to step back to").
pub(super) fn reads_words(decode: &Decode, actor: &str) -> Option<String> {
    let prefix = format!("{actor}|place|");
    let places: Vec<&str> = decode.reads.iter().filter(|(k, p)| k.starts_with(&prefix) && **p >= 0.5).map(|(k, _)| &k[prefix.len()..]).collect();
    let back = decode.reads.get(&format!("{actor}|back")).copied();
    if places.is_empty() && back.is_none() {
        return None;
    }
    let places = if places.is_empty() { "no place".to_string() } else { places.join(", ") };
    Some(match back {
        Some(p) if p < 0.5 => format!("{places}; NO place to step back to"),
        _ => places,
    })
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
        assert_eq!(fuse(&mut menus, &decode, &nobody, &mut || 0.5), (3, 0));
        let m = &menus[0].moves;
        assert!(!m[1].fused && m[2].fused && !m[5].fused);
        assert_eq!((m[3].marked, m[4].marked), (Some(false), Some(true)));
        assert_eq!(reads_words(&decode, "group_A").as_deref(), Some("spot_30"));
        // The middle is nobody's to fuse; a missing step-back place is said.
        decode.reads.insert("group_A|place|spot_62".into(), 0.4);
        decode.reads.insert("group_A|detach|2".into(), 0.5);
        decode.reads.insert("group_A|back".into(), 0.1);
        let mut menus = vec![group()];
        assert_eq!(fuse(&mut menus, &decode, &nobody, &mut || 0.5), (1, 0));
        assert!(!menus[0].moves[2].fused && menus[0].moves[4].marked.is_none());
        assert_eq!(reads_words(&decode, "group_A").as_deref(), Some("spot_30; NO place to step back to"));
        // The audit: a fused move drawn under the share is asked anyway, and still not played.
        decode.reads.insert("group_A|place|spot_62".into(), 0.03);
        let mut menus = vec![group()];
        assert_eq!(fuse(&mut menus, &decode, &nobody, &mut || 0.001), (0, 2));
        assert!(menus[0].moves[2].fused && menus[0].moves[2].audit && menus[0].moves[2].asked());
        // An actor that appeared after the packet landed is not read and not fused: the packet's silence about it
        // is no reading.
        let newer: BTreeSet<String> = ["group_A".to_string()].into();
        let mut menus = vec![group()];
        assert!(needed(&menus, &Decode::default(), &newer, &what).is_empty());
        assert_eq!(fuse(&mut menus, &decode, &newer, &mut || 0.5), (0, 0));
        assert!(menus[0].moves.iter().all(|m| !m.fused && m.marked.is_none()));
    }
}
