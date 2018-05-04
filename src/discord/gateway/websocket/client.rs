use discord::gateway::websocket::{ClientDecoder, ClientEncoder};

use std::io::BufReader;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{FramedRead, FramedWrite};

pub type Reader = FramedRead<BufReader<Box<AsyncRead + Send>>, ClientDecoder>;
pub type Writer = FramedWrite<Box<AsyncWrite + Send>, ClientEncoder>;

pub struct Client {
    pub reader: Reader,
    pub writer: Writer,
}
