use discord::gateway::websocket::{Frame, OpCode};

#[derive(Debug)]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Close(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
}

impl Message {
    pub fn to_frame(self) -> Frame {
        match self {
            Message::Text(text) => Frame::new(OpCode::Text, text),
            Message::Binary(data) => Frame::new(OpCode::Binary, data),
            Message::Close(data) => Frame::new(OpCode::Close, data),
            Message::Ping(data) => Frame::new(OpCode::Ping, data),
            Message::Pong(data) => Frame::new(OpCode::Pong, data),
        }
    }
}
