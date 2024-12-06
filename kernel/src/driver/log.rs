use log::{set_logger, set_max_level, Level, Record};
use log::{LevelFilter, Log, Metadata};

use crate::{println, serial_println};

pub fn init() {
    static LOGGER: Logger = Logger;
    set_logger(&LOGGER).unwrap();
    set_max_level(LevelFilter::Debug);
}

macro_rules! log_output {
    ($color:expr, $level:expr, $args:expr, $($extra:tt)*) => {
        serial_println!(
            "[{}] {}{}",
            format_args!("\x1b[{}m{}\x1b[0m", $color, $level),
            $args,
            format_args!($($extra)*)
        );
        println!(
            "[{}] {}{}",
            format_args!("\x1b[{}m{}\x1b[0m", $color, $level),
            $args,
            format_args!($($extra)*)
        );
    };
}

struct Logger;

impl Logger {
    fn log_message(&self, record: &Record, with_location: bool) {
        let color = match record.level() {
            Level::Error => "31",
            Level::Warn => "33",
            Level::Info => "32",
            Level::Debug => "34",
            Level::Trace => "36",
        };

        if with_location {
            let file = record.file().unwrap();
            let line = record.line().unwrap();
            log_output!(color, record.level(), record.args(), ", {}:{}", file, line);
        } else {
            log_output!(color, record.level(), record.args(), "");
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let with_location = matches!(record.level(), Level::Debug | Level::Trace);
            self.log_message(record, with_location);
        }
    }

    fn flush(&self) {}
}
