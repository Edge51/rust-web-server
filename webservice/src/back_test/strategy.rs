use super::data::{Event, Order, OrderType};

pub trait Strategy {
    type OrderIter: Iterator<Item = Order>;
    fn generate_orders(&mut self, event: Event) -> Self::OrderIter;
}

pub struct DefaultStrategy;

impl Strategy for DefaultStrategy {
    type OrderIter = std::vec::IntoIter<Order>;
    fn generate_orders(&mut self, event: Event) -> Self::OrderIter {
        let order = match event {
            Event::OnCandle(candle) => {
                if candle.close >= 400.0 {
                    Some(Order::new(candle.instrument_id.clone(), OrderType::Sell, candle.close, 100))
                } else if candle.close <= 200.0 {
                    Some(Order::new(candle.instrument_id.clone(), OrderType::Buy, candle.close, 100))
                } else {
                    None
                }
            },
        };
        let mut orders = Vec::new();
        if let Some(order) = order {
            orders.push(order);
        }
        orders.into_iter()
    }
}
