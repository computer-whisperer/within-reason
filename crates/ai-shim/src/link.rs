//! Connection to the bot process. Never blocks the engine thread for long: reads are
//! non-blocking and writes carry a short timeout.

use std::io;
use std::os::unix::net::UnixStream;
use std::time::Duration;

use ai_host::Transport;
use bot_protocol::{Commands, FrameReader, ToBot, handshake, socket_path, write_frame};

const WRITE_TIMEOUT: Duration = Duration::from_millis(250);
/// The bot answers the handshake from its accept loop at once; a bot from before the handshake never answers, and the
/// engine thread waits this long to find out.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_millis(250);

pub struct Link {
    stream: UnixStream,
    reader: FrameReader,
}

impl Transport for Link {
    fn connect() -> io::Result<Self> {
        let mut stream = UnixStream::connect(socket_path())?;
        stream.set_write_timeout(Some(WRITE_TIMEOUT))?;
        stream.set_read_timeout(Some(HANDSHAKE_TIMEOUT))?;
        // A bot of another protocol, or one from before the handshake (no answer in time), is the same fault to the
        // host: `InvalidData`, said once in the chat.
        handshake(&mut stream).map_err(|e| match e.kind() {
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::UnexpectedEof => {
                io::Error::new(io::ErrorKind::InvalidData, format!("no handshake from the bot ({e})"))
            }
            _ => e,
        })?;
        stream.set_read_timeout(None)?;
        Ok(Link { stream, reader: FrameReader::default() })
    }

    fn send(&mut self, message: ToBot) -> io::Result<()> {
        write_frame(&mut self.stream, &message)
    }

    /// Returns the bot's reply if one has fully arrived.
    fn poll(&mut self) -> io::Result<Option<Commands>> {
        self.stream.set_nonblocking(true)?;
        let result = self.reader.read(&mut self.stream);
        self.stream.set_nonblocking(false)?;
        result
    }

    /// Blocks until the bot's reply has arrived. Lockstep mode: the game stands still while the bot thinks.
    fn wait(&mut self) -> io::Result<Commands> {
        loop {
            if let Some(commands) = self.reader.read(&mut self.stream)? {
                return Ok(commands);
            }
        }
    }
}
