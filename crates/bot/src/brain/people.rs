//! Who is in the game, from the start script: every person by the name the lobby knows them by, playing a seat or
//! watching, and the person our chat goes out under.

use bot_protocol::ChatTo;
use serde_json::{Value, json};

use super::Brain;

/// Whether a player on our side would be shown a line (H-CHAT-PERMISSIONS, the engine's rule in
/// `CGame::HandleChatMsg`): a player's all-chat and a spectator's while the players can see it, team chat from a
/// player of our own side; never the spectators' channel or a whisper. A line with no label (`relay` false: no
/// relay widget on the host's client) is taken only from a player of our side; with the widget running an
/// unlabelled line is one the host was not shown.
pub(super) fn hearing(to: ChatTo, ours: bool, relay: bool) -> bool {
    match to {
        ChatTo::Everyone | ChatTo::Watching => true,
        ChatTo::Allies => ours,
        ChatTo::Spectators | ChatTo::Private => false,
        ChatTo::Unknown => !relay && ours,
    }
}

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

/// A mod option of the lobby's, from the script's `[modoptions]` section (`deathmode` for the game-end rule).
pub(super) fn mod_option(script: &str, key: &str) -> Option<String> {
    let lower = script.to_ascii_lowercase();
    let start = lower.find("[modoptions]")?;
    let open = start + lower[start..].find('{')?;
    let close = open + lower[open..].find('}')?;
    value(&script[open + 1..close], key)
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
    /// The person hosting this AI, when the script names one.
    fn host(&self) -> Option<Person> {
        let script = &self.world.hello.script;
        let host = host_of(script, self.world.hello.team)?;
        people(script).into_iter().find(|p| p.player == host)
    }

    /// The player number our chat goes out under.
    pub(super) fn host_player(&self) -> Option<i32> {
        self.host().map(|p| p.player)
    }

    /// The person hosting this AI watches as a spectator: our lines are a spectator's (docs/design/2026-11-04-chat-permissions.md).
    pub(super) fn host_watches(&self) -> bool {
        self.host().is_some_and(|p| p.spectator)
    }

    /// The person plays a seat on our side.
    pub(super) fn plays_with_us(&self, player: i32) -> bool {
        let hello = &self.world.hello;
        people(&hello.script).into_iter().find(|p| p.player == player).and_then(|p| p.team).and_then(|t| hello.teams.iter().find(|x| x.team == t)).is_some_and(|seat| seat.ally_team == hello.ally_team)
    }

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
        let host_person = self.host();
        let your_chat = match (&host, &host_person) {
            (Some(name), Some(p)) if p.spectator => format!("an AI cannot chat under its own seat: the game sends what you `say` as chat from the person hosting you, {name}, who watches as a spectator. Your lines go to everyone in the game, his side too, tagged as a spectator's; a player can hide spectator chat, and a game with spectator chat off shows them to nobody playing (the report says so when that is the case). Nothing you say can reach our allies alone (`say` to allies is refused). A line of yours should say it is the AI speaking; a line from {name} in your report is that person's own (yours are not shown back to you)."),
            (Some(name), Some(p)) => {
                let side = seat(p).map_or("no seat".to_string(), |s| s.to_string());
                format!("an AI cannot chat under its own seat: the game sends what you `say` as chat from the person hosting you, {name}, who {side}. `say` to everyone reaches both sides; `say` to allies reaches the side {name} plays on alone. A line of yours should say it is the AI speaking; a line from {name} in your report is that person's own (yours are not shown back to you).")
            }
            _ => "an AI cannot chat under its own seat: the game sends what you `say` as chat from the person hosting you, so a line of yours says it is the AI speaking.".to_string(),
        };
        json!({
            "in_the_game": all.iter().filter(|p| !p.name.is_empty()).map(|p| json!({ "name": p.name, "role": seat(p).unwrap_or_else(|| "watching (a spectator)".to_string()) })).collect::<Vec<_>>(),
            "your_chat": your_chat,
            "you_hear": "what a player on our side is shown: all-chat, our allies' team chat, and spectators' lines when the players can see them; not his team chat, the spectators' own channel or others' whispers. Each line in your report carries the name of who said it and whether they play or watch; the AI seats are in `sides`",
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
        let host = people(lobby).into_iter().find(|p| p.player == host_of(lobby, 1).unwrap()).unwrap();
        assert!(host.spectator, "the lobby's host watches");
    }

    /// A player on our side is shown all-chat, our allies' team chat and spectators' visible lines; nothing else.
    /// Unlabelled lines (no relay widget) are taken from allied players only.
    #[test]
    fn a_player_on_our_side_hears_by_the_games_rule() {
        for ours in [false, true] {
            for relay in [false, true] {
                assert!(hearing(ChatTo::Everyone, ours, relay));
                assert!(hearing(ChatTo::Watching, ours, relay));
                assert_eq!(hearing(ChatTo::Allies, ours, relay), ours);
                assert!(!hearing(ChatTo::Spectators, ours, relay));
                assert!(!hearing(ChatTo::Private, ours, relay));
                assert_eq!(hearing(ChatTo::Unknown, ours, relay), ours && !relay);
            }
        }
    }

    /// The lobby's game-end rule is read from the script's `[modoptions]`, whatever the case of the keys.
    #[test]
    fn the_scripts_mod_options_give_the_death_mode() {
        let lobby = "[GAME]\n{\n[player0]\n{\nname=x;\n}\n[MODOPTIONS]\n{\ndeathmode=com;\nstartmetal=1000;\n}\n[allyTeam0]\n{\nnumallies=0;\n}\n}";
        assert_eq!(mod_option(lobby, "deathmode").as_deref(), Some("com"));
        assert_eq!(mod_option(lobby, "startmetal").as_deref(), Some("1000"));
        assert_eq!(mod_option(lobby, "nope"), None);
        assert_eq!(mod_option("[GAME]\n{\n}", "deathmode"), None);
    }
}
