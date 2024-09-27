use std::ops::Deref;
use std::sync::{Arc, Mutex};

macro_rules! assert_match {
    ($x:expr, $p:pat) => {
        if let $p = $x {
        } else {
            panic!(concat!(stringify!($x), " did not match ", stringify!($p)));
        }
    };
}

#[test]
fn it_works() {
    let x = ('x', 42);

    let (mut w, r) = left_right_map::new::<char, (char, usize)>();

    // the map is uninitialized, so all lookups should return None
    assert_match!(r.get(&x.0), None);
    assert_match!(r.enter(), None);
    assert_eq!(r.is_empty(), true);
    assert_eq!(r.contains_key(&x.0), false);

    w.insert(x.0, x);

    // it is empty even after an add (we haven't refreshed yet)
    assert_match!(r.get(&x.0), None);

    w.publish();

    // but after the swap, the record is there!
    assert_eq!(r.len(), 1);
    assert_eq!(*r.get(&x.0).unwrap(), x);
    assert_eq!(r.contains_key(&x.0), true);
    assert_match!(r.enter().unwrap().get(&x.0), Some(x));

    // non-existing records return None
    assert_match!(r.get(&'y'), None);
    assert_eq!(r.contains_key(&'y'), false);

    // if we purge, the readers still see the values
    w.purge();
    assert_eq!(r.len(), 1);
    assert_eq!(*r.get(&x.0).unwrap(), x);

    // but once we refresh, things will be empty
    w.publish();
    assert_match!(r.get(&x.0), None);
    assert_eq!(r.is_empty(), true);
}

#[test]
fn mapref() {
    let x = ('x', 42);

    let (mut w, r) = left_right_map::new();

    // get a read ref to the map
    // scope to ensure it gets dropped and doesn't stall refresh
    {
        assert!(r.enter().is_none());
    }

    w.publish();

    {
        let map = r.enter().unwrap();
        // after the first refresh, it is empty, but ready
        assert!(map.is_empty());
        assert!(!map.contains_key(&x.0));
        assert!(map.get(&x.0).is_none());
    }

    w.insert(x.0, x);

    {
        let map = r.enter().unwrap();
        // it is empty even after an add (we haven't refresh yet)
        assert!(map.is_empty());
        assert!(!map.contains_key(&x.0));
        assert!(map.get(&x.0).is_none());
    }

    w.publish();

    {
        let map = r.enter().unwrap();

        // but after the swap, the record is there!
        assert!(!map.is_empty());
        assert!(map.contains_key(&x.0));
        assert_eq!(map.get(&x.0).unwrap(), &x);
        assert_eq!(map.len(), 1);

        // non-existing records return None
        assert!(map.get(&'y').is_none());

        // if we purge, the readers still see the values
        w.purge();

        assert_eq!(map.get(&x.0).unwrap(), &x);
        assert_eq!(map.len(), 1);
    }

    // but once we refresh, things will be empty
    w.publish();

    {
        let map = r.enter().unwrap();
        assert!(map.is_empty());
        assert!(!map.contains_key(&x.0));
        assert!(map.get(&x.0).is_none());
    }

    drop(w);
    {
        let map = r.enter();
        assert!(map.is_none(), "the map should have been destroyed");
    }
}

#[test]
fn read_after_drop() {
    let x = ('x', 42);

    let (mut w, r) = left_right_map::new();
    w.insert(x.0, x);
    w.publish();
    assert_match!(r.get(&x.0), Some(_));

    // once we drop the writer, the readers should see empty maps
    drop(w);
    assert_match!(r.get(&x.0), None);
}

#[test]
fn replace() {
    let (mut w, r) = left_right_map::new();
    w.insert(1, "a");
    w.insert(2, "c");
    w.update(1, "x");
    w.publish();

    assert_eq!(r.get(&1).unwrap().deref(), &"x");
    assert_eq!(r.get(&2).unwrap().deref(), &"c");
}

#[test]
fn replace_not_existing() {
    let (mut w, r) = left_right_map::new();
    w.update(1, "x");
    w.publish();

    assert_match!(r.get(&1), None);
}

#[test]
fn replace_post_refresh() {
    let (mut w, r) = left_right_map::new();
    w.insert(1, "a");
    w.insert(2, "c");
    w.publish();
    w.update(1, "x");
    w.publish();

    assert_eq!(r.get(&1).unwrap().deref(), &"x");
    assert_eq!(r.get(&2).unwrap().deref(), &"c");
}

#[test]
fn values() {
    let (mut w, r) = left_right_map::new();
    w.insert(1, "a");
    w.insert(2, "b");
    w.insert(3, "c");
    w.publish();
    // not visible until publish is called again
    w.insert(1, "x");

    {
        // so guard is dropped and we can publish
        let guard = r.enter().unwrap();
        let mut values: Vec<&&str> = guard.values().collect();
        values.sort();

        assert_eq!(values, vec![&"a", &"b", &"c"]);
    }

    // now 'x' will replace 'a'
    w.publish();

    let guard = r.enter().unwrap();
    let mut values: Vec<&&str> = guard.values().collect();
    values.sort();

    assert_eq!(values, vec![&"b", &"c", &"x"]);
}

#[test]
fn foreach() {
    let (mut w, r) = left_right_map::new();
    w.insert(1, "a");
    w.insert(2, "b");
    w.publish();
    w.insert(1, "x");

    let r = r.enter().unwrap();
    for (k, vs) in &r {
        match k {
            1 => {
                assert_eq!(vs, &"a");
            }
            2 => {
                assert_eq!(vs, &"b");
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn modify() {
    let (mut w, r) = left_right_map::new();
    w.insert(1, "a");
    w.insert(2, "b");
    w.publish();

    w.modify(1, |val| {
        *val = "modified";
    });

    // haven't publish yet
    assert_eq!(r.get(&1).unwrap().deref(), &"a");
    assert_eq!(r.get(&2).unwrap().deref(), &"b");

    w.publish();
    assert_eq!(r.get(&1).unwrap().deref(), &"modified");
    assert_eq!(r.get(&2).unwrap().deref(), &"b");
}

#[test]
fn example_usage() {
    // map storing <partition, offset>
    let (w, r) = left_right_map::new::<i32, FinishingOffset>();
    let arc_w = Arc::new(Mutex::new(w));

    std::thread::scope(|s| {
        for i in 0..10i32 {
            let arc_w_c = arc_w.clone();
            let _handle = s.spawn(move || {
                let mut guard = arc_w_c.lock().unwrap();
                guard.insert(
                    i,
                    FinishingOffset::new(PartitionFinishMarker::Offset(i as i64)),
                );
                guard.publish();
            });
        }
    });

    assert_eq!(r.len(), 10);
    for i in 0..10 {
        let found = r.get(&(i as i32));
        assert_match!(found.as_deref(), Some(_));
        assert_eq!(
            found.unwrap().deref(),
            &FinishingOffset::new(PartitionFinishMarker::Offset(i))
        );
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PartitionFinishMarker {
    Offset(i64),
    None,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FinishingOffset {
    marker: PartitionFinishMarker,
    reached: bool,
}

impl FinishingOffset {
    pub fn new(marker: PartitionFinishMarker) -> Self {
        Self {
            marker,
            reached: false,
        }
    }
}
