# stocks-rs
Monorepo for various projects related to stocks/backtesting/algorithmic trading/technical analysis

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

## Historical backtesting

```shell
START_DATE=$(date -v -400d +%F)
END_DATE=$(date +%F)

cargo run --bin historical_candle_scraper tradingview 2023-02-01 2023-02-01
cargo run --bin backtester tradingview supertrend SPY 1 2023-02-01 2023-02-01
cargo run --bin backtester polygon supertrend SPY 1 2023-02-01 2023-02-01
cargo run --bin signal_backtester
```

## To validate

```
2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] Supertrend(SupertrendStrategyIndicatorSettings { supertrend_periods: 10, supertrend_multiplier: 3.0 })
2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] BacktestSettings { slippage_percentage: 0.000125, profit_limit_percentage: 0.005, stop_loss_percentage: -0.01, warmed_up_index: 0, backtest_mode: SingleEntry }
2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] BacktestStatistics { total_profit_percentage: 0.005214250188648014, total_win_profit_percentage: 0.009599996650145323, total_loss_profit_percentage: -0.004385746461497308, num_trades: 9, num_dates: 1, num_trades_per_day: 9.0, num_profit_limits: 1, num_stop_losses: 0, num_direction_changes: 8, num_wins: 5, num_losses: 4, num_breakevens: 0, win_loss_ratio: 1.25, win_rate_percentage: 0.5555555555555556, starting_portfolio_value: 1000.0, final_portfolio_value: 1005.2079426702178, portfolio_value_change: 5.20794267021779, portfolio_value_change_percentage: 0.005207942670217789 }

2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] open,1675175400,Long,401.18014125, // 9:30am
2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] close,1675177200,Short,401.17984625,DirectionChange // 10:00am

2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] open,1675177200,Short,401.17984625, // 10:00am
2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] close,1675178940,Long,402.36038876249995,DirectionChange // 10:29am

2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] open,1675178940,Long,402.36038876249995, // 10:29am
2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] close,1675183680,Short,402.98962,DirectionChange // 11:48am

2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] open,1675183680,Short,402.98962, // 11:48am
2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] close,1675185480,Long,403.2504,DirectionChange // 12:18pm

2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] open,1675185480,Long,403.2504, // 12:18pm
2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] close,1675189380,Short,403.57954625,DirectionChange // 1:23pm

2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] open,1675189380,Short,403.57954625, // 1:23pm
2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] close,1675190760,Long,403.90048125000004,DirectionChange // 1:46pm

2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] open,1675190760,Long,403.90048125000004, // 1:46pm
2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] close,1675195800,Short,404.54942500000004,DirectionChange // 3:10pm

2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] open,1675195800,Short,404.54942500000004, // 3:10pm
2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] close,1675197240,Long,404.250525,DirectionChange

2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] open,1675197240,Long,404.250525, // 3:34pm
2023-02-01T05:01:12.964Z INFO  [backtester::backtesting] close,1675198680,Long,406.2209936527968,ProfitLimit // 3:58pm
```

## To validate

```
2023-02-01T05:10:08.306Z INFO  [signal_backtester] csv: open,1675175402,Long,401.28015375,
2023-02-01T05:10:08.306Z INFO  [signal_backtester] csv: close,1675177262,Long,400.97487187499996,DirectionChange

2023-02-01T05:10:08.306Z INFO  [signal_backtester] csv: open,1675177262,Short,400.97487187499996,
2023-02-01T05:10:08.307Z INFO  [signal_backtester] csv: close,1675178402,Short,402.44029875,DirectionChange

2023-02-01T05:10:08.307Z INFO  [signal_backtester] csv: open,1675178402,Long,402.44029875,
2023-02-01T05:10:08.308Z INFO  [signal_backtester] csv: close,1675181582,Long,402.694656875,DirectionChange

2023-02-01T05:10:08.308Z INFO  [signal_backtester] csv: open,1675181582,Short,402.694656875,
2023-02-01T05:10:08.308Z INFO  [signal_backtester] csv: close,1675181642,Short,403.02037125000004,DirectionChange
2023-02-01T05:10:08.308Z INFO  [signal_backtester] csv: open,1675181642,Long,403.02037125000004,
2023-02-01T05:10:08.309Z INFO  [signal_backtester] csv: close,1675183742,Long,402.8496375,DirectionChange
2023-02-01T05:10:08.309Z INFO  [signal_backtester] csv: open,1675183742,Short,402.8496375,
2023-02-01T05:10:08.310Z INFO  [signal_backtester] csv: close,1675185482,Short,403.24039875,DirectionChange
2023-02-01T05:10:08.310Z INFO  [signal_backtester] csv: open,1675185482,Long,403.24039875,
2023-02-01T05:10:08.313Z INFO  [signal_backtester] csv: close,1675189142,Long,403.56904756250003,DirectionChange
2023-02-01T05:10:08.313Z INFO  [signal_backtester] csv: open,1675189142,Short,403.56904756250003,
2023-02-01T05:10:08.314Z INFO  [signal_backtester] csv: close,1675190462,Short,403.87897856250004,DirectionChange
2023-02-01T05:10:08.314Z INFO  [signal_backtester] csv: open,1675190462,Long,403.87897856250004,
2023-02-01T05:10:08.317Z INFO  [signal_backtester] csv: close,1675192802,Long,404.13947625,DirectionChange
2023-02-01T05:10:08.317Z INFO  [signal_backtester] csv: open,1675192802,Short,404.13947625,
2023-02-01T05:10:08.317Z INFO  [signal_backtester] csv: close,1675192862,Short,404.27362788749997,DirectionChange
2023-02-01T05:10:08.317Z INFO  [signal_backtester] csv: open,1675192862,Long,404.27362788749997,
2023-02-01T05:10:08.317Z INFO  [signal_backtester] csv: close,1675193162,Long,404.100081175,DirectionChange
2023-02-01T05:10:08.317Z INFO  [signal_backtester] csv: open,1675193162,Short,404.100081175,
2023-02-01T05:10:08.319Z INFO  [signal_backtester] csv: close,1675194602,Short,404.635573125,DirectionChange
2023-02-01T05:10:08.319Z INFO  [signal_backtester] csv: open,1675194602,Long,404.635573125,
2023-02-01T05:10:08.321Z INFO  [signal_backtester] csv: close,1675195862,Long,404.304455625,DirectionChange
2023-02-01T05:10:08.321Z INFO  [signal_backtester] csv: open,1675195862,Short,404.304455625,
2023-02-01T05:10:08.323Z INFO  [signal_backtester] csv: close,1675197302,Short,404.33553562500003,DirectionChange
2023-02-01T05:10:08.323Z INFO  [signal_backtester] csv: open,1675197302,Long,404.33553562500003,
```