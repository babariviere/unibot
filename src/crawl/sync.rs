use error::*;
use hyper::client::IntoUrl;
use hyper::Url;
use indexer::Indexer;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Return a mutex guard of T
pub fn lock<T>(mutex: &Arc<Mutex<T>>) -> Result<MutexGuard<T>> {
    match mutex.lock() {
        Ok(t) => Ok(t),
        Err(e) => bail!(ErrorKind::PoisonError(e.to_string())),
    }
}

/// Add an url to the queue
pub fn add_to_queue<U: IntoUrl>(indexer: &Arc<Mutex<Indexer>>,
                                queue: &Arc<Mutex<VecDeque<Url>>>,
                                url: U)
                                -> Result<()> {
    let url = url.into_url()?;
    let mut queue = lock(queue)?;
    if !queue.contains(&url) && !lock(indexer)?.is_indexed(&url) {
        queue.push_back(url);
    }
    Ok(())
}

/// Get all item from queue
pub fn queue_items(queue: &Arc<Mutex<VecDeque<Url>>>) -> Result<VecDeque<Url>> {
    let queue = lock(queue)?;
    Ok(queue.clone())
}

/// Check if queue is empty
pub fn is_queue_empty(queue: &Arc<Mutex<VecDeque<Url>>>) -> bool {
    let queue = match lock(queue) {
        Ok(q) => q,
        Err(_) => return true,
    };
    queue.is_empty()
}

/// Pop an url from queue
pub fn pop_queue(queue: &Arc<Mutex<VecDeque<Url>>>) -> Result<Url> {
    let mut queue = lock(queue)?;
    let url = queue.pop_front();
    match url {
        Some(u) => Ok(u),
        None => bail!(ErrorKind::QueueEmpty),
    }
}

/// Get number of slave running
pub fn get_running(running: &Arc<AtomicUsize>) -> usize {
    running.load(Ordering::SeqCst)
}

/// Add one to running count
pub fn add_running(running: &Arc<AtomicUsize>) {
    running.fetch_add(1, Ordering::SeqCst);
}

/// Remove one to running count
pub fn remove_running(running: &Arc<AtomicUsize>) {
    if get_running(running) == 0 {
        return;
    }
    running.fetch_sub(1, Ordering::SeqCst);
}

/// Get stop value
pub fn get_stop(stop: &Arc<AtomicBool>) -> bool {
    stop.load(Ordering::Relaxed)
}

/// Set stop value
pub fn set_stop(stop_async: &Arc<AtomicBool>, stop: bool) {
    stop_async.store(stop, Ordering::Relaxed);
}
