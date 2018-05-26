#![feature(proc_macro, generators, entry_or_default, proc_macro_non_items)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

extern crate base64;
extern crate byteorder;
extern crate bytes;
extern crate chrono;
extern crate direct2d;
extern crate direct3d11;
extern crate directwrite;
extern crate dotenv;
extern crate dxgi;
extern crate env_logger;
extern crate erased_serde;
extern crate flate2;
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

pub mod discord;
pub mod error;
pub mod logging;
pub mod state;

mod demo;

fn main() {
    use logging::FutureLogExt;
    use discord::gateway::packets::*;

    dotenv::dotenv().ok();
    logging::init();

    println!("WOO Packet<Identify> IS {} BYTES", std::mem::size_of::<Packet<Identify>>());

    tokio::run(demo::naive_test().log_errors());
}
