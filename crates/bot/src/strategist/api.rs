//! The API backend (`docs/design/2026-09-27-api-backend.md`): an OpenAI-compatible chat-completions client with tools
//! for models billed per token on an API key, Fireworks.ai first (`--commander-model fw:<model id>`) and any
//! OpenAI-compatible endpoint (`api:<model>`). The session owns its message list; the tools are the MCP tool list in
//! OpenAI's shape, dispatched in-process through the same path as the MCP server; the turn ends at the `orders`
//! call's `wait` with no closing request. The key is read from `~/.config/within-reason/fireworks.env` (`api.env`)
//! or the environment variable of the same name, and is never printed or written: the repository is public.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::shared::Shared;
use super::transcript::Transcript;

const CONFIG_DIR: &str = ".config/within-reason";
const FIREWORKS_BASE_URL: &str = "https://api.fireworks.ai/inference/v1";
const DEFAULT_MAX_TOKENS: u64 = 8192;
/// A game's spend cap in USD, unless the config file says otherwise (the user, 2026-09-27: one dollar a game to start).
const DEFAULT_COST_CAP: f64 = 1.0;
/// The message list is trimmed to this many characters before each call, oldest turns first (about 80k tokens).
const CONTEXT_CHARS_VAR: &str = "API_CONTEXT_CHARS";
const DEFAULT_CONTEXT_CHARS: usize = 320_000;
/// One call's timeout: a slow open model writing a long turn.
const TIMEOUT: Duration = Duration::from_secs(300);
/// A 429 or a 5xx is retried this many times, waiting `retry-after` (at most `RETRY_WAIT_MAX`).
const RETRIES: u32 = 2;
const RETRY_WAIT_MAX: Duration = Duration::from_secs(10);

/// Where the model is and how to reach it, from the model name's prefix and its config file.
pub(super) struct Endpoint {
    pub base_url: String,
    key: String,
    pub model: String,
    max_tokens: u64,
    /// USD per million tokens: fresh input, cached input, output.
    price_input: f64,
    price_cached: f64,
    price_output: f64,
    /// USD a game may spend; the player falls silent when it is reached.
    pub cost_cap: f64,
}

impl Endpoint {
    pub fn is_api_model(model: &str) -> bool {
        model.starts_with("fw:") || model.starts_with("api:")
    }

    /// `fw:<model>` reads `fireworks.env` (`FIREWORKS_API_KEY`, `FIREWORKS_BASE_URL`, `FIREWORKS_MAX_TOKENS`);
    /// `api:<model>` reads `api.env` (`API_KEY`, `API_BASE_URL`, `API_MAX_TOKENS`). Both need the model's prices,
    /// `<VAR>_PRICE_INPUT` and `<VAR>_PRICE_OUTPUT` in USD per million tokens (`<VAR>_PRICE_CACHED` for cached input,
    /// else the input price), because the game's spend is capped at `<VAR>_COST_CAP` USD (default 1): without a price
    /// the cap cannot be kept and the session does not start. An environment variable of the same name overrides the
    /// file.
    pub fn for_model(model: &str) -> std::io::Result<Endpoint> {
        let (prefix, name) = model.split_once(':').ok_or_else(|| std::io::Error::other(format!("{model}: not an API model")))?;
        let (var, file, default_base) = match prefix {
            "fw" => ("FIREWORKS", "fireworks.env", Some(FIREWORKS_BASE_URL)),
            "api" => ("API", "api.env", None),
            _ => return Err(std::io::Error::other(format!("{model}: not an API model"))),
        };
        let path = PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(CONFIG_DIR).join(file);
        let mut values: BTreeMap<String, String> = BTreeMap::new();
        if let Ok(text) = std::fs::read_to_string(&path) {
            for line in text.lines().map(str::trim).filter(|l| !l.starts_with('#')) {
                if let Some((k, v)) = line.split_once('=') {
                    values.insert(k.trim().to_string(), v.trim().trim_matches('"').trim_matches('\'').to_string());
                }
            }
        }
        let get = |k: &str| {
            let full = format!("{var}_{k}");
            std::env::var(&full).ok().filter(|v| !v.trim().is_empty()).or_else(|| values.get(&full).cloned()).filter(|v| !v.is_empty())
        };
        let key = if prefix == "fw" { get("API_KEY") } else { get("KEY") }.ok_or_else(|| std::io::Error::other(format!("no key for {prefix}: put {var}_{}KEY in {}", if prefix == "fw" { "API_" } else { "" }, path.display())))?;
        let base_url = get("BASE_URL").or_else(|| default_base.map(String::from)).ok_or_else(|| std::io::Error::other(format!("no {var}_BASE_URL in {}", path.display())))?;
        let max_tokens = get("MAX_TOKENS").and_then(|v| v.parse().ok()).unwrap_or(DEFAULT_MAX_TOKENS);
        let price = |k: &str| get(k).and_then(|v| v.parse::<f64>().ok()).filter(|p| p.is_finite() && *p >= 0.0);
        let (Some(price_input), Some(price_output)) = (price("PRICE_INPUT"), price("PRICE_OUTPUT")) else {
            return Err(std::io::Error::other(format!("no {var}_PRICE_INPUT and {var}_PRICE_OUTPUT (USD per million tokens) in {}: the cost cap cannot be kept without the model's prices", path.display())));
        };
        let price_cached = price("PRICE_CACHED").unwrap_or(price_input);
        let cost_cap = price("COST_CAP").unwrap_or(DEFAULT_COST_CAP);
        Ok(Endpoint { base_url: base_url.trim_end_matches('/').to_string(), key, model: name.to_string(), max_tokens, price_input, price_cached, price_output, cost_cap })
    }

    /// What a call cost, from the usage it reported (`prompt_tokens` counts the cached tokens too).
    pub fn cost(&self, prompt_tokens: u64, cached_tokens: u64, completion_tokens: u64) -> f64 {
        let cached = cached_tokens.min(prompt_tokens);
        ((prompt_tokens - cached) as f64 * self.price_input + cached as f64 * self.price_cached + completion_tokens as f64 * self.price_output) / 1e6
    }

    pub fn describe(&self) -> String {
        format!("the API at {} ({}; ${}/${} per million tokens in/out, the cap ${} a game)", self.base_url, self.model, self.price_input, self.price_output, self.cost_cap)
    }
}

/// One player session on the API: its own message list, trimmed to the budget; a thread per turn.
pub(super) struct ApiSession {
    endpoint: Arc<Endpoint>,
    effort: String,
    tools: Arc<Vec<Value>>,
    messages: Arc<Mutex<Vec<Value>>>,
    /// `reasoning_effort` is sent until the endpoint refuses it.
    reasoning: Arc<AtomicBool>,
    /// Cleared by `end`: a call still in flight then drops its answer instead of playing it. The driver replaces a
    /// session whose turn passed the cap, and the replaced session's thread keeps waiting on the endpoint; its late
    /// `orders` would otherwise land in the next session's turn and its `wait` end that turn (fw-deepseek-v41-flash-1:
    /// the second turn recorded as over at 29 s while its own call ran 165 s).
    alive: Arc<AtomicBool>,
    agent: ureq::Agent,
    shared: Arc<Shared>,
    transcript: Arc<Transcript>,
    turn_done_tx: Sender<()>,
}

impl ApiSession {
    pub fn start(endpoint: Endpoint, effort: &str, system_prompt: &str, tools: Vec<Value>, shared: Arc<Shared>, transcript: Arc<Transcript>, turn_done_tx: Sender<()>) -> ApiSession {
        let config = ureq::Agent::config_builder().timeout_global(Some(TIMEOUT)).http_status_as_error(false).user_agent("within-reason").build();
        let effort = match effort {
            "xhigh" | "max" => "high".to_string(),
            other => other.to_string(),
        };
        ApiSession {
            endpoint: Arc::new(endpoint),
            effort,
            tools: Arc::new(tools),
            messages: Arc::new(Mutex::new(vec![json!({ "role": "system", "content": system_prompt })])),
            reasoning: Arc::new(AtomicBool::new(true)),
            alive: Arc::new(AtomicBool::new(true)),
            agent: ureq::Agent::new_with_config(config),
            shared,
            transcript,
            turn_done_tx,
        }
    }

    /// One turn: the report as a user message, then calls until the model's `orders` carried its `wait` (the turn is
    /// over: no closing request) or it answered without a tool call. The transcript gets the CLI backends' lines.
    /// The session is over: whatever its thread is still waiting for is dropped when it comes.
    pub fn end(&self) {
        self.alive.store(false, Ordering::Relaxed);
    }

    pub fn send(&self, prompt: &str) -> bool {
        if self.shared.cost_capped.load(Ordering::Relaxed) {
            return false;
        }
        self.messages.lock().unwrap().push(json!({ "role": "user", "content": prompt }));
        let (endpoint, effort, tools, messages, reasoning, alive, agent, shared, transcript, done) = (
            self.endpoint.clone(),
            self.effort.clone(),
            self.tools.clone(),
            self.messages.clone(),
            self.reasoning.clone(),
            self.alive.clone(),
            self.agent.clone(),
            self.shared.clone(),
            self.transcript.clone(),
            self.turn_done_tx.clone(),
        );
        let budget = std::env::var(CONTEXT_CHARS_VAR).ok().and_then(|v| v.parse().ok()).unwrap_or(DEFAULT_CONTEXT_CHARS);
        std::thread::spawn(move || {
            let (mut api_ms, mut calls, mut input, mut output, mut cached) = (0u128, 0u32, 0u64, 0u64, 0u64);
            let mut ended_by = "response";
            let mut turn_cost = 0.0;
            let mut spent = shared.spent_micro_usd.load(Ordering::Relaxed) as f64 / 1e6;
            loop {
                let body = {
                    let mut list = messages.lock().unwrap();
                    trim(&mut list, budget);
                    let mut body = json!({ "model": endpoint.model, "messages": *list, "tools": *tools, "tool_choice": "auto", "max_tokens": endpoint.max_tokens });
                    if reasoning.load(Ordering::Relaxed) && !effort.is_empty() {
                        body["reasoning_effort"] = json!(effort);
                    }
                    body
                };
                let t0 = Instant::now();
                let response = match post(&agent, &endpoint, body, &reasoning) {
                    Ok(v) => v,
                    Err(problem) => {
                        eprintln!("[strategist] api: {problem}");
                        transcript.record(json!({ "kind": "error", "message": problem }));
                        ended_by = "error";
                        break;
                    }
                };
                api_ms += t0.elapsed().as_millis();
                calls += 1;
                if !alive.load(Ordering::Relaxed) {
                    // The driver gave up on this turn and replaced the session: the answer is not played.
                    transcript.record(json!({ "kind": "late", "message": format!("the endpoint answered after the session was replaced ({} s): dropped", api_ms / 1000) }));
                    return;
                }
                let usage = &response["usage"];
                let (call_in, call_out, call_cached) = (usage["prompt_tokens"].as_u64().unwrap_or(0), usage["completion_tokens"].as_u64().unwrap_or(0), usage["prompt_tokens_details"]["cached_tokens"].as_u64().unwrap_or(0));
                input += call_in;
                output += call_out;
                cached += call_cached;
                let call_cost = endpoint.cost(call_in, call_cached, call_out);
                turn_cost += call_cost;
                spent = (shared.spent_micro_usd.fetch_add((call_cost * 1e6).round() as u64, Ordering::Relaxed) as f64 + call_cost * 1e6) / 1e6;
                // The cap: this response's tool calls still land (orders written are orders sent), then the player
                // is silent for the rest of the game and the hands carry on under its last packet.
                let capped = spent >= endpoint.cost_cap;
                if capped && !shared.cost_capped.swap(true, Ordering::Relaxed) {
                    eprintln!("[player] cost cap reached: ${spent:.2} of ${:.2} spent on {}; the player is silent from here and the hands carry on under its last orders", endpoint.cost_cap, endpoint.model);
                    transcript.record(json!({ "kind": "cost_cap", "spent_usd": spent, "cap_usd": endpoint.cost_cap }));
                }
                let message = response["choices"][0]["message"].clone();
                let text = message["content"].as_str().unwrap_or_default().to_string();
                let tool_calls: Vec<Value> = message["tool_calls"].as_array().cloned().unwrap_or_default();
                let mut content = Vec::new();
                if !text.is_empty() {
                    content.push(json!({ "type": "text", "text": text }));
                }
                for call in &tool_calls {
                    content.push(json!({ "type": "tool_use", "name": call["function"]["name"], "input": arguments_of(call) }));
                }
                transcript.record(json!({ "kind": "assistant", "message": { "message": { "model": endpoint.model, "content": content } } }));
                let mut assistant = json!({ "role": "assistant", "content": if text.is_empty() { Value::Null } else { json!(text) } });
                if !tool_calls.is_empty() {
                    assistant["tool_calls"] = json!(tool_calls);
                }
                messages.lock().unwrap().push(assistant);
                if tool_calls.is_empty() {
                    if capped {
                        ended_by = "cost_cap";
                    }
                    break;
                }
                let mut over = false;
                for call in &tool_calls {
                    let name = call["function"]["name"].as_str().unwrap_or_default();
                    let name = name.rsplit("__").next().unwrap_or(name);
                    let result = match super::mcp::call_recorded(name, &arguments_of(call), &shared, &transcript) {
                        Ok(text) => text,
                        Err(problem) => format!("REFUSED: {problem}"),
                    };
                    messages.lock().unwrap().push(json!({ "role": "tool", "tool_call_id": call["id"], "content": result }));
                    over |= shared.turn_over.load(Ordering::Relaxed);
                }
                if capped {
                    ended_by = "cost_cap";
                    break;
                }
                if over {
                    ended_by = "wait";
                    break;
                }
            }
            transcript.record(json!({ "kind": "result", "message": {
                "type": "result", "duration_api_ms": api_ms, "num_turns": calls, "ended_by": ended_by,
                "total_cost_usd": turn_cost, "spent_usd": spent, "cap_usd": endpoint.cost_cap,
                "usage": { "input_tokens": input, "output_tokens": output, "cache_read_input_tokens": cached },
                "modelUsage": { endpoint.model.clone(): { "inputTokens": input, "outputTokens": output } },
            } }));
            let _ = done.send(());
        });
        true
    }
}

fn arguments_of(call: &Value) -> Value {
    match &call["function"]["arguments"] {
        Value::String(s) => serde_json::from_str(s).unwrap_or_else(|_| json!({})),
        Value::Object(o) => Value::Object(o.clone()),
        _ => json!({}),
    }
}

/// Drops whole turns, oldest first, until the list fits the budget; the system prompt and the newest turn stay.
fn trim(messages: &mut Vec<Value>, budget: usize) {
    let size = |m: &Value| m.to_string().len();
    while messages.iter().map(size).sum::<usize>() > budget {
        let users: Vec<usize> = messages.iter().enumerate().filter(|(_, m)| m["role"] == "user").map(|(i, _)| i).collect();
        if users.len() < 2 {
            return;
        }
        messages.drain(users[0]..users[1]);
    }
}

/// One request with the retries: a 429 or 5xx waits `retry-after`; a 400 naming `reasoning_effort` drops the
/// parameter for the session and retries once.
fn post(agent: &ureq::Agent, endpoint: &Endpoint, mut body: Value, reasoning: &AtomicBool) -> Result<Value, String> {
    let url = format!("{}/chat/completions", endpoint.base_url);
    let mut retries = 0;
    loop {
        let mut response = agent
            .post(&url)
            .header("Authorization", &format!("Bearer {}", endpoint.key))
            .header("Content-Type", "application/json")
            .send_json(&body)
            .map_err(|e| format!("transport: {e}"))?;
        let status = response.status().as_u16();
        if (status == 429 || status >= 500) && retries < RETRIES {
            let wait = response.headers().get("retry-after").and_then(|v| v.to_str().ok()).and_then(|s| s.trim().parse::<f64>().ok()).map_or(Duration::from_secs(2), Duration::from_secs_f64);
            retries += 1;
            std::thread::sleep(wait.min(RETRY_WAIT_MAX));
            continue;
        }
        let text = response.body_mut().read_to_string().map_err(|e| format!("transport: {e}"))?;
        if status == 400 && text.contains("reasoning_effort") && reasoning.swap(false, Ordering::Relaxed) {
            body.as_object_mut().map(|o| o.remove("reasoning_effort"));
            continue;
        }
        if !(200..300).contains(&status) {
            return Err(format!("HTTP {status}: {}", text.chars().take(300).collect::<String>()));
        }
        return serde_json::from_str(&text).map_err(|e| format!("malformed: {e}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_drops_whole_turns_oldest_first_and_keeps_the_system_prompt() {
        let mut list = vec![
            json!({ "role": "system", "content": "s" }),
            json!({ "role": "user", "content": "a".repeat(100) }),
            json!({ "role": "assistant", "content": null, "tool_calls": [] }),
            json!({ "role": "tool", "tool_call_id": "1", "content": "x" }),
            json!({ "role": "user", "content": "b".repeat(100) }),
            json!({ "role": "assistant", "content": "done" }),
            json!({ "role": "user", "content": "c".repeat(100) }),
        ];
        trim(&mut list, 400);
        assert_eq!(list.len(), 4, "the oldest turn went, the two newer stayed");
        assert_eq!(list[0]["role"], "system");
        assert!(list[1]["content"].as_str().unwrap().starts_with('b'));
        trim(&mut list, 10);
        assert_eq!(list.len(), 2, "the newest turn stays whatever the budget");
        assert!(list[1]["content"].as_str().unwrap().starts_with('c'));
    }

    #[test]
    fn a_call_is_priced_by_fresh_cached_and_output_tokens() {
        let e = Endpoint { base_url: String::new(), key: String::new(), model: String::new(), max_tokens: 1, price_input: 1.0, price_cached: 0.25, price_output: 4.0, cost_cap: 1.0 };
        // 1,000,000 prompt tokens of which 400,000 cached, 100,000 output: 0.6 + 0.1 + 0.4.
        assert!((e.cost(1_000_000, 400_000, 100_000) - 1.1).abs() < 1e-9);
        // Cached tokens never exceed the prompt.
        assert!((e.cost(10, 20, 0) - 10.0 * 0.25 / 1e6).abs() < 1e-12);
    }

    #[test]
    fn tool_arguments_come_as_a_string_or_an_object() {
        assert_eq!(arguments_of(&json!({ "function": { "arguments": "{\"calls\": []}" } })), json!({ "calls": [] }));
        assert_eq!(arguments_of(&json!({ "function": { "arguments": { "calls": [] } } })), json!({ "calls": [] }));
        assert_eq!(arguments_of(&json!({ "function": { "arguments": "not json" } })), json!({}));
    }
}
