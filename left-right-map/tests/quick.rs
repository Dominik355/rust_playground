#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use std::collections::HashSet;
use std::hash::Hash;

fn set<'a, T: 'a, I>(iter: I) -> HashSet<T>
where
    I: IntoIterator<Item = &'a T>,
    T: Copy + Hash + Eq,
{
    iter.into_iter().cloned().collect()
}

#[quickcheck]
fn contains(insert: Vec<u32>) -> bool {
    let (mut w, r) = left_right_map::new();
    for &key in &insert {
        w.insert(key, ());
    }
    w.publish();

    insert.iter().all(|&key| r.get(&key).is_some())
}

#[quickcheck]
fn contains_not(insert: Vec<u8>, not: Vec<u8>) -> bool {
    let (mut w, r) = left_right_map::new();
    for &key in &insert {
        w.insert(key, ());
    }
    w.publish();

    let nots = &set(&not) - &set(&insert);
    nots.iter().all(|&key| r.get(&key).is_none())
}

#[quickcheck]
fn insert_empty(insert: Vec<u8>, remove: Vec<u8>) -> bool {
    let (mut w, r) = left_right_map::new();
    for &key in &insert {
        w.insert(key, ());
    }
    w.publish();
    for &key in &remove {
        w.remove(key);
    }
    w.publish();
    let elements = &set(&insert) - &set(&remove);
    r.len() == elements.len() && elements.iter().all(|k| r.get(k).is_some())
}
