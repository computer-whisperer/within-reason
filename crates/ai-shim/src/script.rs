//! What we read from the game's start script (`doc/StartScriptFormat.txt` in the engine repo): each ally team's start
//! box, and who plays each team (`[PLAYERn]`, `[AIn]`). The AI interface has no call for either.

use bot_protocol::Controller;

/// `(ally team, left, top, right, bottom)` as fractions of the map, for every `[ALLYTEAMn]` section that has a box.
pub fn start_rects(script: &str) -> Vec<(i32, [f32; 4])> {
    let lower = script.to_ascii_lowercase();
    let mut rects = Vec::new();
    let mut rest = lower.as_str();
    while let Some(at) = rest.find("[allyteam") {
        rest = &rest[at + "[allyteam".len()..];
        let Some(close) = rest.find(']') else { break };
        let Ok(ally_team) = rest[..close].trim().parse::<i32>() else { continue };
        let Some(open) = rest.find('{') else { break };
        let body = &rest[open + 1..rest[open..].find('}').map_or(rest.len(), |end| open + end)];
        let value = |key: &str| {
            body.split(';').find_map(|pair| {
                let (k, v) = pair.split_once('=')?;
                (k.trim() == key).then(|| v.trim().parse::<f32>().ok()).flatten()
            })
        };
        if let (Some(left), Some(top), Some(right), Some(bottom)) =
            (value("startrectleft"), value("startrecttop"), value("startrectright"), value("startrectbottom"))
        {
            rects.push((ally_team, [left, top, right, bottom]));
        }
    }
    rects
}

/// Every `[name] { ... }` section of the script with its body, nested braces kept inside the body, keys lowercased.
fn sections(script: &str) -> Vec<(String, String)> {
    let lower = script.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(open) = lower[i..].find('[') {
        let start = i + open;
        let Some(close) = lower[start..].find(']') else { break };
        let name = lower[start + 1..start + close].trim().to_string();
        let Some(brace) = lower[start + close..].find('{') else { break };
        let body_start = start + close + brace + 1;
        let mut depth = 1;
        let mut j = body_start;
        while j < bytes.len() && depth > 0 {
            match bytes[j] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            j += 1;
        }
        out.push((name, lower[body_start..j.saturating_sub(1).max(body_start)].to_string()));
        // Nested sections are found on their own by continuing just past the opening brace.
        i = body_start;
    }
    out
}

/// `key=value;` pairs at the top level of a section body (nested sections skipped).
fn value(body: &str, key: &str) -> Option<String> {
    let mut depth = 0;
    let mut pair = String::new();
    let mut pairs: Vec<String> = Vec::new();
    for c in body.chars() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            ';' if depth == 0 => pairs.push(std::mem::take(&mut pair)),
            _ if depth == 0 => pair.push(c),
            _ => {}
        }
    }
    pairs.push(pair);
    pairs.iter().find_map(|p| {
        let (k, v) = p.split_once('=')?;
        // A nested section's name may precede the key in the same run ("[options] { .. } team=1" is split by ';').
        let k = k.rsplit(']').next().unwrap_or(k).trim();
        (k == key).then(|| v.trim().to_string())
    })
}

/// Who plays each team: `(team, controller)` for every non-spectator player and every AI in the script.
pub fn controllers(script: &str) -> Vec<(i32, Controller)> {
    let all = sections(script);
    let mut out = Vec::new();
    for (name, body) in &all {
        if name.starts_with("player") && name[6..].chars().all(|c| c.is_ascii_digit()) {
            if value(body, "spectator").is_some_and(|v| v == "1") {
                continue;
            }
            let Some(team) = value(body, "team").and_then(|t| t.parse::<i32>().ok()) else { continue };
            let skill = value(body, "skill").and_then(|v| v.trim_matches(|c| c == '[' || c == ']').parse::<f32>().ok());
            out.push((team, Controller::Person { name: value(body, "name").unwrap_or_default(), skill }));
        } else if name.starts_with("ai") && name[2..].chars().all(|c| c.is_ascii_digit()) && name.len() > 2 {
            let Some(team) = value(body, "team").and_then(|t| t.parse::<i32>().ok()) else { continue };
            let options = body.find("[options]").and_then(|at| {
                let rest = &body[at..];
                let open = rest.find('{')?;
                let close = rest[open..].find('}')?;
                Some(rest[open + 1..open + close].to_string())
            });
            let profile = options.as_deref().and_then(|o| value(o, "profile")).filter(|p| !p.is_empty());
            out.push((
                team,
                Controller::Ai {
                    name: value(body, "name").unwrap_or_default(),
                    short_name: value(body, "shortname").unwrap_or_default(),
                    version: value(body, "version").unwrap_or_default(),
                    profile,
                },
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use bot_protocol::Controller;

    #[test]
    fn reads_the_seats_of_a_lobby_script_and_of_the_arena() {
        let lobby = "[GAME]\n{\n[player0] { team=0; name=[gecko]u6bkep; rank=4; skill=[19.24]; spectator=0; }\n[player1] { name=watcher; spectator=1; }\n[ai0] { [options] { ally_base=true; profile=medium; } team=1; host=0; name=BARbarianAI(1); shortname=BARb; }\n[team0] { allyteam=0; }\n}";
        let seats = super::controllers(lobby);
        assert_eq!(seats.len(), 2);
        assert_eq!(seats[0], (0, Controller::Person { name: "[gecko]u6bkep".into(), skill: Some(19.24) }));
        assert_eq!(seats[1], (1, Controller::Ai { name: "barbarianai(1)".into(), short_name: "barb".into(), version: String::new(), profile: Some("medium".into()) }));
        let arena = "[GAME]\n{\n\t[PLAYER0] { Name=arena; Spectator=1; }\n\t[AI0] { Name=ai0; Team=0; Host=0; ShortName=WReason; Version=0.1; }\n\t[AI1] { Name=ai1; Team=1; Host=0; ShortName=BARb; Version=stable; [OPTIONS] { profile=hard_aggressive; random_seed=1; disabledunits=; } }\n}";
        let seats = super::controllers(arena);
        assert_eq!(seats.len(), 2);
        assert!(matches!(&seats[0].1, Controller::Ai { short_name, profile: None, .. } if short_name == "wreason"));
        assert!(matches!(&seats[1].1, Controller::Ai { short_name, profile: Some(p), version, .. } if short_name == "barb" && p == "hard_aggressive" && version == "stable"));
    }

    #[test]
    fn reads_boxes_and_skips_ally_teams_without_one() {
        let script = "[GAME]\n{\n[TEAM0] { AllyTeam=0; }\n[ALLYTEAM0] { NumAllies=0; StartRectLeft=0; StartRectTop=0; StartRectRight=0.3; StartRectBottom=0.25; }\n[allyteam1]\n{\nnumallies=0;\n}\n[ALLYTEAM2] { StartRectLeft=0.7; StartRectTop=0.7; StartRectRight=1; StartRectBottom=1; }\n}";
        assert_eq!(super::start_rects(script), vec![(0, [0.0, 0.0, 0.3, 0.25]), (2, [0.7, 0.7, 1.0, 1.0])]);
    }
}
