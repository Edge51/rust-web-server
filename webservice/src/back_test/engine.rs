use std::collections::HashMap;
use crate::back_test::data::{Audit, Candle, Event, Order, Portfolio, Deal, OrderType, ExecutionResult, OrderRejectedReason, OrderExecutionReport, OrderRejected, BacktestConfig, SlippageModel,};
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

    pub fn execute_orders(&self, backtest_config: &BacktestConfig, slippage_model: &mut SlippageModel, portfolio: &Portfolio, candle: &Candle, orders: Vec<Order>) -> OrderExecutionReport {
        let mut deals = Vec::new();
        let mut orders_rejected = Vec::new();
        for order in orders {
            let execute_result  = match order.order_type {
                OrderType::Buy => {
                    if order.price < candle.low {
                        ExecutionResult::Rejected(OrderRejectedReason::BuyPriceNotMatched)
                    } else if candle.open <= order.price {
                        ExecutionResult::Filled{ price: candle.open }
                    } else { // remaining order.price >= candle.low
                        ExecutionResult::Filled{ price: order.price }
                    }
                },
                OrderType::Sell => {
                    if candle.open >= order.price {
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
                    let slipped_price = slippage_model.slipped_price(price, &order.order_type);
                    let commission = f64::max(backtest_config.min_commission, slipped_price * order.amount as f64 * backtest_config.commission_rate);
                    match order.order_type {
                        OrderType::Buy => {
                            if !portfolio.has_sufficient_cash(commission + slipped_price * order.amount as f64) {
                                orders_rejected.push(OrderRejected::new(order, OrderRejectedReason::CashNotEnough));
                            } else {
                                deals.push(Deal::new(order.instrument_id, order.order_type, slipped_price, order.amount, commission));
                            }
                        },
                        OrderType::Sell => {
                            if !portfolio.has_sufficient_position(&order.instrument_id, order.amount) {
                                orders_rejected.push(OrderRejected::new(order, OrderRejectedReason::PositionNotEnough));
                            } else {
                                deals.push(Deal::new(order.instrument_id, order.order_type, slipped_price, order.amount, commission));
                            }
                        }
                    }
                },
                ExecutionResult::Rejected(order_rejected_reason) => {
                    orders_rejected.push(OrderRejected::new(order, order_rejected_reason));
                }
            }
        }
        OrderExecutionReport::new(deals, orders_rejected)
    }

    pub fn run_backtest(&mut self, backtest_config: BacktestConfig, candles: Vec<Candle>) -> Audit {
        let initial_capital = 100000f64;
        let mut orders = Vec::new();
        let mut portfolio = Portfolio::new(initial_capital);
        let mut current_prices = HashMap::new();
        let mut slippage_model = SlippageModel::from(&backtest_config.slippage_config);
        let mut order_execution_report_summary = OrderExecutionReport::default();

        for candle in candles {
            let order_execution_report = self.execute_orders(&backtest_config, &mut slippage_model, &portfolio, &candle, orders);
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

}

#[cfg(test)]
mod test {
    use crate::back_test::data::{BacktestConfig, SlippageConfig};
    use crate::back_test::data_loader::read_data_from_csv;
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
        let candles = read_data_from_csv();
        let slippage_config = SlippageConfig::new(666, 0.01, 0.01, 0.01, 3);
        let backtest_config = BacktestConfig { slippage_config, commission_rate: 8.54e-5, min_commission: 5.0};
        let current_price = HashMap::from([(candles.last().unwrap().instrument_id.clone(), candles.last().unwrap().close)]);
        let mut engine = Engine::new(DefaultStrategy);
        let audit = engine.run_backtest(backtest_config, candles);

        let profit = calculate_profit_with_deals(&audit.order_execution_report.deals, &audit.portfolio, &current_price);
        let diff = (audit.profit - profit).abs();
        assert!(diff / profit.abs() < 1e-6);
    }

    #[test]
    fn test_execute_orders() {
        let engine = Engine::new(DefaultStrategy);
        let slippage_config = SlippageConfig::new(666, 0.01, 0.01, 0.01, 3);
        let backtest_config = BacktestConfig { slippage_config, commission_rate: 8.54e-5, min_commission: 5.0};
        let mut slippage_model = SlippageModel::from(&backtest_config.slippage_config);
        let portfolio = Portfolio::new(10000f64);
        let candle = Candle::new_checked("test".to_string(), 10.0, 11.0, 11.5, 9.5, 100.0, 1100.0).unwrap();
        let orders = vec![Order::new("test".to_string(), OrderType::Buy, 11.0, 100)];
        let report = engine.execute_orders(&backtest_config, &mut slippage_model, &portfolio, &candle, orders);
        assert!(report.deals.len() > 0);
    }
}