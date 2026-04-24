use super::data::{Event, Order};

pub trait Strategy {
    type OrderIter: Iterator<Item = Order>;
    fn generate_orders(&mut self, event: Event) -> Self::OrderIter;
}