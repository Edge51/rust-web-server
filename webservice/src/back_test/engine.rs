use std::collections::HashMap;
use std::fs::File;
use log::debug;
use crate::back_test::data::{Audit, Candle, Event, StockData, Order, Portfolio, Deal, OrderType, ExecutionResult, OrderRejectedReason, OrderExecutionReport, OrderRejected};
use crate::back_test::strategy::Strategy;

pub struct Engine<OrderStrategy>
where
    OrderStrategy: 'static + Strategy + Send
{
    pub strategy: OrderStrategy,
}

impl<OrderStrategy> Engine<OrderStrategy>
where OrderStrategy: 'static + Strategy + Send
{
    pub fn new(strategy: OrderStrategy) -> Self {
        Self {
            strategy,
        }
    }

    pub fn execute_orders(&self, portfolio: &Portfolio, candle: &Candle, orders: Vec<Order>) -> OrderExecutionReport {
        let mut deals = Vec::new();
        let mut orders_rejected = Vec::new();
        for order in orders {
            let commission = 0.0;
            let execute_result  = match order.order_type {
                OrderType::Buy => {
                    if !portfolio.has_sufficient_cash(order.price * order.amount as f64) {
                        ExecutionResult::Rejected(OrderRejectedReason::CashNotEnough)
                    } else if candle.open <= order.price {
                        ExecutionResult::Filled{ price: candle.open }
                    } else if candle.low <= order.price {
                        ExecutionResult::Filled{ price: order.price }
                    } else {
                        ExecutionResult::Rejected(OrderRejectedReason::BuyPriceNotMatched)
                    }
                },
                OrderType::Sell => {
                    if !portfolio.has_sufficient_position(&order.instrument_id, order.amount) {
                        ExecutionResult::Rejected(OrderRejectedReason::PositionNotEnough)
                    } else if candle.open >= order.price {
                        ExecutionResult::Filled{ price: candle.open }
                    } else if candle.high >= order.price {
                        ExecutionResult::Filled{ price: order.price }
                    } else {
                        ExecutionResult::Rejected(OrderRejectedReason::SellPriceNotMatched)
                    }
                }
            };
            match execute_result {
                ExecutionResult::Filled {price} => {
                    deals.push(Deal::new(order.instrument_id, order.order_type, price, order.amount, commission));
                },
                ExecutionResult::Rejected(order_rejected_reason) => {
                    orders_rejected.push(OrderRejected::new(order, order_rejected_reason));
                }
            }
        }
        OrderExecutionReport::new(deals, orders_rejected)
    }

    pub fn run_backtest(&mut self) -> Audit {
        let candles = Self::read_data_from_csv();
        let initial_capital = 100000f64;
        let mut orders = Vec::new();
        let mut portfolio = Portfolio::new(initial_capital);
        let mut current_prices = HashMap::new();
        let mut order_execution_report_summary = OrderExecutionReport::default();

        for candle in candles {
            let order_execution_report = self.execute_orders(&portfolio, &candle, orders);
            for deal in &order_execution_report.deals {
                portfolio.apply_deal(deal).unwrap();
            }
            order_execution_report_summary.merge(order_execution_report);
            orders = self.strategy.generate_orders(Event::OnCandle(candle.clone())).collect();
            current_prices.entry(candle.instrument_id)
                .and_modify(|value|{ *value = candle.close})
                .or_insert(candle.close);
        }

        Audit::new(portfolio.total_value(&current_prices) - initial_capital, order_execution_report_summary, portfolio)
    }
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
}

#[cfg(test)]
mod test {
    use crate::back_test::strategy::DefaultStrategy;
    use super::*;

    pub fn calculate_profit_with_deals(deals: &Vec<Deal>, portfolio: &Portfolio, current_price: &HashMap<String, f64>) -> f64 {
        let mut profit = 0.0;
        for deal in deals {
            match deal.order_type {
                OrderType::Buy => {
                    profit -= deal.price * deal.amount as f64 + deal.commission;
                },
                OrderType::Sell => {
                    profit += deal.price * deal.amount as f64 - deal.commission;
                }
            }
        }
        profit += portfolio.position_value(current_price);
        profit
    }

    #[test]
    fn test_run_backtest() {
        let candles = Engine::<DefaultStrategy>::read_data_from_csv();
        let current_price = HashMap::from([(candles.last().unwrap().instrument_id.clone(), candles.last().unwrap().close)]);
        let mut engine = Engine::new(DefaultStrategy);
        let audit = engine.run_backtest();

        let profit = calculate_profit_with_deals(&audit.order_execution_report.deals, &audit.portfolio, &current_price);
        let diff = (audit.profit - profit).abs();
        assert!(diff / profit.abs() < 1e-6);
    }
}