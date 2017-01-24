
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
