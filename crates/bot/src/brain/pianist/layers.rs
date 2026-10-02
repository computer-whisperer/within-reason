//! The layers (`docs/design/2026-10-01-hands-rebuild.md` §8): the base version asks every move of every asked actor
//! and every party at every gate, and a layer is a saving with a switch. A layer changes which questions are sent or
//! when, never the menu, the words or what a move does; and it audits itself: for a small share of what it skips
//! it asks anyway, in the same request, and the log sets what it assumed beside what came back.
//!
//! `WITHIN_REASON_HANDS_LAYERS` (the arena's `--hands-layers`) names the layers that are on, comma-separated;
//! unset, `news` is on, and `none` switches every layer off. Built so far: `news`. Naming a layer that is not
//! built stops the bot at its start rather than running a game without it.

use std::collections::{BTreeMap, BTreeSet};

use super::super::FRAMES_PER_SECOND;
use super::menu::{Kind, Menu};

/// An actor with a course is asked again this long after its last ask even when nothing has happened to it.
pub(super) const RE_ASK: i32 = 20 * FRAMES_PER_SECOND;
/// The share of what a layer skips that is asked anyway, for its audit.
const AUDIT_SHARE: f64 = 0.02;
/// The layers of the design note, in its order; the first `BUILT` of them exist.
const KNOWN: [&str; 8] = ["news", "same", "fuse", "openers", "split", "tick", "one", "local"];
const BUILT: usize = 1;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Layers {
    /// Ask on change, actor by actor (the user's law, 2026-09-30): an actor with a course is asked when it has
    /// news or after `RE_ASK`; an actor with no course is asked at every gate.
    pub news: bool,
}

impl Layers {
    pub(super) fn parse(value: Option<&str>) -> Result<Layers, String> {
        let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else { return Ok(Layers { news: true }) };
        let mut layers = Layers { news: false };
        for name in value.split(',').map(str::trim).filter(|n| !n.is_empty() && *n != "none") {
            match KNOWN.iter().position(|k| *k == name) {
                Some(0) => layers.news = true,
                Some(k) if k >= BUILT => return Err(format!("the hands' layer `{name}` is not built yet (built: {})", KNOWN[..BUILT].join(", "))),
                _ => return Err(format!("`{name}` is not a layer of the hands (the layers: {})", KNOWN.join(", "))),
            }
        }
        Ok(layers)
    }

    pub(super) fn from_env() -> Result<Layers, String> {
        Layers::parse(std::env::var("WITHIN_REASON_HANDS_LAYERS").ok().as_deref())
    }

    /// The layers that are on, for the log's header.
    pub(super) fn names(&self) -> Vec<&'static str> {
        [(self.news, "news")].into_iter().filter(|(on, _)| *on).map(|(_, name)| name).collect()
    }
}

/// An actor at its last ask: its course by kind, the party in its entry, and the frame.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Asked {
    pub course: String,
    pub party: Option<String>,
    pub frame: i32,
}

/// What has changed since the gate last went, beyond each actor's own course.
pub(super) struct News<'a> {
    pub frame: i32,
    /// The events since the last gate (`schedule.rs`); one that starts with an actor's name is that actor's.
    pub events: &'a BTreeSet<String>,
    /// The packet or a list changed.
    pub orders: bool,
    /// The store crossed empty or full, energy began or stopped stalling, or the store came to cover an idle
    /// factory's cheapest unit.
    pub store: bool,
}

/// An actor's course as the news rule sees it: its kind, and its idleness. The kind, not the move: a constructor
/// going from one spot to the next is on the same course (onepass-smoke-2: 309 asking seconds of 880, most of them
/// a constructor's course changing spot).
pub(super) fn course(menu: &Menu) -> String {
    menu.course.to_string()
}

/// The `news` layer (H-HANDS-LAYERS): closes every actor that has a course and no news, so that neither its
/// `change` nor its own moves are asked and the pick does not change its course by them. Its moves aimed at a party
/// are not the layer's to skip: a party is asked about at every gate, and so are its answers. News for an actor: its course changed or ended, an event
/// names it (a hit, a lost soldier, a sighting, a place reached), the packet or a list changed, a party came into
/// its entry or left it, the store crossed empty or full (a builder or a factory), or `RE_ASK` has passed since it
/// was last asked. Of the actors it closes, a share of `AUDIT_SHARE` are asked anyway (`draw` gives each a number in
/// 0 to 1) and marked for the audit. player-29-hard (`docs/studies/2026-09-30-jev-load.md`): without the rule
/// every option of every actor was re-asked 58-60 times a minute, and the builders' 258,000 option questions bought
/// 194 moves. Returns the questions the closed actors would have been asked, and how many were audited.
pub(super) fn news(menus: &mut [Menu], asked: &BTreeMap<String, Asked>, news: &News, draw: &mut dyn FnMut() -> f64) -> (usize, usize) {
    let (mut skipped, mut audited) = (0, 0);
    for menu in menus.iter_mut() {
        if menu.idle || news.orders {
            continue;
        }
        let Some(before) = asked.get(&menu.name) else { continue };
        let named = news.events.iter().any(|e| e.strip_prefix(menu.name.as_str()).is_some_and(|rest| rest.starts_with(' ')));
        let outside = match menu.kind {
            Kind::Group(_) => false,
            _ => news.store,
        };
        if before.course == course(menu) && before.party == menu.near_party && news.frame - before.frame < RE_ASK && !named && !outside {
            menu.quiet = true;
            if draw() < AUDIT_SHARE {
                menu.audit = true;
                audited += 1;
            } else {
                // The change question and one per move of its own.
                skipped += 1 + menu.moves.iter().skip(1).filter(|m| m.party.is_none()).count();
            }
        }
    }
    (skipped, audited)
}

/// What the picture looks like to the gate: the parties, and every actor's course by kind. A change asks; the same
/// picture does not (the gate then goes on events and on the actors' own re-asks, `mod.rs`).
pub(super) fn signature(menus: &[Menu], parties: &[super::Party]) -> String {
    let parties = parties.iter().map(|p| format!("{}:{}{}", p.name, p.ids.len(), if p.harming.is_some() { "!" } else { "" }));
    let actors = menus.iter().map(|m| format!("{}:{}{}", m.name, course(m), m.aimed_at.as_ref().map_or(String::new(), |p| format!(">{p}"))));
    parties.chain(actors).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::super::menu::tests::{menu, mv};
    use super::super::menu::Order;
    use super::*;
    use bot_protocol::UnitId;

    #[test]
    fn the_layers_are_named_and_an_unbuilt_one_stops_the_start() {
        assert_eq!(Layers::parse(None), Ok(Layers { news: true }));
        assert_eq!(Layers::parse(Some("news")), Ok(Layers { news: true }));
        assert_eq!(Layers::parse(Some("none")), Ok(Layers { news: false }));
        assert!(Layers::parse(Some("news,same")).unwrap_err().contains("not built yet"));
        assert!(Layers::parse(Some("gnus")).unwrap_err().contains("not a layer"));
        assert_eq!(Layers { news: true }.names(), ["news"]);
    }

    /// Ask on change, actor by actor: an actor with a course is closed until its own course changes, an event
    /// names it, the orders change, a party comes into its entry or leaves it, its kind's outside news comes (the
    /// store, for a builder) or the re-ask is due; an idle actor is never closed. A closed actor drawn for the
    /// audit is asked and marked.
    #[test]
    fn an_actor_with_a_course_is_asked_again_only_on_its_own_news() {
        let builder = || menu("constructor_3", Kind::Builder(UnitId(3)), false, vec![mv("go_spot_4", Order::Go("spot_4".into()), None), mv("send_1_party_1", Order::Send(Vec::new(), "party_1".into()), Some("party_1"))]);
        let group = || menu("group_A", Kind::Group("A".into()), false, vec![mv("go_spot_4", Order::Go("spot_4".into()), None)]);
        let none = BTreeSet::new();
        let calm = |frame: i32| News { frame, events: &none, orders: false, store: false };
        let closed = |menu: Menu, asked: &BTreeMap<String, Asked>, news_: &News| {
            let mut menus = vec![menu];
            let counts = news(&mut menus, asked, news_, &mut || 0.5);
            (menus[0].quiet, counts)
        };
        let at = |course: &str, party: Option<&str>| Asked { course: course.to_string(), party: party.map(str::to_string), frame: 300 };
        let asked: BTreeMap<String, Asked> = [("constructor_3".to_string(), at("go", None)), ("group_A".to_string(), at("go", None))].into();
        assert!(!closed(builder(), &BTreeMap::new(), &calm(330)).0, "never asked: asked");
        assert_eq!(closed(builder(), &asked, &calm(330)), (true, (2, 0)), "the same course a second later: closed; its change and its own move are not sent, its answer to the party still is");
        assert!(!closed(builder(), &asked, &calm(300 + RE_ASK)).0, "the re-ask is due");
        assert!(!closed(builder(), &[("constructor_3".to_string(), at("help", None))].into(), &calm(330)).0, "its course changed");
        let mut idle = builder();
        idle.idle = true;
        assert!(!closed(idle, &asked, &calm(330)).0, "an idle actor is asked at every gate");
        let hit: BTreeSet<String> = ["constructor_3 hit".to_string()].into();
        assert!(!closed(builder(), &asked, &News { events: &hit, ..calm(330) }).0, "an event names it");
        let other: BTreeSet<String> = ["constructor_30 hit".to_string(), "group_A hit".to_string()].into();
        assert!(closed(builder(), &asked, &News { events: &other, ..calm(330) }).0, "another actor's event");
        assert!(!closed(builder(), &asked, &News { orders: true, ..calm(330) }).0, "the orders changed");
        assert!(!closed(builder(), &asked, &News { store: true, ..calm(330) }).0 && closed(group(), &asked, &News { store: true, ..calm(330) }).0, "the store opens a builder, not a group");
        let mut near = group();
        near.near_party = Some("party_7".into());
        assert!(!closed(near, &asked, &calm(330)).0, "a party came into its entry");
        let mut same = group();
        same.near_party = Some("party_7".into());
        assert!(closed(same, &[("group_A".to_string(), at("go", Some("party_7")))].into(), &calm(330)).0, "the same party in its entry is no news");
        // The audit: a closed actor drawn under the share is asked anyway and marked.
        let mut menus = vec![builder()];
        assert_eq!(news(&mut menus, &asked, &calm(330), &mut || 0.001), (0, 1));
        assert!(menus[0].quiet && menus[0].audit && menus[0].asks_own() && !menus[0].open());
    }
}
