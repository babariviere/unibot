extern crate libunibot;
extern crate log;
extern crate term;

use libunibot::crawl::Crawler;
use log::{Log, LogRecord, LogLevel, LogMetadata, SetLoggerError};
use term::stdout;
use term::color::*;

fn main() {
    let mut args = ::std::env::args();
    args.next();
    init_with_level(LogLevel::Debug).unwrap();
    let mut crawler = Crawler::new();
    for arg in args {
        crawler.add_to_queue(&arg).unwrap();
    }
    let v = match crawler.crawl_site() {
        Ok(v) => v,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };
    println!("VISITED:");
    for url in v {
        println!("- {}", url);
    }
}

pub fn init_with_level(log_level: LogLevel) -> Result<(), SetLoggerError> {
    log::set_logger(|max_log_level| {
        max_log_level.set(log_level.to_log_level_filter());
        Box::new(Logger { level: log_level })
    })
}


pub struct Logger {
    level: LogLevel,
}

impl Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.level && metadata.target().contains("libunibot")
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            self.log_result(record);
        }
    }
}

impl Logger {
    fn log_result(&self, record: &LogRecord) {
        let mut t = stdout().unwrap();
        t.fg(BRIGHT_BLUE).unwrap();
        match record.level() {
            LogLevel::Error => t.fg(BRIGHT_RED).unwrap(),
            LogLevel::Warn => t.fg(BRIGHT_YELLOW).unwrap(),
            LogLevel::Info => t.fg(BRIGHT_GREEN).unwrap(),
            LogLevel::Debug => t.fg(BRIGHT_CYAN).unwrap(),
            LogLevel::Trace => t.fg(BRIGHT_WHITE).unwrap(),
        };
        write!(t, "[{:<5}] ", record.level()).unwrap();
        t.reset().unwrap();
        writeln!(t, "{}", record.args()).unwrap();
    }
}
