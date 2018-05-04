#![feature(proc_macro, generators)]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

extern crate base64;
extern crate byteorder;
extern crate bytes;
extern crate chrono;
extern crate direct2d;
extern crate direct3d11;
extern crate directwrite;
extern crate dxgi;
extern crate erased_serde;
extern crate futures_await as futures;
extern crate http;
extern crate httparse;
extern crate hyper;
extern crate hyper_tls;
extern crate itoa;
extern crate native_tls;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate sha1;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_tls;

use error::DResult;

use futures::prelude::*;

pub mod discord;
pub mod error;
pub mod state;

#[async]
fn naive_test() -> DResult<()> {
    let gateway = await!(discord::api::gateway::get())?;

    println!("Connecting to {}...", gateway.url);
    let uri = format!("{}/?v=6", gateway.url).parse().unwrap();
    let ws_client = await!(discord::gateway::websocket::connect(uri))?;

    #[async]
    for packet in ws_client.reader {
        println!("{:?}", packet);
    }

    Ok(())
}

fn main() {
    tokio::run(naive_test().map_err(|e| eprintln!("Error: {:?}", e)));
}
