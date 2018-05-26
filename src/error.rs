use std::io;
use std::fmt;

use hyper;
use hyper_tls;
use serde_json;
use discord::gateway::websocket;
use dxgi::Error as DError;

#[derive(Debug)]
pub enum Error {
    Api(ApiError),
    Io(io::Error),
    Json(serde_json::Error),
    Hyper(hyper::Error),
    Tls(hyper_tls::Error),
    Websocket(websocket::Error),
    Graphics(i32),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Api(err) => write!(fmt, "Api error: {:?}", err),
            Error::Io(err) => write!(fmt, "I/O error: {}", err),
            Error::Json(err) => write!(fmt, "Json error: {}", err),
            Error::Hyper(err) => write!(fmt, "Http error: {}", err),
            Error::Tls(err) => write!(fmt, "Tls error: {}", err),
            Error::Websocket(err) => write!(fmt, "WebSocket error: {}", err),
            Error::Graphics(hr) => write!(fmt, "Graphics error: {:x} {}", hr, DError(*hr)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    UnknownEndpoint,
}

pub type DResult<T> = Result<T, Error>;

impl From<ApiError> for Error {
    fn from(e: ApiError) -> Error {
        Error::Api(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Json(e)
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Error {
        Error::Hyper(e)
    }
}

impl From<hyper_tls::Error> for Error {
    fn from(e: hyper_tls::Error) -> Error {
        Error::Tls(e)
    }
}


impl From<websocket::Error> for Error {
    fn from(e: websocket::Error) -> Error {
        Error::Websocket(e)
    }
}
