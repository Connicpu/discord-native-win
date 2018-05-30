use discord::gateway::websocket::{Error, Message};

use futures::prelude::*;

//pub type Reader = FramedRead<BufReader<Box<AsyncRead + Send>>, ClientDecoder>;
pub type Reader = Box<Stream<Item = Message, Error = Error> + Send + 'static>;
//pub type Writer = FramedWrite<Box<AsyncWrite + Send>, ClientEncoder>;
pub type Writer = Box<Sink<SinkItem = Message, SinkError = Error> + Send + 'static>;

pub struct Client {
    pub reader: Reader,
    pub writer: Writer,
}
