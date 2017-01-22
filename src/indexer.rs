use errors::*;
use hyper::client::IntoUrl;
use site::Site;

#[derive(Debug, Default)]
pub struct Indexer {
    sites: Vec<Site>,
}

impl Indexer {
    pub fn new() -> Indexer {
        Indexer { sites: Vec::new() }
    }

    // TODO - index site - is indexed - get all main urls - get url - get sub url

    /// Add a url to indexer
    ///
    /// If url is not indexed, then it will create a new site with this url
    pub fn add_url<U: IntoUrl>(&mut self, url: U) -> Result<()> {
        let url = url.into_url()?;
        for site in &mut self.sites {
            if site.contains_url(url.clone()) {
                return Ok(());
            }
            if site.is_same_host(url.clone()) {
                site.add_sub_url(url);
                return Ok(());
            }
        }

        self.sites.push(Site::new(url)?);
        Ok(())
    }

    /// Return all sites that indexer provide
    pub fn get_sites(&self) -> &Vec<Site> {
        &self.sites
    }

    /// Check if url is indexed
    pub fn is_indexed<U: IntoUrl>(&self, url: U) -> Result<bool> {
        let url = url.into_url()?;
        for site in &self.sites {
            if site.contains_url(url.clone()) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod unit_tests {
    use super::Indexer;

    #[test]
    fn add_url() {
        let mut indexer = Indexer::new();
        indexer.add_url("http://example.com").unwrap();
        indexer.add_url("http://example.com/hello").unwrap();
        indexer.add_url("http://example.com/hello/world").unwrap();
        indexer.add_url("http://google.com").unwrap();
        assert_eq!(indexer.get_sites().len(), 2);
    }

    #[test]
    fn is_indexed() {
        let mut indexer = Indexer::new();
        indexer.add_url("http://example.com").unwrap();
        indexer.add_url("http://example.com/hello").unwrap();
        indexer.add_url("http://example.com/hello/world").unwrap();
        indexer.add_url("http://google.com").unwrap();
        assert!(indexer.is_indexed("http://google.com").unwrap());
        assert!(indexer.is_indexed("http://example.com").unwrap());
        assert!(!indexer.is_indexed("http://bing.com").unwrap());
    }
}
