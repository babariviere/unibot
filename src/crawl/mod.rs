pub mod config;
mod slave;
mod sync;

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
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use self::config::CrawlerConfig;
use self::slave::CrawlerSlave;

// Add settings to go deeper or else
#[derive(Debug)]
pub struct Crawler {
    slaves: Vec<CrawlerSlave>,
    indexer: Arc<Mutex<Indexer>>,
    queue: Arc<Mutex<VecDeque<Url>>>,
    running: Arc<Mutex<usize>>,
    stop: Arc<Mutex<bool>>,
}

impl Crawler {
    pub fn new() -> Crawler {
        let mut crawler = Crawler {
            slaves: Vec::new(),
            indexer: Arc::new(Mutex::new(Indexer::new())),
            queue: Arc::new(Mutex::new(VecDeque::new())),
            running: Arc::new(Mutex::new(0)),
            stop: Arc::new(Mutex::new(false)),
        };
        crawler.add_slave();
        crawler
    }

    /// Add a slave to slaves list
    fn add_slave(&mut self) {
        let indexer = self.indexer();
        let queue = self.queue();
        let running = self.running();
        let stop = self.stop();
        self.slaves.push(CrawlerSlave::new_shared(indexer, queue, running, stop));
    }

    /// Create a set of new slave
    pub fn create_slaves(&mut self, number: usize) {
        self.slaves.clear();
        for _ in 0..number {
            self.add_slave();
        }
        if self.slaves.len() == 0 {
            self.add_slave();
        }
    }

    /// Return a copy of indexer
    pub fn indexer(&self) -> Arc<Mutex<Indexer>> {
        self.indexer.clone()
    }

    /// Return a copy of queue
    pub fn queue(&self) -> Arc<Mutex<VecDeque<Url>>> {
        self.queue.clone()
    }

    /// Return a copy of running
    pub fn running(&self) -> Arc<Mutex<usize>> {
        self.running.clone()
    }

    /// Return a copy of stop
    pub fn stop(&self) -> Arc<Mutex<bool>> {
        self.stop.clone()
    }

    /// Add to queue an url
    pub fn add_to_queue<U: IntoUrl>(&mut self, url: U) -> Result<()> {
        sync::add_to_queue(&self.indexer, &self.queue, url)
    }

    /// Get all items from queue
    pub fn queue_items(&self) -> Result<VecDeque<Url>> {
        sync::queue_items(&self.queue)
    }

    /// Get the number of slave that are running
    pub fn get_running(&self) -> usize {
        sync::get_running(&self.running)
    }

    /// Get stop value
    pub fn get_stop(&self) -> bool {
        sync::get_stop(&self.stop)
    }

    /// Set stop value
    pub fn set_stop(&mut self, stop: bool) {
        sync::set_stop(&self.stop, stop);
    }

    /// Crawl site from queue, index it and return url and the body.
    pub fn crawl(&mut self) -> Result<(Url, String)> {
        self.slaves[0].crawl()
    }

    /// Crawl site, index it and return the url and the parsed body.
    pub fn crawl_doc(&mut self) -> Result<(Url, Document)> {
        self.slaves[0].crawl_doc()
    }

    /// Crawl site only
    pub fn crawl_site(&mut self) -> Result<Vec<Receiver<Url>>> {
        self.crawl_recursive(&CrawlerConfig::new_site_only())
    }

    /// Crawl site recursively until queue is empty with a filter
    pub fn crawl_recursive(&mut self, config: &CrawlerConfig) -> Result<Vec<Receiver<Url>>> {
        let mut rxs = Vec::new();
        while let Some(mut slave) = self.slaves.pop() {
            sync::add_running(&self.running);
            let (tx, rx) = mpsc::channel();
            let config = config.clone();
            thread::spawn(move || slave.crawl_recursive(config, tx));
            rxs.push(rx);
        }
        Ok(rxs)
    }
}
