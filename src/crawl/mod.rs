pub mod config;
mod slave;
mod sync;

use error::*;
use hyper::client::IntoUrl;
use hyper::Url;
use indexer::Indexer;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use self::config::CrawlerConfig;
use self::slave::CrawlerSlave;

// Add settings to go deeper or else
#[derive(Debug)]
pub struct Crawler {
    slaves: Vec<CrawlerSlave>,
    indexer: Arc<Mutex<Indexer>>,
    queue: Arc<Mutex<VecDeque<Url>>>,
    running: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
}

impl Crawler {
    pub fn new() -> Crawler {
        let mut crawler = Crawler {
            slaves: Vec::new(),
            indexer: Arc::new(Mutex::new(Indexer::new())),
            queue: Arc::new(Mutex::new(VecDeque::new())),
            running: Arc::new(AtomicUsize::new(0)),
            stop: Arc::new(AtomicBool::new(false)),
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
        if self.slaves.is_empty() {
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
    pub fn running(&self) -> Arc<AtomicUsize> {
        self.running.clone()
    }

    /// Return a copy of stop
    pub fn stop(&self) -> Arc<AtomicBool> {
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

impl Default for Crawler {
    fn default() -> Self {
        Self::new()
    }
}
