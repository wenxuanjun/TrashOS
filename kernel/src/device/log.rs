use colorz::ansi::{Blue, Green, Red, Yellow};
use colorz::{Colorize, Style};

use crate::{println, serial_println};

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
}

const ERROR_STYLE: Style = Style::new().fg(Red).const_into_runtime_style();
const WARN_STYLE: Style = Style::new().fg(Yellow).const_into_runtime_style();
const INFO_STYLE: Style = Style::new().fg(Green).const_into_runtime_style();
const DEBUG_STYLE: Style = Style::new().fg(Blue).const_into_runtime_style();
const DEFAULT_STYLE: Style = Style::new().const_into_runtime_style();

struct Logger;

impl Logger {
    fn get_style(level: log::Level) -> Style {
        match level {
            log::Level::Error => ERROR_STYLE,
            log::Level::Warn => WARN_STYLE,
            log::Level::Info => INFO_STYLE,
            log::Level::Debug => DEBUG_STYLE,
            _ => DEFAULT_STYLE,
        }
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let level = record.level();
            let level_style = Logger::get_style(level);

            match record.level() {
                log::Level::Debug | log::Level::Trace => {
                    serial_println!(
                        "[{}] {}, {}:{}",
                        level.style_with(level_style),
                        record.args(),
                        record.file().unwrap_or("unknown"),
                        record.line().unwrap_or(0)
                    );
                    println!(
                        "[{}] {}, {}:{}",
                        level.style_with(level_style),
                        record.args(),
                        record.file().unwrap_or("unknown"),
                        record.line().unwrap_or(0)
                    );
                }
                _ => {
                    serial_println!("[{}] {}", level.style_with(level_style), record.args());
                    println!("[{}] {}", level.style_with(level_style), record.args());
                }
            }
        }
    }
    fn flush(&self) {}
}
