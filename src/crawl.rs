use common::href_to_url;
use error::*;
use hyper::client::{Client, IntoUrl};
use hyper::net::HttpsConnector;
use hyper::Url;
use hyper_native_tls::NativeTlsClient;
use indexer::Indexer;
use scrap::scrap_attr;
use select::document::Document;
use std::collections::VecDeque;
use std::io::Read;
use std::sync::{Arc, Mutex, MutexGuard};

// Add settings to go deeper or else
#[derive(Debug, Default)]
pub struct Crawler {
    client: Client,
    indexer: Arc<Mutex<Indexer>>,
    queue: Arc<Mutex<VecDeque<Url>>>,
    count: usize,
}

impl Crawler {
    pub fn new() -> Crawler {
        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);
        Crawler {
            client: Client::with_connector(connector),
            indexer: Arc::new(Mutex::new(Indexer::new())),
            queue: Arc::new(Mutex::new(VecDeque::new())),
            count: 0,
        }
    }

    pub fn new_shared(indexer: Arc<Mutex<Indexer>>, queue: Arc<Mutex<VecDeque<Url>>>) -> Crawler {
        let mut crawler = Crawler::new();
        crawler.indexer = indexer;
        crawler.queue = queue;
        crawler
    }

    /// Return a mutable reference to queue
    fn lock_queue(&self) -> Result<MutexGuard<VecDeque<Url>>> {
        match self.queue.lock() {
            Ok(q) => Ok(q),
            Err(e) => bail!(ErrorKind::PoisonError(e.to_string())),
        }
    }

    /// Return a mutable reference to indexer
    fn lock_indexer(&self) -> Result<MutexGuard<Indexer>> {
        match self.indexer.lock() {
            Ok(i) => Ok(i),
            Err(e) => bail!(ErrorKind::PoisonError(e.to_string())),
        }
    }

    /// Return a copy of queue
    pub fn queue(&self) -> Arc<Mutex<VecDeque<Url>>> {
        self.queue.clone()
    }

    /// Return a copy of indexer
    pub fn indexer(&self) -> Arc<Mutex<Indexer>> {
        self.indexer.clone()
    }

    /// Add an url to the queue
    pub fn add_to_queue<U: IntoUrl>(&mut self, url: U) -> Result<()> {
        let url = url.into_url()?;
        let mut queue = self.lock_queue()?;
        if !queue.contains(&url) && !self.lock_indexer()?.is_indexed(&url) {
            queue.push_back(url);
        }
        Ok(())
    }

    /// Get all item from queue
    pub fn queue_items(&self) -> Result<VecDeque<Url>> {
        let queue = self.lock_queue()?;
        Ok(queue.clone())
    }

    /// Check if queue is empty
    pub fn is_queue_empty(&self) -> bool {
        let queue = match self.lock_queue() {
            Ok(q) => q,
            Err(_) => return true,
        };
        queue.is_empty()
    }

    /// Pop an url from queue
    pub fn pop_queue(&mut self) -> Result<Url> {
        let mut queue = self.lock_queue()?;
        let url = queue.pop_front();
        match url {
            Some(u) => Ok(u),
            None => bail!(ErrorKind::QueueEmpty),
        }
    }

    /// Free queue
    pub fn free_queue(&mut self) -> Result<()> {
        let mut queue = self.lock_queue()?;
        queue.clear();
        Ok(())
    }

    /// Crawl site from queue, index it and return url and the body.
    pub fn crawl(&mut self) -> Result<(Url, String)> {
        let url = self.pop_queue()?;
        let mut reponse = self.client.get(url.clone()).send()?;
        self.lock_indexer()?.add_url(url.clone())?;
        let mut buf = Vec::new();
        let body = match reponse.read_to_end(&mut buf) {
            Ok(_) => String::from_utf8_lossy(&*buf).into_owned(),
            Err(e) => bail!(e),
        };
        Ok((url, body))
    }

    /// Crawl site, index it and return the url and the parsed body.
    pub fn crawl_doc(&mut self) -> Result<(Url, Document)> {
        let (url, body) = self.crawl()?;
        let doc = Document::from(body.as_str());
        Ok((url, doc))
    }

    /// Crawl site recursively until queue is empty with a filter
    pub fn crawl_recursive(&mut self, config: &CrawlerConfig) -> Result<Vec<Url>> {
        let mut crawled = Vec::new();
        while !self.is_queue_empty() {
            let (v_url, doc) = match self.crawl_doc() {
                Ok(t) => t,
                Err(e) => return Err(e),
            };
            config.crawled(&v_url, &doc);
            crawled.push(v_url.clone());
            self.count += 1;
            let hrefs = scrap_attr(&doc, "href");
            for href in hrefs {
                if href.starts_with('#') {
                    continue;
                }
                let url = match href_to_url(&v_url, &href) {
                    Some(u) => u,
                    None => continue,
                };
                if config.filter(&v_url, &url) {
                    if let Err(_e) = self.add_to_queue(url) {
                        continue;
                    }
                }
            }
        }
        Ok(crawled)
    }

    /// Crawl only a site
    pub fn crawl_site(&mut self) -> Result<Vec<Url>> {
        let config = CrawlerConfig::new_site_only();
        self.crawl_recursive(&config)
    }
}

pub fn create_multiple_crawler(queue: Vec<&str>, crawler_size: usize) -> Vec<Crawler> {
    let mut crawlers = Vec::new();
    let mut crawler = Crawler::new();
    for url in queue {
        let _ = crawler.add_to_queue(url);
    }
    for _ in 1..crawler_size {
        let indexer = crawler.indexer();
        let queue = crawler.queue();
        let crawler = Crawler::new_shared(indexer, queue);
        crawlers.push(crawler);
    }
    crawlers.push(crawler);
    crawlers
}

pub struct CrawlerConfig {
    crawled: Box<Fn(&Url, &Document)>,
    filter: Box<Fn(&Url, &Url) -> bool>,
}

impl CrawlerConfig {
    pub fn new() -> CrawlerConfig {
        CrawlerConfig {
            crawled: Box::new(|_, _| {}),
            filter: Box::new(|_, _| true),
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

    pub fn set_crawled<F>(mut self, crawled: F) -> CrawlerConfig
        where F: 'static + Fn(&Url, &Document)
    {
        self.crawled = Box::new(crawled);
        self
    }

    pub fn set_filter<F>(mut self, filter: F) -> CrawlerConfig
        where F: 'static + Fn(&Url, &Url) -> bool
    {
        self.filter = Box::new(filter);
        self
    }
}

#[cfg(test)]
mod unit_tests {
    use hyper::client::IntoUrl;
    use hyper::Url;
    use super::Crawler;

    fn url_list() -> Vec<Url> {
        let vec = vec!["http://example.com", "http://google.com", "http://duckduckgo.com"];
        vec.iter().map(|u| u.into_url().unwrap()).collect()
    }

    #[test]
    fn queue() {
        let mut crawler = Crawler::new();
        for url in url_list() {
            crawler.add_to_queue(url).unwrap();
        }
        let url_list = url_list();
        for (i, q_url) in crawler.queue_items().unwrap().iter().enumerate() {
            assert_eq!(q_url, &url_list[i]);
        }
    }
}
