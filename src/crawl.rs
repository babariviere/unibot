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
use std::sync::{Arc, Mutex};

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

    /// Add an url to the queue
    pub fn add_to_queue<U: IntoUrl>(&mut self, url: U) -> Result<()> {
        let url = url.into_url()?;
        let mut queue = match self.queue.lock() {
            Ok(q) => q,
            Err(e) => bail!(ErrorKind::PoisonError(e.to_string())),
        };
        debug!("QUEUE PUSH {}", url);
        queue.push(url);
        Ok(())
    }

    /// Get all item from queue
    pub fn queue_items(&self) -> Result<Vec<Url>> {
        let queue = match self.queue.lock() {
            Ok(q) => q,
            Err(e) => bail!(ErrorKind::PoisonError(e.to_string())),
        };
        Ok(queue.clone())
    }

    /// Pop an url from queue
    pub fn pop_queue(&mut self) -> Result<Url> {
        let mut queue = match self.queue.lock() {
            Ok(q) => q,
            Err(e) => bail!(ErrorKind::PoisonError(e.to_string())),
        };
        let url = queue.pop();
        match url {
            Some(u) => {
                debug!("QUEUE POP {}", u);
                Ok(u)
            }
            None => bail!(ErrorKind::QueueEmpty),
        }
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

    /// Crawl site recursively
    pub fn crawl_recursive(&mut self) -> Result<Vec<(Url, Document)>> {
        let mut crawled = Vec::new();
        let (url, doc) = match self.crawl_doc() {
            Ok(u) => u,
            Err(Error(ErrorKind::UrlAlreadyIndexed, _)) => return Ok(crawled),
            Err(e) => return Err(e),
        };
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
            if href.starts_with("/www") || href.starts_with("http") {
                self.add_to_queue(href)?;
            } else if href.starts_with('/') {
                let mut url = url.clone();
                let href = beautify_url(&format!("{}/{}", url.path(), href));
                url.set_path(&href);
                self.add_to_queue(url)?;
            }
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
