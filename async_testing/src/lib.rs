mod test;

use std::sync::atomic::Ordering::SeqCst;

#[cfg(not(test))]
use std::sync::atomic::AtomicU32;
#[cfg(test)]
use test::managed_thread::AtomicU32;

// https://matklad.github.io/2024/07/05/properly-testing-concurrent-data-structures.html
// just a basic atomic counter
#[derive(Default)]
pub struct Counter {
    value: AtomicU32,
}

impl Counter {
    pub fn increment(&self) {
        // self.value.fetch_add(1, SeqCst);
        let value = self.value.load(SeqCst);
        self.value.store(value + 1, SeqCst);
    }

    pub fn get(&self) -> u32 {
        self.value.load(SeqCst)
    }
}
