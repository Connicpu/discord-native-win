use discord;
use discord::gateway::packets::{Heartbeat, Hello, Packet};
use discord::gateway::websocket::Message;
use error::{DResult, Error};
use logging::FutureLogExt;

use std::time::{Duration, Instant};

use futures::prelude::*;
use futures::sync::{mpsc, oneshot};
use tokio::timer::Interval;
use spin::RwLock;

#[async]
pub fn naive_test() -> DResult<()> {
    let gateway = await!(discord::api::gateway::get())?;

    info!("Connecting to {:?}...", gateway.url);
    let client = await!(discord::gateway::connect(&gateway.url))?;

    let (tx, rx) = mpsc::unbounded::<String>();
    let mut writer = client.writer;
    tokio::spawn(async_block! {
        #[async]
        for packet in rx {
            trace!("Sending packet: {}", packet);
            writer = await!(writer.send(Message::Text(packet)).log_errors())?;
        }
        Ok(())
    });

    let htx = tx.clone();
    let (hellotx, hellorx) = oneshot::channel::<()>();
    let hellotx = RwLock::new(Some(hellotx));
    client.dispatcher.handle_opcode(move |packet: Hello| {
        trace!("Hello packet received: {:?}", packet);

        if let Some(tx) = hellotx.write().take() {
            tx.send(()).ok();
        }

        let tx = htx.clone();
        let handle_heartbeats = async_block! {
            let freq = Duration::from_millis(packet.heartbeat_interval);
            let start = Instant::now();
            let timer = Interval::new(start, freq);

            let mut i = 0;
            #[async]
            for _instant in timer {
                let heartbeat = Packet::new(Heartbeat(Some(i)));
                trace!("Sending heartbeat: {:?}", heartbeat);
                let payload = serde_json::to_string(&heartbeat).unwrap();
                tx.unbounded_send(payload).map_err(|_| Error::FutureError)?;
                i += 1;
            }

            Ok(()): DResult<()>
        };
        tokio::spawn(handle_heartbeats.log_errors());
    });

    let (closetx, closerx) = oneshot::channel();
    client.dispatcher.handle_close(move |status, reason| {
        info!(
            "Websocket Closed: {} {}",
            status,
            reason.unwrap_or_default()
        );
        let _ = closetx.send(());
    });

    let _ = await!(hellorx);



    let _ = await!(closerx);
    println!("foo");

    discord::api::dispose();
    Ok(())
}
