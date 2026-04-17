use std::ops::Bound::Included;
use anyhow::Result;
use chrono::NaiveDate;
use serde::Deserialize;

pub const FAILED: &str = "Condition failed";
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
    pub close: Option<f64>,
    #[serde(rename = "最高")]
    pub high: Option<f64>,
    #[serde(rename = "最低")]
    pub low: Option<f64>,
    #[serde(rename = "成交量")]
    pub volume: Option<f64>,
    #[serde(rename = "成交额")]
    pub amount: Option<f64>,
    #[serde(rename = "振幅")]
    pub amplitude: Option<f64>,
    #[serde(rename = "涨跌幅")]
    pub diff_ref: Option<f64>,
    #[serde(rename = "涨跌额")]
    pub diff: Option<f64>,
    #[serde(rename = "换手率")]
    pub change: Option<f64>,
}
