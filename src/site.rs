
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

impl Site {
    /// Create a new instance of site
    pub fn new(url: String) -> Site {
        Site {
            url: url,
            subs_url: Vec::new(),
            trap: false,
            fully_crawled: false,
        }
    }

    /// Add an url that site provide
    pub fn add_sub_url(&mut self, sub_url: String) {
        if sub_url.starts_with(&self.url) {
            self.subs_url.push(sub_url);
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
