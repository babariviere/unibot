use errors::*;
use url::Url;

/// A structure to define a site.
///
/// `url` - Main url
///
/// `subs_url` - All url provided by site
///
/// `trap` - If site contains spider trap
///
/// `fully_crawled` - If site is fully crawled
#[derive(Debug)]
pub struct Site {
    url: Url,
    subs_url: Vec<Url>,
    trap: bool,
    fully_crawled: bool,
}

// TODO replace string url with a url struct

impl Site {
    /// Create a new instance of site
    pub fn new<S: AsRef<str>>(url: S) -> Result<Site> {
        let url = Url::parse(url.as_ref())?;
        Ok(Site {
            url: url,
            subs_url: Vec::new(),
            trap: false,
            fully_crawled: false,
        })
    }

    /// Add an url that site provide
    pub fn add_sub_url<S: AsRef<str>>(&mut self, sub_url: S) {
        let sub_url = sub_url.as_ref();
        let sub_url = match Url::parse(sub_url) {
            Ok(u) => u,
            Err(_) => return,
        };
        if self.url.host_str() == sub_url.host_str() {
            self.subs_url.push(sub_url);
        }
    }

    /// Add a set of url that site provide
    pub fn add_subs_url<S: AsRef<str>>(&mut self, subs_url: &[S]) {
        for sub_url in subs_url {
            self.add_sub_url(sub_url);
        }
    }

    /// Check if site contains url and is crawled
    pub fn contains_url<S: AsRef<str>>(&self, url: S) -> bool {
        let url = url.as_ref();
        let url = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => return false,
        };
        let host_str = url.host_str();
        if host_str == self.url.host_str() {
            return true;
        }
        for sub_url in &self.subs_url {
            if sub_url.host_str() == host_str {
                return true;
            }
        }
        false
    }

    /// Check if url has the same host as this site
    pub fn is_same_host<S: AsRef<str>>(&self, url: S) -> bool {
        let url = match Url::parse(url.as_ref()) {
            Ok(u) => u,
            Err(_) => return false,
        };
        self.url.host_str() == url.host_str()
    }

    /// Return the main url
    pub fn get_url(&self) -> &str {
        self.url.as_str()
    }

    /// Return all subs url
    pub fn get_subs_url(&self) -> Vec<&str> {
        self.subs_url.iter().map(|u| u.as_str()).collect()
    }

    /// Check if site contains trap
    pub fn is_trap(&self) -> bool {
        self.trap
    }

    /// Set the trap state of the site
    ///
    /// true if site contains trap else false
    pub fn set_trap_state(&mut self, contain_trap: bool) {
        self.trap = contain_trap;
    }

    /// Return if site is fully crawled
    pub fn is_fully_crawled(&self) -> bool {
        self.fully_crawled
    }

    /// Set the site as fully crawled
    pub fn fully_crawled(&mut self) {
        self.fully_crawled = true;
    }
}

#[cfg(test)]
mod unit_tests {
    use super::Site;

    const EXAMPLE: &'static str = "http://example.com/";

    fn vec_multiple_sub_url() -> Vec<&'static str> {
        vec!["http://example.com/hello", "http://example.com/yo", "http://example.com/world"]
    }

    #[test]
    fn new_site() {
        let site = Site::new(EXAMPLE).unwrap();
        assert_eq!(site.get_url(), EXAMPLE);
    }

    #[test]
    fn add_one_sub_url() {
        let mut site = Site::new(EXAMPLE).unwrap();
        site.add_sub_url(&format!("{}/sub", EXAMPLE));
        assert_eq!(site.get_subs_url()[0], format!("{}/sub", EXAMPLE));
    }

    #[test]
    fn add_multiple_sub_url() {
        let mut site = Site::new(EXAMPLE).unwrap();
        site.add_subs_url(&vec_multiple_sub_url());
        let subs_url = vec_multiple_sub_url();
        for (i, sub_url) in site.get_subs_url().iter().enumerate() {
            assert_eq!(sub_url, &subs_url[i]);
        }
    }

    #[test]
    fn add_wrong_sub_url() {
        let mut site = Site::new(EXAMPLE).unwrap();
        site.add_sub_url("http://google.com/sub_url");
        assert!(site.get_subs_url().len() == 0);
    }

    #[test]
    fn contains_url() {
        let mut site = Site::new(EXAMPLE).unwrap();
        site.add_sub_url("http://example.com/sub_url");
        assert!(site.contains_url(&EXAMPLE));
        assert!(site.contains_url("http://example.com/sub_url"));
        assert!(site.contains_url("https://example.com/"));
        assert!(!site.contains_url("http://dev.example.com/"));
    }

    #[test]
    fn same_host() {
        let site = Site::new(EXAMPLE).unwrap();
        assert!(site.is_same_host("http://example.com/sub_url"));
        assert!(!site.is_same_host("http://google.com"));
    }
}
