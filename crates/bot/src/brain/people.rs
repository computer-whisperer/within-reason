//! Who is in the game, from the start script: every person by the name the lobby knows them by, playing a seat or
//! watching, and the person our chat goes out under.

use serde_json::{Value, json};

use super::Brain;

/// A `[PLAYERn]` section of the start script. `player` is the engine's player number, the one a chat line carries.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Person {
    pub player: i32,
    pub name: String,
    pub spectator: bool,
    pub team: Option<i32>,
}

/// The body of every section named `prefix` followed by a number, with that number: both shapes the scripts come
/// in (`[player0]` on a line of its own and `[PLAYER0] { ... }` inline), any case.
fn numbered_sections(script: &str, prefix: &str) -> Vec<(i32, String)> {
    let lower = script.to_ascii_lowercase();
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(found) = lower[from..].find(&format!("[{prefix}")) {
        let name = from + found + 1 + prefix.len();
        from = name;
        let Some(close) = lower[name..].find(']') else { break };
        let Ok(number) = lower[name..name + close].parse::<i32>() else { continue };
        let Some(open) = lower[name + close..].find('{').map(|i| name + close + i) else { break };
        // The matching brace: an AI's section holds an `[options]` block of its own.
        let mut depth = 0;
        let end = lower[open..].char_indices().find_map(|(i, c)| {
            depth += match c {
                '{' => 1,
                '}' => -1,
                _ => 0,
            };
            (depth == 0).then_some(open + i)
        });
        let Some(end) = end else { break };
        out.push((number, script[open + 1..end].to_string()));
        from = end;
    }
    out
}

/// The value of `key` in a section's own body (not in a block nested in it), any case of the key.
fn value(body: &str, key: &str) -> Option<String> {
    // A name may hold brackets ("[gecko]thebluegecko"), so the nested block is found by its brace.
    let own = body.find('{').map_or(body, |open| &body[..body[..open].rfind('[').unwrap_or(open)]);
    own.split(';').filter_map(|pair| pair.split_once('=')).find(|(k, _)| k.trim().eq_ignore_ascii_case(key)).map(|(_, v)| v.trim().to_string())
}

pub(super) fn people(script: &str) -> Vec<Person> {
    numbered_sections(script, "player")
        .into_iter()
        .map(|(player, body)| {
            let spectator = value(&body, "spectator").is_some_and(|v| v == "1");
            Person { player, name: value(&body, "name").unwrap_or_default(), spectator, team: value(&body, "team").and_then(|t| t.parse().ok()).filter(|_| !spectator) }
        })
        .collect()
}

/// The player number of the person hosting the AI that plays `team`: an AI's chat goes out as that person's.
pub(super) fn host_of(script: &str, team: i32) -> Option<i32> {
    numbered_sections(script, "ai").into_iter().find(|(_, body)| value(body, "team").and_then(|t| t.parse::<i32>().ok()) == Some(team)).and_then(|(_, body)| value(&body, "host")?.parse().ok())
}

impl Brain {
    /// Who a chat line is from, as the player reads it: the name, and whether that person plays a seat or watches.
    pub(super) fn speaker(&self, player: i32) -> String {
        let hello = &self.world.hello;
        match people(&hello.script).into_iter().find(|p| p.player == player) {
            Some(p) if p.name.is_empty() => format!("player {player}"),
            Some(p) => match p.team.and_then(|t| hello.teams.iter().find(|x| x.team == t)) {
                Some(seat) if seat.ally_team == hello.ally_team => format!("{} (plays a seat on our side)", p.name),
                Some(_) => format!("{} (plays against us)", p.name),
                None => format!("{} (watching)", p.name),
            },
            None => format!("player {player}"),
        }
    }

    /// The `people` entry of the map description: everyone in the game by name, and whose name our chat carries.
    pub(super) fn people_description(&self) -> Value {
        let hello = &self.world.hello;
        let all = people(&hello.script);
        let host = host_of(&hello.script, hello.team).and_then(|h| all.iter().find(|p| p.player == h)).map(|p| p.name.clone()).filter(|n| !n.is_empty());
        let seat = |p: &Person| -> Option<String> {
            let seat = hello.teams.iter().find(|t| Some(t.team) == p.team)?;
            Some(format!("plays team {}, {}", seat.team, if seat.ally_team == hello.ally_team { "on our side" } else { "against us" }))
        };
        json!({
            "in_the_game": all.iter().filter(|p| !p.name.is_empty()).map(|p| json!({ "name": p.name, "role": seat(p).unwrap_or_else(|| "watching (a spectator)".to_string()) })).collect::<Vec<_>>(),
            "your_chat": match &host {
                Some(name) => format!("an AI cannot chat under its own seat: the game sends what you `say` as chat from the person hosting you, {name}. Everyone reads your lines under {name}'s name, so a line of yours says it is the AI speaking; a line from {name} in your report is that person's own (yours are not shown back to you)."),
                None => "an AI cannot chat under its own seat: the game sends what you `say` as chat from the person hosting you, so a line of yours says it is the AI speaking.".to_string(),
            },
            "note": "chat lines in your report carry the name of who said them and whether they play or watch; the AI seats are in `sides`",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_script_names_the_people_and_the_host_in_both_shapes() {
        let lobby = "[GAME]\n{\n[player0]\n{\nrank=4;\nteam=0;\nname=[gecko]thebluegecko;\nspectator=0;\n}\n[player3]\n{\nname=computer_whisperer;\nspectator=1;\n}\n[ai0]\n{\nteam=1;\nhost=3;\nshortname=WReason;\n[options]\n{\nprofile=x;\n}\n}\n}";
        let all = people(lobby);
        assert_eq!(all, vec![
            Person { player: 0, name: "[gecko]thebluegecko".into(), spectator: false, team: Some(0) },
            Person { player: 3, name: "computer_whisperer".into(), spectator: true, team: None },
        ]);
        assert_eq!(host_of(lobby, 1), Some(3));
        let arena = "[GAME]\n{\n\t[PLAYER0] { Name=arena; Spectator=1; }\n\t[AI0] { Name=ai0; Team=0; Host=0; ShortName=WReason; Version=0.1; }\n\t[AI1] { Name=ai1; Team=1; Host=0; ShortName=BARb; [OPTIONS] { profile=hard; } }\n}";
        assert_eq!(people(arena), vec![Person { player: 0, name: "arena".into(), spectator: true, team: None }]);
        assert_eq!(host_of(arena, 1), Some(0));
        assert_eq!(host_of(arena, 5), None);
    }
}
