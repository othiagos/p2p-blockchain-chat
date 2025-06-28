use colored::Colorize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, OnceLock};
use std::fs::{OpenOptions, File};
use std::io::{self, Write};
use chrono::{Utc, DateTime};
use std::path::Path;

static LOG_LEVEL: AtomicUsize = AtomicUsize::new(LogLevel::Info as usize);
static USE_TIME: AtomicBool = AtomicBool::new(true);
static LOG_FILE: OnceLock<Mutex<Option<File>>> = OnceLock::new();

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[allow(dead_code)]
pub fn set_log_level(level: LogLevel) {
    LOG_LEVEL.store(level as usize, Ordering::SeqCst);
}

#[allow(dead_code)]
pub fn set_log_file(path: &str) -> io::Result<()> {

    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    let lock = LOG_FILE.get_or_init(|| Mutex::new(None));
    let mut guard = lock.lock().unwrap();
    *guard = Some(file);

    Ok(())
}

#[allow(dead_code)]
pub fn set_log_time(use_time: bool) {
    USE_TIME.store(use_time, Ordering::SeqCst);
}

fn get_time_string() -> String {
    if !USE_TIME.load(Ordering::SeqCst) {
        return String::new();
    }
    
    let now: DateTime<Utc> = Utc::now();
    now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true) + " "
}

fn log_message(level: LogLevel, message: &str) {
    if (level as usize) < LOG_LEVEL.load(Ordering::SeqCst) {
        return;
    }

    let level_str = match level {
        LogLevel::Debug => "[DEBUG]".blue(),
        LogLevel::Info => "[INFO]".green(),
        LogLevel::Warn => "[WARN]".yellow(),
        LogLevel::Error => "[ERROR]".red(),
        LogLevel::Fatal => "[FATAL]".on_red().white().bold(),
    };

    let time_str = get_time_string().dimmed();

    let formatted = format!("{}{} {}\n", time_str, level_str, message);

    try_log_to_file(level, message, formatted);
}

fn try_log_to_file(level: LogLevel, message: &str, formatted: String) {
    if let Some(lock) = LOG_FILE.get() {
        let mut guard = lock.lock().unwrap();

        if let Some(file) = guard.as_mut() {

            let plain = format!(
                "{}[{}] {}\n",
                get_time_string(),
                match level {
                    LogLevel::Debug => "DEBUG",
                    LogLevel::Info => "INFO",
                    LogLevel::Warn => "WARN",
                    LogLevel::Error => "ERROR",
                    LogLevel::Fatal => "FATAL",
                },
                message
            );

            let _ = file.write_all(plain.as_bytes());
            let _ = file.flush();
            return;
        }
    }

    print!("{}", formatted);
}

#[allow(dead_code)]
pub fn debug(message: &str) {
    log_message(LogLevel::Debug, message);
}

#[allow(dead_code)]
pub fn info(message: &str) {
    log_message(LogLevel::Info, message);
}

#[allow(dead_code)]
pub fn warn(message: &str) {
    log_message(LogLevel::Warn, message);
}

#[allow(dead_code)]
pub fn error(message: &str) {
    log_message(LogLevel::Error, message);
}

#[allow(dead_code)]
pub fn fatal(message: &str) {
    log_message(LogLevel::Fatal, message);
}
