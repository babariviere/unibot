extern crate clap;
extern crate libunibot;
extern crate term;

use clap::{App, Arg};
use libunibot::crawl::Crawler;
use libunibot::crawl::config::CrawlerConfig;
use std::thread;

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

    let sites = app.values_of("sites").unwrap();
    let site_only = app.is_present("site-only");
    let jobs = app.value_of("jobs").unwrap_or("1").trim().parse::<usize>().unwrap_or(1);

    let mut crawler = Crawler::new();
    crawler.create_slaves(jobs);
    for site in sites {
        crawler.add_to_queue(site).unwrap();
    }
    let receivers = if site_only {
        crawler.crawl_recursive(&CrawlerConfig::new_site_only().set_sleep_ms(500)).unwrap()
    } else {
        crawler.crawl_recursive(&CrawlerConfig::new().set_sleep_ms(500)).unwrap()
    };
    while crawler.get_running() > 0 {
        for receiver in &receivers {
            match receiver.try_recv() {
                Ok(u) => {
                    println!("Visited {}", u);
                }
                Err(_) => continue,
            }
        }
        thread::sleep(::std::time::Duration::from_secs(1));
    }
}
