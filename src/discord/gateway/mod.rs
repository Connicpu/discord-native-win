use error::Error;

use flate2::Decompress;
use futures::prelude::*;

pub mod compression;
pub mod websocket;

pub enum GatewayMessage {
    Packet(String),
    OtherFrame(websocket::Message),
}

pub struct Client {
    pub reader: compression::MessageDeflater,
    pub writer: websocket::Writer,
}

pub fn connect(gateway: &str) -> impl Future<Item = Client, Error = Error> {
    let uri = format!("{}/?v=6&encoding=json&compress=zlib-stream", gateway);
    let uri = uri.parse().unwrap();
    websocket::connect(uri).map(|ws_client| Client {
        reader: compression::MessageDeflater {
            reader: ws_client.reader,
            zlib: Decompress::new(true),
        },
        writer: ws_client.writer,
    })
}
