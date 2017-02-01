use hyper::Url;
use hyper::client::IntoUrl;

pub fn beautify_url<S: AsRef<str>>(url: S) -> String {
    let mut new_url = String::new();
    let url = url.as_ref();
    let mut last_char = ' ';
    for c in url.chars() {
        if c == '/' && last_char == '/' {
            continue;
        }
        new_url.push(c);
        last_char = c;
    }
    new_url
}

/// Convert an href to an url
pub fn href_to_url(url: &Url, href: &str) -> Option<Url> {
    let url = if href.starts_with("//") {
        let scheme = url.scheme();
        match format!("{}:{}", scheme, href).into_url() {
            Ok(u) => u,
            _ => return None,
        }
    } else if href.starts_with("http") {
        match href.into_url() {
            Ok(u) => u,
            _ => return None,
        }
    } else if href.starts_with('/') {
        let mut url = url.clone();
        url.set_path(href);
        url
    } else if href.starts_with("javascript") {
        return None;
    } else {
        let path = url.path();
        if path.ends_with(href) {
            return None;
        }
        let mut url = url.clone();
        let href = beautify_url(format!("{}/{}", url, href));
        url.set_path(&href);
        url
    };
    Some(url)
}
