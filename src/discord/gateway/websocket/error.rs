use discord::gateway::websocket::OpCode;

use std::io;

use httparse;

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

    InvalidResponseCode(Option<u16>),
    FrameTooLarge(usize),
    UnexpectedFrame(OpCode, &'static [OpCode]),
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
