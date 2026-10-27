//! Bakes the protocol's fingerprint (`bot_protocol::FINGERPRINT`): a hash of the message and framing sources with
//! their comments, blank lines and test modules left out, so the shim and the bot can tell at connection whether they speak the same
//! encoding. Postcard is not self-describing: a field one side lacks shifts every byte after it (human 2026-10-26: a
//! shim installed before `BuildSite::exact` read the bot's orders one byte off and dropped the connection five times a
//! seat, each reconnection a fresh player session).
use std::fmt::Write;

fn main() {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for file in ["src/messages.rs", "src/framing.rs"] {
        println!("cargo:rerun-if-changed={file}");
        let text = std::fs::read_to_string(file).expect("the protocol source");
        // A test module (`#[cfg(test)]` to its unindented closing brace) is not the protocol.
        let mut in_test = false;
        for line in text.lines() {
            if line.trim() == "#[cfg(test)]" {
                in_test = true;
            }
            if in_test {
                in_test = line != "}";
                continue;
            }
            let code = line.split("//").next().unwrap_or("").trim();
            if code.is_empty() {
                continue;
            }
            for byte in code.bytes().chain([b'\n']) {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
            }
        }
    }
    let mut hex = String::new();
    write!(hex, "{hash:016x}").unwrap();
    println!("cargo:rustc-env=WITHIN_REASON_PROTOCOL={hex}");
}
