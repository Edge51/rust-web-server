use std::fs::File;
use crate::back_test::data::{Candle, StockData};

pub fn read_data_from_csv() -> Vec<Candle> {
    let file = File::open("600000_daily_data.csv").expect("file open failed");
    let mut reader = csv::Reader::from_reader(file);
    let mut candles: Vec<Candle> = Vec::new();
    for row in reader.deserialize() {
        let record: StockData = row.unwrap();
        match record.into_candle() {
            Ok(candle) => candles.push(candle),
            Err(e) => eprintln!("Skipping invalid candle: {:?}", e),
        }
    }
    candles
}