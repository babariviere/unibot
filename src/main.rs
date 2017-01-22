extern crate libunibot;

use libunibot::crawl::Crawler;

fn main() {
    println!("Welcome to my crawler!");
    let mut args = ::std::env::args();
    args.next();
    let mut crawler = Crawler::new();
    for arg in args {
        let (url, body) = match crawler.crawl(&arg) {
            Ok((url, body)) => (url, body),
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };
        println!("{}", url);
    }
    println!("{:?}", crawler);
}
