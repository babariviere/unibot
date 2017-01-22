
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
    url: String,
    subs_url: Vec<String>,
    trap: bool,
    fully_crawled: bool,
}

// TODO replace string url with a url struct

impl Site {
    /// Create a new instance of site
    pub fn new<S: AsRef<str>>(url: S) -> Site {
        Site {
            url: url.as_ref().to_owned(),
            subs_url: Vec::new(),
            trap: false,
            fully_crawled: false,
        }
    }

    /// Add an url that site provide
    pub fn add_sub_url<S: AsRef<str>>(&mut self, sub_url: S) {
        let sub_url = sub_url.as_ref().to_owned();
        if sub_url.starts_with(&self.url) {
            self.subs_url.push(sub_url);
        }
    }

    /// Add a set of url that site provide
    pub fn add_subs_url<S: AsRef<str>>(&mut self, subs_url: &Vec<S>) {
        for sub_url in subs_url {
            self.add_sub_url(sub_url);
        }
    }

    /// Check if site contains url and is crawled
    pub fn contains_url<S: AsRef<str>>(&self, url: S) -> bool {
        let url = url.as_ref();
        if url == self.url {
            return true;
        }
        for sub_url in &self.subs_url {
            if sub_url == url {
                return true;
            }
        }
        false
    }

    /// Return the main url
    pub fn get_url(&self) -> &str {
        &self.url
    }

    /// Return all subs url
    pub fn get_subs_url(&self) -> &Vec<String> {
        &self.subs_url
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

    const EXAMPLE: &'static str = "http://example.com";

    fn vec_multiple_sub_url() -> Vec<&'static str> {
        vec!["http://example.com/hello", "http://example.com/yo", "http://example.com/world"]
    }

    #[test]
    fn new_site() {
        let site = Site::new(EXAMPLE);
        assert_eq!(site.get_url(), EXAMPLE);
    }

    #[test]
    fn add_one_sub_url() {
        let mut site = Site::new(EXAMPLE);
        site.add_sub_url(&format!("{}/sub", EXAMPLE));
        assert_eq!(site.get_subs_url()[0], format!("{}/sub", EXAMPLE));
    }

    #[test]
    fn add_multiple_sub_url() {
        let mut site = Site::new(EXAMPLE);
        site.add_subs_url(&vec_multiple_sub_url());
        let subs_url = vec_multiple_sub_url();
        for (i, sub_url) in site.get_subs_url().iter().enumerate() {
            assert_eq!(sub_url, subs_url[i]);
        }
    }

    #[test]
    fn add_wrong_sub_url() {
        let mut site = Site::new(EXAMPLE);
        site.add_sub_url("http://google.com/sub_url");
        assert!(site.get_subs_url().len() == 0);
    }
}
