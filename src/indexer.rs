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
}
