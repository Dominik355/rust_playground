use crate::inner::Inner;
use crate::read::ReadHandle;
use crate::write::WriteHandle;
use std::fmt;
use std::hash::Hash;

mod inner;
pub mod read;
pub mod write;

pub fn new<K, V>() -> (WriteHandle<K, V>, ReadHandle<K, V>)
where
    K: Hash + Eq + Clone + fmt::Debug,
    V: Clone,
{
    let inner = Inner::default();
    let (mut w, r) = left_right::new_from_empty(inner);
    w.append(write::Operation::MarkReady);
    (WriteHandle::new(w), ReadHandle::new(r))
}
