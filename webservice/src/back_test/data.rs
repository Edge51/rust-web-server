use std::collections::HashMap;
use anyhow::{bail, Result};
use chrono::{NaiveDate};
use serde::Deserialize;

pub const FAILED: &str = "Condition failed";
#[derive(Debug, Clone)]
pub struct Candle {
    pub instrument_id: String,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
    pub amount: f64
}

#[inline(always)]
pub fn check_predicate_true(predicate: bool, fail_msg: &str) -> anyhow::Result<()> {
    if !predicate {
        anyhow::bail!(FAILED.to_string() + ": " + fail_msg);
    }
    Ok(())
}

impl Candle {
    pub fn new_checked(
        instrument_id: String,
        open: f64,
        close: f64,
        high: f64,
        low: f64,
        volume: f64,
        amount: f64
    ) -> Result<Self> {
        check_predicate_true(high >= low, "high >= low")?;
        check_predicate_true(high >= open, "hign >= open")?;
        check_predicate_true(high >= close, "hign >= close")?;
        check_predicate_true(low <= close, "low <= close")?;
        check_predicate_true(low <= open, "low <= open")?;
        Ok(Self{
            instrument_id,
            open,
            close,
            high,
            low,
            volume,
            amount,
        })
    }
}
#[derive(Deserialize, Debug)]
pub struct StockData {
    #[serde(rename = "日期")]
    pub date: NaiveDate,
    #[serde(rename = "股票代码")]
    pub code: String,
    #[serde(rename = "开盘")]
    pub open: f64,
    #[serde(rename = "收盘")]
    pub close: f64,
    #[serde(rename = "最高")]
    pub high: f64,
    #[serde(rename = "最低")]
    pub low: f64,
    #[serde(rename = "成交量")]
    pub volume: f64,
    #[serde(rename = "成交额")]
    pub amount: f64,
    #[serde(rename = "振幅")]
    pub amplitude: f64,
    #[serde(rename = "涨跌幅")]
    pub diff_ref: f64,
    #[serde(rename = "涨跌额")]
    pub diff: f64,
    #[serde(rename = "换手率")]
    pub change: f64,
}

impl StockData {
    pub fn into_candle(self) -> Result<Candle> {
        Candle::new_checked(
            self.code,
            self.open,
            self.close,
            self.high,
            self.low,
            self.volume,
            self.amount,
        )
    }
}

#[derive(Debug)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(Debug)]
pub struct Order {
    pub instrument_id: String,
    pub order_type: OrderType,
    pub price: f64,
    pub amount: u32,
}
impl Order {
    pub fn new(code: String, order_type: OrderType, price: f64, amount: u32) -> Self {
        Self { instrument_id: code, order_type, price, amount }
    }
}

pub enum OrderRejectedReason {
    CashNotEnough,
    BuyPriceNotMatched,
    PositionNotEnough,
    SellPriceNotMatched,
}

pub enum ExecutionResult {
    Filled{ price: f64 },
    Rejected(OrderRejectedReason),
}

#[derive(Debug)]
pub struct Deal {
    pub instrument_id: String,
    pub order_type: OrderType,
    pub price: f64,
    pub amount: u32,
    pub commission: f64,
}
impl Deal {
    pub fn new(instrument_id: String, order_type: OrderType, price: f64, amount: u32, commission: f64) -> Self {
        Self { instrument_id, order_type, price, amount, commission }
    }
}

pub struct Position {
    avg_cost: f64,
    amount: u32,
}

impl Position {
    pub fn new(price: f64, amount: u32) -> Self {
        Self { avg_cost: price, amount }
    }
    pub fn update_position(&mut self, deal: &Deal) {
        match deal.order_type {
            OrderType::Buy => {
                self.avg_cost = (self.avg_cost * self.amount as f64 + deal.price * deal.amount as f64) / (self.amount as f64 + deal.amount as f64);
                self.amount += deal.amount;
            },
            OrderType::Sell => {
                debug_assert!(self.amount >= deal.amount);
                self.amount -= deal.amount;
            }
        }
    }
}

pub struct Portfolio {
    pub cash: f64,
    pub positions: HashMap<String, Position>,
}

impl Portfolio {
    pub fn new(cash: f64) -> Self {
        Self {
            cash,
            positions: HashMap::new(),
        }
    }
    pub fn apply_deal(&mut self, deal: &Deal) -> Result<()> {
        match deal.order_type {
            OrderType::Buy => {
                if deal.price * deal.amount as f64 > self.cash {
                    bail!(FAILED);
                } else {
                    self.positions.entry(deal.instrument_id.clone())
                        .and_modify(|po| {po.update_position(deal)})
                        .or_insert(Position::new(deal.price, deal.amount));
                    self.cash -= deal.price * deal.amount as f64 + deal.commission;
                    Ok(())
                }
            },
            OrderType::Sell => {
                if !self.positions.contains_key(&deal.instrument_id) || self.positions[&deal.instrument_id].amount < deal.amount {
                    bail!(FAILED);
                } else {
                    self.positions.get_mut(&deal.instrument_id).unwrap().update_position(&deal);
                    self.cash += deal.price * deal.amount as f64 - deal.commission;
                    Ok(())
                }
            }
        }
    }

    pub fn has_sufficient_cash(&self, cost: f64) -> bool {
        self.cash >= cost
    }

    pub fn has_sufficient_position(&self, instrument_id: &String, amount: u32) -> bool {
        self.positions.contains_key(instrument_id) && self.positions[instrument_id].amount >= amount
    }
    pub fn total_value(&self, current_prices: &HashMap<String, f64>) -> f64 {
        self.positions.iter()
            .fold(self.cash, |acc, (instrument_id, position)| {
                acc + current_prices[instrument_id] * position.amount as f64
            })
    }
    pub fn position_value(&self, current_prices: &HashMap<String, f64>) -> f64 {
        self.positions.iter()
            .fold(0f64, |acc, (instrument_id, position)| {
                acc + current_prices[instrument_id] * position.amount as f64
            })
    }
}

pub enum Event {
    OnCandle(Candle),
}

pub struct OrderRejected {
    order: Order,
    order_rejected_reason: OrderRejectedReason,
}
impl OrderRejected {
    pub fn new(order: Order, order_rejected_reason: OrderRejectedReason) -> Self {
        Self { order, order_rejected_reason }
    }
}

pub struct OrderExecutionReport {
    pub deals: Vec<Deal>,
    pub orders_rejected: Vec<OrderRejected>,
}
impl OrderExecutionReport {
    pub fn default() -> Self {
        Self {
            deals: Vec::new(),
            orders_rejected: Vec::new(),
        }
    }
    pub fn new(deals: Vec<Deal>, orders_rejected: Vec<OrderRejected>) -> Self {
        Self { deals, orders_rejected }
    }
    pub fn merge(&mut self, other: OrderExecutionReport) {
        self.deals.extend(other.deals);
        self.orders_rejected.extend(other.orders_rejected);
    }
}

pub struct Audit {
    pub profit: f64,
    pub order_execution_report: OrderExecutionReport,
    pub portfolio: Portfolio,
}

impl Audit {
    pub fn new(profit: f64, order_execution_report: OrderExecutionReport, portfolio: Portfolio) -> Self {
        Self { profit, order_execution_report, portfolio}
    }

    pub fn profit(&self) -> f64 {
        self.profit
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_new_portfolio_initial_state() {
        let portfolio = Portfolio::new(10.0);
        assert_eq!(portfolio.cash, 10.0);
        assert_eq!(portfolio.positions.len(), 0);
    }

    #[test]
    fn test_buy_one_stock_reduces_cash() {
        let mut portfolio = Portfolio::new(1000.0);
        assert_eq!(portfolio.cash, 1000.0);
        let deal = Deal::new("test".to_string(), OrderType::Buy, 1.0, 100, 0.0);
        assert!(portfolio.apply_deal(&deal).is_ok());
        assert_eq!(portfolio.cash, 900.0);
    }

    #[test]
    fn test_buy_insufficient_cash_returns_error() {
        let mut portfolio = Portfolio::new(10.0);
        assert_eq!(portfolio.cash, 10.0);
        let deal = Deal::new("test".to_string(), OrderType::Buy, 1.0, 100, 0.0);
        assert!(portfolio.apply_deal(&deal).is_err());
    }

    #[test]
    fn test_buy_and_sell_full_cycle() {
        let mut portfolio = Portfolio::new(1000.0);
        assert_eq!(portfolio.cash, 1000.0);
        let deal = Deal::new("test".to_string(), OrderType::Buy, 1.0, 200, 0.0);
        assert!(portfolio.apply_deal(&deal).is_ok());
        assert_eq!(portfolio.cash, 800.0);
        let deal = Deal::new("test".to_string(), OrderType::Sell, 1.0, 200, 0.0);
        assert!(portfolio.apply_deal(&deal).is_ok());
        assert_eq!(portfolio.cash, 1000.0);
    }

    #[test]
    fn test_buy_then_sell_partial() {
        let mut portfolio = Portfolio::new(1000.0);
        assert_eq!(portfolio.cash, 1000.0);
        let deal = Deal::new("test".to_string(), OrderType::Buy, 1.0, 200, 0.0);
        assert!(portfolio.apply_deal(&deal).is_ok());
        assert_eq!(portfolio.cash, 800.0);
        let deal = Deal::new("test".to_string(), OrderType::Sell, 1.0, 50, 0.0);
        assert!(portfolio.apply_deal(&deal).is_ok());
        assert_eq!(portfolio.cash, 850.0);
        assert_eq!(portfolio.positions.len(), 1);
        assert_eq!(portfolio.positions.get(&deal.instrument_id).unwrap().amount, 150);
    }

    #[test]
    fn test_sell_without_holding_returns_error() {
        let mut portfolio = Portfolio::new(10.0);
        assert_eq!(portfolio.cash, 10.0);
        let deal = Deal::new("test".to_string(), OrderType::Sell, 1.0, 100, 0.0);
        assert!(portfolio.apply_deal(&deal).is_err());
    }
}