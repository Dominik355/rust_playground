use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(feature = "parking_lot")]
use sync_parking_lot::{Condvar, Mutex};
#[cfg(not(feature = "parking_lot"))]
use sync_std::{Condvar, Mutex};

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
        }
    }
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    last_seen_version: u64,
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
            last_seen_version: 0,
        }
    }
}

struct Shared<T> {
    lock: Mutex<SharedValue<T>>,
    updated: Condvar,
}

struct SharedValue<T> {
    value: T,
    version: u64,
}

pub fn channel<T: Clone>(value: T) -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Shared {
        lock: Mutex::new(SharedValue { value, version: 0 }),
        updated: Condvar::new(),
    });
    (
        Sender {
            shared: shared.clone(),
        },
        Receiver {
            shared,
            last_seen_version: 0,
        },
    )
}

impl<T: Clone> Receiver<T> {
    pub fn new_sender(&self) -> Sender<T> {
        Sender {
            shared: self.shared.clone(),
        }
    }
}

impl<T: Clone> Receiver<T> {
    pub fn receive(&mut self) -> T {
        let lock = self.shared.lock.lock();
        self.last_seen_version = lock.version;
        lock.value.clone()
    }

    pub fn get_if_new(&mut self) -> Option<T> {
        let lock = self.shared.lock.lock();
        if self.last_seen_version == lock.version {
            return None;
        }
        self.last_seen_version = lock.version;
        Some(lock.value.clone())
    }

    pub fn receive_blocking(&mut self) -> T {
        let mut lock = self.shared.lock.lock();

        while lock.version == self.last_seen_version {
            lock = self.shared.updated.wait(lock);
        }

        self.last_seen_version = lock.version;
        lock.value.clone()
    }

    pub fn wait_timeout(&mut self, duration: Duration) -> Option<T> {
        let mut lock = self.shared.lock.lock();
        let deadline = Instant::now() + duration;

        while lock.version == self.last_seen_version {
            let timeout = deadline.saturating_duration_since(Instant::now());

            lock = self.shared.updated.wait_timeout(lock, timeout)?;

            // Note: checking after `on_update.wait_timeout` to call it at least once,
            // even when `duration` was zero.
            if timeout.is_zero() && lock.version == self.last_seen_version {
                return None;
            }
        }

        self.last_seen_version = lock.version;
        Some(lock.value.clone())
    }
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) {
        {
            let mut lock = self.shared.lock.lock();
            lock.value = value;
            lock.version = lock.version.wrapping_add(1);
        }
        self.shared.updated.notify_all();
    }

    pub fn subscribe(&self) -> Receiver<T> {
        let last_seen_version = {
            let lock = self.shared.lock.lock();
            lock.version
        };

        Receiver {
            shared: self.shared.clone(),
            last_seen_version,
        }
    }

    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        {
            let mut lock = self.shared.lock.lock();
            f(&mut lock.value);
            lock.version = lock.version.wrapping_add(1);
        }
        self.shared.updated.notify_all();
    }
}

#[cfg(feature = "parking_lot")]
mod sync_parking_lot {
    use parking_lot::MutexGuard;
    use std::time::Duration;

    pub struct Mutex<T> {
        inner: parking_lot::Mutex<T>,
    }

    impl<T> Mutex<T> {
        pub fn new(value: T) -> Self {
            Self {
                inner: parking_lot::Mutex::new(value),
            }
        }

        pub fn lock(&self) -> MutexGuard<'_, T> {
            self.inner.lock()
        }
    }

    pub struct Condvar {
        inner: parking_lot::Condvar,
    }
    impl Condvar {
        pub fn new() -> Self {
            Self {
                inner: parking_lot::Condvar::new(),
            }
        }

        pub fn wait<'a, T>(&self, mut guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
            self.inner.wait(&mut guard);
            guard
        }

        pub fn wait_timeout<'a, T>(
            &self,
            mut guard: MutexGuard<'a, T>,
            duration: Duration,
        ) -> Option<MutexGuard<'a, T>> {
            if self.inner.wait_for(&mut guard, duration).timed_out() {
                None
            } else {
                Some(guard)
            }
        }

        pub fn notify_all(&self) {
            self.inner.notify_all();
        }
    }
}

#[cfg(not(feature = "parking_lot"))]
mod sync_std {
    use std::{sync::MutexGuard, time::Duration};

    pub struct Mutex<T> {
        inner: std::sync::Mutex<T>,
    }

    impl<T> Mutex<T> {
        pub fn new(value: T) -> Self {
            Self {
                inner: std::sync::Mutex::new(value),
            }
        }

        pub fn lock(&self) -> MutexGuard<'_, T> {
            self.inner.lock().unwrap_or_else(|err| err.into_inner())
        }
    }

    pub struct Condvar {
        inner: std::sync::Condvar,
    }
    impl Condvar {
        pub fn new() -> Self {
            Self {
                inner: std::sync::Condvar::new(),
            }
        }

        pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
            self.inner
                .wait(guard)
                .unwrap_or_else(|err| err.into_inner())
        }

        pub fn wait_timeout<'a, T>(
            &self,
            guard: MutexGuard<'a, T>,
            duration: Duration,
        ) -> Option<MutexGuard<'a, T>> {
            match self.inner.wait_timeout(guard, duration) {
                Ok((guard, _)) => Some(guard),
                Err(_) => None,
            }
        }
        pub fn notify_all(&self) {
            self.inner.notify_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let (tx, mut rx) = channel(1);
        let mut rx_2 = rx.clone();
        assert_eq!(rx.receive(), 1);
        assert_eq!(tx.subscribe().receive(), 1);
        assert_eq!(rx.clone().receive(), 1);

        tx.send(2);
        assert_eq!(rx.receive(), 2);
        assert_eq!(rx.receive(), 2);
        assert_eq!(rx.get_if_new(), None);
        assert_eq!(rx_2.get_if_new(), Some(2));
        assert_eq!(tx.subscribe().receive(), 2);
        assert_eq!(rx.clone().receive(), 2);
    }
}
