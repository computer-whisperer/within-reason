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

/// Every `[name] { ... }` section of the script: the name lowercased, the body as written (nested braces kept).
fn sections(script: &str) -> Vec<(String, &str)> {
    let lower = script.to_ascii_lowercase();
    let bytes = script.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(open) = lower[i..].find('[') {
        let start = i + open;
        let Some(close) = lower[start..].find(']') else { break };
        let name = lower[start + 1..start + close].trim().to_string();
        // A section's brace follows its name with nothing but whitespace between: `name=[gecko]thebluegecko;` and
        // `skill=[6.02];` are values, and taking them for sections lost every section after them (the person's
        // seat "not in the script", our first seat not ours, in every game with people to 2026-09-27).
        let after = start + close + 1;
        let brace = lower[after..].len() - lower[after..].trim_start().len();
        if !lower[after..].trim_start().starts_with('{') {
            i = after;
            continue;
        }
        let body_start = after + brace + 1;
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
        out.push((name, &script[body_start..j.saturating_sub(1).max(body_start)]));
        // Nested sections are found on their own by continuing just past the opening brace.
        i = body_start;
    }
    out
}

/// The value of `key=value;` at the top level of a section body (keys matched without case, nested sections skipped).
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
        k.eq_ignore_ascii_case(key).then(|| v.trim().to_string())
    })
}

/// The teams the script declares (`[TEAMn]`): a team the engine lists beyond these is its Gaia team.
pub fn teams(script: &str) -> Vec<i32> {
    sections(script).into_iter().filter_map(|(name, _)| name.strip_prefix("team").and_then(|n| n.parse::<i32>().ok())).collect()
}

/// Each team's lobby colour: `(team, [r, g, b])` from `rgbcolor=r g b` (0-1) of every `[TEAMn]` that has one.
pub fn colors(script: &str) -> Vec<(i32, [f32; 3])> {
    sections(script)
        .into_iter()
        .filter_map(|(name, body)| {
            let team = name.strip_prefix("team")?.parse::<i32>().ok()?;
            let parts: Vec<f32> = value(body, "rgbcolor")?.split_whitespace().filter_map(|v| v.parse::<f32>().ok()).collect();
            (parts.len() == 3).then(|| (team, [parts[0], parts[1], parts[2]]))
        })
        .collect()
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
            let options = body.to_ascii_lowercase().find("[options]").and_then(|at| {
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
        assert_eq!(seats[1], (1, Controller::Ai { name: "BARbarianAI(1)".into(), short_name: "BARb".into(), version: String::new(), profile: Some("medium".into()) }));
        assert_eq!(super::teams(lobby), vec![0]);
        let coloured = "[GAME]\n{\n[team0] { allyteam=0; rgbcolor=0.63922 0.08235 0.88235; side=Armada; }\n[team1] { allyteam=1; side=Random; }\n}";
        assert_eq!(super::colors(coloured), vec![(0, [0.63922, 0.08235, 0.88235])]);
        // Brackets inside values are not sections (the lobby's clan tags and skill brackets).
        let tagged = "[GAME]\n{\n[player0] { team=0; name=[gecko]thebluegecko; skill=[34.34]; spectator=0; }\n[player1] { name=[gecko]u6bkep; skill=[19.24]; spectator=1; }\n[ai0] { team=1; host=3; name=WReason0.1(1); shortname=WReason; }\n[mapoptions]\n{\n}\n[team1] { allyteam=1; side=Random; }\n[team0] { allyteam=0; }\n}";
        let seats = super::controllers(tagged);
        assert_eq!(seats.len(), 2, "{seats:?}");
        assert_eq!(seats[0], (0, Controller::Person { name: "[gecko]thebluegecko".into(), skill: Some(34.34) }));
        assert!(matches!(&seats[1], (1, Controller::Ai { short_name, .. }) if short_name == "WReason"));
        assert_eq!(super::teams(tagged), vec![1, 0]);
        let arena = "[GAME]\n{\n\t[PLAYER0] { Name=arena; Spectator=1; }\n\t[AI0] { Name=ai0; Team=0; Host=0; ShortName=WReason; Version=0.1; }\n\t[AI1] { Name=ai1; Team=1; Host=0; ShortName=BARb; Version=stable; [OPTIONS] { profile=hard_aggressive; random_seed=1; disabledunits=; } }\n}";
        let seats = super::controllers(arena);
        assert_eq!(seats.len(), 2);
        assert!(matches!(&seats[0].1, Controller::Ai { short_name, profile: None, .. } if short_name == "WReason"));
        assert!(matches!(&seats[1].1, Controller::Ai { short_name, profile: Some(p), version, .. } if short_name == "BARb" && p == "hard_aggressive" && version == "stable"));
    }

    #[test]
    fn reads_boxes_and_skips_ally_teams_without_one() {
        let script = "[GAME]\n{\n[TEAM0] { AllyTeam=0; }\n[ALLYTEAM0] { NumAllies=0; StartRectLeft=0; StartRectTop=0; StartRectRight=0.3; StartRectBottom=0.25; }\n[allyteam1]\n{\nnumallies=0;\n}\n[ALLYTEAM2] { StartRectLeft=0.7; StartRectTop=0.7; StartRectRight=1; StartRectBottom=1; }\n}";
        assert_eq!(super::start_rects(script), vec![(0, [0.0, 0.0, 0.3, 0.25]), (2, [0.7, 0.7, 1.0, 1.0])]);
    }
}

#[cfg(test)]
mod probe {
    /// `WR_SCRIPT=<file> cargo test -p ai-shim -- --ignored probe` prints what the parser reads from a real script.
    #[test]
    #[ignore]
    fn sections_of_a_recorded_script() {
        let script = std::fs::read_to_string(std::env::var("WR_SCRIPT").unwrap()).unwrap();
        for (name, body) in super::sections(&script) {
            println!("[{name}] {} bytes: {}", body.len(), body.replace('\n', " ").chars().take(80).collect::<String>());
        }
        println!("teams {:?}", super::teams(&script));
        println!("controllers {:?}", super::controllers(&script));
        println!("colors {:?}", super::colors(&script));
    }
}
