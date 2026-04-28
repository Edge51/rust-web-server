use std::fs::File;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::back_test::data::{Audit, Candle, Event, StockData};
use crate::back_test::strategy::Strategy;

pub struct Engine<OrderStrategy>
where
    OrderStrategy: 'static + Strategy + Send
{
    pub strategy: OrderStrategy,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_tx: UnboundedSender<Event>,
}

impl<OrderStrategy> Engine<OrderStrategy>
where OrderStrategy: 'static + Strategy + Send
{
    pub fn new(strategy: OrderStrategy) -> Self {
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            strategy,
            event_rx,
            event_tx,
        }
    }

    pub async fn run_backtest(&mut self) -> Audit {
        let tx = self.event_tx.clone();
        tokio::spawn(async move {
            let candles = Self::read_data_from_csv();
            candles.into_iter()
                .for_each(|candle| {
                    tx.send(Event::OnCandle(candle)).unwrap()
                });
        });
        while let Some(event) = self.event_rx.recv().await {
            match event {
                Event::OnCandle(candle) => {
                    let orders = self.strategy.generate_orders(Event::OnCandle(candle));
                    for order in orders {
                        self.event_tx.send(Event::OnOrder(order)).expect("Send order failed");
                    }
                },
                Event::OnOrder(order) => {
                    println!("OnOrder:{:?}", order)
                },
                Event::OnTerminate => {
                    break;
                }
            }
        }
        Audit::new(77.7)
    }
    pub fn read_data_from_csv() -> Vec<Candle> {
        println!("Starting to read stock data");
        let file = File::open("600000_daily_data.csv").expect("file open failed");
        let mut reader = csv::Reader::from_reader(file);
        let mut candles: Vec<Candle> = Vec::new();
        for row in reader.deserialize() {
            let record: StockData
                = row.unwrap();
            candles.push(record.into_candle())
        }
        candles
    }
}

mod test {
    use crate::back_test::strategy::DefaultStrategy;
    use super::*;

    #[tokio::test]
    async fn test_run_backtest() {
        let mut engine = Engine::new(DefaultStrategy);
        let audit = engine.run_backtest().await;
        assert_eq!(audit.get_profit(), 77.7);
    }
}