use colored::Colorize;
use log::{Level, LevelFilter, Metadata, Record};

pub struct MinimalLogger;

impl log::Log for MinimalLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        // Filter out INFO messages from commitlog
        if record.level() == Level::Info && record.target().starts_with("commitlog") {
            return;
        }

        // Inspired by https://github.com/borntyping/rust-simple_logger/blob/ce8ec4bbe5f81cfd2f7a852f68e308369ef7fa5f/src/lib.rs#L199-L203
        let level_string = match record.level() {
            Level::Error => record.level().to_string().red(),
            Level::Warn => record.level().to_string().yellow(),
            Level::Info => record.level().to_string().cyan(),
            Level::Debug => record.level().to_string().purple(),
            Level::Trace => record.level().to_string().normal(),
        };

        if record.level() > LevelFilter::Error {
            println!("{:<5} {}", level_string, record.args())
        } else {
            eprintln!("{:<5} {}", level_string, record.args())
        }
    }

    fn flush(&self) {}
}
