use std::fmt::Arguments;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

// Define Log Levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// This function performs the "Calendar Math" to convert
/// Unix seconds into a human-readable string without dependencies.
fn get_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let total_seconds = now.as_secs();
    let millis = now.as_millis() % 1000;

    // Calculate seconds, minutes, hours
    let sec = total_seconds % 60;
    let min = (total_seconds / 60) % 60;
    let hour = (total_seconds / 3600) % 24;

    // Calculate Days since Epoch
    let mut days = total_seconds / 86400;

    // Calculate Year
    let mut year = 1970;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if leap { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    // Calculate Month (based on remaining days)
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let month_days = [
        31,
        if is_leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for m_days in month_days.iter() {
        if days < *m_days {
            break;
        }
        days -= m_days;
        month += 1;
    }
    // If we exited the loop because we ran out of months, 'month' might be index-off
    let month = if month > 12 { 12 } else { month };

    // Return formatted string: [YYYY-MM-DD HH:MM:SS.mmm]
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
        year,
        month,
        days + 1, // days is zero-indexed
        hour,
        min,
        sec,
        millis
    )
}

pub fn print_log(level: LogLevel, module: &str, args: Arguments<'_>) {
    // let now = Local::now();
    let now = get_timestamp();

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    let _ = writeln!(
        handle,
        "[{}] [{}] [{}] - {}",
        now,
        // now.format("%Y-%m-%d %H:%M:%S%.3f"),
        level.as_str(),
        module,
        args
    );
}

// The Macros
// Capture `module_path!()` at the call site automatically

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        $crate::print_log($crate::LogLevel::Debug, module_path!(), format_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        $crate::print_log($crate::LogLevel::Info, module_path!(), format_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        $crate::print_log($crate::LogLevel::Warn, module_path!(), format_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        $crate::print_log($crate::LogLevel::Error, module_path!(), format_args!($($arg)+));
    };
}
