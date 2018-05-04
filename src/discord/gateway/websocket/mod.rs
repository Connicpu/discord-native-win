pub use discord::gateway::websocket::client::{Client, Reader, Writer};
pub use discord::gateway::websocket::codec::{ClientCodec, ClientDecoder, ClientEncoder};
pub use discord::gateway::websocket::connect::{connect, connect_with_auth, connect_with_settings};
pub use discord::gateway::websocket::error::Error;
pub use discord::gateway::websocket::frame::{Frame, OpCode};
pub use discord::gateway::websocket::message::Message;

pub mod client;
pub mod codec;
pub mod connect;
pub mod error;
pub mod frame;
pub mod message;
