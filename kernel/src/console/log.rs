use crate::serial_println;
use super::printk::Color;

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            if record.level() < log::Level::Debug {
                serial_println!("[{}] {}", record.level(), record.args());
                super::printk::_print(
                    record.level().color(),
                    format_args!("[{}] {}\n", record.level(), record.args()),
                );
            } else {
                serial_println!(
                    "[{}] {}, {}:{}",
                    record.level(),
                    record.args(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0)
                );
                super::printk::_print(
                    record.level().color(),
                    format_args!(
                        "[{}] {}, {}:{}\n",
                        record.level(),
                        record.args(),
                        record.file().unwrap_or("unknown"),
                        record.line().unwrap_or(0)
                    ),
                );
            }
        }
    }

    fn flush(&self) {}
}

trait LogLevel {
    fn color(&self) -> Color;
}

impl LogLevel for log::Level {
    fn color(&self) -> Color {
        match self {
            log::Level::Error => Color::Red,
            log::Level::Warn => Color::Yellow,
            log::Level::Info => Color::Green,
            log::Level::Debug => Color::Blue,
            _ => Color::White,
        }
    }
}
