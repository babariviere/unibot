extern crate libunibot;

use libunibot::crawl::Crawler;

fn main() {
    let mut args = ::std::env::args();
    args.next();
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
