use crate::inner::Inner;
use crate::read::ReadHandle;
use left_right::Absorb;
use std::fmt;
use std::hash::Hash;
use std::sync::{Arc, Mutex, MutexGuard};

/// Takes read_handle out of mutex, so we can read through WriteHandle without locking
#[derive(Clone)]
pub struct SharedWriteHandle<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    handle: Arc<Mutex<left_right::WriteHandle<Inner<K, V>, Operation<K, V>>>>,
    read_handle: ReadHandle<K, V>,
}

impl<K, V> From<WriteHandle<K, V>> for SharedWriteHandle<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn from(value: WriteHandle<K, V>) -> Self {
        Self {
            handle: Arc::new(Mutex::new(value.handle)),
            read_handle: value.read_handle,
        }
    }
}

impl<K, V> fmt::Debug for SharedWriteHandle<K, V>
where
    K: Eq + Hash + Clone + fmt::Debug,
    V: fmt::Debug + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedWriteHandle")
            .field("handle", &self.handle)
            .field("read_handle", &self.read_handle)
            .finish()
    }
}

impl<K, V> SharedWriteHandle<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn inner(
        &mut self,
    ) -> MutexGuard<'_, left_right::WriteHandle<Inner<K, V>, Operation<K, V>>> {
        self.handle.lock().unwrap()
    }

    pub fn publish(&mut self) {
        self.handle.lock().unwrap().publish();
    }

    pub(crate) fn add_op(&mut self, op: Operation<K, V>) -> &mut Self {
        self.handle.lock().unwrap().append(op);
        self
    }

    pub fn has_pending(&self) -> bool {
        self.handle.lock().unwrap().has_pending_operations()
    }

    pub fn insert(&mut self, k: K, v: V) {
        self.add_op(Operation::Add(k, v));
    }

    pub fn remove(&mut self, k: K) {
        self.add_op(Operation::Remove(k));
    }

    pub fn update(&mut self, k: K, v: V) {
        self.add_op(Operation::Replace(k, v));
    }

    pub fn purge(&mut self) {
        self.add_op(Operation::Purge);
    }

    pub fn modify<F>(&mut self, k: K, modifier: F) -> &mut Self
    where
        F: Fn(&mut V) -> () + Send + 'static,
    {
        self.add_op(Operation::Modify(k, Modifier(Box::new(modifier))))
    }
}

pub struct WriteHandle<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    handle: left_right::WriteHandle<Inner<K, V>, Operation<K, V>>,
    read_handle: ReadHandle<K, V>,
}

impl<K, V> fmt::Debug for WriteHandle<K, V>
where
    K: Eq + Hash + Clone + fmt::Debug,
    V: fmt::Debug + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WriteHandle")
            .field("handle", &self.handle)
            .field("read_handle", &self.read_handle)
            .finish()
    }
}

impl<K, V> WriteHandle<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub(crate) fn new(handle: left_right::WriteHandle<Inner<K, V>, Operation<K, V>>) -> Self {
        let read_handle = ReadHandle::new(left_right::ReadHandle::clone(&*handle));
        Self {
            handle,
            read_handle,
        }
    }

    pub fn publish(&mut self) -> &mut Self {
        self.handle.publish();
        self
    }

    pub fn has_pending(&self) -> bool {
        self.handle.has_pending_operations()
    }

    pub(crate) fn add_op(&mut self, op: Operation<K, V>) -> &mut Self {
        self.handle.append(op);
        self
    }

    pub fn insert(&mut self, k: K, v: V) -> &mut Self {
        self.add_op(Operation::Add(k, v))
    }

    pub fn remove(&mut self, k: K) -> &mut Self {
        self.add_op(Operation::Remove(k))
    }

    pub fn update(&mut self, k: K, v: V) -> &mut Self {
        self.add_op(Operation::Replace(k, v))
    }

    pub fn purge(&mut self) -> &mut Self {
        self.add_op(Operation::Purge)
    }

    pub fn modify<F>(&mut self, k: K, modifier: F) -> &mut Self
    where
        F: Fn(&mut V) -> () + Send + 'static,
    {
        self.add_op(Operation::Modify(k, Modifier(Box::new(modifier))))
    }
}

pub(crate) enum Operation<K, V> {
    Add(K, V),
    Replace(K, V),
    Remove(K),
    Modify(K, Modifier<V>),
    Purge,
    MarkReady,
}

impl<K, V> fmt::Debug for Operation<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Operation::Replace(ref a, ref b) => f.debug_tuple("Replace").field(a).field(b).finish(),
            Operation::Add(ref a, ref b) => f.debug_tuple("Add").field(a).field(b).finish(),
            Operation::Remove(ref a) => f.debug_tuple("Remove").field(a).finish(),
            Operation::Modify(ref a, ref b) => f.debug_tuple("Modify").field(a).field(b).finish(),
            Operation::Purge => f.debug_tuple("Purge").finish(),
            Operation::MarkReady => f.debug_tuple("MarkReady").finish(),
        }
    }
}

impl<K, V> Absorb<Operation<K, V>> for Inner<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn absorb_first(&mut self, operation: &mut Operation<K, V>, _other: &Self) {
        match operation {
            Operation::Add(ref key, ref value) => {
                self.data.insert(key.clone(), value.clone());
            }
            Operation::Replace(ref key, ref value) => {
                if let Some(v) = self.data.get_mut(key) {
                    *v = value.clone();
                }
            }
            Operation::Remove(ref key) => {
                self.data.remove(key);
            }
            Operation::Modify(ref key, ref modifier) => {
                if let Some(v) = self.data.get_mut(key) {
                    modifier.modify(v);
                }
            }
            Operation::Purge => {
                self.data.clear();
            }
            Operation::MarkReady => {
                self.ready = true;
            }
        }
    }

    fn absorb_second(&mut self, operation: Operation<K, V>, _other: &Self) {
        match operation {
            Operation::Add(key, value) => {
                self.data.insert(key, value);
            }
            Operation::Replace(key, value) => {
                if let Some(v) = self.data.get_mut(&key) {
                    *v = value.clone();
                }
            }
            Operation::Remove(key) => {
                self.data.remove(&key);
            }
            Operation::Modify(key, modifier) => {
                if let Some(v) = self.data.get_mut(&key) {
                    modifier.modify(v);
                }
            }
            Operation::Purge => {
                self.data.clear();
            }
            Operation::MarkReady => {
                self.ready = true;
            }
        }
    }

    fn sync_with(&mut self, first: &Self) {
        self.data = first.data.clone();
        self.ready = first.ready;
    }
}

pub(super) struct Modifier<V: ?Sized>(Box<dyn Fn(&mut V) -> () + Send>);

impl<V: ?Sized> Modifier<V> {
    fn modify(&self, value: &mut V) -> () {
        (*self.0)(value)
    }
}

impl<V: ?Sized> fmt::Debug for Modifier<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Modifier")
            .field(&format_args!("{:p}", &*self.0 as *const _))
            .finish()
    }
}
