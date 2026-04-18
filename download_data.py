import akshare as ak

stock_zh_a_hist_df = ak.stock_zh_a_hist(
    symbol="002050",
    period="daily",
    start_date="20200501",
    end_date="20260106",
    adjust="hfq"
)
print(stock_zh_a_hist_df)
stock_zh_a_hist_df.to_csv('600000_daily_data.csv', index=False)
