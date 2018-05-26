use discord;
use discord::gateway::websocket::Message;
use discord::gateway::GatewayMessage;
use error::DResult;

use futures::prelude::*;

#[async]
pub fn naive_test() -> DResult<()> {
    let gateway = await!(discord::api::gateway::get())?;

    info!("Connecting to {:?}...", gateway.url);
    let client = await!(discord::gateway::connect(&gateway.url))?;

    /*#[async]
    for packet in client.dispatcher {
        match packet {
            GatewayMessage::Packet(_payload) => {
                //println!("packet: {}", payload);
            }

            GatewayMessage::OtherFrame(Message::Close { status, reason }) => {
                let reason = reason.as_ref().map(|s| &s[..]).unwrap_or("");
                info!("closed: {} {}", status, reason);
                break;
            }

            GatewayMessage::OtherFrame(frame) => {
                info!("other websocket packet: {:?}", frame);
            }
        }
    }*/

    discord::api::dispose();
    Ok(())
}
