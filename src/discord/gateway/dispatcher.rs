use discord::gateway::compression::MessageDeflater;
use discord::gateway::packets::{DataOnlyPacket, PacketData};

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde_json as json;

#[derive(Clone)]
pub struct Dispatcher {
    state: Arc<DispatcherState>,
}

impl Dispatcher {
    pub fn handle_opcode<H, P>(&self, opcode: u32, handler: H)
    where
        H: Fn(P) + 'static,
        P: PacketData,
    {
        let raw_handler = move |data: &str| {
            let payload = match json::from_str::<DataOnlyPacket<P>>(data) {
                Ok(data) => data.payload,
                Err(e) => return error!("Failed to deserialize gateway packet: {}", e),
            };

            handler(payload);
        };

        self.handle_opcode_raw(opcode, Box::new(raw_handler));
    }

    pub fn handle_event<H, P>(&self, event: &'static str, handler: H)
    where
        H: Fn(P) + 'static,
        P: PacketData,
    {
        let raw_handler = move |data: &str| {
            let payload = match json::from_str::<DataOnlyPacket<P>>(data) {
                Ok(data) => data.payload,
                Err(e) => return error!("Failed to deserialize gateway packet: {}", e),
            };

            handler(payload);
        };

        self.handle_event_raw(event, Box::new(raw_handler));
    }

    pub fn handle_opcode_raw(&self, opcode: u32, handler: Box<Fn(&str)>) {
        let mut handlers = self.state.opcode_handlers.write().unwrap();
        handlers.entry(opcode).or_default().push(handler);
    }

    pub fn handle_event_raw(&self, event: &'static str, handler: Box<Fn(&str)>) {
        let mut handlers = self.state.event_handlers.write().unwrap();
        handlers.entry(event).or_default().push(handler);
    }
}

struct DispatcherState {
    opcode_handlers: RwLock<HashMap<u32, Vec<Box<Fn(&str)>>>>,
    event_handlers: RwLock<HashMap<&'static str, Vec<Box<Fn(&str)>>>>,
}

pub fn create(reader: MessageDeflater) -> Dispatcher {
    unimplemented!()
}
