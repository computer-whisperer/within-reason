//! How late the engine runs our orders (H-HANDS-ORDER-LAG). A seat's commands go to the game's server and run at the
//! frame it stamps them with, so a host behind the server (a client stalled at the start, catching up after) has its
//! orders run late; an idle builder then reads as a refused order (human 1792971120 on All That Glitters Extended:
//! the banner said at 0:02 came back at 0:32, six opening orders were called refused and the plant landed on the
//! first extractor's spot). Every line we say is a probe: it travels the same way as the unit orders sent beside it
//! and comes back as chat (K-mech-chat-echoes-our-own-lines), and the frames it took are the lag from then on.

use bot_protocol::Command;

use super::FRAMES_PER_SECOND;

/// A probe not heard back in this long is dropped.
const PROBE_FRAMES: i32 = 180 * FRAMES_PER_SECOND;
/// With no line heard back this long after the first was said, the lag is taken as none (chat lost or muted).
const FIRST_ECHO_WAIT: i32 = 60 * FRAMES_PER_SECOND;

#[derive(Default)]
pub(super) struct Echo {
    /// The lines said and not yet heard back: the words the echo is known by and the frame said.
    probes: Vec<(String, i32)>,
    first_said: Option<i32>,
    lag: Option<i32>,
}

/// What a said line is known by when it comes back: the game fills in `{name}` and cuts a line at about 120
/// characters, so the first words after the name (as `relay_chat` knows them). Another seat of ours from the same
/// build says the same banner words; it runs in the same engine, so its echo times the same way.
fn key(text: &str) -> String {
    let text = text.replace("{name}", "");
    text.get(..24).unwrap_or(&text).to_string()
}

impl Echo {
    /// Our own lines among the commands sent this frame.
    pub(super) fn said(&mut self, commands: &[Command], frame: i32) {
        for command in commands {
            if let Command::Say { text } = command {
                self.probes.push((key(text), frame));
                self.first_said.get_or_insert(frame);
            }
        }
        self.probes.retain(|(_, at)| frame - at < PROBE_FRAMES);
    }

    /// A chat line heard: when it is one of ours, the frames it took become the lag, returned.
    pub(super) fn heard(&mut self, text: &str, frame: i32) -> Option<i32> {
        let i = self.probes.iter().position(|(key, at)| *at < frame && text.contains(key.as_str()))?;
        let (_, at) = self.probes.remove(i);
        self.lag = Some(frame - at);
        self.lag
    }

    /// The frames an order takes to run, as last measured; `None` while the first line said has not come back (an
    /// order's outcome is not judged yet).
    pub(super) fn lag(&self, frame: i32) -> Option<i32> {
        self.lag.or_else(|| self.first_said.filter(|at| frame - at > FIRST_ECHO_WAIT).map(|_| 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn say(text: &str) -> Vec<Command> {
        vec![Command::Say { text: text.to_string() }]
    }

    #[test]
    fn unknown_until_the_first_echo_then_the_latest() {
        let mut echo = Echo::default();
        assert_eq!(echo.lag(0), None);
        // human 1792971120: the banner at frame 75 came back at 960, the player's "gl hf" at 2160 at 2169.
        echo.said(&say("{name} is Within Reason 02eb1f3+ | player (claude-opus-5-5, effort medium) | seat ai1 team 3 random"), 75);
        assert_eq!(echo.lag(900), None);
        assert_eq!(echo.heard("gl hf", 900), None);
        assert_eq!(echo.heard("HiMom is Within Reason 02eb1f3+ | player (claude-opus-5-5, effort medium) | seat", 960), Some(885));
        assert_eq!(echo.lag(1000), Some(885));
        echo.said(&say("[WReason] gl hf (the AI commanding yellow)"), 2160);
        assert_eq!(echo.heard("[WReason] gl hf (the AI commanding yellow)", 2169), Some(9));
        assert_eq!(echo.lag(2200), Some(9));
    }

    #[test]
    fn no_echo_at_all_is_no_lag_after_a_minute() {
        let mut echo = Echo::default();
        echo.said(&say("{name} is Within Reason"), 60);
        assert_eq!(echo.lag(60 + FIRST_ECHO_WAIT), None);
        assert_eq!(echo.lag(61 + FIRST_ECHO_WAIT), Some(0));
    }

    #[test]
    fn a_line_heard_the_frame_it_was_said_is_not_its_echo() {
        let mut echo = Echo::default();
        echo.said(&say("hello there, everyone in the game"), 100);
        assert_eq!(echo.heard("hello there, everyone in the game", 100), None);
        assert_eq!(echo.heard("hello there, everyone in the game", 106), Some(6));
    }
}
