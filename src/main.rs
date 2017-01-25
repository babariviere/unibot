extern crate libunibot;
extern crate log;
extern crate log4rs;

use libunibot::crawl::Crawler;
use log::LogLevelFilter;
use log::LogRecord;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::{Filter, Response};

fn main() {
    let mut args = ::std::env::args();
    args.next();
    init_logger();
    let mut crawler = Crawler::new();
    for arg in args {
        crawler.add_to_queue(&arg).unwrap();
    }
    let _v = match crawler.crawl_recursive() {
        Ok(v) => v,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };
    println!("{:?}", crawler);
}

fn init_logger() {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{h({l})} {m}\n")))
        .build();

    let config = Config::builder()
        .appender(Appender::builder()
            .filter(Box::new(CustomFilter))
            .build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LogLevelFilter::Debug))
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();
}

#[derive(Debug)]
struct CustomFilter;

impl Filter for CustomFilter {
    fn filter(&self, record: &LogRecord) -> Response {
        if record.target().contains("libunibot") {
            Response::Accept
        } else {
            Response::Reject
        }
    }
}
