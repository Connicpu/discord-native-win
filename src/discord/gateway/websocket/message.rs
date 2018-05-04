use discord::gateway::websocket::{Frame, OpCode};

use byteorder::{BigEndian, ByteOrder};

#[derive(Debug)]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Close { status: u16, reason: Option<String> },
    Ping(Vec<u8>),
    Pong(Vec<u8>),
}

impl Message {
    pub fn to_frame(self) -> Frame {
        match self {
            Message::Text(text) => Frame::new(OpCode::Text, text),
            Message::Binary(data) => Frame::new(OpCode::Binary, data),
            Message::Close { status, reason } => {
                let mut statb = [0; 2];
                BigEndian::write_u16(&mut statb, status);
                let mut buf = Vec::with_capacity(reason.map(|r| r.len()).unwrap_or(0) + 2);

                Frame::new(OpCode::Close, buf)
            }
            Message::Ping(data) => Frame::new(OpCode::Ping, data),
            Message::Pong(data) => Frame::new(OpCode::Pong, data),
        }
    }
}
