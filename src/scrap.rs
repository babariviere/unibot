use select::document::Document;
use select::predicate::Attr;

pub fn scrap_attr(doc: &Document, attr: &str) -> Vec<String> {
    let mut attrs = Vec::new();
    let nodes = doc.find(Attr(attr, ()));
    for node in nodes.iter() {
        let attr = match node.attr("href") {
            Some(a) => a.to_string(),
            None => continue,
        };
        attrs.push(attr);
    }
    attrs
}
