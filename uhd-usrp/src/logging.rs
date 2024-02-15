use std::path::{Path, PathBuf};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warning = 3,
    Error = 4,
    Fatal = 5,
    Off = 6,
}

impl LogLevel {
    fn from_env_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "trace" => Some(LogLevel::Trace),
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warning" => Some(LogLevel::Warning),
            "error" => Some(LogLevel::Error),
            "fatal" => Some(LogLevel::Fatal),
            "off" => Some(LogLevel::Off),
            _ => None,
        }
    }
}

impl ToString for LogLevel {
    fn to_string(&self) -> String {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
            LogLevel::Fatal => "fatal",
            LogLevel::Off => "off",
        }
        .to_owned()
    }
}

/// Enable or disable fastpath logging.
///
/// Fastpath logging is used by UHD for fast logging in performance-critical functions.
/// When fastpath logging is enabled, `U`/`O`/`S`/`D`/`L` characters will be printed
/// to the console during streaming to indicate various conditions.
pub fn set_fastpath_logging_enabled(en: bool) {
    std::env::set_var("UHD_LOG_FASTPATH_DISABLE", if en { "OFF" } else { "ON" });
}

pub fn fastpath_logging_enabled() -> bool {
    let var = std::env::var("UHD_LOG_FASTPATH_DISABLE").unwrap_or("OFF".to_owned());
    match var.as_str() {
        "ON" => false,
        _ => true,
    }
}

/// Set the global minimum log level.
pub fn set_global_log_level(level: LogLevel) {
    std::env::set_var("UHD_LOG_LEVEL", level.to_string());
}

pub fn global_log_level() -> LogLevel {
    log_level_from_var("UHD_LOG_LEVEL")
}

/// Set the minimum log level for files.
pub fn set_file_log_level(level: LogLevel) {
    std::env::set_var("UHD_LOG_FILE_LEVEL", level.to_string());
}

pub fn file_log_level() -> LogLevel {
    log_level_from_var("UHD_LOG_FILE_LEVEL")
}

/// Set the minimum log level for the console.
pub fn set_console_log_level(level: LogLevel) {
    std::env::set_var("UHD_LOG_CONSOLE_LEVEL", level.to_string());
}

pub fn console_log_level() -> LogLevel {
    log_level_from_var("UHD_LOG_CONSOLE_LEVEL")
}

/// Set the minimum log level for the console.
pub fn set_log_file(path: Option<impl AsRef<Path>>) {
    let path = path
        .as_ref()
        .map(|p| p.as_ref().to_string_lossy().to_string())
        .unwrap_or("".to_owned());
    std::env::set_var("UHD_LOG_FILE", path);
}

/// Set the minimum log level for the console.
pub fn log_file() -> Option<PathBuf> {
    std::env::var("UHD_LOG_FILE").ok().map(|p| PathBuf::from(p))
}

fn log_level_from_var(var_name: &str) -> LogLevel {
    let var = std::env::var(var_name).unwrap_or("".to_owned());
    LogLevel::from_env_str(&var).unwrap_or(LogLevel::Debug)
}
