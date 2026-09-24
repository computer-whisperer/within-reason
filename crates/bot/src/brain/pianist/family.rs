//! The two-level layout of a group's `do` question (H-HANDS-TWO-LEVEL; K-hands-a-family-question-frees-the-course):
//! in place of one flat choice, a `kind` question over families of the options (stay, fight, go, back; any other
//! option as itself) and one refinement question per family with the original options and their words, all in the
//! one request, since batching costs nothing (`../jev_experiments`, battery D). The `do` answer is composed here:
//! the family with the most mass, then its refinement's argmax; an option's probability is its family's mass times
//! its share of the refinement, so the log, the replays and the switch margin read it as before. Offline the layout
//! took the moments where a flat `continue` held 0.5 to 0.9 from 0% to 51-59% right (`docs/knowledge/jev.md`).
use std::collections::BTreeMap;

use jev::{Answer, Question};
use serde_json::{Value, json};

/// The families and their members, in the order the `kind` question lists them.
pub(super) const FAMILIES: [(&str, &[&str]); 4] = [
    ("stay", &["hold", "continue", "wait"]),
    ("fight", &["engage", "send_against", "attack_unit"]),
    ("go", &["move_to", "fight_to"]),
    ("back", &["retreat", "fall_back"]),
];

pub(super) fn family_of(option: &str) -> Option<&'static str> {
    FAMILIES.iter().find(|(_, members)| members.contains(&option)).map(|(family, _)| *family)
}

fn family_words(family: &str) -> &'static str {
    match family {
        "stay" => "Stay: hold where it is, or carry on with what it is doing (which is asked apart). Nothing beyond its reach is protected by staying.",
        "fight" => "Fight: attack an enemy party now, with the whole group, a detachment, or every soldier on one unit (which is asked apart; the party is `whom`). How this group weighs against the nearest party is on its own `enemies_near` line.",
        "go" => "Go: walk or advance to the place in `where` (which way is asked apart).",
        _ => "Back: retreat home, or fall back to a station away from the enemy (which is asked apart).",
    }
}

fn family_ask(family: &str, name: &str) -> String {
    match family {
        "stay" => format!("If {name} stays, which way: standing where it is and fighting what comes within reach, or carrying on with what it is doing?"),
        "fight" => format!("If {name} fights, how: the whole group after the party in `whom`, a detachment of the soldiers nearest it, or every soldier on one unit of it?"),
        "go" => format!("If {name} goes to the place in `where`, how: walking without stopping to fight, or advancing and fighting everything on the way?"),
        _ => format!("If {name} goes back, which way: home, or falling back to a station away from the enemy?"),
    }
}

/// The questions sent in place of `<name>.do`: `<name>.kind`, and `<name>.<family>` for every family with more than
/// one option offered. The hold option's cost clause (what holding leaves to die) goes onto the stay family.
pub(super) fn questions(name: &str, flat: &Question) -> Vec<(String, Question)> {
    let Question::Choice { instructions, criteria } = flat else { return vec![(format!("{name}.do"), flat.clone())] };
    let mut kind: BTreeMap<String, Value> = BTreeMap::new();
    let mut out = Vec::new();
    for (family, members) in FAMILIES {
        let present: Vec<&str> = members.iter().copied().filter(|m| criteria.contains_key(*m)).collect();
        if present.is_empty() {
            continue;
        }
        let mut words = family_words(family).to_string();
        if family == "stay"
            && let Some(hold) = criteria.get("hold").and_then(Value::as_str)
            && let Some(i) = hold.find("Holding now leaves")
        {
            words.push(' ');
            words.push_str(&hold[i..]);
        }
        kind.insert(family.to_string(), json!(words));
        if present.len() > 1 {
            let refinement: BTreeMap<String, Value> = present.iter().map(|m| (m.to_string(), criteria[*m].clone())).collect();
            out.push((format!("{name}.{family}"), Question::Choice { instructions: json!(family_ask(family, name)), criteria: refinement }));
        }
    }
    for (option, words) in criteria {
        if family_of(option).is_none() {
            kind.insert(option.clone(), words.clone());
        }
    }
    out.insert(0, (format!("{name}.kind"), Question::Choice { instructions: instructions.clone(), criteria: kind }));
    out
}

/// The `do` answer composed from the `kind` answer and the refinements; None without a `kind` answer (a flat ask).
pub(super) fn compose(name: &str, flat: &Question, answers: &BTreeMap<String, Answer>) -> Option<Answer> {
    let Some(Answer::Choice { probabilities: kind, confidence, .. }) = answers.get(&format!("{name}.kind")) else { return None };
    let Question::Choice { criteria, .. } = flat else { return None };
    let mut probabilities: BTreeMap<String, f64> = BTreeMap::new();
    // (the family's mass, the option it picks)
    let mut best: Option<(f64, String)> = None;
    for (family, mass) in kind {
        match FAMILIES.iter().find(|(f, _)| f == family) {
            None => {
                probabilities.insert(family.clone(), *mass);
                if best.as_ref().is_none_or(|b| *mass > b.0) {
                    best = Some((*mass, family.clone()));
                }
            }
            Some((_, members)) => {
                let refinement: BTreeMap<String, f64> = match answers.get(&format!("{name}.{family}")) {
                    Some(Answer::Choice { probabilities, .. }) => probabilities.clone(),
                    // A family of one option offered has no refinement: the option takes the whole mass.
                    _ => members.iter().find(|m| criteria.contains_key(**m)).map(|m| BTreeMap::from([(m.to_string(), 1.0)])).unwrap_or_default(),
                };
                let total: f64 = refinement.values().sum::<f64>().max(1e-9);
                let mut inside: Option<(f64, String)> = None;
                for (option, share) in &refinement {
                    probabilities.insert(option.clone(), mass * share / total);
                    if inside.as_ref().is_none_or(|b| *share > b.0) {
                        inside = Some((*share, option.clone()));
                    }
                }
                if let Some((_, option)) = inside
                    && best.as_ref().is_none_or(|b| *mass > b.0)
                {
                    best = Some((*mass, option));
                }
            }
        }
    }
    let (_, choice) = best?;
    Some(Answer::Choice { choice, probabilities, confidence: *confidence })
}

/// The mass of the option's family (the option itself when it has none).
pub(super) fn family_mass(option: &str, probabilities: &BTreeMap<String, f64>) -> f64 {
    match family_of(option) {
        Some(family) => FAMILIES.iter().find(|(f, _)| *f == family).map_or(0.0, |(_, members)| members.iter().map(|m| probabilities.get(*m).copied().unwrap_or(0.0)).sum()),
        None => probabilities.get(option).copied().unwrap_or(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat() -> Question {
        Question::choice("what next?", [
            ("continue", "Carry on."), ("hold", "Stand. Holding now leaves what party_9 is killing to die."), ("engage", "Attack the party."),
            ("send_against", "A detachment."), ("move_to", "Walk to where."), ("retreat", "Go home."), ("join_group_B", "Merge into group_B."),
        ])
    }

    #[test]
    fn the_layout_has_a_kind_question_and_a_refinement_per_family_of_two_or_more() {
        let qs = questions("group_A", &flat());
        let ids: Vec<&str> = qs.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, ["group_A.kind", "group_A.stay", "group_A.fight"]);
        let Question::Choice { criteria, .. } = &qs[0].1 else { panic!() };
        assert_eq!(criteria.keys().cloned().collect::<Vec<_>>(), ["back", "fight", "go", "join_group_B", "stay"]);
        assert!(criteria["stay"].as_str().unwrap().contains("Holding now leaves what party_9 is killing to die."));
        let Question::Choice { criteria, .. } = &qs[2].1 else { panic!() };
        assert_eq!(criteria.keys().cloned().collect::<Vec<_>>(), ["engage", "send_against"]);
    }

    #[test]
    fn the_answer_is_the_family_with_the_most_mass_then_its_refinements_pick() {
        let choice = |c: &str, p: &[(&str, f64)]| Answer::Choice { choice: c.into(), probabilities: p.iter().map(|(k, v)| (k.to_string(), *v)).collect(), confidence: 0.5 };
        let answers = BTreeMap::from([
            ("group_A.kind".to_string(), choice("stay", &[("stay", 0.4), ("fight", 0.45), ("go", 0.1), ("back", 0.03), ("join_group_B", 0.02)])),
            ("group_A.stay".to_string(), choice("continue", &[("continue", 0.7), ("hold", 0.3)])),
            ("group_A.fight".to_string(), choice("send_against", &[("engage", 0.45), ("send_against", 0.55)])),
        ]);
        let Some(Answer::Choice { choice, probabilities, .. }) = compose("group_A", &flat(), &answers) else { panic!() };
        assert_eq!(choice, "send_against");
        assert!((probabilities["continue"] - 0.28).abs() < 1e-9 && (probabilities["engage"] - 0.2025).abs() < 1e-9);
        // A family of one option offered (go: move_to alone) takes its whole mass; the sum stays one.
        assert!((probabilities["move_to"] - 0.1).abs() < 1e-9 && (probabilities.values().sum::<f64>() - 1.0).abs() < 1e-9);
        assert!((family_mass("hold", &probabilities) - 0.4).abs() < 1e-9 && (family_mass("join_group_B", &probabilities) - 0.02).abs() < 1e-9);
        assert!(compose("group_A", &flat(), &BTreeMap::new()).is_none());
    }
}
