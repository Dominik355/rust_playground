mod callback;
mod channels;
pub mod rc;
mod reference;

#[derive(Debug, Clone)]
pub enum Event {
    Registered,
    Element(usize),
}
