use discord::gateway::websocket::{ClientDecoder, Error, Message};
use discord::gateway::GatewayMessage;

use std::io::BufReader;

use flate2::{self, FlushDecompress, Status};
use futures::prelude::*;
use tokio::io::AsyncRead;
use tokio_io::codec::FramedRead;

pub struct MessageDeflater {
    pub reader: FramedRead<BufReader<Box<AsyncRead + Send>>, ClientDecoder>,
    pub zlib: flate2::Decompress,
}

impl Stream for MessageDeflater {
    type Item = GatewayMessage;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<GatewayMessage>, Error> {
        match self.reader.poll() {
            Ok(Async::Ready(Some(Message::Binary(data)))) => {
                if is_zlib(&data) {
                    let mut buf = Vec::with_capacity(256);
                    let flush = FlushDecompress::None;
                    let res = self.zlib.decompress_vec(&data, &mut buf, flush);

                    match res {
                        Ok(Status::Ok) | Ok(Status::StreamEnd) => {
                            let text = String::from_utf8(buf).map_err(|_| Error::TextFrameNotUtf8)?;
                            debug!("gateway packet decompressed");
                            Ok(Async::Ready(Some(GatewayMessage::Packet(text))))
                        }
                        _ => return Err(Error::BadCompression),
                    }
                } else {
                    Ok(Async::Ready(Some(GatewayMessage::OtherFrame(
                        Message::Binary(data),
                    ))))
                }
            }
            Ok(Async::Ready(Some(Message::Text(text)))) => {
                Ok(Async::Ready(Some(GatewayMessage::Packet(text))))
            }
            Ok(Async::Ready(Some(frame))) => {
                Ok(Async::Ready(Some(GatewayMessage::OtherFrame(frame))))
            }
            Ok(Async::Ready(None)) => Ok(Async::Ready(None)),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(e),
        }
    }
}

const ZLIB_SUFFIX: [u8; 4] = [0, 0, 255, 255];
fn is_zlib(data: &[u8]) -> bool {
    data.len() >= 4 && &data[data.len() - 4..] == &ZLIB_SUFFIX
}
