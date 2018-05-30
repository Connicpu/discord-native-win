use discord::gateway::websocket::OpCode;

use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Headers(httparse::Error),

    InvalidUri,
    ServerNotFound,
    TextFrameNotUtf8,
    NonFinalControlFrame,
    IncompleteHeaders,
    BadUpgrade,
    BadSecretKey,
    UnexpectedExtensions,
    BadCompression,

    InvalidResponseCode(Option<u16>),
    FrameTooLarge(usize),
    UnexpectedFrame(OpCode, &'static [OpCode]),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(err) => write!(fmt, "{}", err),
            Error::Headers(err) => write!(fmt, "bad headers: {}", err),
            Error::InvalidUri => fmt.write_str("Invalid connection uri"),
            Error::ServerNotFound => fmt.write_str("Server not found"),
            Error::TextFrameNotUtf8 => fmt.write_str("Received a non-utf8 text frame"),
            Error::NonFinalControlFrame => fmt.write_str("Received a control frame with FIN=0"),
            Error::IncompleteHeaders => fmt.write_str("Server sent incomplete HTTP headers"),
            Error::BadUpgrade => fmt.write_str("Server sent invalid Upgrade/Connection header"),
            Error::BadSecretKey => fmt.write_str("Server sent invalid secret key"),
            Error::UnexpectedExtensions => {
                fmt.write_str("Server enabled unexpected extensions/protocols")
            }
            Error::BadCompression => fmt.write_str("Failed to decompress a compressed payload"),
            Error::InvalidResponseCode(Some(code)) => {
                write!(fmt, "Server replied with unexpected HTTP {:03}", code)
            }
            Error::InvalidResponseCode(None) => write!(fmt, "Server replied with no HTTP code"),
            Error::FrameTooLarge(size) => write!(fmt, "Server sent {} byte frame", size),
            Error::UnexpectedFrame(sent, expected) => write!(
                fmt,
                "Server sent a {:?} frame when only 1 of {:?} was expected",
                sent, expected,
            ),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<httparse::Error> for Error {
    fn from(e: httparse::Error) -> Error {
        Error::Headers(e)
    }
}
