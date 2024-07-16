use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

fn create_reader(name: i32, lock: Arc<RwLock<u32>>) -> JoinHandle<()> {
    thread::spawn(move || {
        println!("reader {}: acquiring read lock...", name);
        let guard = lock.read().unwrap();

        println!("reader {}: sleeping...", name);
        thread::sleep(Duration::from_secs(3));

        println!("reader {}: end: {}", name, *guard);
    })
}

fn create_writer(name: i32, lock: Arc<RwLock<u32>>) -> JoinHandle<()> {
    thread::spawn(move || {
        println!("writer {}: acquiring write lock...", name);
        let guard = lock.write().unwrap();

        println!("writer {}: sleeping...", name);
        thread::sleep(Duration::from_secs(3));

        println!("writer {}: end: {}", name, *guard);
    })
}

/// default std RwLock behaves differently on different platforms :
///
///
/// Windows: Readers and writers are fairly queued
///
/// Linux: Readers will starve the writers
///
/// macOS: Readers and writers are fairly queued
///
/// Expected results from this test: the "sleeping..." message for readers and writers
/// should be interleaved, with reader 1 & 2 sleeping first, followed by writer 1.
///
/// This won't happen especially with linux, where with read-heavy workloads,
/// the writer tasks will be starved.
fn main() {
    let mut threads = Vec::new();
    let shared_lock = Arc::new(RwLock::new(0));

    // Start two reader threads
    let lock = Arc::clone(&shared_lock);
    threads.push(create_reader(1, lock));

    let lock = Arc::clone(&shared_lock);
    threads.push(create_reader(2, lock));

    // Wait for threads to sleep
    thread::sleep(Duration::from_millis(100));

    // Start a writer thread
    let lock = Arc::clone(&shared_lock);
    threads.push(create_writer(1, lock));

    // Wait for threads to sleep
    thread::sleep(Duration::from_millis(100));

    // Start another two reader threads
    let lock = Arc::clone(&shared_lock);
    threads.push(create_reader(3, lock));

    let lock = Arc::clone(&shared_lock);
    threads.push(create_reader(4, lock));

    // Wait for threads to sleep
    thread::sleep(Duration::from_millis(100));

    // Start another writer thread
    let lock = Arc::clone(&shared_lock);
    threads.push(create_writer(2, lock));

    // Wait for all threads to exit
    for t in threads {
        t.join().unwrap();
    }
}
