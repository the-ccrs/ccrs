#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Fine = 5,
    Finer = 6,
    Finest = 7,
}

type LogFn = dyn Fn(LogLevel, &str, u32, std::fmt::Arguments) + Send + Sync;

pub static LOGGER: std::sync::OnceLock<Box<LogFn>> = std::sync::OnceLock::new();

pub fn init_logger<F>(callback: F)
where
    F: Fn(LogLevel, &str, u32, std::fmt::Arguments) + Send + Sync + 'static,
{
    LOGGER.set(Box::new(callback)).unwrap_or_else(|_| {
        panic!("Logger already initialized");
    });
}

#[cfg(feature = "max_log_level_error")]
pub const MAX_LOG_LEVEL: LogLevel = LogLevel::Error;

#[cfg(feature = "max_log_level_warn")]
pub const MAX_LOG_LEVEL: LogLevel = LogLevel::Warn;

#[cfg(feature = "max_log_level_info")]
pub const MAX_LOG_LEVEL: LogLevel = LogLevel::Info;

#[cfg(feature = "max_log_level_debug")]
pub const MAX_LOG_LEVEL: LogLevel = LogLevel::Debug;

#[cfg(feature = "max_log_level_fine")]
pub const MAX_LOG_LEVEL: LogLevel = LogLevel::Fine;

#[cfg(feature = "max_log_level_finer")]
pub const MAX_LOG_LEVEL: LogLevel = LogLevel::Finer;

#[cfg(feature = "max_log_level_finest")]
pub const MAX_LOG_LEVEL: LogLevel = LogLevel::Finest;

#[cfg(not(any(
    feature = "max_log_level_error",
    feature = "max_log_level_warn",
    feature = "max_log_level_info",
    feature = "max_log_level_debug",
    feature = "max_log_level_fine",
    feature = "max_log_level_finer",
    feature = "max_log_level_finest"
)))]
pub const MAX_LOG_LEVEL: LogLevel = LogLevel::Error;

#[inline(always)]
pub fn log_internal(level: LogLevel, file: &str, line: u32, args: std::fmt::Arguments) {
    if level <= MAX_LOG_LEVEL
        && let Some(cb) = LOGGER.get()
    {
        cb(level, file, line, args);
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        if $crate::logger::LogLevel::Error <= $crate::logger::MAX_LOG_LEVEL {
            $crate::logger::log_internal(
                $crate::logger::LogLevel::Error,
                file!(),
                line!(),
                format_args!($($arg)+),
            );
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        if $crate::logger::LogLevel::Warn <= $crate::logger::MAX_LOG_LEVEL {
            $crate::logger::log_internal(
                $crate::logger::LogLevel::Warn,
                file!(),
                line!(),
                format_args!($($arg)+),
            );
        }
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        if $crate::logger::LogLevel::Info <= $crate::logger::MAX_LOG_LEVEL {
            $crate::logger::log_internal(
                $crate::logger::LogLevel::Info,
                file!(),
                line!(),
                format_args!($($arg)+),
            );
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        if $crate::logger::LogLevel::Debug <= $crate::logger::MAX_LOG_LEVEL {
            $crate::logger::log_internal(
                $crate::logger::LogLevel::Debug,
                file!(),
                line!(),
                format_args!($($arg)+),
            );
        }
    };
}

#[macro_export]
macro_rules! fine {
    ($($arg:tt)+) => {
        if $crate::logger::LogLevel::Fine <= $crate::logger::MAX_LOG_LEVEL {
            $crate::logger::log_internal(
                $crate::logger::LogLevel::Fine,
                file!(),
                line!(),
                format_args!($($arg)+),
            );
        }
    };
}

#[macro_export]
macro_rules! finer {
    ($($arg:tt)+) => {
        if $crate::logger::LogLevel::Finer <= $crate::logger::MAX_LOG_LEVEL {
            $crate::logger::log_internal(
                $crate::logger::LogLevel::Finer,
                file!(),
                line!(),
                format_args!($($arg)+),
            );
        }
    };
}

#[macro_export]
macro_rules! finest {
    ($($arg:tt)+) => {
        if $crate::logger::LogLevel::Finest <= $crate::logger::MAX_LOG_LEVEL {
            $crate::logger::log_internal(
                $crate::logger::LogLevel::Finest,
                file!(),
                line!(),
                format_args!($($arg)+),
            );
        }
    };
}
