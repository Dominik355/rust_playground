use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub(crate) struct Inner<K, V>
where
    K: Eq + Hash,
{
    pub(crate) data: HashMap<K, V>,
    pub(crate) ready: bool,
}

impl<K, V> Default for Inner<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Inner {
            data: HashMap::new(),
            ready: false,
        }
    }
}

impl<K, V> Inner<K, V>
where
    K: Eq + Hash,
{
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Inner {
            data: HashMap::with_capacity(capacity),
            ready: false,
        }
    }
}
