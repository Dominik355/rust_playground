use crate::observer::Event;

trait IObserver {
    fn update(&self, event: &Event);
}

trait ISubject<'a, T: IObserver> {
    fn attach(&mut self, observer: &'a T);
    fn detach(&mut self, observer: &'a T);
    fn notify_observers(&self, event: Event);
}

struct Subject<'a, T: IObserver> {
    observers: Vec<&'a T>,
}
impl<'a, T: IObserver + PartialEq> Subject<'a, T> {
    fn new() -> Subject<'a, T> {
        Subject {
            observers: Vec::new(),
        }
    }
}

impl<'a, T: IObserver + PartialEq> ISubject<'a, T> for Subject<'a, T> {
    fn attach(&mut self, observer: &'a T) {
        self.observers.push(observer);
        self.observers.last().unwrap().update(&Event::Registered)
    }
    fn detach(&mut self, observer: &'a T) {
        if let Some(idx) = self.observers.iter().position(|x| *x == observer) {
            self.observers.remove(idx);
        }
    }
    fn notify_observers(&self, event: Event) {
        for item in self.observers.iter() {
            item.update(&event);
        }
    }
}

#[derive(PartialEq)]
struct ConcreteObserver {
    id: i32,
}
impl IObserver for ConcreteObserver {
    fn update(&self, event: &Event) {
        println!("{} | received event: {:?}", self.id, event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let mut subject = Subject::new();
        let observer_a = ConcreteObserver { id: 1 };
        let observer_b = ConcreteObserver { id: 2 };

        subject.attach(&observer_a);
        subject.attach(&observer_b);
        subject.notify_observers(Event::Element(2));

        subject.detach(&observer_b);
        subject.notify_observers(Event::Element(3));
    }
}
