use arc_swap::access::Access;
use arc_swap::ArcSwap;
use std::sync::{Arc, Mutex, RwLock};

pub mod left_right_cell;

pub trait ValueGetter: Clone + Send {
    fn get_value(&self) -> u64;
}

fn main() {}

#[derive(Default, Clone)]
pub struct InnerValue {
    value: u64,
}

impl InnerValue {
    fn get_value(&self) -> u64 {
        self.value
    }
}

// Arc Swap
#[derive(Default, Clone)]
pub struct ArcSwapVersion {
    inner: Arc<ArcSwap<InnerValue>>,
}

impl ValueGetter for ArcSwapVersion {
    fn get_value(&self) -> u64 {
        let inner = Access::<InnerValue>::load(&self.inner);
        inner.get_value()
    }
}

// Left Right
#[derive(Clone)]
pub struct LeftRightVersion {
    pub inner_w: Arc<Mutex<left_right_cell::WriteHandle<InnerValue>>>,
    pub inner_r: left_right_cell::ReadHandle<InnerValue>,
}

impl ValueGetter for left_right_cell::ReadHandle<InnerValue> {
    fn get_value(&self) -> u64 {
        self.get().unwrap().get_value()
    }
}

pub fn left_right_version() -> (
    Arc<Mutex<left_right_cell::WriteHandle<InnerValue>>>,
    left_right_cell::ReadHandle<InnerValue>,
) {
    let (inner_w, inner_r) = left_right_cell::new_default::<InnerValue>();
    (Arc::new(Mutex::new(inner_w)), inner_r)
}

// Mutex
#[derive(Default, Clone)]
pub struct MutexVersion {
    inner: Arc<Mutex<InnerValue>>,
}

impl ValueGetter for MutexVersion {
    fn get_value(&self) -> u64 {
        self.inner.lock().unwrap().get_value()
    }
}

// RW Lock
#[derive(Default, Clone)]
pub struct RwLockVersion {
    inner: Arc<RwLock<InnerValue>>,
}

impl ValueGetter for RwLockVersion {
    fn get_value(&self) -> u64 {
        self.inner.read().unwrap().get_value()
    }
}
