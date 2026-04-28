use super::data::{Event, Order};

pub trait Strategy {
    type OrderIter: Iterator<Item = Order>;
    fn generate_orders(&mut self, event: Event) -> Self::OrderIter;
}

pub struct DefaultStrategy;

impl Strategy for DefaultStrategy {
    type OrderIter = std::vec::IntoIter<Order>;
    fn generate_orders(&mut self, event: Event) -> Self::OrderIter {
        match event {
            Event::OnCandle(candle) => {
                if candle.close > candle.low {
                    println!("CLOSE: {} > OPEN :{}, candle{:?}", candle.close, candle.open, candle);
                } else {
                    println!("CLOSE: {} <= OPEN :{}, candle{:?}", candle.close, candle.open, candle);
                }
            },
            _ => {
                println!("Event not candle should not pass to strategy default");
            }
        }
        let mut orders = Vec::new();
        let order = Order::new(66.6, 100);
        orders.push(order);
        orders.into_iter()
    }
}
