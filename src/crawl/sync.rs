use common::href_to_url;
use error::*;
use hyper::client::{Client, IntoUrl};
use hyper::net::HttpsConnector;
use hyper::Url;
use hyper_native_tls::NativeTlsClient;
use indexer::Indexer;
use scrap::scrap_attr;
use select::document::Document;
use std::collections::VecDeque;
use std::io::Read;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use super::config::CrawlerConfig;


/// Return a mutable reference to queue
pub fn lock_queue(queue: &Arc<Mutex<VecDeque<Url>>>) -> Result<MutexGuard<VecDeque<Url>>> {
    match queue.lock() {
        Ok(q) => Ok(q),
        Err(e) => bail!(ErrorKind::PoisonError(e.to_string())),
    }
}

/// Return a mutable reference to indexer
pub fn lock_indexer(indexer: &Arc<Mutex<Indexer>>) -> Result<MutexGuard<Indexer>> {
    match indexer.lock() {
        Ok(i) => Ok(i),
        Err(e) => bail!(ErrorKind::PoisonError(e.to_string())),
    }
}

/// Stop loop if there is a running one
pub fn lock_stop(stop: &Arc<Mutex<bool>>) -> Result<MutexGuard<bool>> {
    match stop.lock() {
        Ok(s) => Ok(s),
        Err(e) => bail!(ErrorKind::PoisonError(e.to_string())),
    }
}

/// Add an url to the queue
pub fn add_to_queue<U: IntoUrl>(indexer: &Arc<Mutex<Indexer>>,
                                queue: &Arc<Mutex<VecDeque<Url>>>,
                                url: U)
                                -> Result<()> {
    let url = url.into_url()?;
    let mut queue = lock_queue(queue)?;
    if !queue.contains(&url) && !lock_indexer(&indexer)?.is_indexed(&url) {
        queue.push_back(url);
    }
    Ok(())
}

/// Get all item from queue
pub fn queue_items(queue: &Arc<Mutex<VecDeque<Url>>>) -> Result<VecDeque<Url>> {
    let queue = lock_queue(queue)?;
    Ok(queue.clone())
}

/// Check if queue is empty
pub fn is_queue_empty(queue: &Arc<Mutex<VecDeque<Url>>>) -> bool {
    let queue = match lock_queue(queue) {
        Ok(q) => q,
        Err(_) => return true,
    };
    queue.is_empty()
}

/// Pop an url from queue
pub fn pop_queue(queue: &Arc<Mutex<VecDeque<Url>>>) -> Result<Url> {
    let mut queue = lock_queue(queue)?;
    let url = queue.pop_front();
    match url {
        Some(u) => Ok(u),
        None => bail!(ErrorKind::QueueEmpty),
    }
}

/// Free queue
pub fn free_queue(queue: &Arc<Mutex<VecDeque<Url>>>) -> Result<()> {
    let mut queue = lock_queue(queue)?;
    queue.clear();
    Ok(())
}

/// Get stop value
pub fn get_stop(stop: &Arc<Mutex<bool>>) -> bool {
    match lock_stop(stop) {
        Ok(b) => *b,
        Err(_) => true,
    }
}

/// Set stop value
pub fn set_stop(stop_async: &Arc<Mutex<bool>>, stop: bool) {
    let mut lock = match lock_stop(stop_async) {
        Ok(l) => l,
        Err(_) => return,
    };
    *lock = stop;
}
