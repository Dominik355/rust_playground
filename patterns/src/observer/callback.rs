pub struct Notifier<E, Ctx> {
    subscribers: Vec<Box<dyn Fn(&E, &mut Ctx)>>,
}

impl<E, Ctx> Notifier<E, Ctx> {
    pub fn new() -> Notifier<E, Ctx> {
        Notifier {
            subscribers: Vec::new(),
        }
    }

    pub fn register<F>(&mut self, callback: F)
    where
        F: 'static + Fn(&E, &mut Ctx),
    {
        self.subscribers.push(Box::new(callback));
    }

    pub fn notify(&self, event: E, ctx: &mut Ctx) {
        for callback in &self.subscribers {
            callback(&event, ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observer::Event;

    #[test]
    fn test_1() {
        let mut n = Notifier::<Event, i32>::new();
        n.register(|i, ctx| {
            println!("event received: {:?}, ctx: {:?}", i, ctx);
            *ctx += 1;
        });
        println!("sending event...");
        let mut ctx = 0;
        n.notify(Event::Registered, &mut ctx);
        println!("{ctx}");
    }
}
