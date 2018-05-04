use discord::gateway::websocket::{ClientEncoder, ClientDecoder};

use std::io::BufReader;

use tokio_io::codec::{FramedRead, FramedWrite};
use tokio::io::{AsyncRead, AsyncWrite};

pub struct Client {
    pub reader: FramedRead<BufReader<Box<AsyncRead + Send>>, ClientDecoder>,
    pub writer: FramedWrite<Box<AsyncWrite + Send>, ClientEncoder>,
}
