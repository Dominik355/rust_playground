use crate::observer::Event;
use std::sync::mpsc::Sender;

trait Observable {
    fn register(&mut self, observer: Sender<Event>);
    fn event(&mut self, event: Event);
}

#[derive(Default)]
struct ObservableImpl {
    observers: Vec<Sender<Event>>,
}

impl Observable for ObservableImpl {
    fn register(&mut self, tx: Sender<Event>) {
        if let Ok(_) = tx.send(Event::Registered) {
            self.observers.push(tx);
        }
    }

    fn event(&mut self, event: Event) {
        self.observers.retain(|tx| match tx.send(event.clone()) {
            Ok(_) => true,
            Err(_) => false,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    #[test]
    fn test_1() {
        let mut observable = ObservableImpl::default();

        let (tx, rx) = channel();
        observable.register(tx);
        observable.event(Event::Element(3));

        println!("first: {:?}", rx.recv().unwrap());
        println!("second: {:?}", rx.recv().unwrap());
    }
}
