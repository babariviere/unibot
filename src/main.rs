extern crate clap;
extern crate libunibot;
extern crate log;
extern crate term;

use clap::{App, Arg};
use libunibot::crawl::create_multiple_crawler;
use log::{Log, LogRecord, LogLevel, LogMetadata, SetLoggerError};
use std::thread;
use term::stdout;
use term::color::*;

fn main() {
    let app = App::new("unibot")
        .version("dev")
        .author("Bastien Badzioch <notkild@gmail.com")
        .about("Crawl the web")
        .arg(Arg::with_name("sites")
            .help("Sites to crawl")
            .takes_value(true)
            .multiple(true)
            .required(true)
            .value_name("SITE"))
        .arg(Arg::with_name("site-only")
            .short("s")
            .long("site-only")
            .help("Crawl site only"))
        .arg(Arg::with_name("jobs")
            .short("j")
            .long("jobs")
            .takes_value(true)
            .help("Set number of jobs to use"))
        .get_matches();

    init_with_level(LogLevel::Info).unwrap();

    let sites = app.values_of("sites").unwrap();
    let site_only = app.is_present("site-only");
    let jobs = app.value_of("jobs").unwrap_or("1").trim().parse::<usize>().unwrap_or(1);

    let mut queue = Vec::new();
    for site in sites {
        queue.push(site);
    }

    let crawlers = create_multiple_crawler(queue, jobs);
    let mut threads = Vec::new();
    for mut crawler in crawlers {
        let site_only = site_only;
        let thread = thread::spawn(move || {
            println!("{:?}", crawler);
            let result = if site_only {
                crawler.crawl_site()
            } else {
                crawler.crawl_recursive()
            };
            let v = match result {
                Ok(v) => v,
                Err(_) => {
                    return;
                }
            };
            println!("VISITED:");
            for url in v {
                println!("- {}", url);
            }
        });
        threads.push(thread);
    }
    for thread in threads {
        thread.join().unwrap();
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
