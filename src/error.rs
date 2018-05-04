use std::io;

use hyper;
use hyper_tls;
use serde_json;
use discord::gateway::websocket;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    UnknownEndpointError,
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
