use common::beautify_url;
use error::*;
use hyper::client::{Client, IntoUrl};
use hyper::net::HttpsConnector;
use hyper::Url;
use hyper_native_tls::NativeTlsClient;
use indexer::Indexer;
use select::document::Document;
use select::predicate::Attr;
use std::io::Read;
use std::sync::{Arc, Mutex, MutexGuard};

// Add settings to go deeper or else
#[derive(Debug, Default)]
pub struct Crawler {
    client: Client,
    indexer: Indexer,
    queue: Arc<Mutex<Vec<Url>>>,
    count: usize,
}

impl Crawler {
    pub fn new() -> Crawler {
        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);
        Crawler {
            client: Client::with_connector(connector),
            indexer: Indexer::new(),
            queue: Arc::new(Mutex::new(Vec::new())),
            count: 0,
        }
    }

    /// Return a mutable reference to queue
    fn lock_queue<'a>(&'a self) -> Result<MutexGuard<'a, Vec<Url>>> {
        match self.queue.lock() {
            Ok(q) => Ok(q),
            Err(e) => bail!(ErrorKind::PoisonError(e.to_string())),
        }
    }

    /// Add an url to the queue
    pub fn add_to_queue<U: IntoUrl>(&mut self, url: U) -> Result<()> {
        let url = url.into_url()?;
        let mut queue = self.lock_queue()?;
        if !queue.contains(&url) {
            debug!("QUEUE PUSH {}", url);
            queue.push(url);
        }
        Ok(())
    }

    /// Get all item from queue
    pub fn queue_items(&self) -> Result<Vec<Url>> {
        let queue = self.lock_queue()?;
        Ok(queue.clone())
    }

    /// Pop an url from queue
    pub fn pop_queue(&mut self) -> Result<Url> {
        let mut queue = self.lock_queue()?;
        let url = queue.pop();
        match url {
            Some(u) => {
                debug!("QUEUE POP {}", u);
                Ok(u)
            }
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
        debug!("CRAWLING SITE {}", url);
        let mut reponse = self.client.get(url.clone()).send()?;
        self.indexer.add_url(url.clone())?;
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

    /// Crawl site recursively until queue is empty
    pub fn crawl_recursive(&mut self) -> Result<Vec<Url>> {
        let mut crawled = Vec::new();
        // Only for debug
        if self.count > 50 {
            return Ok(crawled);
        }
        let (url, doc) = match self.crawl_doc() {
            Ok(u) => u,
            Err(e) => {
                error!("{}", e);
                match e {
                    Error(ErrorKind::UrlAlreadyIndexed, _) |
                    Error(ErrorKind::SpiderTrap, _) => return Ok(crawled),
                    _ => return Err(e),
                }
            }
        };
        crawled.push(url.clone());
        self.count += 1;
        info!("[{}] Crawling {}", self.count, url);
        let srcs = doc.find(Attr("src", ()));
        for node in srcs.iter() {
            let src = node.attr("src").unwrap();
            if src.starts_with("http") {
                debug!("SRC {}", src);
            } else {
                debug!("SRC {}{}", url, src);
            }
        }
        let hrefs = doc.find(Attr("href", ()));
        for node in hrefs.iter() {
            let href = node.attr("href").unwrap();
            if href.starts_with('#') {
                continue;
            }
            let url = url.clone();
            let url = if href.starts_with("//") {
                let scheme = url.scheme();
                match format!("{}:{}", scheme, href).into_url() {
                    Ok(u) => u,
                    _ => continue,
                }
            } else if href.starts_with("http") {
                match href.into_url() {
                    Ok(u) => u,
                    _ => continue,
                }
            } else if href.starts_with('/') {
                let mut url = url.clone();
                url.set_path(href);
                url
            } else {
                let mut url = url.clone();
                let href = beautify_url(format!("{}/{}", url, href));
                url.set_path(&href);
                url
            };
            self.add_to_queue(url)?;
            let result = self.crawl_recursive()?;
            crawled.extend(result);
        }
        Ok(crawled)
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
