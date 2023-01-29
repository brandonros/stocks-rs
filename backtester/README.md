# backtester-rs
Finnhub API + SQLite + SPY OHLC candles + technical analysis = algorithmic trading backtesting

## How to use

```shell
./scripts/scrape.sh
./scripts/backtest.sh
```

## Architecture

```
candles -> strategy -> indicators -> signals -> trades
```

## Backtesting flow

```
for every date worth of candle data
  for every possible setting combination
    for every strategy indicator signal direction change
      determiine stop loss, profit limit, or direction change
aggregate statistics into win/loss ratio, profit/loss performance, etc.
```

## ROIs that don't make sense

```
2023-01-27T05:19:16.122Z INFO  [backtester::backtesting] Supertrend(SupertrendStrategyIndicatorSettings { supertrend_periods: 4, supertrend_multiplier: 1.0 })
2023-01-27T05:19:16.122Z INFO  [backtester::backtesting] BacktestSettings { slippage_percentage: 0.00025, profit_limit_percentage: 0.0015, stop_loss_percentage: -0.0025, warmed_up_index: 10, backtest_mode: SingleEntry }
2023-01-27T05:19:16.122Z INFO  [backtester::backtesting] BacktestStatistics { total_profit_percentage: 3.7833349985951554, total_win_profit_percentage: 8.658206314010863, total_loss_profit_percentage: -4.874871315416258, num_trades: 9352, num_dates: 212, num_trades_per_day: 44.113207547169814, num_profit_limits: 4203, num_stop_losses: 22, num_direction_changes: 5127, num_wins: 6349, num_losses: 3003, num_breakevens: 0, win_loss_ratio: 2.114219114219114, win_rate_percentage: 0.6788922155688623, starting_portfolio_value: 1000.0, final_portfolio_value: 43345.473757840075, portfolio_value_change: 42345.473757840075, portfolio_value_change_percentage: 42.345473757840075 }

2023-01-27T05:24:29.153Z INFO  [backtester::backtesting] Supertrend(SupertrendStrategyIndicatorSettings { supertrend_periods: 12, supertrend_multiplier: 1.0 })
2023-01-27T05:24:29.153Z INFO  [backtester::backtesting] BacktestSettings { slippage_percentage: 0.00025, profit_limit_percentage: 0.0015, stop_loss_percentage: -0.0035, warmed_up_index: 10, backtest_mode: SingleEntry }
2023-01-27T05:24:29.153Z INFO  [backtester::backtesting] BacktestStatistics { total_profit_percentage: 3.7373808821038255, total_win_profit_percentage: 8.443660699623134, total_loss_profit_percentage: -4.70627981751986, num_trades: 9097, num_dates: 212, num_trades_per_day: 42.910377358490564, num_profit_limits: 4158, num_stop_losses: 22, num_direction_changes: 4917, num_wins: 6226, num_losses: 2871, num_breakevens: 0, win_loss_ratio: 2.1685823754789273, win_rate_percentage: 0.6844014510278114, starting_portfolio_value: 1000.0, final_portfolio_value: 41415.9380778207, portfolio_value_change: 40415.9380778207, portfolio_value_change_percentage: 40.4159380778207 }

2023-01-27T05:59:31.702Z INFO  [backtester::backtesting] 13.477,16.913636363636364,0.002,-0.0095,Supertrend(SupertrendStrategyIndicatorSettings { supertrend_periods: 17, supertrend_multiplier: 2.25 })

2023-01-27T05:57:30.041Z INFO  [backtester::backtesting] 21.675,23.37727272727273,0.002,-0.006,Supertrend(SupertrendStrategyIndicatorSettings { supertrend_periods: 14, supertrend_multiplier: 1.75 })

2023-01-27T06:03:41.139Z INFO  [backtester::backtesting] 71.988,56.64545454545455,0.0015,-1,Supertrend(SupertrendStrategyIndicatorSettings { supertrend_periods: 22, supertrend_multiplier: 0.75 })

2023-01-27T06:03:41.140Z INFO  [backtester::backtesting] 66.111,75.33181818181818,0.001,-1,Supertrend(SupertrendStrategyIndicatorSettings { supertrend_periods: 26, supertrend_multiplier: 0.5 })

2023-01-27T06:03:41.145Z INFO  [backtester::backtesting] 44.272,104.53636363636363,0.001,-1,Supertrend(SupertrendStrategyIndicatorSettings { supertrend_periods: 6, supertrend_multiplier: 0.25 })
```
