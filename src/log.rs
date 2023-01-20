use crate::printk::Color;

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
            LogLevel::Warn => Color::Orange,
            LogLevel::Error => Color::Red,
        }
    }
}

#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => ({
        $crate::printk::change_print_level($level.color());
        $crate::printk::_print(format_args!("[{}] {}\n", $level.name(), format_args!($($arg)*)));
    });
}