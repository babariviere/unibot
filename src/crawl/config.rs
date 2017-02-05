use hyper::Url;
use select::document::Document;
use std::sync::Arc;

#[derive(Clone)]
pub struct CrawlerConfig {
    crawled: Arc<Fn(&Url, &Document) + Send + Sync>,
    filter: Arc<Fn(&Url, &Url) -> bool + Send + Sync>,
    sleep_ms: u64,
}

impl CrawlerConfig {
    pub fn new() -> CrawlerConfig {
        CrawlerConfig {
            crawled: Arc::new(|_, _| {}),
            filter: Arc::new(|_, _| true),
            sleep_ms: 1000,
        }
    }

    pub fn new_site_only() -> CrawlerConfig {
        CrawlerConfig::new().set_filter(|old, new| old.domain() == new.domain())
    }

    pub fn crawled(&self, url: &Url, doc: &Document) {
        (self.crawled)(url, doc)
    }

    pub fn filter(&self, old_url: &Url, new_url: &Url) -> bool {
        (self.filter)(old_url, new_url)
    }

    pub fn sleep_ms(&self) -> u64 {
        self.sleep_ms
    }

    pub fn set_crawled<F>(mut self, crawled: F) -> CrawlerConfig
        where F: 'static + Send + Sync + Fn(&Url, &Document)
    {
        self.crawled = Arc::new(crawled);
        self
    }

    pub fn set_filter<F>(mut self, filter: F) -> CrawlerConfig
        where F: 'static + Send + Sync + Fn(&Url, &Url) -> bool
    {
        self.filter = Arc::new(filter);
        self
    }

    pub fn set_sleep_ms(mut self, sleep_ms: u64) -> CrawlerConfig {
        self.sleep_ms = sleep_ms;
        self
    }
}
