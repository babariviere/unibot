use error::*;
use hyper::client::IntoUrl;
use hyper::Url;
use site::Site;

/// Handle all crawled url
#[derive(Debug, Default)]
pub struct Indexer {
    sites: Vec<Site>,
}

impl Indexer {
    pub fn new() -> Indexer {
        Indexer { sites: Vec::new() }
    }

    /// Add a url to indexer
    ///
    /// If url is not indexed, then it will create a new site with this url
    pub fn add_url<U: IntoUrl>(&mut self, url: U) -> Result<()> {
        let url = url.into_url()?;
        for site in &mut self.sites {
            if site.contains_url(&url) {
                bail!(ErrorKind::UrlAlreadyIndexed);
            }
            if site.is_same_host(&url) {
                if site.is_trap() {
                    bail!(ErrorKind::SpiderTrap);
                }
                // TODO
                if url.as_str().len() > 200 {
                    site.set_trap_state(true);
                    bail!(ErrorKind::SpiderTrap);
                }
                debug!("ADDING TO INDEX {}", url);
                site.add_sub_url(url);
                return Ok(());
            }
        }

        debug!("ADDING SITE {}", url);
        self.sites.push(Site::new(url)?);
        Ok(())
    }

    /// Return all sites that indexer provide
    pub fn get_sites(&self) -> &Vec<Site> {
        &self.sites
    }

    /// Return all url from all sites
    pub fn get_all_urls(&self) -> Vec<&Url> {
        let mut vec = Vec::new();
        for site in &self.sites {
            vec.push(site.get_url());
            vec.extend(site.get_subs_url());
        }
        vec
    }

    /// Return all mains url
    pub fn get_all_main_urls(&self) -> Vec<&Url> {
        self.sites.iter().map(|s| s.get_url()).collect()
    }

    /// Return all subs url
    pub fn get_all_subs_urls(&self) -> Vec<&Url> {
        let mut vec = Vec::new();
        for site in &self.sites {
            vec.extend(site.get_subs_url());
        }
        vec
    }

    /// Check if url is indexed
    pub fn is_indexed(&self, url: &Url) -> Result<bool> {
        for site in &self.sites {
            if site.contains_url(url) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod unit_tests {
    use hyper::client::IntoUrl;
    use super::Indexer;

    fn add_set_of_url(indexer: &mut Indexer) {
        for url in &all_urls() {
            indexer.add_url(*url).unwrap();
        }
    }

    fn all_urls() -> Vec<&'static str> {
        let mut vec = main_urls();
        vec.extend(subs_urls());
        vec
    }

    fn main_urls() -> Vec<&'static str> {
        vec!["http://example.com/", "http://google.com/"]
    }

    fn subs_urls() -> Vec<&'static str> {
        vec!["http://example.com/hello", "http://example.com/hello/world"]
    }

    #[test]
    fn add_url() {
        let mut indexer = Indexer::new();
        add_set_of_url(&mut indexer);
        assert_eq!(indexer.get_sites().len(), 2);
    }

    #[test]
    fn is_indexed() {
        let mut indexer = Indexer::new();
        add_set_of_url(&mut indexer);
        assert!(indexer.is_indexed(&"http://google.com".into_url().unwrap()).unwrap());
        assert!(indexer.is_indexed(&"http://example.com".into_url().unwrap()).unwrap());
        assert!(!indexer.is_indexed(&"http://bing.com".into_url().unwrap()).unwrap());
    }

    #[test]
    fn get_all_urls() {
        let mut indexer = Indexer::new();
        add_set_of_url(&mut indexer);
        let mut urls = all_urls();
        urls.sort();
        for (i, url) in indexer.get_all_urls().iter().enumerate() {
            assert_eq!(url.as_str(), urls[i]);
        }
    }

    #[test]
    fn get_all_main_urls() {
        let mut indexer = Indexer::new();
        add_set_of_url(&mut indexer);
        let urls = main_urls();
        for (i, url) in indexer.get_all_main_urls().iter().enumerate() {
            assert_eq!(url.as_str(), urls[i]);
        }
    }

    #[test]
    fn get_all_subs_urls() {
        let mut indexer = Indexer::new();
        add_set_of_url(&mut indexer);
        let urls = subs_urls();
        for (i, url) in indexer.get_all_subs_urls().iter().enumerate() {
            assert_eq!(url.as_str(), urls[i]);
        }
    }
}
