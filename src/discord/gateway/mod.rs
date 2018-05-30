use error::{DResult, Error};

use flate2::Decompress;
use futures::prelude::*;
use futures::sync::BiLock;

pub use discord::gateway::dispatcher::Dispatcher;

pub mod compression;
pub mod dispatcher;
pub mod heartbeat;
pub mod packets;
pub mod websocket;

#[derive(Debug)]
pub enum GatewayMessage {
    Packet(String),
    OtherFrame(websocket::Message),
}

pub struct Writer {
    writer: BiLock<websocket::Writer>,
}

pub struct Client {
    pub dispatcher: dispatcher::Dispatcher,
    pub writer: Writer,
}

struct PartialClient {
    pub reader: compression::MessageDeflater,
    pub writer: websocket::Writer,
}

pub fn connect(gateway: &str) -> impl Future<Item = Client, Error = Error> {
    let uri = format!("{}/?v=6&encoding=json&compress=zlib-stream", gateway);
    async_block! {
        let PartialClient { reader, writer } = await!(new_connection(uri))?;
        let (dispatcher, writer) = dispatcher::create(reader, writer);

        Ok(Client { dispatcher, writer })
    }
}

#[async]
fn new_connection(uri: String) -> DResult<PartialClient> {
    let uri = uri.parse().unwrap();
    let ws_client = await!(websocket::connect(uri))?;

    let deflater = compression::MessageDeflater {
        reader: ws_client.reader,
        zlib: Decompress::new(true),
    };

    Ok(PartialClient {
        reader: deflater,
        writer: ws_client.writer,
    })
}

impl Writer {
    fn create(writer: websocket::Writer) -> (Writer, Writer) {
        let (left, right) = BiLock::new(writer);
        (Writer { writer: left }, Writer { writer: right })
    }
}

impl Writer {
    #[async]
    pub fn send(self, mut message: websocket::Message) -> DResult<Writer> {
        let mut writer = await!(self.writer.lock()).map_err(|_| Error::FutureError)?;

        loop {
            match writer.start_send(message)? {
                AsyncSink::Ready => break,
                AsyncSink::NotReady(msg) => {
                    message = msg;
                    yield Async::NotReady;
                }
            }
        }

        loop {
            match writer.poll_complete()? {
                Async::Ready(_) => break,
                Async::NotReady => yield Async::NotReady,
            }
        }

        let writer = writer.unlock();
        Ok(Writer { writer })
    }
}
