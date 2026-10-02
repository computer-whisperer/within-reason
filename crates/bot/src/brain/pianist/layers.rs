//! The layers (`docs/design/2026-10-01-hands-rebuild.md` §8): the base version asks every move of every asked actor
//! and every party at every gate, and a layer is a saving with a switch. A layer changes which questions are sent or
//! when, never the menu, the words or what a move does; and it audits itself: for a small share of what it skips
//! it asks anyway, in the same request, and the log sets what it assumed beside what came back.
//!
//! `WITHIN_REASON_HANDS_LAYERS` (the arena's `--hands-layers`) names the layers that are on, comma-separated;
//! unset, `news` is on, and `none` switches every layer off. Built so far: `news`, `same`, `fuse`, `openers`, `tick`. Naming a layer that is
//! not built stops the bot at its start rather than running a game without it.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use jev::{Answer, Question};

use super::super::FRAMES_PER_SECOND;
use super::menu::{Kind, Menu};

/// An actor with a course is asked again this long after its last ask even when nothing has happened to it.
pub(super) const RE_ASK: i32 = 20 * FRAMES_PER_SECOND;
/// The share of what a layer skips that is asked anyway, for its audit.
const AUDIT_SHARE: f64 = 0.02;
/// The layers of the design note, in its order, and those that exist.
const KNOWN: [&str; 8] = ["news", "same", "fuse", "openers", "split", "tick", "one", "local"];
const BUILT: [&str; 5] = ["news", "same", "fuse", "openers", "tick"];
/// The `tick` layer: a gate that no event calls for waits until this long after the last one.
pub(super) const TICK: i32 = 2 * FRAMES_PER_SECOND;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Layers {
    /// Ask on change, actor by actor (the user's law, 2026-09-30): an actor with a course is asked when it has
    /// news or after `RE_ASK`; an actor with no course is asked at every gate.
    pub news: bool,
    /// A noul whose words are those of its last ask, under `RE_ASK` old, is not sent again: its last answer stands.
    pub same: bool,
    /// The packet decoded once (`decode.rs`): a move the instructions clearly do not give an actor is fused off,
    /// and a detachment's forbidden mark the decode reads clearly is set without the second's own question.
    pub fuse: bool,
    /// An actor's own moves are not asked while its `change` said no at the last gate, nor a party's answers
    /// while its `answer` said no; the opener is asked every time and the moves follow the second after it opens.
    pub openers: bool,
    /// A gate that no event calls for waits until `TICK` after the last one; an event still asks at once. It only
    /// delays, so it has no audit.
    pub tick: bool,
}

impl Layers {
    pub(super) fn parse(value: Option<&str>) -> Result<Layers, String> {
        let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else { return Ok(Layers { news: true, same: false, fuse: false, openers: false, tick: false }) };
        let mut layers = Layers { news: false, same: false, fuse: false, openers: false, tick: false };
        for name in value.split(',').map(str::trim).filter(|n| !n.is_empty() && *n != "none") {
            match name {
                "news" => layers.news = true,
                "same" => layers.same = true,
                "fuse" => layers.fuse = true,
                "openers" => layers.openers = true,
                "tick" => layers.tick = true,
                _ if KNOWN.contains(&name) => return Err(format!("the hands' layer `{name}` is not built yet (built: {})", BUILT.join(", "))),
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
        [(self.news, "news"), (self.same, "same"), (self.fuse, "fuse"), (self.openers, "openers"), (self.tick, "tick")].into_iter().filter(|(on, _)| *on).map(|(_, name)| name).collect()
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

/// The `openers` layer (H-HANDS-LAYERS): a move's opener is its actor's `change` (a move of its own) or the
/// `answer` of the party it is aimed at; a move aimed at a party is opened by either. While every opener of a move
/// said no at its last ask (under `RE_ASK` ago), the move is held: not asked this gate. The openers themselves are
/// asked every time, and when one comes back at the flag or over the gate goes again the next second with its moves
/// (`mod.rs`: an "opened" event). An idle actor has no `change` and its own moves are never held. `opened` holds
/// each opener's last answer and frame. A share of `AUDIT_SHARE` of what is held is asked anyway. Returns the
/// questions not sent and the moves audited.
pub(super) fn openers(menus: &mut [Menu], opened: &HashMap<String, (f64, i32)>, frame: i32, flag: f64, draw: &mut dyn FnMut() -> f64) -> (usize, usize) {
    let said_no = |name: &str| opened.get(name).filter(|(_, f)| frame - f < RE_ASK).is_some_and(|(p, _)| *p < flag);
    let (mut skipped, mut audited) = (0, 0);
    for menu in menus.iter_mut() {
        // The actor's own opener: no when its `change` said no, and no when the `news` layer has closed it.
        let own_no = !menu.idle && (menu.quiet || said_no(&menu.name));
        let own_asked = menu.asks_own();
        for m in menu.moves.iter_mut().skip(1).filter(|m| !m.fused) {
            let hold = match &m.party {
                Some(party) => own_no && said_no(party),
                None => own_asked && !menu.idle && said_no(&menu.name),
            };
            if hold {
                m.held = true;
                if draw() < AUDIT_SHARE {
                    m.audit = true;
                    audited += 1;
                } else {
                    skipped += if m.detachment && m.marked.is_none() { 2 } else { 1 };
                }
            }
        }
    }
    (skipped, audited)
}

/// A noul as it was last asked: its words and the counts it was asked over (hashed together), when, and what
/// came back.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Said {
    pub words: u64,
    pub frame: i32,
    pub noul: f64,
}

fn words_of(question: &Question, counts: u64) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    match question {
        Question::Noul { instructions, .. } | Question::Choice { instructions, .. } | Question::Score { instructions, .. } => instructions.to_string().hash(&mut hasher),
    }
    counts.hash(&mut hasher);
    hasher.finish()
}

/// What a packet's tables key on, as the picture says it: how many extractors, generators, labs, constructors,
/// radars and turrets stand, the army's size as its word, and the economy's lines without their numbers ("low",
/// "STALLING", "in balance"). An answer of the `same` layer stands only while these are what they were when it
/// was given: rebuild-smoke-7, 0:22, "a solar" rated 0.28 with one extractor standing was played with two, against
/// a packet that says "with 2 extractors and 0 solar collectors: a solar collector", and the game was lost
/// without a soldier made (K-hands-the-same-layer-keeps-an-answer-the-picture-has-outdated).
pub(super) fn counts(state: &serde_json::Value) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    if let Some(ours) = state["ours"].as_object() {
        for (key, value) in ours.iter().filter(|(key, _)| *key != "wrecks") {
            key.hash(&mut hasher);
            match value.as_str() {
                // "a group: 11 worth 894 metal (...)": the size's word, not the count.
                Some(text) => text.split(':').next().unwrap_or(text).hash(&mut hasher),
                None => value.to_string().hash(&mut hasher),
            }
        }
    }
    for line in ["metal", "energy"] {
        let words: String = state["economy"][line].as_str().unwrap_or_default().chars().filter(|c| !c.is_ascii_digit()).collect();
        words.hash(&mut hasher);
    }
    hasher.finish()
}

/// What the `same` layer made of a gate's questions: those to send, the answers it lets stand for the rest, and
/// the audited ones (sent anyway) with the answer it would have let stand.
pub(super) struct Same {
    pub send: Vec<(String, Question)>,
    pub stand: BTreeMap<String, Answer>,
    pub audited: BTreeMap<String, f64>,
}

/// The `same` layer (H-HANDS-LAYERS): a noul whose words are exactly those of its last ask, asked under `RE_ASK`
/// ago over the same `counts`, is not sent; its last answer stands for it. The rest of the picture beside the
/// question has moved on, which is the layer's bet: asked again as recorded, an unchanged question's answer
/// crosses 0.5 in 0.1 to 0.7% of asks (`docs/studies/2026-10-01-jev-token-budget.md`). A share of `AUDIT_SHARE` of
/// what it would skip is sent anyway and set beside the standing answer; the standing answer is the one played.
pub(super) fn same(questions: Vec<(String, Question)>, said: &HashMap<String, Said>, counts: u64, frame: i32, draw: &mut dyn FnMut() -> f64) -> Same {
    let mut out = Same { send: Vec::new(), stand: BTreeMap::new(), audited: BTreeMap::new() };
    for (id, question) in questions {
        match said.get(&id).filter(|s| frame - s.frame < RE_ASK && s.words == words_of(&question, counts)) {
            Some(before) => {
                out.stand.insert(id.clone(), Answer::Noul { noul: before.noul });
                if draw() < AUDIT_SHARE {
                    out.audited.insert(id.clone(), before.noul);
                    out.send.push((id, question));
                }
            }
            None => out.send.push((id, question)),
        }
    }
    out
}

/// Remembers a gate's fresh nouls for the `same` layer, and forgets what is too old to stand.
pub(super) fn remember(said: &mut HashMap<String, Said>, questions: &BTreeMap<String, Question>, answers: &BTreeMap<String, Answer>, skip: &BTreeMap<String, f64>, counts: u64, frame: i32) {
    said.retain(|_, s| frame - s.frame < RE_ASK);
    for (id, question) in questions {
        if let Some(Answer::Noul { noul }) = answers.get(id)
            && !skip.contains_key(id)
        {
            said.insert(id.clone(), Said { words: words_of(question, counts), frame, noul: *noul });
        }
    }
}

/// What the picture looks like to the gate: the parties, and every actor's course by kind, with whether an actor
/// aimed at a party has it in reach. A change asks; the same picture does not (the gate then goes on events and on
/// the actors' own re-asks, `mod.rs`).
pub(super) fn signature(menus: &[Menu], parties: &[super::Party]) -> String {
    let parties = parties.iter().map(|p| format!("{}:{}{}", p.name, p.ids.len(), if p.harming.is_some() { "!" } else { "" }));
    let actors = menus.iter().map(|m| format!("{}:{}{}", m.name, course(m), m.aimed_at.as_ref().map_or(String::new(), |p| format!(">{p}{}", if m.against.as_ref().is_some_and(|s| !s.met()) { "~" } else { "" }))));
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
        let none = Layers { news: false, same: false, fuse: false, openers: false, tick: false };
        assert_eq!(Layers::parse(None), Ok(Layers { news: true, ..none.clone() }));
        assert_eq!(Layers::parse(Some("news")), Ok(Layers { news: true, ..none.clone() }));
        assert_eq!(Layers::parse(Some("none")), Ok(none.clone()));
        assert_eq!(Layers::parse(Some("news, same,fuse,openers,tick")), Ok(Layers { news: true, same: true, fuse: true, openers: true, tick: true }));
        assert!(Layers::parse(Some("news,split")).unwrap_err().contains("not built yet"));
        assert!(Layers::parse(Some("gnus")).unwrap_err().contains("not a layer"));
        assert_eq!(Layers { news: true, same: true, ..none }.names(), ["news", "same"]);
    }

    /// The `openers` layer: a move is held while every opener of its said no at the last gate; an idle actor's own
    /// moves never are, and a move aimed at a party is asked when either the party or the actor is open.
    #[test]
    fn a_move_waits_for_its_opener() {
        let group = |idle: bool| menu("group_A", Kind::Group("A".into()), idle, vec![mv("go_spot_4", Order::Go("spot_4".into()), None), mv("attack_party_1", Order::Attack("party_1".into()), Some("party_1"))]);
        let held = |menu: Menu, opened: &[(&str, f64, i32)]| {
            let opened: HashMap<String, (f64, i32)> = opened.iter().map(|(k, p, f)| (k.to_string(), (*p, *f))).collect();
            let mut menus = vec![menu];
            let counts = openers(&mut menus, &opened, 330, 0.5, &mut || 0.5);
            (menus[0].moves[1].held, menus[0].moves[2].held, counts)
        };
        assert_eq!(held(group(false), &[]), (false, false, (0, 0)), "never asked: everything is asked");
        assert_eq!(held(group(false), &[("group_A", 0.2, 300), ("party_1", 0.1, 300)]), (true, true, (2, 0)), "both openers said no");
        assert_eq!(held(group(false), &[("group_A", 0.2, 300), ("party_1", 0.8, 300)]), (true, false, (1, 0)), "the party is open: its answers are asked");
        assert_eq!(held(group(false), &[("group_A", 0.7, 300), ("party_1", 0.1, 300)]), (false, false, (0, 0)), "the actor is open: every move of its is asked");
        assert_eq!(held(group(false), &[("group_A", 0.2, 330 - RE_ASK), ("party_1", 0.1, 300)]), (false, false, (0, 0)), "an answer too old is no answer");
        assert_eq!(held(group(true), &[("group_A", 0.2, 300), ("party_1", 0.1, 300)]), (false, false, (0, 0)), "an idle actor has no opener of its own");
        let mut closed = group(false);
        closed.quiet = true;
        assert_eq!(held(closed, &[("party_1", 0.1, 300)]), (false, true, (1, 0)), "closed by the news layer: its own moves are not asked anyway, and its answer waits for the party");
    }

    /// The `same` layer: a noul asked again in the same words within the re-ask is not sent and its last answer
    /// stands; changed words, an answer too old, or a change in the counts it was asked over (a second extractor
    /// stands, the store's word is another) are sent; an audited one is sent and its standing answer kept.
    #[test]
    fn a_question_in_the_same_words_is_not_sent_again() {
        let q = |text: &str| Question::noul(serde_json::json!(text));
        let mut said: HashMap<String, Said> = HashMap::new();
        let asked: BTreeMap<String, Question> = [("a.go_x".to_string(), q("walks to x (near)")), ("a.go_y".to_string(), q("walks to y (far)")), ("a.old".to_string(), q("old"))].into();
        let answers: BTreeMap<String, Answer> = [("a.go_x".to_string(), Answer::Noul { noul: 0.7 }), ("a.go_y".to_string(), Answer::Noul { noul: 0.2 }), ("a.old".to_string(), Answer::Noul { noul: 0.9 })].into();
        let picture = |extractors: i32, metal: &str| serde_json::json!({ "ours": { "extractors": extractors, "soldiers": "a group: 11 worth 894 metal", "wrecks": [extractors] }, "economy": { "metal": metal, "energy": "25 of 1600 stored (nearly empty)" } });
        let at = counts(&picture(1, "171 of 1650 stored (low)"));
        assert_eq!(at, counts(&picture(1, "188 of 1650 stored (low)")), "the numbers move every second; the words do not");
        remember(&mut said, &asked, &answers, &BTreeMap::new(), at, 300);
        said.get_mut("a.old").unwrap().frame = 300 - RE_ASK;
        let next = vec![("a.go_x".to_string(), q("walks to x (near)")), ("a.go_y".to_string(), q("walks to y (some way off)")), ("a.old".to_string(), q("old")), ("a.new".to_string(), q("new"))];
        let out = same(next.clone(), &said, at, 330, &mut || 0.5);
        assert_eq!(out.send.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(), ["a.go_y", "a.old", "a.new"]);
        assert_eq!(out.stand, [("a.go_x".to_string(), Answer::Noul { noul: 0.7 })].into());
        assert!(out.audited.is_empty());
        // A second extractor stands, or the store's word changed: nothing stands.
        for moved in [counts(&picture(2, "171 of 1650 stored (low)")), counts(&picture(1, "900 of 1650 stored (plenty in store)"))] {
            let fresh = same(next.clone(), &said, moved, 330, &mut || 0.5);
            assert!(fresh.stand.is_empty() && fresh.send.len() == 4);
        }
        let audit = same(next, &said, at, 330, &mut || 0.001);
        assert_eq!(audit.send.len(), 4, "the audited question is sent anyway");
        assert_eq!(audit.audited, [("a.go_x".to_string(), 0.7)].into());
        // What stood is not remembered afresh: it expires by its own ask.
        remember(&mut said, &asked, &answers, &audit.audited, at, 330);
        assert_eq!(said["a.go_x"].frame, 300);
        assert_eq!(said["a.go_y"].frame, 330);
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
