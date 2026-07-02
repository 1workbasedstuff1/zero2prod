use std::fs::metadata;

// NOTE:
// code             log crate       env_logger
// info!("msg") ->  Log trait  ->   impl Log    -> Terminal
// [-]
// but we dont see it get passed to the function in startup.rs
// thats because it writes to a global atomic pointer

use log::{LevelFilter, Log, Metadata, Record};

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init() {
    log::set_logger(&SimpleLogger)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("logger already set");
}
