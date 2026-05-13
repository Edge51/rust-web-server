use std::collections::HashMap;
use std::ops::Bound::Included;
use anyhow::Result;
use chrono::{NaiveDate, SecondsFormat};
use serde::Deserialize;

pub const FAILED: &str = "Condition failed";
#[derive(Debug)]
pub struct Candle {
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
        anyhow::bail!(FAILED);
    }
    Ok(())
}

impl Candle {
    pub fn new(
        open: f64,
        close: f64,
        high: f64,
        low: f64,
        volume: f64,
        amount:f64
    ) -> Self {
        Self::new_checked(open, close, high, low, volume, amount)
            .expect(FAILED)
    }

    pub fn new_checked(
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
    code: String,
    order_type: OrderType,
    price: f64,
    amount: u32,
}
impl Order {
    pub fn new(code: String, order_type: OrderType, price: f64, amount: u32) -> Self {
        Self { code, order_type, price, amount }
    }
}

pub struct Position {
    price: f64,
    amount: u32,
}

pub struct Portfolio {
    pub cash: f64,
    pub positions: HashMap<String, Position>,
    pub total_value: f64,
}

impl Portfolio {
    pub fn new(cash: f64) -> Self {
        Self {
            cash,
            positions: HashMap::new(),
            total_value: cash,
        }
    }
    pub fn sell(&mut self, order: Order) -> Result<()>{
        let position = self.positions.get(&order.code).ok_or_else(|| anyhow::anyhow!("No position"))?;
        if order.amount > position.amount {
            anyhow::bail!("Insufficient shares")
        }
        let remaining = position.amount - order.amount;
        self.cash += order.price * order.amount as f64;
        self.total_value -= order.price * order.amount as f64;
        if remaining == 0 {
            self.positions.remove(&order.code);
        } else {
            self.positions.insert(order.code.clone(), Position { price: position.price, amount: remaining });
        }
        Ok(())
    }

    pub fn buy(&mut self, order: Order) -> Result<()> {
        if self.cash < order.price * order.amount as f64 {
            anyhow::bail!("Insufficient cash")
        }
        self.cash -= order.price * order.amount as f64;
        self.total_value += order.price * order.amount as f64;
        Ok(())
    }
}

pub enum Event {
    OnCandle(Candle),
    OnOrder(Order),
    OnTerminate,
}

pub struct Audit {
    profit: f64
}

impl Audit {
    pub fn new(profit: f64) -> Self {
        Self { profit }
    }
    pub fn show_profit(&self) {
        println!("{}", self.profit);
    }
    pub fn get_profit(&self) -> f64 {
        self.profit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 创建一笔买入订单的辅助函数，测试时不用每次都写 OrderType
    fn buy_order(price: f64, amount: u32) -> Order {
        Order { code: "TEST".to_string(), order_type: OrderType::Buy, price, amount }
    }

    fn sell_order(price: f64, amount: u32) -> Order {
        Order { code: "TEST".to_string(), order_type: OrderType::Sell, price, amount }
    }

    #[test]
    fn test_new_portfolio_initial_state() {
        let p = Portfolio::new(100_000.0);
        // 初始现金 = 本金，总资产 = 本金，无持仓
        // 需要访问 cash / total_value 来验证，当前这两个字段可见性不同
        // 提示：可以给 Portfolio 加一个 getter: pub fn cash_balance(&self) -> f64
        todo!("断言 portfolio.cash 等于 100_000, total_value 等于 100_000");
    }

    #[test]
    fn test_buy_one_stock_reduces_cash() {
        let mut p = Portfolio::new(100_000.0);
        p.buy(buy_order(10.0, 100));
        // 买入 100 股 × 10 元 = 1000 元，现金应减少 1000
        assert_eq!(p.cash, 90_000.0);
    }

    #[test]
    fn test_buy_insufficient_cash_returns_error() {
        let mut p = Portfolio::new(1_000.0);
        // 尝试买入 1000 股 × 10 元 = 10,000 元，远超现金
        // buy 方法当前返回 ()，如果现金不足应该怎么处理？
        // 提示：把 buy 的返回值从 () 改为 Result<()>
        let r = p.buy(buy_order(10.0, 1000));
        assert_eq!(r.is_err(), true);
    }

    #[test]
    fn test_buy_same_stock_twice_averages_cost() {
        let mut p = Portfolio::new(100_000.0);
        p.buy(buy_order(10.0, 100));  // 第一次买入: 100股 @ 10元
        p.buy(buy_order(20.0, 100));  // 第二次买入: 100股 @ 20元
        // 总持仓 = 200股，总成本 = 1000 + 2000 = 3000
        // 加权平均成本 = 3000 / 200 = 15 元/股
        // 现金应减少 3000
        // 提示：需要 Position 有访问方法，或者给 Portfolio 加查询持仓的方法
        todo!("断言仓位存在，成本均价 = 15.0，持仓量 = 200 股");
    }

    #[test]
    fn test_buy_and_sell_full_cycle() {
        let mut p = Portfolio::new(100_000.0);
        p.buy(buy_order(10.0, 100));   // 买入 100股
        p.sell(sell_order(15.0, 100));  // 全部卖出

        // 现金变化：-1000 (买) + 1500 (卖) = +500
        // 最终现金 = 100_000 + 500 = 100_500
        todo!("断言 cash 为 100_500");
    }

    #[test]
    fn test_buy_then_sell_partial() {
        let mut p = Portfolio::new(100_000.0);
        p.buy(buy_order(10.0, 200));    // 买入 200股 @ 10元
        p.sell(sell_order(15.0, 50));    // 卖出 50股 @ 15元

        // 现金: -2000 + 750 = -1250, 剩余 98750
        // 剩余持仓: 150股
        todo!("断言现金 = 98_750, 剩余持仓 150 股");
    }

    #[test]
    fn test_sell_more_than_held_returns_error() {
        let mut p = Portfolio::new(100_000.0);
        p.buy(buy_order(10.0, 100));     // 买入 100股
        // 尝试卖出 200 股，持仓不足
        todo!("断言 sell 返回错误，持仓不变");
    }

    #[test]
    fn test_sell_without_holding_returns_error() {
        let mut p = Portfolio::new(100_000.0);
        // 没有买入过任何股票，直接卖出
        todo!("断言 sell 返回错误");
    }

    #[test]
    fn test_buy_zero_amount_should_fail() {
        let mut p = Portfolio::new(100_000.0);
        // 买入 0 股应该被视为无效操作
        todo!("断言 buy 返回错误或无副作用");
    }

    #[test]
    fn test_sell_zero_amount_should_fail() {
        let mut p = Portfolio::new(100_000.0);
        p.buy(buy_order(10.0, 100));
        // 卖出 0 股应该被视为无效操作
        todo!("断言 sell 返回错误或持仓不变");
    }
}