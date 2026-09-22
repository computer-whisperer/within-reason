//! The policy: the player's Lua script as a second pair of hands beside Jev (`docs/design/2026-09-22-policy-replay.md`,
//! "The runtime"). Once a second `decide(S)` is called with the picture as a table and returns orders for the actors
//! it names; those become answers at probability one and are played by the same actuators as Jev's. The Lua state
//! lives for the game (globals persist across seconds and amendments); an amendment runs in it, replacing what it
//! redefines, and the script's text is kept as named blocks so the policy shown to the player stays flat.

use std::collections::BTreeMap;
use std::time::Instant;

use mlua::{Lua, LuaOptions, LuaSerdeExt, StdLib, Table, Value as LuaValue};
use serde_json::{Value, json};

/// The helpers every script may use: plain substring tests that take nil as "".
const HELPERS: &str = r#"
function has(s, w) if s == nil then return false end if type(s) ~= "string" then s = tostring(s) end return string.find(s, w, 1, true) ~= nil end
function starts(s, w) if s == nil then return false end if type(s) ~= "string" then s = tostring(s) end return string.sub(s, 1, #w) == w end
"#;

/// One actor's order for this second.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Order {
    pub choice: String,
    /// `where`, `whom`, `how_many`, `where_scout`, `where_extractor`.
    pub params: BTreeMap<String, String>,
}

/// What the policy did since the player last looked (the report's line).
#[derive(Clone, Debug, Default)]
pub struct PolicyStats {
    pub runs: u32,
    pub errors: u32,
    pub first_error: Option<String>,
    /// Orders given, by option.
    pub given: BTreeMap<String, u32>,
    /// Orders not carried out: an option the actor was not offered, an unknown actor or place.
    pub illegal: Vec<String>,
    pub ms_total: f32,
}

impl PolicyStats {
    /// Adds a later count to this one (the brain hands its counts to the report each ask).
    pub fn merge(&mut self, other: PolicyStats) {
        self.runs += other.runs;
        self.errors += other.errors;
        if self.first_error.is_none() {
            self.first_error = other.first_error;
        }
        for (k, v) in other.given {
            *self.given.entry(k).or_insert(0) += v;
        }
        if self.illegal.len() < 20 {
            self.illegal.extend(other.illegal);
        }
        self.ms_total += other.ms_total;
    }

    pub fn words(&self) -> String {
        if self.runs == 0 {
            return "your policy has not run since your last turn".into();
        }
        let mut line = format!("your policy ran {} times since your last turn ({:.0} ms in all)", self.runs, self.ms_total);
        if self.errors > 0 {
            line += &format!("; {} runs raised an error (the first: {})", self.errors, self.first_error.as_deref().unwrap_or("?"));
        }
        if self.given.is_empty() {
            line += "; it gave no orders";
        } else {
            let mut given: Vec<(&String, &u32)> = self.given.iter().collect();
            given.sort_by(|a, b| b.1.cmp(a.1));
            line += &format!("; orders given: {}", given.iter().map(|(k, v)| format!("{k} {v}")).collect::<Vec<_>>().join(", "));
        }
        if !self.illegal.is_empty() {
            line += &format!("; {} orders could not be carried out (e.g. {})", self.illegal.len(), self.illegal[0]);
        }
        line
    }
}

pub struct Policy {
    lua: Lua,
    /// The script as top-level blocks: (the name a block defines, its text), in order; unnamed blocks are kept as
    /// they came.
    blocks: Vec<(Option<String>, String)>,
    pub stats: PolicyStats,
    /// How many times the script was set or amended.
    pub version: u32,
}

fn fresh_lua() -> Result<Lua, String> {
    let lua = Lua::new_with(StdLib::STRING | StdLib::TABLE | StdLib::MATH, LuaOptions::default()).map_err(|e| e.to_string())?;
    lua.load(HELPERS).exec().map_err(|e| e.to_string())?;
    Ok(lua)
}

/// A line that opens a new top-level block: at column 0 and not the tail of the block before it.
fn opens_block(line: &str) -> bool {
    let first = line.chars().next();
    if first.is_none_or(char::is_whitespace) {
        return false;
    }
    let word = line.split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap_or_default();
    !matches!(word, "end" | "else" | "elseif" | "until") && !line.starts_with(['}', ')', ']'])
}

/// The name a block defines: `function NAME`, `local function NAME`, `NAME = ...`.
fn block_name(text: &str) -> Option<String> {
    let code = text.lines().find(|l| !l.trim_start().starts_with("--") && !l.trim().is_empty())?;
    let mut words = code.split_whitespace();
    let first = words.next()?;
    let ident = |w: &str| w.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.').collect::<String>();
    match first {
        "function" => words.next().map(ident).filter(|n| !n.is_empty()),
        "local" if words.clone().next() == Some("function") => words.nth(1).map(ident).filter(|n| !n.is_empty()),
        _ => {
            let name = ident(first);
            let rest = code[name.len()..].trim_start();
            (!name.is_empty() && rest.starts_with('=') && !rest.starts_with("==")).then_some(name)
        }
    }
}

/// The script split into its top-level blocks, comments attached to the block that follows them.
pub(super) fn split_blocks(text: &str) -> Vec<(Option<String>, String)> {
    let mut blocks: Vec<(Option<String>, String)> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut comments: Vec<&str> = Vec::new();
    let flush = |current: &mut Vec<&str>, blocks: &mut Vec<(Option<String>, String)>| {
        if current.iter().any(|l| !l.trim().is_empty()) {
            let text = current.join("\n").trim_end().to_string();
            blocks.push((block_name(&text), text));
        }
        current.clear();
    };
    for line in text.lines() {
        if line.trim().is_empty() {
            if current.is_empty() {
                comments.clear();
            } else {
                current.push(line);
            }
            continue;
        }
        if line.starts_with("--") {
            if current.is_empty() {
                comments.push(line);
            } else {
                current.push(line);
            }
            continue;
        }
        if opens_block(line) {
            flush(&mut current, &mut blocks);
            current.append(&mut comments);
        }
        current.push(line);
    }
    flush(&mut current, &mut blocks);
    blocks
}

impl Policy {
    /// A syntax check without running anything: what the `policy` tool answers at once.
    pub fn check(script: &str) -> Result<(), String> {
        let lua = fresh_lua()?;
        lua.load(script).set_name("policy").into_function().map(|_| ()).map_err(|e| e.to_string())
    }

    /// A whole script in a fresh state. On an error nothing changes.
    pub fn set(script: &str) -> Result<Policy, String> {
        let lua = fresh_lua()?;
        lua.load(script).set_name("policy").exec().map_err(|e| e.to_string())?;
        let decide: Result<mlua::Function, _> = lua.globals().get("decide");
        if decide.is_err() {
            return Err("the script defines no `decide` function".into());
        }
        Ok(Policy { lua, blocks: split_blocks(script), stats: PolicyStats::default(), version: 1 })
    }

    /// A chunk run in the living state: what it redefines is replaced, in Lua and in the text. On an error nothing
    /// changes in the text (the state may have run part of the chunk).
    pub fn amend(&mut self, chunk: &str) -> Result<Vec<String>, String> {
        self.lua.load(chunk).set_name("amendment").exec().map_err(|e| e.to_string())?;
        let mut replaced = Vec::new();
        for (name, text) in split_blocks(chunk) {
            match name.as_ref().and_then(|n| self.blocks.iter().position(|(b, _)| b.as_ref() == Some(n))) {
                Some(i) => {
                    self.blocks[i].1 = text;
                    replaced.push(format!("{} replaced", name.unwrap_or_default()));
                }
                None => {
                    replaced.push(format!("{} added", name.clone().unwrap_or_else(|| "a block".into())));
                    self.blocks.push((name, text));
                }
            }
        }
        self.version += 1;
        Ok(replaced)
    }

    /// The policy in force, flat.
    pub fn text(&self) -> String {
        self.blocks.iter().map(|(_, t)| t.as_str()).collect::<Vec<_>>().join("\n\n")
    }

    pub fn lines(&self) -> usize {
        self.blocks.iter().map(|(_, t)| t.lines().count() + 1).sum::<usize>().saturating_sub(1)
    }

    /// `decide(S)` over this second's picture: the orders by actor name.
    pub(super) fn decide(&mut self, state: &Value) -> Result<BTreeMap<String, Order>, String> {
        let started = Instant::now();
        self.stats.runs += 1;
        let result = self.call(state);
        self.stats.ms_total += started.elapsed().as_secs_f32() * 1000.0;
        if let Err(e) = &result {
            self.stats.errors += 1;
            if self.stats.first_error.is_none() {
                self.stats.first_error = Some(e.chars().take(200).collect());
            }
        }
        result
    }

    fn call(&self, state: &Value) -> Result<BTreeMap<String, Order>, String> {
        let options = mlua::serde::SerializeOptions::new().serialize_none_to_null(false).serialize_unit_to_null(false);
        let table: LuaValue = self.lua.to_value_with(state, options).map_err(|e| e.to_string())?;
        let decide: mlua::Function = self.lua.globals().get("decide").map_err(|_| "no `decide` function".to_string())?;
        let returned: LuaValue = decide.call(table).map_err(|e| e.to_string())?;
        let mut orders = BTreeMap::new();
        let LuaValue::Table(t) = returned else { return Ok(orders) };
        for pair in t.pairs::<String, LuaValue>() {
            let (actor, value) = pair.map_err(|e| e.to_string())?;
            match value {
                LuaValue::String(s) => {
                    orders.insert(actor, Order { choice: s.to_str().map_err(|e| e.to_string())?.to_string(), params: BTreeMap::new() });
                }
                LuaValue::Table(o) => {
                    let mut choice = None;
                    let mut params = BTreeMap::new();
                    for pair in o.pairs::<String, LuaValue>() {
                        let (k, v) = pair.map_err(|e| e.to_string())?;
                        let v = match v {
                            LuaValue::String(s) => s.to_str().map_err(|e| e.to_string())?.to_string(),
                            LuaValue::Integer(i) => i.to_string(),
                            LuaValue::Number(n) => n.to_string(),
                            _ => continue,
                        };
                        if k == "do" { choice = Some(v) } else { params.insert(k, v); }
                    }
                    if let Some(choice) = choice {
                        orders.insert(actor, Order { choice, params });
                    }
                }
                _ => {}
            }
        }
        Ok(orders)
    }

    /// The table the script reads: the picture's state with each actor's `kind` and `options` (the menu's words),
    /// the groups with their centres, the places with coordinates; the replay harness's shape exactly.
    pub(super) fn state_for(picture_state: &Value, options: &BTreeMap<String, BTreeMap<String, Value>>, groups: &[Value], places: &[Value], frame: i32) -> Value {
        let mut state = json!({ "clock": super::picture::clock(frame), "frame": frame });
        let mut actors = serde_json::Map::new();
        if let Some(entries) = picture_state["actors"].as_object() {
            for (name, entry) in entries {
                let mut a = entry.clone();
                let kind = if name.starts_with("group_") { "group" } else if name.starts_with("lab_") || name.starts_with("plant_") { "lab" } else { "builder" };
                a["kind"] = json!(kind);
                a["options"] = json!(options.get(name).cloned().unwrap_or_default());
                if a.get("enemies_at_our_extractors").is_none() {
                    a["enemies_at_our_extractors"] = json!([]);
                }
                actors.insert(name.clone(), a);
            }
        }
        state["actors"] = Value::Object(actors);
        let mut group_map = serde_json::Map::new();
        for g in groups {
            let name = format!("group_{}", g["name"].as_str().unwrap_or_default());
            group_map.insert(name, json!({ "x": g["at"][0], "z": g["at"][1], "members": g["members"].as_array().map_or(0, Vec::len), "task": g["task"]["kind"] }));
        }
        state["groups"] = Value::Object(group_map);
        let mut place_map = serde_json::Map::new();
        for p in places {
            place_map.insert(p["name"].as_str().unwrap_or_default().to_string(), json!({ "x": p["x"], "z": p["z"], "spot": p["spot"] }));
        }
        if let Some(words) = picture_state["places"].as_object() {
            for (name, entry) in words {
                let slot = place_map.entry(name.clone()).or_insert_with(|| json!({}));
                if let (Some(slot), Some(entry)) = (slot.as_object_mut(), entry.as_object()) {
                    for (k, v) in entry {
                        slot.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        state["places"] = Value::Object(place_map);
        for key in ["economy", "enemy", "ours"] {
            state[key] = picture_state.get(key).cloned().unwrap_or_else(|| json!({}));
        }
        state
    }
}

/// A `Table` is not `Send`; the policy lives on the brain's thread only.
#[allow(dead_code)]
fn _table_is_thread_local(_: Table) {}

#[cfg(test)]
mod tests {
    use super::*;

    const SCRIPT: &str = r#"
-- the opening
step = step or 1
COM = { "extractor", "armsolar" }

function commander(S, a)
  if a.options.extractor then return { ["do"] = "extractor", where = "spot_45" } end
end

function decide(S)
  local orders = {}
  for name, a in pairs(S.actors) do
    if name == "commander" then orders[name] = commander(S, a) end
    if a.kind == "group" and a.options.hold and has(a.enemies_near, "we outweigh it") then orders[name] = { ["do"] = "engage", whom = "party_1" } end
  end
  step = step + 1
  orders.plant_1 = "armflash"
  return orders
end
"#;

    fn state() -> Value {
        json!({
            "clock": "0:02", "frame": 60,
            "actors": {
                "commander": { "kind": "builder", "doing": "idle", "options": { "extractor": "an extractor", "armsolar": "a solar" }, "enemies_at_our_extractors": [] },
                "group_A": { "kind": "group", "doing": "holding", "enemies_near": "party_1 (1 armflash): we outweigh it", "options": { "hold": "", "engage": "" }, "enemies_at_our_extractors": [] },
                "plant_1": { "kind": "lab", "options": { "armflash": "", "nothing": "" }, "enemies_at_our_extractors": [] }
            },
            "groups": {}, "places": { "spot_45": { "x": 1, "z": 2 } }, "economy": {}, "enemy": {}, "ours": {}
        })
    }

    #[test]
    fn a_script_decides_and_keeps_its_state() {
        let mut policy = Policy::set(SCRIPT).unwrap();
        let orders = policy.decide(&state()).unwrap();
        assert_eq!(orders["commander"].choice, "extractor");
        assert_eq!(orders["commander"].params["where"], "spot_45");
        assert_eq!(orders["group_A"].choice, "engage");
        assert_eq!(orders["plant_1"].choice, "armflash");
        policy.decide(&state()).unwrap();
        let step: i64 = policy.lua.globals().get("step").unwrap();
        assert_eq!(step, 3, "globals persist between seconds");
        assert_eq!(policy.stats.runs, 2);
    }

    #[test]
    fn an_amendment_replaces_the_block_in_place_and_keeps_the_state() {
        let mut policy = Policy::set(SCRIPT).unwrap();
        policy.decide(&state()).unwrap();
        let before = policy.blocks.len();
        let done = policy.amend("function commander(S, a)\n  return { [\"do\"] = \"armsolar\" }\nend\n").unwrap();
        assert_eq!(done, vec!["commander replaced"]);
        assert_eq!(policy.blocks.len(), before);
        assert!(policy.text().contains("armsolar\" }") && !policy.text().contains("where = \"spot_45\""));
        assert_eq!(policy.decide(&state()).unwrap()["commander"].choice, "armsolar");
        let step: i64 = policy.lua.globals().get("step").unwrap();
        assert_eq!(step, 3, "an amendment keeps the globals");
        assert!(policy.amend("function group(S, name, a) return nil end").unwrap()[0].ends_with("added"));
    }

    #[test]
    fn errors_are_reported_and_counted() {
        assert!(Policy::check("function decide(S) return {} ").is_err());
        assert!(Policy::check(SCRIPT).is_ok());
        assert!(Policy::set("x = 1").is_err(), "no decide");
        let mut policy = Policy::set("function decide(S) return #S.actors.commander.enemies_near end").unwrap();
        assert!(policy.decide(&state()).is_err());
        assert_eq!(policy.stats.errors, 1);
        assert!(policy.stats.first_error.is_some());
        assert!(policy.amend("function decide(S) return {").is_err());
        assert_eq!(policy.version, 1);
    }

    #[test]
    fn blocks_are_split_by_the_name_they_define() {
        let names: Vec<Option<String>> = split_blocks(SCRIPT).into_iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec![Some("step".into()), Some("COM".into()), Some("commander".into()), Some("decide".into())]);
        let blocks = split_blocks("-- about x\nlocal function x()\n  return 1\nend\n\nif a then\n  b()\nend\n");
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].0.as_deref(), Some("x"));
        assert!(blocks[0].1.starts_with("-- about x"));
        assert_eq!(blocks[1].0, None);
    }

    #[test]
    fn the_state_carries_kinds_options_groups_and_places() {
        let picture = json!({ "actors": { "commander": { "doing": "idle" }, "group_A": { "doing": "holding" } }, "places": { "home": { "what": "our start" } }, "economy": { "metal": "low" }, "enemy": {}, "ours": {} });
        let mut options = BTreeMap::new();
        options.insert("commander".to_string(), BTreeMap::from([("extractor".to_string(), json!("words"))]));
        let groups = vec![json!({ "name": "A", "members": [1, 2], "at": [10, 20], "task": { "kind": "hold" } })];
        let places = vec![json!({ "name": "home", "x": 1, "z": 2, "spot": null })];
        let s = Policy::state_for(&picture, &options, &groups, &places, 90);
        assert_eq!(s["actors"]["commander"]["kind"], "builder");
        assert_eq!(s["actors"]["commander"]["options"]["extractor"], "words");
        assert_eq!(s["actors"]["group_A"]["options"], json!({}));
        assert_eq!(s["actors"]["group_A"]["enemies_at_our_extractors"], json!([]));
        assert_eq!(s["groups"]["group_A"]["members"], 2);
        assert_eq!(s["places"]["home"]["what"], "our start");
        assert_eq!(s["places"]["home"]["x"], 1);
        assert_eq!(s["clock"], "0:03");
    }
}
