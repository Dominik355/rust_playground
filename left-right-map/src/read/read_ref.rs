use crate::inner::Inner;
use left_right::ReadGuard;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::hash::{BuildHasher, Hash};

pub struct MapReadRef<'rh, K, V>
where
    K: Eq + Hash,
{
    pub(super) guard: ReadGuard<'rh, Inner<K, V>>,
}

impl<'rh, K, V> MapReadRef<'rh, K, V>
where
    K: Hash + Eq,
{
    pub fn iter(&self) -> ReadGuardIter<'_, K, V> {
        ReadGuardIter {
            iter: self.guard.data.iter(),
        }
    }

    pub fn len(&self) -> usize {
        self.guard.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.guard.data.is_empty()
    }

    pub fn get<'a, Q: ?Sized>(&'a self, key: &'_ Q) -> Option<&'a V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.guard.data.get(key)
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.guard.data.contains_key(key)
    }

    pub fn values(&self) -> ValuesIter<'_, K, V> {
        ValuesIter {
            iter: self.guard.data.iter(),
        }
    }
}

impl<'rg, 'rh, K, V> IntoIterator for &'rg MapReadRef<'rh, K, V>
where
    K: Eq + Hash,
{
    type Item = (&'rg K, &'rg V);
    type IntoIter = ReadGuardIter<'rg, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct ReadGuardIter<'rg, K, V>
where
    K: Eq + Hash,
{
    iter: <&'rg HashMap<K, V> as IntoIterator>::IntoIter,
}

impl<'rg, K, V> fmt::Debug for ReadGuardIter<'rg, K, V>
where
    K: Eq + Hash + fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ReadGuardIter").field(&self.iter).finish()
    }
}

impl<'rg, K, V> Iterator for ReadGuardIter<'rg, K, V>
where
    K: Eq + Hash,
{
    type Item = (&'rg K, &'rg V);
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct ValuesIter<'rg, K, V>
where
    K: Eq + Hash,
{
    iter: <&'rg HashMap<K, V> as IntoIterator>::IntoIter,
}

impl<'rg, K, V> fmt::Debug for ValuesIter<'rg, K, V>
where
    K: Eq + Hash + fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ValuesIter").field(&self.iter).finish()
    }
}

impl<'rg, K, V> Iterator for ValuesIter<'rg, K, V>
where
    K: Eq + Hash,
{
    type Item = &'rg V;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }
}
