use tokio::sync::mpsc::{Receiver, Sender};
use crate::back_test::data::{Audit, Event};
use crate::back_test::strategy::Strategy;

pub struct Engine<OrderStrategy>
where
    OrderStrategy: 'static + Strategy + Send
{
    pub strategy: OrderStrategy,
    pub event_rx: Receiver<Event>,
    pub event_tx: Sender<Event>,
}

impl<OrderStrategy> Engine<OrderStrategy>
where OrderStrategy: 'static + Strategy + Send
{
    pub fn new(strategy: OrderStrategy) -> Self {
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(100);
        Self {
            strategy,
            event_rx,
            event_tx,
        }
    }

    pub async fn run_backtest(&mut self) -> Audit {
        while let Some(event) = self.event_rx.recv().await {
            match event {
                Event::OnCandle(candle) => {
                    let orders = self.strategy.generate_orders(Event::OnCandle(candle));
                    for order in orders {
                        self.event_tx.send(Event::OnOrder(order)).await.expect("Send order failed");
                    }
                },
                Event::OnOrder(order) => {println!("OnOrder:{:?}", order)},
            }
        }
        todo!()
    }
}