use discord::gateway::compression::MessageDeflater;
use discord::gateway::packets::{DataOnlyPacket, IgnoreData, Packet, PacketData};
use discord::gateway::websocket::{self, Message};
use discord::gateway::{GatewayMessage, Writer};
use error::DResult;
use logging::FutureLogExt;

use std::boxed::FnBox;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Weak};
use std::time::SystemTime;

use futures::prelude::*;
use serde_json as json;
use spin::{Mutex, RwLock};

#[derive(Clone)]
pub struct Dispatcher {
    state: Arc<DispatcherState>,
}

impl Dispatcher {
    pub fn handle_opcode<H, P>(&self, handler: H)
    where
        H: Fn(P) + Send + Sync + 'static,
        P: PacketData,
    {
        let raw_handler = move |data: &str| {
            let payload = match json::from_str::<DataOnlyPacket<P>>(data) {
                Ok(data) => data.payload,
                Err(e) => return error!("Failed to deserialize gateway packet: {}", e),
            };

            handler(payload);
        };

        self.handle_opcode_raw(P::OPCODE, Box::new(raw_handler));
    }

    pub fn handle_event<H, P>(&self, handler: H)
    where
        H: Fn(P) + Send + Sync + 'static,
        P: PacketData,
    {
        let raw_handler = move |data: &str| {
            let payload = match json::from_str::<DataOnlyPacket<P>>(data) {
                Ok(data) => data.payload,
                Err(e) => return error!("Failed to deserialize gateway packet: {}", e),
            };

            handler(payload);
        };

        self.handle_event_raw(P::EVENT, Box::new(raw_handler));
    }

    pub fn handle_close<F>(&self, event: F)
    where
        F: FnOnce(u16, Option<String>) + Send + 'static,
    {
        *self.state.close_handler.lock() = Some(Box::new(event))
    }

    pub fn handle_opcode_raw(&self, opcode: u32, handler: EventHandler) {
        let mut handlers = self.state.opcode_handlers.write();
        handlers.entry(opcode).or_default().write().push(handler);
    }

    pub fn handle_event_raw(&self, event: &'static str, handler: EventHandler) {
        let mut handlers = self.state.event_handlers.write();
        handlers.entry(event).or_default().write().push(handler);
    }
}

pub type EventHandler = Box<Fn(&str) + Send + Sync>;

type HandlerList = Arc<RwLock<Vec<EventHandler>>>;
type HandlerMap<K> = RwLock<HashMap<K, HandlerList>>;

#[derive(Default)]
struct DispatcherState {
    opcode_handlers: HandlerMap<u32>,
    event_handlers: HandlerMap<&'static str>,
    close_handler: Mutex<Option<Box<FnBox(u16, Option<String>) + Send>>>,
    last_ping: AtomicUsize,
}

pub fn create(reader: MessageDeflater, writer: websocket::Writer) -> (Dispatcher, Writer) {
    let state = Arc::new(DispatcherState::default());
    let handler_state = Arc::downgrade(&state);
    let dispatcher = Dispatcher { state };

    let (writer, handler_writer) = Writer::create(writer);
    let handle_messages = handle_messages(handler_state, reader, handler_writer);
    tokio::spawn(handle_messages.log_errors());

    (dispatcher, writer)
}

#[async]
fn handle_messages(
    state: Weak<DispatcherState>,
    reader: MessageDeflater,
    mut writer: Writer,
) -> DResult<()> {
    #[async]
    for message in reader {
        trace!("Decoding packet");
        if let GatewayMessage::Packet(payload) = message {
            let frame: Packet<IgnoreData> = json::from_str(&payload)?;
            let handlers = state.upgrade().map(|s| {
                if let Some(ref event) = frame.event {
                    debug!("{} packet received", event);
                    s.event_handlers.read().get(event.as_ref()).cloned()
                } else {
                    debug!("Op({}) packet received", frame.opcode);
                    s.opcode_handlers.read().get(&frame.opcode).cloned()
                }
            });

            match handlers {
                Some(Some(handlers)) => for (i, handler) in handlers.read().iter().enumerate() {
                    trace!("Calling packet handler {}", i);
                    (*handler)(&payload);
                },
                Some(None) => {
                    trace!("No handlers installed");
                }
                None => {
                    info!("Dispatcher was closed");
                }
            }
        } else if let GatewayMessage::OtherFrame(frame) = message {
            match frame {
                Message::Text(_) => (),
                Message::Binary(_) => (),
                Message::Ping(data) => {
                    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
                    let now = now.as_secs() * 1000 + now.subsec_millis() as u64;
                    state.upgrade().map(|s| s.last_ping.store(now as usize, Ordering::SeqCst));
                    writer = await!(writer.send(Message::Pong(data)))?;
                }
                Message::Pong(data) => {
                    trace!("pong received: {}", String::from_utf8_lossy(&data));
                }
                Message::Close { status, reason } => {
                    let handler = state.upgrade().and_then(|s| s.close_handler.lock().take());
                    if let Some(handler) = handler {
                        (handler)(status, reason)
                    }
                }
            }
        }
    }

    Ok(())
}
