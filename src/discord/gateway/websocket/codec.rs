use discord::gateway::websocket::{Error, Frame, Message, OpCode};

use bytes::BytesMut;
use rand::{thread_rng, Rng};
use tokio_io::codec::{Decoder, Encoder};

pub struct ClientCodec {
    pub encoder: ClientEncoder,
    pub decoder: ClientDecoder,
}

impl Encoder for ClientCodec {
    type Item = Message;
    type Error = Error;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Error> {
        self.encoder.encode(item, dst)
    }
}

impl Decoder for ClientCodec {
    type Item = Message;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Message>, Error> {
        self.decoder.decode(src)
    }
}

pub struct ClientEncoder;

impl Encoder for ClientEncoder {
    type Item = Message;
    type Error = Error;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Error> {
        let frame = item.to_frame().with_mask(thread_rng().gen());

        let size = frame.frame_size();
        dst.reserve(size);
        frame.encode(dst);
        Ok(())
    }
}

pub struct ClientDecoder {
    max_recv_message_size: usize,
    decode_frames: Vec<Frame>,
    decoded_size: usize,
}

impl ClientDecoder {
    pub fn new() -> Self {
        ClientDecoder::with_limit(1_000_000)
    }

    pub fn with_limit(max_recv: usize) -> Self {
        ClientDecoder {
            max_recv_message_size: max_recv,
            decode_frames: Vec::with_capacity(16),
            decoded_size: 0,
        }
    }

    fn clear_state(&mut self) {
        self.decode_frames.clear();
        self.decoded_size = 0;
    }

    fn drain_frames(&mut self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.decoded_size);
        self.decoded_size = 0;
        for frame in self.decode_frames.drain(..) {
            buf.extend_from_slice(&frame.payload);
        }
        buf
    }

    fn combine_frames(&mut self) -> Result<Message, Error> {
        let opcode = self.decode_frames[0].flags.opcode();

        let buf = match opcode {
            OpCode::Text | OpCode::Binary => self.drain_frames(),
            op => {
                self.clear_state();
                return Err(Error::UnexpectedFrame(op, &VALID_FRAGMENT_FRAMES));
            }
        };

        match opcode {
            OpCode::Text => {
                let text = String::from_utf8(buf).map_err(|_| Error::TextFrameNotUtf8)?;
                Ok(Message::Text(text))
            }
            OpCode::Binary => Ok(Message::Binary(buf)),
            _ => unreachable!(),
        }
    }
}

impl Decoder for ClientDecoder {
    type Item = Message;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Message>, Error> {
        let (flags, len) = match Frame::is_complete(src) {
            Some(info) => info,
            None => return Ok(None),
        };

        let combined_size = self.decoded_size.saturating_add(len);
        if combined_size > self.max_recv_message_size {
            self.clear_state();
            return Err(Error::FrameTooLarge(combined_size));
        }

        let msg = match flags.opcode() {
            OpCode::Continuation if !self.decode_frames.is_empty() => {
                let frame = Frame::decode(src).unmask();
                self.decode_frames.push(frame);
                self.decoded_size += len;

                if flags.is_final() {
                    Some(self.combine_frames()?)
                } else {
                    None
                }
            }

            OpCode::Text | OpCode::Binary if self.decode_frames.is_empty() => {
                let frame = Frame::decode(src).unmask();
                self.decode_frames.push(frame);
                self.decoded_size += len;

                if flags.is_final() {
                    Some(self.combine_frames()?)
                } else {
                    None
                }
            }

            OpCode::Close | OpCode::Ping | OpCode::Pong => {
                if !flags.is_final() {
                    return Err(Error::NonFinalControlFrame);
                }

                let frame = Frame::decode(src).unmask();
                match frame.flags.opcode() {
                    OpCode::Close => Some(Message::Close(frame.payload)),
                    OpCode::Ping => Some(Message::Ping(frame.payload)),
                    OpCode::Pong => Some(Message::Pong(frame.payload)),
                    _ => unreachable!(),
                }
            }

            op => {
                if self.decode_frames.is_empty() {
                    return Err(Error::UnexpectedFrame(op, &VALID_START_CODES));
                } else {
                    return Err(Error::UnexpectedFrame(op, &VALID_CONTINUATION_CODES));
                }
            }
        };

        *src = src.split_off(len);
        Ok(msg)
    }
}

static VALID_START_CODES: &'static [OpCode] = &[
    OpCode::Text,
    OpCode::Binary,
    OpCode::Close,
    OpCode::Ping,
    OpCode::Pong,
];

static VALID_CONTINUATION_CODES: &'static [OpCode] = &[
    OpCode::Continuation,
    OpCode::Close,
    OpCode::Ping,
    OpCode::Pong,
];

static VALID_FRAGMENT_FRAMES: &'static [OpCode] = &[OpCode::Text, OpCode::Binary];
