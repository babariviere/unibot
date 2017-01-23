extern crate libunibot;

use libunibot::crawl::Crawler;

fn main() {
    let mut args = ::std::env::args();
    args.next();
    let mut crawler = Crawler::new();
    for arg in args {
        let v = match crawler.crawl_recursive(&arg) {
            Ok(v) => v,
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };
    }
    println!("{:?}", crawler);
}
