//! The flagship's skirmish AI library: the host (`crates/ai-host`) over a socket to the bot process (`crates/bot`).
//!
//! The engine loads this as `libSkirmishAI.so` (renamed by `run/install_ai.sh`). It holds no strategy: the bot
//! process can be restarted, can crash and can wait on a model without the game going down with it. See `DESIGN.md`.

mod link;

ai_host::export!(link::Link);
