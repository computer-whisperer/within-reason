//! The pass over the menus (`docs/design/2026-10-01-hands-rebuild.md` §4, H-HANDS-ONE-PASS): a gate of nouls asks
//! whether each actor should change course, whether each enemy party needs answering, and whether each move is the
//! move; the best-rated moves are joined into worlds, each with its consequence in words; and the pick is two
//! questions, the best of the changes and then that change against changing nothing. A world gives every actor
//! exactly one move, so nothing is settled between menus.

use std::collections::{BTreeMap, BTreeSet};

use jev::{Answer, Question};
use serde_json::{Value, json};

use super::menu::{Kind, Menu, Order, Site, under};
use super::picture::{Party, place_of};

/// Worlds in one question at most, by default (`WITHIN_REASON_WORLDS` sets it): the API's cap on a Choice's options.
/// Battery J (`../jev_experiments/results/j.md` §2) holds 0.95-1.00 to 255 worlds when each carries its computed
/// consequence.
pub(super) const CAP: usize = 255;
/// Characters of deviation lines in one pick at most: the stage-one Choice cannot be split into batches, and the
/// service refuses a request past its context (player-27, 10:20-13:00: 145-255 worlds with 120-240k characters of
/// lines refused 172 times, HTTP 400 max_tokens_exceeded, no pick for three minutes with 2,200 metal banked). The
/// worlds are cut to the longest prefix that fits: the singles first, then the joint.
pub(super) const LINE_CHARS: usize = 40_000;
/// A noul at or above this opens its actor or its party, and puts its move forward.
pub(super) const FLAG: f64 = 0.5;
/// A detachment whose `forbidden` noul is at or above this says on its line in the pick that the player's
/// instructions forbid it (the user's law, 2026-09-30; player-29-hard, `docs/studies/2026-09-29-pick-framing.md`
/// §8: the packets said the ball "never hunts and never sends detachments" and the pick played 61 hunts by a few of
/// its soldiers; this noul rated the ball's hunts a median 0.84 against the early ordered hunts' 0.53, and with the
/// sentence on the line the forbidden hunts beat world 1 in 10 of 181 picks for 172).
pub(super) const FORBIDDEN: f64 = 0.7;
/// An actor's moves in the worlds at most: its best by the gate.
const DEPTH: usize = 2;
/// An idle actor with no move at the flag still puts its best move to the pick when it is rated this high
/// (onepass-medium-2: the commander idled six minutes on an extractor rated 0.48).
pub(super) const IDLE_BAR: f64 = 0.3;

/// One world: for each menu, the index of its move; 0 is `stay`.
pub(crate) type World = Vec<usize>;

/// What the gate said of an actor's moves: the two it rated best, for world 1's line and the player's report.
pub(super) type Rated = BTreeMap<String, Vec<(String, f64)>>;

/// Who is on a party now, in words: each actor whose course is aimed at it, with where that actor stands against
/// it (`Standing`: an actor told to attack a party it has not reached is not "on it").
fn on_it(party: &Party, menus: &[Menu]) -> String {
    let who: Vec<String> = menus
        .iter()
        .filter(|m| m.aimed_at.as_deref() == Some(party.name.as_str()))
        .map(|m| match &m.against {
            Some(s) if s.met() => format!("{} on it: {}", m.name, s.words()),
            Some(s) if s.young.is_some() => format!("{} is told to attack it: {}", m.name, s.words()),
            Some(s) => format!("{} is told to attack it, but {}", m.name, s.words()),
            None => format!("{} is told to attack it", m.name),
        })
        .collect();
    if who.is_empty() { "nobody moves for it".to_string() } else { who.join("; ") }
}

/// A party as the gate's question and a world's line say it.
fn party_line(party: &Party, places: &[super::Place]) -> String {
    let place = place_of(places, party.at).map_or("in sight".to_string(), |pl| format!("at {pl}"));
    let composition = if party.has_commander { format!("THEIR COMMANDER, whose death wins the game, with {}", party.composition) } else { party.composition.clone() };
    format!("{} ({composition}, {place}{}{})", party.name, under(party), party.harming.as_ref().map_or(String::new(), |(what, _)| format!(", {what}")))
}

/// The gate's nouls: per party whether it needs answering by someone other than what stands; per asked actor
/// whether it should change course (an idle actor has no course to keep) and one per move whether it is the move.
/// A move aimed at a party is asked as an answer to that party; every other move is asked of the actor. A
/// detachment is asked about a second time, as a reading of the instructions (`FORBIDDEN`). With `once` (the diet's
/// `only_used`), a move of the actor's own says the fight's facts its course has just said as "(as said above)".
/// With `builder_words` (the diet's), a builder's own move is asked as its words and "Rather than: {course}." with
/// the framing said once in the rules (`rules.md` `<<builder_words>>`): the question-cuts study §2, a builder's
/// answers hold without the framing and a group's do not. With `places_on` the `places` layer's questions go first.
pub(super) fn gate_questions(menus: &[Menu], parties: &[Party], places: &[super::Place], once: bool, builder_words: bool, places_on: bool) -> Vec<(String, Question)> {
    let mut out = if places_on { place_questions(menus) } else { Vec::new() };
    for party in parties {
        out.push((
            format!("{}.answer", party.name),
            Question::noul(json!(format!(
                "Given `enemy`, `actors`, `player` and the player's `instructions`: does {} need answering this second by someone other than what stands ({})? Yes when it is killing, taking apart or about to kill something of ours that nothing there can stop; no when a turret handles it, it is leaving, or the instructions say to leave such a party.",
                party_line(party, places),
                on_it(party, menus)
            ))),
        ));
    }
    for menu in menus.iter().filter(|m| m.asked()) {
        let standing = &menu.moves[0].words;
        let own = menu.asks_own();
        if own && !menu.idle {
            out.push((
                format!("{}.change", menu.name),
                Question::noul(json!(format!(
                    "Given `actors.{}`, `economy`, `enemy` and the player's `instructions`: should {} do something other than what it does now ({standing})? Yes when the instructions call for a different job now, when what it does is finished or pointless, or when something near it needs answering. No when its course is what the instructions want and nothing has changed.",
                    menu.name, menu.name
                ))),
            ));
        }
        for m in menu.moves.iter().skip(1).filter(|m| (own || m.party.is_some()) && m.asked()) {
            let id = menu.id(m);
            let text = match m.party.as_ref().and_then(|name| parties.iter().find(|p| p.name == *name)) {
                Some(party) => format!("Is this the move to make against {} now, rather than what stands ({})? The move: {}.", party.name, on_it(party, menus), m.words),
                None => {
                    let fight = menu.fight.as_ref().zip(menu.near_party.as_ref()).filter(|(fight, _)| once && standing.contains(fight.as_str()));
                    let words = fight.map_or(m.words.clone(), |(fight, party)| m.words.replacen(fight.as_str(), &format!("{party} (as said above)"), 1));
                    if builder_words && matches!(menu.kind, Kind::Builder(_)) {
                        format!("{words}. Rather than: {standing}.")
                    } else {
                        format!("Given `actors.{}`, `economy`, `ours` and the player's `instructions`: is this what {} should do now, rather than {standing}? The move: {words}.", menu.name, menu.name)
                    }
                }
            };
            out.push((id.clone(), Question::noul(json!(text))));
            // The forbidden mark: asked each second unless the decode has it clearly on or off (the `fuse` layer),
            // and then for its audit alone.
            if m.detachment && (m.marked.is_none() || m.audit) {
                out.push((format!("{id}.forbidden"), Question::noul(json!(format!("Read the player's `instructions` alone: do they forbid this move for the group that would make it? The move: {}.", m.words)))));
            }
        }
    }
    out
}

/// The `places` layer's questions (`layers::places`): one noul per place with a move of an asked actor's own aimed
/// at it, blanked or not, in the words the question-cuts study tested (`run/jev_cut_ab.py`, wording `v0`): what
/// stands there and how far, from the first move's words, and who could go, each with its verb and its distance.
/// The id is `<place>.place`.
pub(super) fn place_questions(menus: &[Menu]) -> Vec<(String, Question)> {
    let mut by: BTreeMap<String, Vec<(&Menu, &super::menu::Move)>> = BTreeMap::new();
    for menu in menus.iter().filter(|m| m.asks_own()) {
        for m in menu.moves.iter().skip(1).filter(|m| !m.fused && !m.held && m.party.is_none()) {
            if let Some(place) = m.place() {
                by.entry(place).or_default().push((menu, m));
            }
        }
    }
    by.into_iter()
        .map(|(place, moves)| {
            let mut facts = "";
            let mut who: Vec<String> = Vec::new();
            for (menu, m) in &moves {
                let inside = parenthetical(&m.words, &place);
                if facts.is_empty() {
                    facts = inside;
                }
                let distance = inside.split(',').next().unwrap_or("").trim();
                let verb = m.words.strip_prefix(&format!("{} ", menu.name)).and_then(|rest| rest.split(&format!(" {place}")).next()).map_or("", |v| v.strip_suffix(" at").or_else(|| v.strip_suffix(" to")).unwrap_or(v));
                let entry = if distance.is_empty() { format!("{} ({verb})", menu.name) } else { format!("{} ({verb}, {distance})", menu.name) };
                if !who.contains(&entry) {
                    who.push(entry);
                }
            }
            let others = if who.len() > 12 { " and others" } else { "" };
            who.truncate(12);
            let text = format!("Given `actors`, `enemy`, `economy`, `ours` and the player's `instructions`: does any of ours have a reason to go to, fight at or build at {place} now ({facts}), rather than carry on with what it does? Those with a move there: {}{others}. No when nothing there needs doing now or the instructions send everyone elsewhere.", who.join("; "));
            (format!("{place}.place"), Question::noul(json!(text)))
        })
        .collect()
}

/// The balanced parenthetical that follows " {place} (" in a move's words: "far, about 50 s of walking, our
/// extractor (beside it: our Sentry (armllt))"; empty when the words have none.
fn parenthetical<'a>(words: &'a str, place: &str) -> &'a str {
    let Some(i) = words.find(&format!(" {place} (")) else { return "" };
    let start = i + place.len() + 3;
    let mut depth = 1;
    for (j, c) in words[start..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return &words[start..start + j];
                }
            }
            _ => {}
        }
    }
    ""
}

/// What the gate rated: every answer by its id, and each open actor's two best moves.
pub(super) fn rated(menus: &[Menu], answers: &BTreeMap<String, Answer>) -> Rated {
    let mut out = Rated::new();
    for menu in menus.iter().filter(|m| m.open()) {
        let mut mine: Vec<(String, f64)> = menu.moves.iter().skip(1).filter(|m| m.playable()).filter_map(|m| if let Some(Answer::Noul { noul }) = answers.get(&menu.id(m)) { Some((m.said.clone(), *noul)) } else { None }).collect();
        mine.sort_by(|a, b| b.1.total_cmp(&a.1));
        mine.truncate(DEPTH);
        if !mine.is_empty() {
            out.insert(menu.name.clone(), mine);
        }
    }
    out
}

/// Per menu, the moves that go to the pick, best first: (rank, move index). A move goes when Jev rated it at the
/// flag or over and its opener is open (the actor's `change`, or the `answer` of the party it is aimed at); an idle
/// actor's best move goes at `IDLE_BAR` when none reaches the flag. A party's `answer` opens a move in place of the
/// actor's `change`, never in place of the move's own rating (until 2026-10-03 the `DEPTH` best-rated moves aimed at
/// a party that needs answering went whatever their rating: K-jev-a-partys-answer-carries-low-rated-moves-and-they-
/// move-armies). Of an actor's moves the `DEPTH` best go on. `flags` receives what the gate said.
fn candidates(menus: &[Menu], answers: &BTreeMap<String, Answer>, flags: &mut BTreeMap<String, f64>) -> Vec<Vec<(f64, usize)>> {
    let noul = |id: &str| match answers.get(id) {
        Some(Answer::Noul { noul }) => Some(*noul),
        _ => None,
    };
    for (id, answer) in answers {
        if let Answer::Noul { noul } = answer
            && (id.ends_with(".answer") || id.ends_with(".change") || id.ends_with(".forbidden") || id.ends_with(".place"))
        {
            flags.insert(id.clone(), *noul);
        }
    }
    let answer_of = |party: &str| noul(&format!("{party}.answer"));
    let mut out = Vec::new();
    for menu in menus {
        let mut mine: Vec<(f64, usize)> = Vec::new();
        // A closed actor's own moves stay out (an audited one's answers are logged, not played); its moves aimed at
        // a party go by the party's answer.
        let change = if !menu.open() { None } else if menu.idle { Some(1.0) } else { noul(&format!("{}.change", menu.name)) };
        let opened = change.is_some_and(|c| c >= FLAG);
        let best = menu.moves.iter().skip(1).filter_map(|m| noul(&menu.id(m))).fold(0.0, f64::max);
        let bar = if menu.idle && best < FLAG { IDLE_BAR } else { FLAG };
        for (ti, m) in menu.moves.iter().enumerate().skip(1) {
            let Some(r) = noul(&menu.id(m)) else { continue };
            flags.insert(menu.id(m), r);
            // A fused or held move asked for its layer's audit is logged, not played.
            if !m.playable() {
                continue;
            }
            let by_actor = (opened && r >= bar).then(|| change.unwrap_or(0.0) * r);
            // The party's answer opens the move in place of the actor's `change`; the move's own rating still has
            // to reach the flag (the user, 2026-10-03, after player-49: 23 of group_O's picks were moves rated
            // under 0.5 carried by a party's answer, the army sent to shell one Blitz 3,900 away at 0.28-0.39).
            let by_party = m.party.as_deref().and_then(answer_of).filter(|a| *a >= FLAG && r >= FLAG).map(|a| a * r);
            if let Some(rank) = [by_actor, by_party].into_iter().flatten().reduce(f64::max) {
                mine.push((rank, ti));
            }
        }
        mine.sort_by(|a, b| b.0.total_cmp(&a.0));
        mine.truncate(DEPTH);
        out.push(mine);
    }
    out
}

/// Two moves that cannot both be made in one world: two builders taking the same metal spot, two groups joining
/// each other.
fn clash(menus: &[Menu], world: &World) -> bool {
    let mut spots: Vec<usize> = Vec::new();
    let mut joins: Vec<(&str, &str)> = Vec::new();
    for (menu, i) in menus.iter().zip(world) {
        match &menu.moves[*i].order {
            Order::Build(_, Site::Spot(spot)) => {
                if spots.contains(spot) {
                    return true;
                }
                spots.push(*spot);
            }
            Order::Join(other) => {
                if let Kind::Group(me) = &menu.kind {
                    if joins.iter().any(|(a, b)| *a == other.as_str() && *b == me.as_str()) {
                        return true;
                    }
                    joins.push((me.as_str(), other.as_str()));
                }
            }
            _ => {}
        }
    }
    false
}

/// The worlds after the gate: world 1 is every actor at `stay`; the single changes are taken round-robin over the
/// actors, each actor's best first, so one actor's many moves never starve another's (onepass-smoke-1); then the
/// joint worlds (the user's one-decider design, restored 2026-09-29): every combination of the best-rated actors'
/// changes, one per actor, over as many actors as fit under `cap`, ordered by the product of their ranks. None when
/// nothing opened. Also returns each actor's best candidate, for world 1's line.
#[cfg(test)]
pub(super) fn compose(menus: &[Menu], answers: &BTreeMap<String, Answer>, flags: &mut BTreeMap<String, f64>, cap: usize) -> Option<(Vec<World>, Vec<Option<usize>>)> {
    let per_menu = candidates(menus, answers, flags);
    if per_menu.iter().all(Vec::is_empty) {
        return None;
    }
    let next: Vec<Option<usize>> = per_menu.iter().map(|mine| mine.first().map(|(_, ti)| *ti)).collect();
    Some((worlds_over(menus, &per_menu, cap), next))
}

/// The worlds over the candidates given: world 1, the single changes round-robin, then the joint worlds.
fn worlds_over(menus: &[Menu], per_menu: &[Vec<(f64, usize)>], cap: usize) -> Vec<World> {
    let base: World = vec![0; menus.len()];
    let cap = cap.max(2);
    let mut out = vec![base.clone()];
    let mut round = 0;
    while out.len() < cap {
        let mut any = false;
        for (mi, mine) in per_menu.iter().enumerate() {
            if let Some((_, ti)) = mine.get(round) {
                any = true;
                let mut w = base.clone();
                w[mi] = *ti;
                out.push(w);
                if out.len() >= cap {
                    break;
                }
            }
        }
        if !any {
            break;
        }
        round += 1;
    }
    // The joint worlds: the actors ranked by their best change, as many as the cap has room for the product of.
    let mut ranked: Vec<usize> = (0..per_menu.len()).filter(|mi| !per_menu[*mi].is_empty()).collect();
    ranked.sort_by(|a, b| per_menu[*b][0].0.total_cmp(&per_menu[*a][0].0));
    let mut active: Vec<usize> = Vec::new();
    for mi in ranked {
        let with: Vec<usize> = active.iter().copied().chain([mi]).collect();
        let product: usize = with.iter().map(|k| 1 + per_menu[*k].len()).product();
        let singles: usize = with.iter().map(|k| per_menu[*k].len()).sum();
        if out.len() + product - 1 - singles > cap {
            break;
        }
        active = with;
    }
    if active.len() >= 2 {
        let mut joint: Vec<(f64, World)> = Vec::new();
        let radix: Vec<usize> = active.iter().map(|mi| 1 + per_menu[*mi].len()).collect();
        let mut digits = vec![0usize; active.len()];
        loop {
            // The next combination, mixed radix; done when it wraps.
            let mut carry = 0;
            while carry < digits.len() {
                digits[carry] += 1;
                if digits[carry] < radix[carry] {
                    break;
                }
                digits[carry] = 0;
                carry += 1;
            }
            if carry == digits.len() {
                break;
            }
            if digits.iter().filter(|d| **d > 0).count() < 2 {
                continue;
            }
            let mut w = base.clone();
            let mut rank = 1.0;
            for (k, d) in digits.iter().enumerate().filter(|(_, d)| **d > 0) {
                let (r, ti) = per_menu[active[k]][d - 1];
                rank *= r;
                w[active[k]] = ti;
            }
            if !clash(menus, &w) {
                joint.push((rank, w));
            }
        }
        joint.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (_, w) in joint {
            if out.len() >= cap {
                break;
            }
            out.push(w);
        }
    }
    out
}

/// A component's worlds in a split pick: world 1 and at most this many changes, each of which is asked against
/// world 1 in the same request.
const SPLIT_CAP: usize = 9;

/// One component of a split pick (the `split` layer, H-HANDS-LAYERS): the actors whose candidate changes touch
/// each other, the worlds over their changes alone, and each world's line, said over the component's own actors
/// and parties.
#[derive(Clone, Debug)]
pub(crate) struct Component {
    pub menus: Vec<usize>,
    pub worlds: Vec<World>,
    pub lines: Vec<String>,
}

/// What a component's lines speak of: its actors, and the parties its courses and its changes are aimed at.
pub(super) struct Scope {
    menus: BTreeSet<usize>,
    parties: BTreeSet<String>,
}

/// The actors with a change on offer, grouped into components: two are of one component when a change of one
/// names the other (a join, a follow, a help), when a change or the course of each is aimed at the same party, or
/// when both would build on the same spot. Components share no actor, group, builder, party or spot, so each can
/// be picked on its own; what they can still share is the metal in the store.
fn components(menus: &[Menu], per_menu: &[Vec<(f64, usize)>]) -> Vec<Vec<usize>> {
    let name_of = |id: bot_protocol::UnitId| menus.iter().find(|m| matches!(&m.kind, Kind::Builder(b) | Kind::Factory(b) if *b == id)).map(|m| m.name.clone());
    let touches: Vec<BTreeSet<String>> = menus
        .iter()
        .zip(per_menu)
        .map(|(menu, mine)| {
            let mut set: BTreeSet<String> = BTreeSet::from([menu.name.clone()]);
            set.extend(menu.aimed_at.clone());
            for (_, ti) in mine {
                let m = &menu.moves[*ti];
                set.extend(m.party.clone());
                match &m.order {
                    Order::Join(other) | Order::Follow(super::groups::Ward::Group(other)) => {
                        set.insert(format!("group_{other}"));
                    }
                    Order::Follow(super::groups::Ward::Builder(id)) | Order::Help(id) => set.extend(name_of(*id)),
                    Order::Build(_, Site::Spot(spot)) => {
                        set.insert(format!("the spot {spot}"));
                    }
                    _ => {}
                }
            }
            set
        })
        .collect();
    let active: Vec<usize> = (0..menus.len()).filter(|mi| !per_menu[*mi].is_empty()).collect();
    let mut of: Vec<usize> = (0..menus.len()).collect();
    fn root(of: &mut [usize], mut i: usize) -> usize {
        while of[i] != i {
            of[i] = of[of[i]];
            i = of[i];
        }
        i
    }
    for (k, a) in active.iter().enumerate() {
        for b in &active[k + 1..] {
            if !touches[*a].is_disjoint(&touches[*b]) {
                let (ra, rb) = (root(&mut of, *a), root(&mut of, *b));
                of[rb] = ra;
            }
        }
    }
    let mut out: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for mi in active {
        let r = root(&mut of, mi);
        out.entry(r).or_default().push(mi);
    }
    out.into_values().collect()
}

/// The split pick's components over what the gate flagged, each with its worlds and their lines.
fn split(menus: &[Menu], per_menu: &[Vec<(f64, usize)>], said: &Lines) -> Vec<Component> {
    components(menus, per_menu)
        .into_iter()
        .map(|members| {
            let masked: Vec<Vec<(f64, usize)>> = per_menu.iter().enumerate().map(|(mi, mine)| if members.contains(&mi) { mine.clone() } else { Vec::new() }).collect();
            let worlds = worlds_over(menus, &masked, SPLIT_CAP);
            let mut parties: BTreeSet<String> = members.iter().filter_map(|mi| menus[*mi].aimed_at.clone()).collect();
            parties.extend(members.iter().flat_map(|mi| per_menu[*mi].iter().filter_map(|(_, ti)| menus[*mi].moves[*ti].party.clone())));
            let scope = Scope { menus: members.iter().copied().collect(), parties };
            let said = Lines { scope: Some(&scope), ..*said };
            let lines = worlds.iter().enumerate().map(|(i, w)| consequence(w, menus, &said, (i > 0).then_some(&worlds[0]))).collect();
            Component { menus: members, worlds, lines }
        })
        .collect()
}

/// The split pick's questions, all for one request: per component a Choice over its changes (when it has more
/// than one) and, for every change, that change against the component's "nothing changes".
pub(super) fn split_questions(components: &[Component]) -> BTreeMap<String, Question> {
    let mut out = BTreeMap::new();
    for (j, c) in components.iter().enumerate() {
        if c.lines.len() > 2 {
            out.insert(format!("worlds.c{j}.pick"), stage_one(&c.lines));
        }
        for k in 1..c.lines.len() {
            out.insert(format!("worlds.c{j}.w{}", k + 1), stage_two(&c.lines, k));
        }
    }
    out
}

/// The split pick read back: per component the candidate (sampled from its Choice, or its one change) and that
/// change's own question against "nothing changes"; the world of every change taken, and a record per component.
pub(super) fn split_pick(answers: &BTreeMap<String, Answer>, components: &[Component], menus: &[Menu], draw: &mut dyn FnMut() -> f64) -> (World, Vec<Value>) {
    let mut world: World = vec![0; menus.len()];
    let mut parts = Vec::new();
    for (j, c) in components.iter().enumerate() {
        let candidate = if c.lines.len() == 2 { Some((1, 1.0)) } else { sample(answers, &format!("worlds.c{j}.pick"), c.lines.len(), draw()) };
        let Some((k, p)) = candidate else { continue };
        let taken = match answers.get(&format!("worlds.c{j}.w{}", k + 1)) {
            Some(Answer::Choice { choice, confidence, .. }) => Some((*choice == format!("w{}", k + 1), *confidence)),
            _ => None,
        };
        if let Some((true, _)) = taken {
            for (mi, ti) in c.worlds[k].iter().enumerate().filter(|(_, ti)| **ti != 0) {
                world[mi] = *ti;
            }
        }
        parts.push(json!({
            "actors": c.menus.iter().map(|mi| menus[*mi].name.as_str()).collect::<Vec<_>>(),
            "changes": c.lines.len() - 1,
            "candidate": k + 1,
            "sampled_p": p,
            "taken": taken.map(|(t, _)| t),
            "confidence": taken.map(|(_, c)| c),
        }));
    }
    (world, parts)
}

/// What a world's line needs beside the menus: the parties, the gate's word on which need answering, each idle
/// actor's best candidate, the store's words, the detachments the gate reads as forbidden.
pub(super) struct Lines<'a> {
    pub parties: &'a [Party],
    pub places: &'a [super::Place],
    pub flags: &'a BTreeMap<String, f64>,
    pub next: &'a [Option<usize>],
    pub store: &'a str,
    pub forbidden: &'a BTreeSet<String>,
    /// A split pick's component: the lines speak of its actors and parties alone. None: of everything.
    pub scope: Option<&'a Scope>,
}

impl Clone for Lines<'_> {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for Lines<'_> {}

/// A world's parts beside its moves.
#[derive(Default)]
struct Parts {
    /// The changes from world 1, in each move's words.
    moves: Vec<String>,
    /// A party with something of ours in reach of it, and whose.
    met: Vec<String>,
    /// A party a move of this world sends an actor at.
    sent: Vec<String>,
    /// A party an actor is told to attack and has not reached, with where that actor stands.
    not_met: Vec<String>,
    /// A party Jev says needs answering that nobody is aimed at.
    unmet: Vec<String>,
    /// The idle actors, each with the move the gate rated best for it.
    idle: Vec<String>,
}

/// A world's moves and what follows from them. A party is met only by an actor with something in reach of it
/// (H-HANDS-STANDING): a course aimed at it from afar is said as that, with the distance, the idleness and whether
/// it can be caught, and a move made at it this second is said as a sending.
fn parts_of(world: &World, menus: &[Menu], lines: &Lines) -> Parts {
    let mut parts = Parts { moves: menus.iter().zip(world).filter(|(_, i)| **i != 0).map(|(menu, i)| menu.moves[*i].words.clone()).collect(), ..Parts::default() };
    for party in lines.parties.iter().filter(|p| lines.scope.is_none_or(|s| s.parties.contains(&p.name))) {
        let aimed = |menu: &&Menu| menu.aimed_at.as_deref() == Some(party.name.as_str());
        let staying: Vec<&Menu> = menus.iter().zip(world).filter(|(_, i)| **i == 0).map(|(menu, _)| menu).filter(aimed).collect();
        let sent: Vec<&str> = menus.iter().zip(world).filter(|(menu, i)| **i != 0 && menu.moves[**i].party.as_deref() == Some(party.name.as_str())).map(|(menu, _)| menu.name.as_str()).collect();
        let (there, away): (Vec<&Menu>, Vec<&Menu>) = staying.into_iter().partition(|m| m.against.as_ref().is_none_or(|s| s.met()));
        let line = party_line(party, lines.places);
        if !there.is_empty() {
            parts.met.push(format!("{line} met by {}", there.iter().map(|m| m.name.as_str()).collect::<Vec<_>>().join(" and ")));
        }
        if !sent.is_empty() {
            parts.sent.push(format!("{line} by {}", sent.join(" and ")));
        }
        if there.is_empty() && !away.is_empty() {
            parts.not_met.push(format!("{line}: {}", away.iter().map(|m| format!("{} is told to attack it{} {}", m.name, if m.against.as_ref().is_some_and(|s| s.young.is_some()) { ":" } else { ", but" }, m.against.as_ref().map_or(String::new(), |s| s.words()))).collect::<Vec<_>>().join("; ")));
        }
        if there.is_empty() && away.is_empty() && sent.is_empty() && lines.flags.get(&format!("{}.answer", party.name)).is_some_and(|a| *a >= FLAG) {
            parts.unmet.push(line);
        }
    }
    parts.idle = menus
        .iter()
        .zip(world)
        .enumerate()
        .filter(|(k, (menu, i))| **i == 0 && menu.idle && lines.scope.is_none_or(|s| s.menus.contains(k)))
        .map(|(k, (menu, _))| match lines.next.get(k).copied().flatten() {
            Some(ti) => format!("{}, with its next move, {}, not ordered", menu.idle_words, menu.moves[ti].said),
            None => menu.idle_words.clone(),
        })
        .collect();
    parts
}

/// A world's line. World 1's says the cost of changing nothing and no more (the courses in force are in `actors`;
/// K-jev-hold-words-carry-the-cost): the parties that need answering and are left to nobody, and each idle actor
/// with the move Jev's own gate rated best for it (K-jev-its-own-best-rated-move-carries-a-route: 24 of 24 next
/// legs picked with the sentence, 10 of 24 with an unordered list). Every other world's says only what it changes
/// from world 1 ("As w1, and ...") and the consequences that differ, so the change is the option's own words. A
/// detachment the gate reads as forbidden says so in a sentence of its own at the line's end.
pub(super) fn consequence(world: &World, menus: &[Menu], lines: &Lines, base: Option<&World>) -> String {
    let mut of = parts_of(world, menus, lines);
    if let Some(b) = base {
        let theirs = parts_of(b, menus, lines);
        let added = |mine: &mut Vec<String>, theirs: &[String]| mine.retain(|m| !theirs.contains(m));
        added(&mut of.met, &theirs.met);
        added(&mut of.sent, &theirs.sent);
        added(&mut of.not_met, &theirs.not_met);
        added(&mut of.unmet, &theirs.unmet);
        added(&mut of.idle, &theirs.idle);
    }
    let Parts { moves, met, sent, not_met, unmet, idle } = of;
    let mut parts: Vec<String> = Vec::new();
    parts.push(match base {
        None => "Nothing changes: every actor keeps the course its entry under `actors` describes".to_string(),
        Some(_) => format!("As w1, and: {}", moves.join("; ")),
    });
    if !met.is_empty() {
        parts.push(format!("Met: {}", met.join("; ")));
    }
    if !sent.is_empty() {
        parts.push(format!("Sent at: {}", sent.join("; ")));
    }
    if !not_met.is_empty() {
        parts.push(format!("Not met: {}", not_met.join("; ")));
    }
    if !unmet.is_empty() {
        parts.push(format!("Left to nobody: {}", unmet.join("; ")));
    }
    if !idle.is_empty() {
        // The cost on the option's own words (onepass-smoke-3: "Idle: plant" beside a full store lost to the Blitz
        // rated 0.68 at 0.65 against 0.35).
        let makers = menus.iter().zip(world).enumerate().any(|(k, (menu, i))| *i == 0 && menu.idle && !matches!(menu.kind, Kind::Group(_)) && lines.scope.is_none_or(|s| s.menus.contains(&k)));
        parts.push(format!("{}{}", idle.join("; "), if makers && !lines.store.is_empty() { format!(", while the metal store reads {}", lines.store) } else { String::new() }));
    }
    if base.is_some() {
        for (menu, i) in menus.iter().zip(world) {
            let m = &menu.moves[*i];
            if *i != 0 && lines.forbidden.contains(&menu.id(m)) {
                parts.push(format!("The player's instructions forbid this detachment for {}", menu.name));
            }
        }
    }
    format!("{}.", parts.join(". "))
}

/// The pick is two questions (the user, 2026-09-29, after the offline replay of player-26: with a median 96
/// worlds a pick, near-duplicate combinations split the mass and "nothing changes" took the plurality 142 of 315
/// picks; two stages built a factory 29 of 92 idle seconds against 9). Stage one: a Choice over the deviations
/// alone, its answer sampled, not taken at the top (the user's ruling: the mass a change holds across the worlds
/// it sits in is its chance of being the candidate). Stage two: a plain Choice between world 1 and the candidate.
const COMMON: &str = "Given `economy`, `ours`, `enemy`, `actors`, `player` and the player's `instructions`, which plan is best this second? Each option is one world: who changes course to do what, with what it costs and gives up, which enemy parties are met and which are left to nobody, who stays idle. An idle factory or builder with metal in the store is a cost, not a course, unless the instructions say to wait. The instructions were written before this picture: where they name a place, a party, a building, a unit or a rule, follow them; where the situation has changed, pick the world they would call for.";

/// Stage one: the deviations alone, keyed by their world numbers.
pub(super) fn stage_one(lines: &[String]) -> Question {
    let instructions = json!(format!("{COMMON} Every option is a change from what stands (w1, not offered here): one or more actors' courses changed, the line saying only those changes and what they alter. Pick the best of the changes; whether to change at all is asked next."));
    Question::Choice { instructions, criteria: lines.iter().enumerate().skip(1).map(|(i, l)| (format!("w{}", i + 1), json!(l))).collect() }
}

/// Stage two: world 1 against the candidate, under its own world number.
pub(super) fn stage_two(lines: &[String], candidate: usize) -> Question {
    let instructions = json!(format!("{COMMON} w1 changes nothing: every actor keeps its course and the idle ones stay idle. The other option is the best change on offer this second, its line saying only what it changes from w1 and what that alters. Pick one."));
    Question::Choice { instructions, criteria: BTreeMap::from([("w1".to_string(), json!(lines[0])), (format!("w{}", candidate + 1), json!(lines[candidate]))]) }
}

/// A world drawn from a Choice's probabilities with a uniform draw in `u` (0 to 1): the index and its probability.
pub(super) fn sample(answers: &BTreeMap<String, Answer>, key: &str, worlds: usize, u: f64) -> Option<(usize, f64)> {
    let Some(Answer::Choice { probabilities, choice, .. }) = answers.get(key) else { return None };
    let index = |k: &str| k.strip_prefix('w').and_then(|n| n.parse::<usize>().ok()).filter(|n| (2..=worlds).contains(n)).map(|n| n - 1);
    let total: f64 = probabilities.iter().filter(|(k, _)| index(k).is_some()).map(|(_, p)| p).sum();
    if total <= 0.0 {
        return index(choice).map(|i| (i, 1.0));
    }
    let mut acc = 0.0;
    let mut last = None;
    for (k, p) in probabilities {
        let Some(i) = index(k) else { continue };
        acc += p / total;
        last = Some((i, *p));
        if u < acc {
            return last;
        }
    }
    last
}

/// A uniform draw in 0 to 1 from the standard library's random hasher seed (no `rand` dependency).
pub(super) fn draw() -> f64 {
    use std::hash::{BuildHasher, Hasher};
    let h = std::collections::hash_map::RandomState::new().build_hasher().finish();
    (h >> 11) as f64 / (1u64 << 53) as f64
}

/// The two calls of the pick, made in place: stage one over the deviations (skipped when there is one), its answer
/// sampled into the candidate, then stage two. Each stage's request and answer come back for the log.
pub(super) struct PickRun {
    pub stage_one: Option<(jev::Request, Result<jev::Response, jev::Error>)>,
    pub candidate: Option<(usize, f64)>,
    pub stage_two: Option<(jev::Request, Result<jev::Response, jev::Error>)>,
}

pub(super) fn run_pick(ask: &dyn Fn(&jev::Request) -> Result<jev::Response, jev::Error>, state: &Value, lines: &[String], u: f64) -> PickRun {
    if lines.len() < 2 {
        return PickRun { stage_one: None, candidate: None, stage_two: None };
    }
    let (stage_one, candidate) = if lines.len() == 2 {
        (None, Some((1, 1.0)))
    } else {
        let request = jev::Request { state: state.clone(), questions: BTreeMap::from([("worlds.pick".to_string(), stage_one(lines))]) };
        let result = ask(&request);
        let candidate = result.as_ref().ok().and_then(|r| sample(&r.answers, "worlds.pick", lines.len(), u));
        (Some((request, result)), candidate)
    };
    let stage_two = candidate.map(|(c, _)| {
        let request = jev::Request { state: state.clone(), questions: BTreeMap::from([("worlds.pick".to_string(), stage_two(lines, c))]) };
        let result = ask(&request);
        (request, result)
    });
    PickRun { stage_one, candidate, stage_two }
}

/// The picked world's index and the pick's confidence.
pub(super) fn pick(answers: &BTreeMap<String, Answer>, worlds: &[World]) -> Option<(usize, f64)> {
    match answers.get("worlds.pick") {
        Some(Answer::Choice { choice, confidence, .. }) => choice.strip_prefix('w').and_then(|n| n.parse::<usize>().ok()).filter(|n| (1..=worlds.len()).contains(n)).map(|n| (n - 1, *confidence)),
        _ => None,
    }
}

/// What follows a gate: what it flagged, each actor's best-rated moves, and, when something opened, the worlds,
/// their lines and the pick's state.
pub(super) struct Followed {
    pub flags: BTreeMap<String, f64>,
    pub rated: Rated,
    pub worlds: Option<(Vec<World>, Vec<String>, Value)>,
    /// The same changes as a split pick's components, with the state their one request goes with.
    pub components: Option<(Vec<Component>, Value)>,
}

/// After the gate: the worlds over what it flagged, their lines, and the pick's state, cut to what the worlds name
/// (`pick_state`). Pure, so the realtime worker runs it the instant the gate answers.
pub(super) fn follow_up(menus: &[Menu], parties: &[Party], places: &[super::Place], answers: &BTreeMap<String, Answer>, store: &str, cap: usize, state: &Value) -> Followed {
    let mut flags: BTreeMap<String, f64> = BTreeMap::new();
    let rated = rated(menus, answers);
    let per_menu = candidates(menus, answers, &mut flags);
    if per_menu.iter().all(Vec::is_empty) {
        return Followed { flags, rated, worlds: None, components: None };
    }
    let next: Vec<Option<usize>> = per_menu.iter().map(|mine| mine.first().map(|(_, ti)| *ti)).collect();
    let mut worlds = worlds_over(menus, &per_menu, cap);
    // The detachments the gate reads as forbidden by the instructions (`FORBIDDEN`): said on their lines.
    // The mark the decode set stands over the second's own reading, which is then asked for the audit alone.
    let decoded = |id: &str| menus.iter().find_map(|menu| menu.moves.iter().find(|m| menu.id(m) == id).and_then(|m| m.marked));
    let mut forbidden: BTreeSet<String> = flags.iter().filter(|(_, p)| **p >= FORBIDDEN).filter_map(|(id, _)| id.strip_suffix(".forbidden").map(str::to_string)).filter(|id| decoded(id).is_none()).collect();
    forbidden.extend(menus.iter().flat_map(|menu| menu.moves.iter().filter(|m| m.marked == Some(true)).map(|m| menu.id(m))));
    let said = Lines { parties, places, flags: &flags, next: &next, store, forbidden: &forbidden, scope: None };
    let mut lines: Vec<String> = worlds.iter().enumerate().map(|(i, w)| consequence(w, menus, &said, (i > 0).then_some(&worlds[0]))).collect();
    let keep = fit(&lines, LINE_CHARS);
    worlds.truncate(keep);
    lines.truncate(keep);
    let components = split(menus, &per_menu, &said);
    let split_state = {
        let all: Vec<World> = components.iter().flat_map(|c| c.worlds.iter().cloned()).collect();
        let text: Vec<String> = components.iter().flat_map(|c| c.lines.iter().cloned()).collect();
        pick_state(state, menus, &all, &text)
    };
    let state = pick_state(state, menus, &worlds, &lines);
    Followed { flags, rated, worlds: Some((worlds, lines, state)), components: Some((components, split_state)) }
}

/// How many worlds fit the line budget: world 1 and at least one deviation, then the longest prefix whose
/// deviation lines total `budget` characters or fewer.
pub(super) fn fit(lines: &[String], budget: usize) -> usize {
    let mut total = 0;
    for (i, line) in lines.iter().enumerate().skip(1) {
        total += line.len();
        if total > budget && i > 1 {
            return i;
        }
    }
    lines.len()
}

/// The pick's state: the gate's, with every actor no world moves cut to one line and every place the worlds'
/// lines, the instructions and the kept actors do not name dropped (onepass-player-8: 25 KB a pick, `places` 10 KB
/// of it); the pick judges the lines, which say what each world does and to whom.
pub(super) fn pick_state(state: &Value, menus: &[Menu], worlds: &[World], lines: &[String]) -> Value {
    let mut state = state.clone();
    let moved: BTreeSet<&str> = worlds.iter().flat_map(|w| menus.iter().zip(w).filter(|(_, i)| **i != 0).map(|(m, _)| m.name.as_str())).collect();
    let mut text = lines.join("\n");
    text.push_str(state["instructions"].as_str().unwrap_or_default());
    if let Some(actors) = state["actors"].as_object_mut() {
        for (name, entry) in actors.iter_mut() {
            if moved.contains(name.as_str()) || super::diet::names(&text, name) {
                text.push_str(&entry.to_string());
            } else {
                super::diet::brief(entry);
            }
        }
    }
    if let Some(places) = state["places"].as_object_mut() {
        places.retain(|name, _| super::diet::names(&text, name));
    }
    state
}

/// The menus for the log.
pub(super) fn log_menus(menus: &[Menu]) -> Value {
    json!(menus
        .iter()
        .map(|menu| {
            json!({
                "name": menu.name,
                "kind": menu.kind.word(),
                "idle": menu.idle,
                "course": menu.course,
                "quiet": menu.quiet,
                "audit": menu.audit,
                "moves": menu.moves.iter().map(|m| if !m.playable() { json!({ "id": menu.id(m), "words": m.words, "party": m.party, "fused": m.fused, "held": m.held, "blanked": m.blanked, "audit": m.audit }) } else { json!({ "id": menu.id(m), "words": m.words, "party": m.party }) }).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::super::groups::Ward;
    use super::super::menu::Standing;
    use super::super::menu::tests::{menu, mv};
    use super::*;
    use bot_protocol::{UnitDefId, UnitId, Vec3};

    fn party(name: &str) -> Party {
        Party { name: name.to_string(), ids: vec![UnitId(1)], at: Vec3::default(), metal: 100.0, composition: "2 Ticks".into(), has_commander: false, harming: None, unarmed: false, turret_metal: 0.0, turret_metal_air: 0.0, turrets: String::new() }
    }

    fn nouls(list: &[(&str, f64)]) -> BTreeMap<String, Answer> {
        list.iter().map(|(id, p)| (id.to_string(), Answer::Noul { noul: *p })).collect()
    }

    fn group(name: &str, idle: bool, moves: Vec<super::super::menu::Move>) -> Menu {
        menu(&format!("group_{name}"), Kind::Group(name.to_string()), idle, moves)
    }

    fn said<'a>(parties: &'a [Party], flags: &'a BTreeMap<String, f64>, next: &'a [Option<usize>], forbidden: &'a BTreeSet<String>) -> Lines<'a> {
        Lines { parties, places: &[], flags, next, store: "340 of 500 stored", forbidden, scope: None }
    }

    /// One menu per actor, one move per actor a world: a group's own way out stands beside its answers to a party
    /// in the same menu, and nothing closes either. (player-33: 0 of 14 attacking groups had a move of their own in
    /// the pick; the party's menu took the group and its own was closed.)
    #[test]
    fn a_groups_way_out_and_its_answer_to_a_party_are_one_menu() {
        let menus = vec![group("A", false, vec![mv("go_spot_30", Order::Go("spot_30".into()), None), mv("attack_party_7", Order::Attack("party_7".into()), Some("party_7")), mv("go_spot_31", Order::Go("spot_31".into()), None)])];
        let parties = vec![party("party_7")];
        let ids: Vec<String> = gate_questions(&menus, &parties, &[], false, false, false).into_iter().map(|(id, _)| id).collect();
        assert_eq!(ids, ["party_7.answer", "group_A.change", "group_A.go_spot_30", "group_A.attack_party_7", "group_A.go_spot_31"]);
        // The group's own change opens its walk; the party's answer opens the attack on it.
        let answers = nouls(&[("party_7.answer", 0.8), ("group_A.change", 0.9), ("group_A.go_spot_30", 0.7), ("group_A.attack_party_7", 0.6), ("group_A.go_spot_31", 0.2)]);
        let mut flags = BTreeMap::new();
        let (worlds, _) = compose(&menus, &answers, &mut flags, CAP).unwrap();
        assert_eq!(worlds, vec![vec![0], vec![1], vec![2]], "world 1, then its two best moves, each a world of its own: one move per actor");
        assert_eq!(flags["group_A.go_spot_30"], 0.7);
    }

    /// Which moves go to the pick: at the flag with the opener open; an idle actor's best at the low bar; and a
    /// party that needs answering puts forward the two best moves aimed at it whatever their rating.
    #[test]
    fn a_move_goes_to_the_pick_by_its_rating_and_its_opener() {
        let menus = vec![
            group("A", false, vec![mv("go_spot_30", Order::Go("spot_30".into()), None), mv("attack_party_7", Order::Attack("party_7".into()), Some("party_7")), mv("send_2_party_7", Order::Send(vec![UnitId(9)], "party_7".into()), Some("party_7"))]),
            group("B", false, vec![mv("attack_party_7", Order::Attack("party_7".into()), Some("party_7"))]),
            menu("constructor_3", Kind::Builder(UnitId(3)), true, vec![mv("build_armmex_spot_4", Order::Build(UnitDefId(1), Site::Spot(4)), None), mv("build_armsolar", Order::Build(UnitDefId(2), Site::Planned), None)]),
        ];
        let rank = |answers: &BTreeMap<String, Answer>| candidates(&menus, answers, &mut BTreeMap::new());
        // The group's change says no and the party needs no answer: nothing of the group goes, whatever its moves rate.
        let quiet = rank(&nouls(&[("party_7.answer", 0.2), ("group_A.change", 0.3), ("group_A.go_spot_30", 0.9), ("group_A.attack_party_7", 0.9), ("constructor_3.build_armmex_spot_4", 0.2), ("constructor_3.build_armsolar", 0.1)]));
        assert!(quiet[0].is_empty() && quiet[2].is_empty(), "{quiet:?}");
        // The party needs answering: that opens the group in place of its own change, for the answers at the flag
        // or over; the ones under it stay home however much the party needs answering.
        let raid = rank(&nouls(&[("party_7.answer", 0.9), ("group_A.change", 0.3), ("group_A.go_spot_30", 0.9), ("group_A.attack_party_7", 0.4), ("group_A.send_2_party_7", 0.55), ("group_B.attack_party_7", 0.1), ("group_B.change", 0.1)]));
        assert_eq!(raid[0].iter().map(|(_, ti)| *ti).collect::<Vec<_>>(), vec![3], "{raid:?}");
        assert!(raid[1].is_empty(), "{raid:?}");
        // An idle builder with nothing at the flag: its moves at the low bar or over go.
        let idle = rank(&nouls(&[("constructor_3.build_armmex_spot_4", 0.35), ("constructor_3.build_armsolar", 0.31)]));
        assert_eq!(idle[2].iter().map(|(_, ti)| *ti).collect::<Vec<_>>(), vec![1, 2]);
        let low = rank(&nouls(&[("constructor_3.build_armmex_spot_4", 0.25), ("constructor_3.build_armsolar", 0.1)]));
        assert!(low[2].is_empty());
        // With one at the flag the bar is the flag: the 0.35 stays out.
        let flagged = rank(&nouls(&[("constructor_3.build_armmex_spot_4", 0.35), ("constructor_3.build_armsolar", 0.6)]));
        assert_eq!(flagged[2].iter().map(|(_, ti)| *ti).collect::<Vec<_>>(), vec![2]);
    }

    /// The joint worlds: every combination of the open actors' best changes under the cap, a world of two builders
    /// on one spot or of two groups joining each other dropped; each line says every change.
    #[test]
    fn the_worlds_join_the_actors_changes_under_the_cap() {
        let menus = vec![
            group("A", true, vec![mv("join_group_B", Order::Join("B".into()), None), mv("go_spot_30", Order::Go("spot_30".into()), None)]),
            group("B", true, vec![mv("join_group_A", Order::Join("A".into()), None)]),
            menu("constructor_3", Kind::Builder(UnitId(3)), true, vec![mv("build_armmex_spot_4", Order::Build(UnitDefId(1), Site::Spot(4)), None)]),
            menu("commander", Kind::Builder(UnitId(4)), true, vec![mv("build_armmex_spot_4", Order::Build(UnitDefId(1), Site::Spot(4)), None)]),
        ];
        let answers = nouls(&[("group_A.join_group_B", 0.9), ("group_A.go_spot_30", 0.6), ("group_B.join_group_A", 0.8), ("constructor_3.build_armmex_spot_4", 0.7), ("commander.build_armmex_spot_4", 0.7)]);
        let (worlds, next) = compose(&menus, &answers, &mut BTreeMap::new(), CAP).unwrap();
        assert_eq!(worlds[0], vec![0, 0, 0, 0]);
        assert_eq!(next, vec![Some(1), Some(1), Some(1), Some(1)]);
        assert!(worlds.contains(&vec![2, 1, 1, 0]), "{worlds:?}");
        assert!(!worlds.iter().any(|w| w[0] == 1 && w[1] == 1), "A into B while B into A is no world: {worlds:?}");
        assert!(!worlds.iter().any(|w| w[2] == 1 && w[3] == 1), "two builders on one spot is no world: {worlds:?}");
        // 3 * 2 * 2 * 2 = 24 combinations, less the 4 with the mutual join and the 6 with both on the spot, 1 in both.
        assert_eq!(worlds.len(), 24 - 4 - 6 + 1, "{worlds:?}");
        let flags = BTreeMap::new();
        let forbidden = BTreeSet::new();
        let line = consequence(&vec![2, 1, 1, 0], &menus, &said(&[], &flags, &next, &forbidden), Some(&worlds[0]));
        assert_eq!(line, "As w1, and: go_spot_30; join_group_A; build_armmex_spot_4.");
        // A cap of 6 holds world 1 and the five single changes; nothing joint fits.
        let (small, _) = compose(&menus, &answers, &mut BTreeMap::new(), 6).unwrap();
        assert_eq!(small.len(), 6);
        assert!(small.iter().all(|w| w.iter().filter(|i| **i != 0).count() <= 1), "{small:?}");
    }

    /// The split pick (the `split` layer): the actors with a change on offer fall into components that share no
    /// actor, party or spot; each has its own worlds and lines, said over its own actors and parties; one request
    /// asks every component its best change and every change against "nothing changes"; what comes back is one
    /// world of the changes taken.
    #[test]
    fn the_pick_is_split_into_components_that_touch_nothing_of_each_other() {
        let menus = vec![
            group("A", true, vec![mv("attack_party_7", Order::Attack("party_7".into()), Some("party_7")), mv("go_spot_30", Order::Go("spot_30".into()), None)]),
            group("B", true, vec![mv("send_1_party_7", Order::Send(vec![UnitId(9)], "party_7".into()), Some("party_7"))]),
            menu("constructor_3", Kind::Builder(UnitId(3)), true, vec![mv("build_armmex_spot_4", Order::Build(UnitDefId(1), Site::Spot(4)), None)]),
            menu("commander", Kind::Builder(UnitId(4)), true, vec![mv("build_armmex_spot_4", Order::Build(UnitDefId(1), Site::Spot(4)), None)]),
            menu("plant_5", Kind::Factory(UnitId(5)), true, vec![mv("make_armflash", Order::Make(UnitDefId(1)), None)]),
            group("C", false, vec![mv("go_spot_31", Order::Go("spot_31".into()), None)]),
        ];
        let parties = vec![party("party_7"), party("party_8")];
        let answers = nouls(&[("party_7.answer", 0.9), ("party_8.answer", 0.9), ("group_A.attack_party_7", 0.8), ("group_A.go_spot_30", 0.6), ("group_B.send_1_party_7", 0.7), ("constructor_3.build_armmex_spot_4", 0.7), ("commander.build_armmex_spot_4", 0.6), ("plant_5.make_armflash", 0.9), ("group_C.change", 0.1), ("group_C.go_spot_31", 0.9)]);
        let followed = follow_up(&menus, &parties, &[], &answers, "340 of 500 stored", CAP, &json!({}));
        let (components, _) = followed.components.expect("something opened");
        // The two groups answer one party, the two builders want one spot, the plant stands alone; group_C is closed.
        assert_eq!(components.iter().map(|c| c.menus.clone()).collect::<Vec<_>>(), vec![vec![0, 1], vec![2, 3], vec![4]]);
        let groups = &components[0];
        assert_eq!(groups.worlds.len(), 1 + 3 + 2, "three single changes and A's two with B's one: {:?}", groups.worlds);
        assert!(groups.worlds.iter().all(|w| w[2..].iter().all(|i| *i == 0)));
        // A component's lines speak of its own: the party it answers, its idle actors; party_8 and the builders are another's.
        assert_eq!(groups.lines[0], "Nothing changes: every actor keeps the course its entry under `actors` describes. Left to nobody: party_7 (2 Ticks, in sight). group_A idle, doing nothing, with its next move, the attack_party_7, not ordered; group_B idle, doing nothing, with its next move, the send_1_party_7, not ordered.");
        assert_eq!(components[1].worlds, vec![vec![0, 0, 0, 0, 0, 0], vec![0, 0, 1, 0, 0, 0], vec![0, 0, 0, 1, 0, 0]], "two builders on one spot is no world");
        assert_eq!(components[2].lines.len(), 2);
        assert!(components[2].lines[0].contains("plant_5 idle, doing nothing, with its next move, the make_armflash, not ordered, while the metal store reads 340 of 500 stored") && !components[2].lines[0].contains("party_"), "{}", components[2].lines[0]);
        let questions = split_questions(&components);
        assert_eq!(questions.keys().map(String::as_str).collect::<Vec<_>>(), ["worlds.c0.pick", "worlds.c0.w2", "worlds.c0.w3", "worlds.c0.w4", "worlds.c0.w5", "worlds.c0.w6", "worlds.c1.pick", "worlds.c1.w2", "worlds.c1.w3", "worlds.c2.w2"]);
        // Read back: the groups' candidate is sampled from their Choice and taken; the builders' is declined; the
        // plant's one change needs no first stage and is taken.
        let choice = |pick: &str, p: &[(&str, f64)]| Answer::Choice { choice: pick.to_string(), confidence: 0.7, probabilities: p.iter().map(|(k, v)| (k.to_string(), *v)).collect() };
        let joint = groups.worlds.iter().position(|w| w[0] == 1 && w[1] == 1).expect("A attacks and B sends") + 1;
        let answers: BTreeMap<String, Answer> = [
            ("worlds.c0.pick".to_string(), choice(&format!("w{joint}"), &[(&format!("w{joint}"), 1.0)])),
            (format!("worlds.c0.w{joint}"), choice(&format!("w{joint}"), &[("w1", 0.3), (&format!("w{joint}"), 0.7)])),
            ("worlds.c1.pick".to_string(), choice("w2", &[("w2", 1.0)])),
            ("worlds.c1.w2".to_string(), choice("w1", &[("w1", 0.7), ("w2", 0.3)])),
            ("worlds.c2.w2".to_string(), choice("w2", &[("w1", 0.3), ("w2", 0.7)])),
        ]
        .into();
        let (world, parts) = split_pick(&answers, &components, &menus, &mut || 0.5);
        assert_eq!(world, vec![1, 1, 0, 0, 1, 0]);
        assert_eq!(parts.iter().map(|p| p["taken"].as_bool()).collect::<Vec<_>>(), vec![Some(true), Some(false), Some(true)]);
    }

    /// World 1's line: the cost of changing nothing. An idle actor is named with the move the gate rated best for
    /// it; a party is left to nobody only when Jev says it needs answering; a party somebody's course is on is met.
    #[test]
    fn world_one_names_each_idle_actors_best_rated_move() {
        let mut a = group("A", true, vec![mv("fight_to_spot_46", Order::FightTo("spot_46".into()), None)]);
        a.idle_words = "group_A holds at spot_49, a stop it has reached".into();
        a.moves[1].said = "the advance to spot_46".into();
        let mut b = group("B", false, vec![mv("follow_group_A", Order::Follow(Ward::Group("A".into())), None)]);
        b.aimed_at = Some("party_8".into());
        b.against = Some(Standing { in_reach: 2, of: 3, way: "right here, a few seconds of walking".into(), idle: 0, catches: Some(false), young: None });
        let menus = vec![a, b, menu("plant_5", Kind::Factory(UnitId(5)), true, vec![mv("make_armflash", Order::Make(UnitDefId(1)), None)])];
        let parties = vec![party("party_7"), party("party_8"), party("party_9")];
        let flags: BTreeMap<String, f64> = [("party_7.answer".to_string(), 0.8), ("party_8.answer".to_string(), 0.9), ("party_9.answer".to_string(), 0.2)].into();
        let next = vec![Some(1), None, None];
        let forbidden = BTreeSet::new();
        let line = consequence(&vec![0, 0, 0], &menus, &said(&parties, &flags, &next, &forbidden), None);
        assert_eq!(
            line,
            "Nothing changes: every actor keeps the course its entry under `actors` describes. Met: party_8 (2 Ticks, in sight) met by group_B. Left to nobody: party_7 (2 Ticks, in sight). group_A holds at spot_49, a stop it has reached, with its next move, the advance to spot_46, not ordered; plant_5 idle, doing nothing, while the metal store reads 340 of 500 stored."
        );
        // The change's line says only what it changes: the group no longer idle is said by its move.
        let line = consequence(&vec![1, 0, 0], &menus, &said(&parties, &flags, &next, &forbidden), Some(&vec![0, 0, 0]));
        assert_eq!(line, "As w1, and: fight_to_spot_46.");
    }

    /// A party is met only by an actor with something in reach of it. One told to attack it from afar is said as
    /// that, in world 1 and in the party's question, with how far it stands, that it stands with no order and that
    /// the party outruns it; a move made at the party is a sending. (player-35 7:53-8:28: "party_17 met by group_W"
    /// and "what stands (group_W on it)" for 35 s while all nine of group_W stood idle 1,000 or more from it.)
    #[test]
    fn a_party_is_met_only_by_what_has_it_in_reach() {
        let mut w = group("W", false, vec![mv("go_spot_30", Order::Go("spot_30".into()), None)]);
        w.aimed_at = Some("party_7".into());
        w.against = Some(Standing { in_reach: 0, of: 9, way: "near, about 10 s of walking".into(), idle: 9, catches: Some(false), young: None });
        let menus = vec![w, group("A", false, vec![mv("attack_party_7", Order::Attack("party_7".into()), Some("party_7"))])];
        let parties = vec![party("party_7")];
        let flags: BTreeMap<String, f64> = [("party_7.answer".to_string(), 0.8)].into();
        let (next, forbidden) = (vec![None, None], BTreeSet::new());
        let away = "group_W is told to attack it, but nothing of it has the party in reach (the nearest of its 9 soldiers is near, about 10 s of walking away, every one of them standing with no order); the party outruns it: it reaches the party only where the party stands still";
        let line = consequence(&vec![0, 0], &menus, &said(&parties, &flags, &next, &forbidden), None);
        assert_eq!(line, format!("Nothing changes: every actor keeps the course its entry under `actors` describes. Not met: party_7 (2 Ticks, in sight): {away}."));
        let line = consequence(&vec![0, 1], &menus, &said(&parties, &flags, &next, &forbidden), Some(&vec![0, 0]));
        assert_eq!(line, "As w1, and: attack_party_7. Sent at: party_7 (2 Ticks, in sight) by group_A.");
        // Taken off the party, and nobody else sent: Jev said it needs answering, so it is left to nobody.
        let line = consequence(&vec![1, 0], &menus, &said(&parties, &flags, &next, &forbidden), Some(&vec![0, 0]));
        assert_eq!(line, "As w1, and: go_spot_30. Left to nobody: party_7 (2 Ticks, in sight).");
        let qs: BTreeMap<String, Question> = gate_questions(&menus, &parties, &[], false, false, false).into_iter().collect();
        let Question::Noul { instructions, .. } = &qs["party_7.answer"] else { panic!("a noul") };
        assert!(instructions.as_str().unwrap().contains(&format!("by someone other than what stands ({away})?")), "{instructions}");
    }

    /// The diet's `only_used`: a move of the group's own says the fight's facts its course has just said once;
    /// the course keeps them, and without the diet the move says them in full.
    #[test]
    fn a_moves_question_says_the_fight_its_course_said_once() {
        let fight = "party_3 (2 armpw): for the 4 of its 7 soldiers near it, we outweigh it heavily; it can follow this group";
        let mut m = group("A", false, vec![mv("go_home", Order::Go("home".into()), None)]);
        m.moves[0].words = format!("group_A attacks party_3; near {fight}");
        m.moves[1].words = format!("group_A walks to home; stepping back from {fight}; leaves party_3");
        m.near_party = Some("party_3".into());
        m.fight = Some(fight.to_string());
        let text = |once: bool| {
            let qs: BTreeMap<String, Question> = gate_questions(std::slice::from_ref(&m), &[], &[], once, false, false).into_iter().collect();
            let Question::Noul { instructions, .. } = &qs["group_A.go_home"] else { panic!("a noul") };
            instructions.as_str().unwrap().to_string()
        };
        assert!(text(true).ends_with(&format!("rather than group_A attacks party_3; near {fight}? The move: group_A walks to home; stepping back from party_3 (as said above); leaves party_3.")), "{}", text(true));
        assert!(text(false).ends_with(&format!("The move: group_A walks to home; stepping back from {fight}; leaves party_3.")), "{}", text(false));
    }

    /// The `places` layer's questions: one per place with a move aimed at it, saying what stands there and who
    /// could go; a builder's own move in the short words when the diet says so, a group's never.
    #[test]
    fn a_place_is_asked_about_once_and_a_builders_move_in_short_words() {
        let mut builder = menu("constructor_3", Kind::Builder(UnitId(3)), false, vec![mv("build_armmex_spot_56", Order::Build(bot_protocol::UnitDefId(1), Site::Spot(56)), None), mv("go_spot_9", Order::Go("spot_9".into()), None), mv("help_plant_1", Order::Help(UnitId(1)), None)]);
        builder.moves[0].words = "constructor_3 walks to spot_68".into();
        builder.moves[1].words = "constructor_3 builds a metal extractor at spot_56 (a few seconds of walking, D6, the free spot it reaches soonest): our 23rd (4 under way already); stops helping constructor_11407".into();
        builder.moves[2].words = "constructor_3 walks to spot_9 (far, about 50 s of walking, our extractor (beside it: our Sentry (armllt)))".into();
        builder.moves[3].words = "constructor_3 helps plant_1 build".into();
        let mut g = group("A", true, vec![mv("fight_to_spot_9", Order::FightTo("spot_9".into()), None), mv("attack_party_1", Order::Attack("party_1".into()), Some("party_1"))]);
        g.moves[1].words = "group_A advances to spot_9 (some way off, about 30 s of walking, our extractor), fighting on the way".into();
        let menus = vec![builder, g];
        let qs: BTreeMap<String, Question> = gate_questions(&menus, &[party("party_1")], &[], false, true, true).into_iter().collect();
        let text = |id: &str| {
            let Question::Noul { instructions, .. } = &qs[id] else { panic!("a noul") };
            instructions.as_str().unwrap().to_string()
        };
        assert_eq!(text("spot_56.place"), "Given `actors`, `enemy`, `economy`, `ours` and the player's `instructions`: does any of ours have a reason to go to, fight at or build at spot_56 now (a few seconds of walking, D6, the free spot it reaches soonest), rather than carry on with what it does? Those with a move there: constructor_3 (builds a metal extractor, a few seconds of walking). No when nothing there needs doing now or the instructions send everyone elsewhere.");
        assert!(text("spot_9.place").contains("spot_9 now (far, about 50 s of walking, our extractor (beside it: our Sentry (armllt))), rather than") && text("spot_9.place").contains("Those with a move there: constructor_3 (walks, far); group_A (advances, some way off)."), "{}", text("spot_9.place"));
        assert!(!qs.keys().any(|k| k.contains("party_1.place")), "a party move has no place");
        assert_eq!(text("constructor_3.build_armmex_spot_56"), "constructor_3 builds a metal extractor at spot_56 (a few seconds of walking, D6, the free spot it reaches soonest): our 23rd (4 under way already); stops helping constructor_11407. Rather than: constructor_3 walks to spot_68.");
        assert!(text("group_A.fight_to_spot_9").starts_with("Given `actors.group_A`, `economy`, `ours` and the player's `instructions`: is this what group_A should do now, rather than"), "a group keeps the framing");
        let long: BTreeMap<String, Question> = gate_questions(&menus, &[], &[], false, false, false).into_iter().collect();
        assert!(!long.keys().any(|k| k.ends_with(".place")));
        let Question::Noul { instructions, .. } = &long["constructor_3.go_spot_9"] else { panic!("a noul") };
        assert!(instructions.as_str().unwrap().starts_with("Given `actors.constructor_3`"));
    }

    /// The forbidden mark: a detachment is asked about a second time, as a reading of the instructions, and one
    /// the gate rates at the bar or over says so on its line; under the bar, and for the whole group's attack,
    /// nothing is said.
    #[test]
    fn a_detachment_the_gate_reads_as_forbidden_says_so_on_its_line() {
        let menus = vec![group("A", false, vec![mv("send_2_party_1", Order::Send(vec![UnitId(9)], "party_1".into()), Some("party_1")), mv("attack_party_1", Order::Attack("party_1".into()), Some("party_1"))])];
        let parties = vec![party("party_1")];
        let qs: BTreeMap<String, Question> = gate_questions(&menus, &parties, &[], false, false, false).into_iter().collect();
        let Question::Noul { instructions, .. } = &qs["group_A.send_2_party_1.forbidden"] else { panic!("a noul") };
        assert!(instructions.as_str().unwrap().starts_with("Read the player's `instructions` alone: do they forbid this move"));
        assert!(!qs.contains_key("group_A.attack_party_1.forbidden"), "detachments only");
        let Question::Noul { instructions, .. } = &qs["group_A.attack_party_1"] else { panic!("a noul") };
        assert!(instructions.as_str().unwrap().starts_with("Is this the move to make against party_1 now, rather than what stands (nobody moves for it)?"), "a move aimed at a party is asked as an answer to it");
        let line_of = |forbidden: f64| {
            let answers = nouls(&[("party_1.answer", 0.8), ("group_A.change", 0.2), ("group_A.send_2_party_1", 0.6), ("group_A.attack_party_1", 0.55), ("group_A.send_2_party_1.forbidden", forbidden)]);
            let followed = follow_up(&menus, &parties, &[], &answers, "", CAP, &json!({}));
            assert_eq!(followed.flags["group_A.send_2_party_1.forbidden"], forbidden);
            let (worlds, lines, _) = followed.worlds.expect("the party opened");
            (lines[worlds.iter().position(|w| w[0] == 1).expect("the detachment's world")].clone(), lines[worlds.iter().position(|w| w[0] == 2).expect("the whole group's world")].clone())
        };
        let (sent, whole) = line_of(0.84);
        assert!(sent.ends_with("The player's instructions forbid this detachment for group_A."), "{sent}");
        assert!(sent.contains("Sent at: party_1") && !whole.contains("forbid"), "{sent} / {whole}");
        assert!(!line_of(0.6).0.contains("forbid"), "under the bar");
    }

    /// The pick's two stages: stage one offers the deviations alone and its answer is sampled by the mass the
    /// worlds hold; stage two is world 1 against the candidate under its own number, which `pick` reads back.
    #[test]
    fn the_pick_samples_stage_one_and_asks_world_one_against_the_candidate() {
        let lines: Vec<String> = ["Nothing changes", "As w1, and: a", "As w1, and: b", "As w1, and: c"].iter().map(|s| s.to_string()).collect();
        let Question::Choice { criteria, .. } = stage_one(&lines) else { panic!("a choice") };
        assert_eq!(criteria.keys().cloned().collect::<Vec<_>>(), vec!["w2", "w3", "w4"]);
        let answers = BTreeMap::from([("worlds.pick".to_string(), Answer::Choice { choice: "w2".into(), probabilities: BTreeMap::from([("w2".to_string(), 0.5), ("w3".to_string(), 0.3), ("w4".to_string(), 0.2)]), confidence: 0.5 })]);
        assert_eq!(sample(&answers, "worlds.pick", 4, 0.10), Some((1, 0.5)));
        assert_eq!(sample(&answers, "worlds.pick", 4, 0.55), Some((2, 0.3)));
        assert_eq!(sample(&answers, "worlds.pick", 4, 0.95), Some((3, 0.2)));
        // A stray key for world 1 or beyond the worlds is not drawn.
        let stray = BTreeMap::from([("worlds.pick".to_string(), Answer::Choice { choice: "w1".into(), probabilities: BTreeMap::from([("w1".to_string(), 0.9), ("w9".to_string(), 0.05), ("w3".to_string(), 0.05)]), confidence: 0.9 })]);
        assert_eq!(sample(&stray, "worlds.pick", 4, 0.5), Some((2, 0.05)));
        let Question::Choice { criteria, .. } = stage_two(&lines, 2) else { panic!("a choice") };
        assert_eq!(criteria.keys().cloned().collect::<Vec<_>>(), vec!["w1", "w3"]);
        let worlds: Vec<World> = vec![vec![0], vec![1], vec![2], vec![3]];
        assert_eq!(pick(&BTreeMap::from([("worlds.pick".to_string(), Answer::Choice { choice: "w3".into(), probabilities: BTreeMap::new(), confidence: 0.6 })]), &worlds), Some((2, 0.6)));
        assert_eq!(pick(&BTreeMap::from([("worlds.pick".to_string(), Answer::Choice { choice: "w1".into(), probabilities: BTreeMap::new(), confidence: 0.6 })]), &worlds), Some((0, 0.6)));
        assert!((0.0..1.0).contains(&draw()));
    }

    /// The line budget (player-27: 145-255 worlds of 120-240k characters refused for three minutes).
    #[test]
    fn the_worlds_are_cut_to_the_line_budget_keeping_world_one_and_a_deviation() {
        let lines: Vec<String> = (0..10).map(|i| format!("w{i} {}", "x".repeat(100))).collect();
        assert_eq!(fit(&lines, 10_000), 10);
        assert_eq!(fit(&lines, 350), 4, "three deviations of 104 fit 350, the fourth does not");
        assert_eq!(fit(&lines, 10), 2, "the first deviation stays even over the budget");
    }

    /// A closed actor (the `news` layer) is not asked about its own moves, and one asked for the layer's audit is
    /// asked and not played; but a party is asked about at every gate and so are its answers: a closed group's
    /// moves aimed at a party are asked, and go to the pick by the party's answer. (rebuild-smoke-1, 4:39-4:47: a
    /// Pawn shot a constructor 300 from a holding group for eight seconds, the party's answer at 0.7-0.8, while
    /// the group was closed and no answer of its was a question.)
    #[test]
    fn a_closed_actors_answers_to_a_party_are_still_asked() {
        let mut closed = group("A", false, vec![mv("go_spot_30", Order::Go("spot_30".into()), None), mv("send_1_party_7", Order::Send(vec![UnitId(9)], "party_7".into()), Some("party_7"))]);
        closed.quiet = true;
        let mut audited = group("B", false, vec![mv("go_spot_30", Order::Go("spot_30".into()), None)]);
        (audited.quiet, audited.audit) = (true, true);
        let menus = vec![closed, audited];
        let parties = vec![party("party_7")];
        let ids: Vec<String> = gate_questions(&menus, &parties, &[], false, false, false).into_iter().map(|(id, _)| id).collect();
        assert_eq!(ids, ["party_7.answer", "group_A.send_1_party_7", "group_A.send_1_party_7.forbidden", "group_B.change", "group_B.go_spot_30"]);
        let answers = nouls(&[("party_7.answer", 0.2), ("group_A.send_1_party_7", 0.9), ("group_B.change", 0.9), ("group_B.go_spot_30", 0.9)]);
        assert!(compose(&menus, &answers, &mut BTreeMap::new(), CAP).is_none(), "the audit's answers move nobody, and a party that needs no answer opens nothing");
        let answers = nouls(&[("party_7.answer", 0.8), ("group_A.send_1_party_7", 0.6), ("group_B.change", 0.9), ("group_B.go_spot_30", 0.9)]);
        let (worlds, _) = compose(&menus, &answers, &mut BTreeMap::new(), CAP).expect("the party opens its answers");
        assert_eq!(worlds, vec![vec![0, 0], vec![2, 0]]);
    }

    #[test]
    fn the_picks_state_keeps_what_the_worlds_name_and_briefs_the_rest() {
        let menus = vec![group("A", false, vec![mv("send_1_party_1", Order::Send(vec![UnitId(9)], "party_1".into()), Some("party_1"))]), menu("constructor_3", Kind::Builder(UnitId(3)), true, vec![mv("build_armmex_spot_4", Order::Build(UnitDefId(1), Site::Spot(4)), None)])];
        let worlds: Vec<World> = vec![vec![0, 0], vec![1, 0]];
        let lines = vec!["Nothing changes.".to_string(), "As w1, and: 1 Blitz of group_A hunt party_1 at spot_4".to_string()];
        let state = json!({
            "instructions": "group_B stands at south_yard",
            "actors": {
                "group_A": { "units": "8 Blitz", "at": "spot_4", "doing": "holding" },
                "constructor_3": { "units": "a constructor", "at": "home", "doing": "idle" },
                "group_B": { "units": "2 Stout", "at": "spot_9", "doing": "holding" }
            },
            "places": { "spot_4": { "what": "free" }, "spot_9": { "what": "free" }, "south_yard": { "what": "a mark" }, "spot_77": { "what": "theirs" } }
        });
        let cut = pick_state(&state, &menus, &worlds, &lines);
        assert!(cut["actors"]["group_A"].is_object(), "an actor a world moves keeps its entry");
        assert!(cut["actors"]["group_B"].is_object(), "an actor the instructions name keeps its entry");
        assert_eq!(cut["actors"]["constructor_3"], json!("a constructor, at home, idle"), "an actor no world touches is one line");
        assert!(cut["places"].get("spot_4").is_some() && cut["places"].get("south_yard").is_some() && cut["places"].get("spot_9").is_some(), "{}", cut["places"]);
        assert!(cut["places"].get("spot_77").is_none(), "a place nothing names is dropped");
    }
}
