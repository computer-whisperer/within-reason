//! What a request's words are on the wire (`Diet::wire_cut`): the request as the hands build and log it, with the
//! two things Jev's meter counts and its answers do not need taken out as it is sent. A backtick is a token of its
//! own, and so is every digit of a unit's engine number (`constructor_10655` is seven tokens); the number is also
//! what Jev has to match a unit by between the picture, the instructions and a question (the user, 2026-10-04: "I
//! wouldn't be suprised if it is bad at matching things by numeric id"). A question's id is not counted and is what
//! the answers come back under, so the ids go as they are (`jev::Client::with_wire`). Replayed on 150 gates of
//! glitters-8v8-hard-1: 1.9% and 1.5% of a gate's tokens, 3.4% together with the flat state not built
//! (`docs/studies/2026-10-04-jev-where-the-tokens-go.md` §15).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// The kinds of actor named by an engine number.
const NUMBERED: [&str; 4] = ["constructor_", "plant_", "lab_", "factory_"];

/// The wire for one seat's hands: backticks dropped, and every unit's engine number replaced by letters, the same
/// unit the same letters for the whole game and in every client that shares this wire.
pub(super) fn wire() -> jev::Wire {
    let names: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    Arc::new(move |text: &str| short(text, &mut names.lock().expect("the names' lock")))
}

/// The nth name in letters: A to Z, then AA, AB and on.
fn letters(mut n: usize) -> String {
    let mut out = Vec::new();
    loop {
        out.push(b'A' + (n % 26) as u8);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    out.reverse();
    String::from_utf8(out).expect("letters")
}

fn word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// `text` without its backticks and with `constructor_10655` as `constructor_A`: a numbered name is rewritten only
/// as a whole word, and takes the next letters the first time it is seen.
fn short(text: &str, names: &mut HashMap<String, String>) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut before = ' ';
    'scan: while let Some(c) = rest.chars().next() {
        if !word(before) {
            for kind in NUMBERED {
                let Some(tail) = rest.strip_prefix(kind) else { continue };
                let digits = tail.chars().take_while(char::is_ascii_digit).count();
                if digits == 0 || tail[digits..].chars().next().is_some_and(word) {
                    continue;
                }
                let name = &rest[..kind.len() + digits];
                let next = names.len();
                out.push_str(names.entry(name.to_string()).or_insert_with(|| format!("{kind}{}", letters(next))));
                rest = &tail[digits..];
                before = '0';
                continue 'scan;
            }
        }
        if c != '`' {
            out.push(c);
        }
        before = c;
        rest = &rest[c.len_utf8()..];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unit keeps its letters wherever it is named; a name inside a longer word, a spot and a party are left;
    /// the backticks go.
    #[test]
    fn a_units_number_goes_as_letters_and_the_backticks_go() {
        let mut names = HashMap::new();
        assert_eq!(short("Given `actors.constructor_10655`: constructor_10655 helps plant_4020 (near spot_43, party_291).", &mut names), "Given actors.constructor_A: constructor_A helps plant_B (near spot_43, party_291).");
        assert_eq!(short("plant_4020 makes; reconstructor_7 and constructor_9x are no units of ours; constructor_10655.", &mut names), "plant_B makes; reconstructor_7 and constructor_9x are no units of ours; constructor_A.");
        assert_eq!((letters(0), letters(25), letters(26), letters(27), letters(701), letters(702)), ("A".into(), "Z".into(), "AA".into(), "AB".into(), "ZZ".into(), "AAA".into()));
    }
}
