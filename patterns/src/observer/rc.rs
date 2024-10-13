use crate::observer::Event;
use std::rc::{Rc, Weak};

trait Observable {
    fn register(&mut self, observer: Weak<Box<dyn Observer>>);
    fn event(&mut self, event: Event);
}

trait Observer {
    fn notify(&self, event: Event);
}

#[derive(Default)]
struct ObservableImpl {
    observers: Vec<Weak<Box<dyn Observer>>>,
}

impl Observable for ObservableImpl {
    fn register(&mut self, observer: Weak<Box<dyn Observer>>) {
        if let Some(observer) = observer.upgrade() {
            observer.notify(Event::Registered);
            self.observers.push(Rc::downgrade(&observer));
        }
    }

    fn event(&mut self, event: Event) {
        self.observers.retain(|observer| match observer.upgrade() {
            None => false,
            Some(o) => {
                o.notify(event.clone());
                true
            }
        });
    }
}

struct ObserverImpl {
    pub name: String,
}

impl ObserverImpl {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Observer for ObserverImpl {
    fn notify(&self, event: Event) {
        println!("{:?} | Received Event: {:?}", self.name, event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let mut observable = ObservableImpl::default();

        let observer_1: Rc<Box<dyn Observer>> = Rc::new(Box::new(ObserverImpl::new("first")));
        observable.register(Rc::downgrade(&observer_1));

        let observer_2: Rc<Box<dyn Observer>> = Rc::new(Box::new(ObserverImpl::new("second")));
        observable.register(Rc::downgrade(&observer_2));

        observable.event(Event::Element(2));
        observable.event(Event::Element(3));
    }
}
