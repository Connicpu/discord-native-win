use error::{DResult, Error};

use flate2::Decompress;
use futures::prelude::*;

pub use discord::gateway::dispatcher::Dispatcher;
pub use discord::gateway::websocket::Writer;

pub mod compression;
pub mod dispatcher;
pub mod heartbeat;
pub mod packets;
pub mod websocket;

pub enum GatewayMessage {
    Packet(String),
    OtherFrame(websocket::Message),
}

pub struct Client {
    pub dispatcher: dispatcher::Dispatcher,
    pub writer: websocket::Writer,
}

struct PartialClient {
    pub reader: compression::MessageDeflater,
    pub writer: websocket::Writer,
}

pub fn connect(gateway: &str) -> impl Future<Item = Client, Error = Error> {
    let uri = format!("{}/?v=6&encoding=json&compress=zlib-stream", gateway);
    async_block! {
        let PartialClient { reader, writer } = await!(new_connection(uri))?;
        let dispatcher = dispatcher::create(reader);

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
