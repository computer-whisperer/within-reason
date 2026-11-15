//! The engine's autohost channel: the game server sends UDP event packets to us and accepts
//! plain-text server commands back (`rts/Net/AutohostInterface.cpp`).

use std::io::{self, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

const SERVER_STARTPLAYING: u8 = 2;
const SERVER_GAMEOVER: u8 = 3;

pub enum Event {
    StartPlaying,
    GameOver { winning_ally_teams: Vec<u8> },
    Other,
}

pub struct Autohost {
    socket: UdpSocket,
    /// Learned from the first packet; commands cannot be sent before then.
    engine: Option<SocketAddr>,
}

impl Autohost {
    pub fn bind(port: u16) -> io::Result<Self> {
        Ok(Autohost { socket: UdpSocket::bind(("127.0.0.1", port))?, engine: None })
    }

    pub fn receive(&mut self, timeout: Duration) -> io::Result<Option<Event>> {
        self.socket.set_read_timeout(Some(timeout))?;
        let mut packet = [0u8; 65536];
        let len = match self.socket.recv_from(&mut packet) {
            Ok((len, from)) => {
                self.engine = Some(from);
                len
            }
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => return Ok(None),
            Err(e) => return Err(e),
        };
        Ok(Some(match packet[..len] {
            [SERVER_STARTPLAYING, ..] => Event::StartPlaying,
            // [id, message size, reporting player, winning ally teams...]
            [SERVER_GAMEOVER, _, _, ref winners @ ..] => Event::GameOver { winning_ally_teams: winners.to_vec() },
            _ => Event::Other,
        }))
    }

    /// The winners the server's game-over packet names, when one comes within `grace`; with no grace, one that has
    /// already arrived. For a game whose end the referee met first by another sign (the engine gone, the bot gone).
    pub fn game_over_within(&mut self, grace: Duration) -> io::Result<Option<Vec<u8>>> {
        let until = Instant::now() + grace;
        loop {
            match self.receive(Duration::from_millis(100))? {
                Some(Event::GameOver { winning_ally_teams }) => return Ok(Some(winning_ally_teams)),
                Some(_) => {}
                None if Instant::now() >= until => return Ok(None),
                None => {}
            }
        }
    }

    pub fn send(&self, command: &str) -> io::Result<()> {
        let Some(engine) = self.engine else {
            return Err(io::Error::other("no packet received from the engine yet"));
        };
        self.socket.send_to(command.as_bytes(), engine).map(drop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The engine's side of the channel, and where the arena listens.
    fn pair() -> (Autohost, UdpSocket, SocketAddr) {
        let host = Autohost::bind(0).unwrap();
        let at = host.socket.local_addr().unwrap();
        (host, UdpSocket::bind(("127.0.0.1", 0)).unwrap(), at)
    }

    #[test]
    fn a_game_over_within_the_grace_is_read_past_the_packets_before_it() {
        let (mut host, engine, at) = pair();
        engine.send_to(&[13, 5, 0, b'g', b'g'], at).unwrap();
        let late = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(400));
            engine.send_to(&[SERVER_GAMEOVER, 4, 0, 1], at).unwrap();
        });
        assert_eq!(host.game_over_within(Duration::from_secs(5)).unwrap(), Some(vec![1]));
        late.join().unwrap();
    }

    #[test]
    fn without_a_grace_only_a_game_over_already_there_is_read() {
        let (mut host, engine, at) = pair();
        assert_eq!(host.game_over_within(Duration::ZERO).unwrap(), None);
        engine.send_to(&[SERVER_GAMEOVER, 4, 0, 0], at).unwrap();
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(host.game_over_within(Duration::ZERO).unwrap(), Some(vec![0]));
    }

    #[test]
    fn no_game_over_within_the_grace_is_none() {
        let (mut host, engine, at) = pair();
        engine.send_to(&[13, 5, 0, b'g', b'g'], at).unwrap();
        assert_eq!(host.game_over_within(Duration::from_millis(300)).unwrap(), None);
    }
}
