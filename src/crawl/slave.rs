use common::href_to_url;
use error::*;
use hyper::client::Client;
use hyper::net::HttpsConnector;
use hyper::Url;
use hyper_native_tls::NativeTlsClient;
use indexer::Indexer;
use scrap::scrap_attr;
use select::document::Document;
use std::collections::VecDeque;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use super::config::CrawlerConfig;
use super::sync;

#[derive(Debug)]
pub struct CrawlerSlave {
    client: Client,
    indexer: Arc<Mutex<Indexer>>,
    queue: Arc<Mutex<VecDeque<Url>>>,
    running: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
}

impl CrawlerSlave {
    pub fn new() -> CrawlerSlave {
        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);
        CrawlerSlave {
            client: Client::with_connector(connector),
            indexer: Arc::new(Mutex::new(Indexer::new())),
            queue: Arc::new(Mutex::new(VecDeque::new())),
            running: Arc::new(AtomicUsize::new(0)),
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn new_shared(indexer: Arc<Mutex<Indexer>>,
                      queue: Arc<Mutex<VecDeque<Url>>>,
                      running: Arc<AtomicUsize>,
                      stop: Arc<AtomicBool>)
                      -> CrawlerSlave {
        let mut crawler = CrawlerSlave::new();
        crawler.indexer = indexer;
        crawler.queue = queue;
        crawler.running = running;
        crawler.stop = stop;
        crawler
    }

    /// Crawl site from queue, index it and return url and the body.
    pub fn crawl(&mut self) -> Result<(Url, String)> {
        let url = sync::pop_queue(&self.queue)?;
        let mut reponse = self.client.get(url.clone()).send()?;
        sync::lock(&self.indexer)?.add_url(url.clone())?;
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
    pub fn crawl_recursive(&mut self, config: CrawlerConfig, tx: Sender<Url>) {
        sync::set_stop(&self.stop, false);
        let sleep = Duration::from_millis(config.sleep_ms());
        while !sync::is_queue_empty(&self.queue) && !sync::get_stop(&self.stop) {
            let (v_url, doc) = match self.crawl_doc() {
                Ok(t) => t,
                Err(_e) => continue,
            };
            config.crawled(&v_url, &doc);
            match tx.send(v_url.clone()) {
                Ok(_) => {}
                Err(_) => continue,
            }
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
                    if let Err(_e) = sync::add_to_queue(&self.indexer, &self.queue, url) {
                        continue;
                    }
                }
            }
            thread::sleep(sleep);
        }
        sync::remove_running(&self.running);
    }
}
