use crate::printk::Color;

pub const LOG_LEVEL: LogLevel = LogLevel::Debug;

#[derive(PartialOrd, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub const fn name(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
    pub const fn color(&self) -> Color {
        match self {
            LogLevel::Debug => Color::Blue,
            LogLevel::Info => Color::Green,
            LogLevel::Warn => Color::Yellow,
            LogLevel::Error => Color::Red,
        }
    }
}

pub fn log(level: LogLevel, args: core::fmt::Arguments) {
    if level >= LOG_LEVEL {
        crate::printk::change_print_level(level.color());
        crate::printk::_print(format_args!("[{}] {}\n", level.name(), args));
    }
}

#[macro_export]
macro_rules! log {
    ($level:expr, $arg:expr) => (
        if $level >= $crate::log::LOG_LEVEL {
            $crate::printk::change_print_level($level.color());
            if $level == $crate::log::LogLevel::Debug {
                $crate::printk::_print(format_args!("[{}] {}:{} {}\n", $level.name(), file!(), line!(), $arg));
            } else {
                $crate::printk::_print(format_args!("[{}] {}\n", $level.name(), $arg));
            }
        }
    );
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ($crate::log!($crate::log::LogLevel::Debug, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ($crate::log!($crate::log::LogLevel::Info, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ($crate::log!($crate::log::LogLevel::Warn, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ($crate::log!($crate::log::LogLevel::Error, format_args!($($arg)*)));
}