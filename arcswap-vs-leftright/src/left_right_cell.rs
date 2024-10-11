use std::ops::Deref;

use left_right::Absorb;

struct SetOperation<T>(T);

impl<T> Absorb<SetOperation<T>> for Inner<T>
where
    T: Clone,
{
    fn absorb_first(&mut self, operation: &mut SetOperation<T>, _: &Self) {
        self.0 = operation.0.clone();
    }

    fn absorb_second(&mut self, operation: SetOperation<T>, _: &Self) {
        self.0 = operation.0;
    }

    fn drop_first(self: Box<Self>) {}

    fn sync_with(&mut self, first: &Self) {
        self.0 = first.0.clone()
    }
}

#[derive(Clone)]
struct Inner<T>(T);

impl<T> Default for Inner<T>
where
    T: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Clone)]
pub struct ReadHandle<T>(left_right::ReadHandle<Inner<T>>);

impl<T> ReadHandle<T> {
    pub fn get(&self) -> Option<ReadGuard<T>> {
        self.0.enter().map(|guard| ReadGuard(guard))
    }

    pub unsafe fn get_unchecked(&self) -> ReadGuard<T> {
        self.0
            .enter()
            .map(|guard| ReadGuard(guard))
            .unwrap_unchecked()
    }
}

pub struct ReadGuard<'a, T>(left_right::ReadGuard<'a, Inner<T>>);

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.as_ref().0
    }
}

pub struct WriteHandle<T: Clone>(left_right::WriteHandle<Inner<T>, SetOperation<T>>);

impl<T> WriteHandle<T>
where
    T: Clone,
{
    pub fn set(&mut self, value: T) {
        self.0.append(SetOperation(value));
    }

    pub fn publish(&mut self) {
        self.0.publish();
    }

    pub fn has_pending(&self) -> bool {
        self.0.has_pending_operations()
    }
}

pub fn new<T: Clone>(value: T) -> (WriteHandle<T>, ReadHandle<T>) {
    let (w, r) = left_right::new_from_empty::<Inner<T>, SetOperation<T>>(Inner(value));
    (WriteHandle(w), ReadHandle(r))
}

pub fn new_default<T: Clone + Default>() -> (WriteHandle<T>, ReadHandle<T>) {
    let (w, r) = left_right::new_from_empty::<Inner<T>, SetOperation<T>>(Inner::default());
    (WriteHandle(w), ReadHandle(r))
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    #[test]
    fn it_works() {
        let (mut w, r) = super::new(false);

        let t = thread::spawn(move || loop {
            let value = r.get().unwrap();
            if *value {
                break;
            }
            thread::sleep(Duration::from_millis(1));
        });

        w.set(true);
        w.publish();
        t.join().unwrap();
        assert!(true);
    }
}
