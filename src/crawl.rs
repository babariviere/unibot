use errors::*;
use hyper::client::{Client, IntoUrl};
use hyper::net::HttpsConnector;
use hyper::Url;
use hyper_native_tls::NativeTlsClient;
use indexer::Indexer;
use std::io::Read;

#[derive(Debug, Default)]
pub struct Crawler {
    client: Client,
    indexer: Indexer, 
    // Add settings to go deeper or else
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

    pub fn crawl<U: IntoUrl>(&mut self, url: U) -> Result<(Url, String)> {
        let mut reponse = self.client.get(url).send()?;
        let url = reponse.url.clone();
        self.indexer.add_url(url.clone())?;
        let mut body = String::new();
        reponse.read_to_string(&mut body)?;
        Ok((url, body))
    }
}
