extern crate clap;
extern crate libunibot;
extern crate term;

use clap::{App, Arg};
use libunibot::crawl::{CrawlerConfig, create_multiple_crawler};
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
            let result = if site_only {
                crawler.crawl_site()
            } else {
                let config = CrawlerConfig::new()
                    .set_crawled(|url, _doc| println!("Crawling {}", url));
                crawler.crawl_recursive(&config)
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
