pub mod read_ref;

use crate::inner::Inner;
use crate::read::read_ref::MapReadRef;
use left_right::ReadGuard;
use std::borrow::Borrow;
use std::fmt::Formatter;
use std::hash::Hash;

#[derive(Clone)]
pub struct ReadHandle<K, V>
where
    K: Eq + Hash,
{
    pub(crate) handle: left_right::ReadHandle<Inner<K, V>>,
}

impl<K: Eq + Hash, V> std::fmt::Debug for ReadHandle<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadHandle")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<K, V> ReadHandle<K, V>
where
    K: Eq + Hash,
{
    pub(crate) fn new(handle: left_right::ReadHandle<Inner<K, V>>) -> Self {
        Self { handle }
    }
}

impl<K, V> ReadHandle<K, V>
where
    K: Eq + Hash,
{
    pub fn enter(&self) -> Option<MapReadRef<'_, K, V>> {
        let guard = self.handle.enter()?;
        if !guard.ready {
            return None;
        }
        Some(MapReadRef { guard })
    }

    fn get_raw<Q: ?Sized>(&self, key: &Q) -> Option<ReadGuard<'_, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let inner = self.handle.enter()?;
        if !inner.ready {
            return None;
        }

        ReadGuard::try_map(inner, |inner| inner.data.get(key))
    }

    pub fn get<'rh, Q: ?Sized>(&'rh self, key: &'_ Q) -> Option<ReadGuard<'rh, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.get_raw(key.borrow())
    }

    /// Returns the number of non-empty keys present in the map.
    pub fn len(&self) -> usize {
        self.enter().map_or(0, |x| x.len())
    }

    /// Returns true if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.enter().map_or(true, |x| x.is_empty())
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.enter().map_or(false, |x| x.contains_key(key))
    }
}
