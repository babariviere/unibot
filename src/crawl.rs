use errors::*;
use hyper::client::{Client, IntoUrl};
use hyper::net::HttpsConnector;
use hyper::Url;
use hyper_native_tls::NativeTlsClient;
use indexer::Indexer;
use select::document::Document;
use select::predicate::Attr;
use std::io::Read;

// Add settings to go deeper or else
#[derive(Debug, Default)]
pub struct Crawler {
    client: Client,
    indexer: Indexer,
}

impl Crawler {
    pub fn new() -> Crawler {
        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);
        Crawler {
            client: Client::with_connector(connector),
            indexer: Indexer::new(),
        }
    }

    /// Crawl site, index it and return the url and the body.
    pub fn crawl<U: IntoUrl>(&mut self, url: U) -> Result<(Url, String)> {
        let mut reponse = self.client.get(url).send()?;
        let url = reponse.url.clone();
        self.indexer.add_url(url.clone())?;
        let mut body = String::new();
        reponse.read_to_string(&mut body)?;
        Ok((url, body))
    }

    /// Crawl site, index it and return the url and the parsed body.
    pub fn crawl_doc<U: IntoUrl>(&mut self, url: U) -> Result<(Url, Document)> {
        let (url, body) = self.crawl(url)?;
        let doc = Document::from(body.as_str());
        Ok((url, doc))
    }

    /// Crawl site recursively
    pub fn crawl_recursive<U: IntoUrl>(&mut self, url: U) -> Result<Vec<(Url, Document)>> {
        let mut crawled = Vec::new();
        let url = url.into_url()?;
        let (url, doc) = match self.crawl_doc(url) {
            Ok(u) => u,
            _ => return Ok(crawled),
        };
        println!("==> Crawling {}", url);
        let hrefs = doc.find(Attr("href", ()));
        for node in hrefs.iter() {
            let href = node.attr("href").unwrap();
            if href.starts_with('/') {
                let mut url = url.clone();
                url.set_path(href);
                let result = self.crawl_recursive(url)?;
                crawled.extend(result);
            } else if href.starts_with("http") {
                let result = self.crawl_recursive(href)?;
                crawled.extend(result);
            }
        }
        Ok(crawled)
    }
}
