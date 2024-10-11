use arc_swap::access::Access;
use arc_swap::ArcSwap;
use std::sync::{Arc, Mutex, RwLock};

pub mod left_right_cell;

pub trait ValueManipulator: Clone + Send {
    fn get_value(&self) -> u64;
    fn set_value(&self, val: u64);
}

fn main() {}

// Arc Swap
#[derive(Default, Clone)]
pub struct ArcSwapVersion {
    inner: Arc<ArcSwap<u64>>,
}

impl ValueManipulator for ArcSwapVersion {
    fn get_value(&self) -> u64 {
        *Access::<u64>::load(&self.inner)
    }

    fn set_value(&self, val: u64) {
        self.inner.store(Arc::new(val));
    }
}

// Left Right
#[derive(Clone)]
pub struct LeftRightVersion {
    pub inner_w: Arc<Mutex<left_right_cell::WriteHandle<u64>>>,
    pub inner_r: left_right_cell::ReadHandle<u64>,
}

impl ValueManipulator for LeftRightVersion {
    fn get_value(&self) -> u64 {
        *self.inner_r.get().unwrap()
    }

    fn set_value(&self, val: u64) {
        let mut lock = self.inner_w.lock().unwrap();
        lock.set(val);
        lock.publish();
    }
}

// impl ValueManipulator for left_right_cell::ReadHandle<u64> {
//     fn get_value(&self) -> u64 {
//         *self.get().unwrap()
//     }
//
//     fn set_value(&self, val: u64) {
//         todo!()
//     }
// }

impl Default for LeftRightVersion {
    fn default() -> Self {
        let (inner_w, inner_r) = left_right_cell::new_default::<u64>();
        Self {
            inner_w: Arc::new(Mutex::new(inner_w)),
            inner_r,
        }
    }
}

// pub fn left_right_version() -> (
//     Arc<Mutex<left_right_cell::WriteHandle<u64>>>,
//     left_right_cell::ReadHandle<u64>,
// ) {
//     let (inner_w, inner_r) = left_right_cell::new_default::<u64>();
//     (Arc::new(Mutex::new(inner_w)), inner_r)
// }

// Mutex
#[derive(Default, Clone)]
pub struct MutexVersion {
    inner: Arc<Mutex<u64>>,
}

impl ValueManipulator for MutexVersion {
    fn get_value(&self) -> u64 {
        *self.inner.lock().unwrap()
    }

    fn set_value(&self, val: u64) {
        *self.inner.lock().unwrap() = val
    }
}

// RW Lock
#[derive(Default, Clone)]
pub struct RwLockVersion {
    inner: Arc<RwLock<u64>>,
}

impl ValueManipulator for RwLockVersion {
    fn get_value(&self) -> u64 {
        *self.inner.read().unwrap()
    }

    fn set_value(&self, val: u64) {
        *self.inner.write().unwrap() = val
    }
}
