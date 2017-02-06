use hyper::Url;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub struct CrawlerConfig {
    filter: Arc<Fn(&Url, &Url) -> bool + Send + Sync>,
    store_path: Option<PathBuf>,
    sleep_ms: u64,
}

impl CrawlerConfig {
    pub fn new() -> CrawlerConfig {
        CrawlerConfig {
            filter: Arc::new(|_, _| true),
            store_path: None,
            sleep_ms: 1000,
        }
    }

    pub fn new_site_only() -> CrawlerConfig {
        CrawlerConfig::new().set_filter(|old, new| old.domain() == new.domain())
    }

    pub fn filter(&self, old_url: &Url, new_url: &Url) -> bool {
        (self.filter)(old_url, new_url)
    }

    pub fn sleep_ms(&self) -> u64 {
        self.sleep_ms
    }

    pub fn store(&self, url: &Url, body: &str) {
        if let Some(ref dir_path) = self.store_path {
            if !dir_path.exists() {
                match fs::create_dir_all(dir_path) {
                    Ok(_) => {}
                    Err(_) => return,
                }
            }
            let url_str = url.to_string().replace(':', "").replace('/', "_").replace('\\', "_");
            let path = dir_path.join(url_str);
            let mut file = match File::create(&path) {
                Ok(f) => f,
                Err(_) => return,
            };
            let _ = file.write_all(body.as_bytes());
        }
    }

    pub fn set_filter<F>(mut self, filter: F) -> CrawlerConfig
        where F: 'static + Send + Sync + Fn(&Url, &Url) -> bool
    {
        self.filter = Arc::new(filter);
        self
    }

    pub fn set_sleep_ms(mut self, sleep_ms: u64) -> CrawlerConfig {
        self.sleep_ms = sleep_ms;
        self
    }

    pub fn set_store_path<P: AsRef<Path>>(mut self, path: Option<P>) -> CrawlerConfig {
        match path {
            Some(path) => self.store_path = Some(path.as_ref().to_path_buf()),
            None => self.store_path = None,
        }
        self
    }
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        Self::new()
    }
}
