use chrono::Utc;
use colored::*;

pub enum LogLevel {
    Info,
    Warning,
    Error,
    Success,
}

pub fn log(level: LogLevel, message: &str) {
    let timestamp = Utc::now().format("%H:%M:%S");

    match level {
        LogLevel::Info => {
            println!(
                "{} {} {}",
                format!("[{}]", timestamp).dimmed(),
                "ℹ️ INFO".blue().bold(),
                message
            );
        }
        LogLevel::Warning => {
            println!(
                "{} {} {}",
                format!("[{}]", timestamp).dimmed(),
                "⚠️ WARNING".yellow().bold(),
                message
            );
        }
        LogLevel::Error => {
            println!(
                "{} {} {}",
                format!("[{}]", timestamp).dimmed(),
                "❌ ERROR".red().bold(),
                message
            );
        }
        LogLevel::Success => {
            println!(
                "{} {} {}",
                format!("[{}]", timestamp).dimmed(),
                "✅ SUCCESS".green().bold(),
                message
            );
        }
    }
}

pub fn log_info(message: &str) {
    log(LogLevel::Info, message);
}

pub fn log_warning(message: &str) {
    log(LogLevel::Warning, message);
}

pub fn log_error(message: &str) {
    log(LogLevel::Error, message);
}

pub fn log_success(message: &str) {
    log(LogLevel::Success, message);
}
