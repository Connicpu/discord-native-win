use std::fmt::Display;
use std::io::Write;

use chrono::Local;
use env_logger;
use futures::prelude::*;
use log::Level;

pub fn init() {
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            write!(
                buf,
                "{pad}[{level}][{time}] ({target}) {message}\n",
                level = record.level(),
                pad = levelpad(record.level()),
                time = Local::now().format("%F %T"),
                message = record.args(),
                target = record.target(),
            )
        })
        .init();
}

fn levelpad(level: Level) -> &'static str {
    match level {
        Level::Error => "",
        Level::Warn => " ",
        Level::Info => " ",
        Level::Debug => "",
        Level::Trace => "",
    }
}

pub trait FutureLogExt
where
    Self: Future + Sized,
    Self::Error: Display,
{
    fn log_errors(self) -> LogFutureErrors<Self>;
}

impl<T> FutureLogExt for T
where
    T: Future,
    T::Error: Display,
{
    fn log_errors(self) -> LogFutureErrors<Self> {
        LogFutureErrors(self)
    }
}

pub struct LogFutureErrors<T>(pub T)
where
    T: Future,
    T::Error: Display;

impl<T> Future for LogFutureErrors<T>
where
    T: Future,
    T::Error: Display,
{
    type Item = T::Item;
    type Error = ();

    fn poll(&mut self) -> Result<Async<T::Item>, ()> {
        match self.0.poll() {
            Ok(result) => Ok(result),
            Err(error) => {
                error!("{}", error);
                Err(())
            }
        }
    }
}
