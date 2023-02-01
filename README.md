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
cargo run --bin historical_candle_scraper 2023-01-31 2023-01-31
cargo run --bin backtester backtest polygon supertrend SPY 1 "2023-01-31 00:00:00" "2023-01-31 00:00:00"
```

## To validate

```
2023-02-01T04:13:48.780Z INFO  [backtester::backtesting] Supertrend(SupertrendStrategyIndicatorSettings { supertrend_periods: 10, supertrend_multiplier: 3.0 })
2023-02-01T04:13:48.780Z INFO  [backtester::backtesting] BacktestSettings { slippage_percentage: 0.000125, profit_limit_percentage: 0.005, stop_loss_percentage: -0.01, warmed_up_index: 0, backtest_mode: SingleEntry }
2023-02-01T04:13:48.780Z INFO  [backtester::backtesting] BacktestStatistics { total_profit_percentage: 0.005326168750817819, total_win_profit_percentage: 0.009661183356849967, total_loss_profit_percentage: -0.004335014606032146, num_trades: 9, num_dates: 1, num_trades_per_day: 9.0, num_profit_limits: 1, num_stop_losses: 0, num_direction_changes: 8, num_wins: 6, num_losses: 3, num_breakevens: 0, win_loss_ratio: 2.0, win_rate_percentage: 0.6666666666666666, starting_portfolio_value: 1000.0, final_portfolio_value: 1005.3204907790355, portfolio_value_change: 5.320490779035481, portfolio_value_change_percentage: 0.005320490779035481 }

2023-02-01T04:13:48.780Z INFO  [backtester::backtesting] open,1675175400,Long,401.18014125, 9:30:00am
2023-02-01T04:13:48.780Z INFO  [backtester::backtesting] close,1675177140,Long,401.1897450125,DirectionChange 9:59:00am

2023-02-01T04:13:48.780Z INFO  [backtester::backtesting] open,1675177200,Short,401.17984625, 10:00:00am
2023-02-01T04:13:48.780Z INFO  [backtester::backtesting] close,1675178880,Short,402.35028750000004,DirectionChange 10:28:00am
2023-02-01T04:13:48.780Z INFO  [backtester::backtesting] open,1675178940,Long,402.36038876249995,
2023-02-01T04:13:48.780Z INFO  [backtester::backtesting] close,1675183620,Long,402.99961875,DirectionChange
2023-02-01T04:13:48.780Z INFO  [backtester::backtesting] open,1675183680,Short,402.98962,
2023-02-01T04:13:48.780Z INFO  [backtester::backtesting] close,1675185420,Short,403.24039875,DirectionChange
2023-02-01T04:13:48.780Z INFO  [backtester::backtesting] open,1675185480,Long,403.2504,
2023-02-01T04:13:48.781Z INFO  [backtester::backtesting] close,1675189320,Long,403.584545625,DirectionChange
2023-02-01T04:13:48.781Z INFO  [backtester::backtesting] open,1675189380,Short,403.57954625,
2023-02-01T04:13:48.781Z INFO  [backtester::backtesting] close,1675190700,Short,403.90048125000004,DirectionChange
2023-02-01T04:13:48.781Z INFO  [backtester::backtesting] open,1675190760,Long,403.90048125000004,
2023-02-01T04:13:48.781Z INFO  [backtester::backtesting] close,1675195740,Long,404.54942500000004,DirectionChange
2023-02-01T04:13:48.781Z INFO  [backtester::backtesting] open,1675195800,Short,404.54942500000004,
2023-02-01T04:13:48.781Z INFO  [backtester::backtesting] close,1675197180,Short,404.250525,DirectionChange
2023-02-01T04:13:48.781Z INFO  [backtester::backtesting] open,1675197240,Long,404.250525,
2023-02-01T04:13:48.781Z INFO  [backtester::backtesting] close,1675198680,Long,406.2209936527968,ProfitLimit
```

## To validate

```
2023-02-01T04:20:25.271Z INFO  [signal_backtester] csv: open,1675175402,Long,401.28015375, 9:30:02am
2023-02-01T04:20:25.284Z INFO  [signal_backtester] csv: close,1675177255,Long,400.9698725,DirectionChange 10:00:55am

2023-02-01T04:20:25.284Z INFO  [signal_backtester] csv: open,1675177255,Short,400.9698725, 10:00:55am
2023-02-01T04:20:25.295Z INFO  [signal_backtester] csv: close,1675178170,Short,402.35028750000004,DirectionChange 10:16:10am

2023-02-01T04:20:25.295Z INFO  [signal_backtester] csv: open,1675178170,Long,402.35028750000004, 10:16:10am
2023-02-01T04:20:25.295Z INFO  [signal_backtester] csv: close,1675178175,Long,402.28970749999996,DirectionChange 10:16:15am

2023-02-01T04:20:25.296Z INFO  [signal_backtester] csv: open,1675178175,Short,402.28970749999996,
2023-02-01T04:20:25.296Z INFO  [signal_backtester] csv: close,1675178226,Short,402.34028625,DirectionChange
2023-02-01T04:20:25.296Z INFO  [signal_backtester] csv: open,1675178226,Long,402.34028625,
2023-02-01T04:20:25.296Z INFO  [signal_backtester] csv: close,1675178231,Long,402.23971375,DirectionChange
2023-02-01T04:20:25.296Z INFO  [signal_backtester] csv: open,1675178231,Short,402.23971375,
2023-02-01T04:20:25.297Z INFO  [signal_backtester] csv: close,1675178236,Short,402.36028875,DirectionChange
2023-02-01T04:20:25.297Z INFO  [signal_backtester] csv: open,1675178236,Long,402.36028875,
2023-02-01T04:20:25.297Z INFO  [signal_backtester] csv: close,1675178241,Long,402.234714375,DirectionChange
2023-02-01T04:20:25.297Z INFO  [signal_backtester] csv: open,1675178241,Short,402.234714375,
2023-02-01T04:20:25.298Z INFO  [signal_backtester] csv: close,1675178320,Short,402.35028750000004,DirectionChange
2023-02-01T04:20:25.298Z INFO  [signal_backtester] csv: open,1675178320,Long,402.35028750000004,
2023-02-01T04:20:25.298Z INFO  [signal_backtester] csv: close,1675178325,Long,402.229715,DirectionChange
2023-02-01T04:20:25.298Z INFO  [signal_backtester] csv: open,1675178325,Short,402.229715,
2023-02-01T04:20:25.298Z INFO  [signal_backtester] csv: close,1675178345,Short,402.390892575,DirectionChange
2023-02-01T04:20:25.298Z INFO  [signal_backtester] csv: open,1675178345,Long,402.390892575,
2023-02-01T04:20:25.298Z INFO  [signal_backtester] csv: close,1675178350,Long,402.23971375,DirectionChange
2023-02-01T04:20:25.299Z INFO  [signal_backtester] csv: open,1675178350,Short,402.23971375,
2023-02-01T04:20:25.299Z INFO  [signal_backtester] csv: close,1675178390,Short,402.4302975,DirectionChange
2023-02-01T04:20:25.299Z INFO  [signal_backtester] csv: open,1675178390,Long,402.4302975,
2023-02-01T04:20:25.336Z INFO  [signal_backtester] csv: close,1675180430,Long,402.5296775,DirectionChange
2023-02-01T04:20:25.336Z INFO  [signal_backtester] csv: open,1675180430,Short,402.5296775,
2023-02-01T04:20:25.336Z INFO  [signal_backtester] csv: close,1675180435,Short,402.650325,DirectionChange
2023-02-01T04:20:25.336Z INFO  [signal_backtester] csv: open,1675180435,Long,402.650325,
2023-02-01T04:20:25.362Z INFO  [signal_backtester] csv: close,1675181580,Long,402.6896575,DirectionChange
2023-02-01T04:20:25.362Z INFO  [signal_backtester] csv: open,1675181580,Short,402.6896575,
2023-02-01T04:20:25.363Z INFO  [signal_backtester] csv: close,1675181585,Short,402.78034125,DirectionChange
2023-02-01T04:20:25.363Z INFO  [signal_backtester] csv: open,1675181585,Long,402.78034125,
2023-02-01T04:20:25.416Z INFO  [signal_backtester] csv: close,1675183505,Long,402.89963124999997,DirectionChange
2023-02-01T04:20:25.416Z INFO  [signal_backtester] csv: open,1675183505,Short,402.89963124999997,
2023-02-01T04:20:25.417Z INFO  [signal_backtester] csv: close,1675183515,Short,402.9903675,DirectionChange
2023-02-01T04:20:25.417Z INFO  [signal_backtester] csv: open,1675183515,Long,402.9903675,
2023-02-01T04:20:25.423Z INFO  [signal_backtester] csv: close,1675183700,Long,402.85963625000005,DirectionChange
2023-02-01T04:20:25.423Z INFO  [signal_backtester] csv: open,1675183700,Short,402.85963625000005,
2023-02-01T04:20:25.423Z INFO  [signal_backtester] csv: close,1675183715,Short,403.0142704875,DirectionChange
2023-02-01T04:20:25.423Z INFO  [signal_backtester] csv: open,1675183715,Long,403.0142704875,
2023-02-01T04:20:25.424Z INFO  [signal_backtester] csv: close,1675183730,Long,402.71965374999996,DirectionChange
2023-02-01T04:20:25.424Z INFO  [signal_backtester] csv: open,1675183730,Short,402.71965374999996,
2023-02-01T04:20:25.482Z INFO  [signal_backtester] csv: close,1675185440,Short,403.2303975,DirectionChange
2023-02-01T04:20:25.482Z INFO  [signal_backtester] csv: open,1675185440,Long,403.2303975,
2023-02-01T04:20:25.484Z INFO  [signal_backtester] csv: close,1675185475,Long,403.07960875,DirectionChange
2023-02-01T04:20:25.484Z INFO  [signal_backtester] csv: open,1675185475,Short,403.07960875,
2023-02-01T04:20:25.484Z INFO  [signal_backtester] csv: close,1675185480,Short,403.26040125,DirectionChange
2023-02-01T04:20:25.484Z INFO  [signal_backtester] csv: open,1675185480,Long,403.26040125,
2023-02-01T04:20:25.618Z INFO  [signal_backtester] csv: close,1675188635,Long,403.509555,DirectionChange
2023-02-01T04:20:25.618Z INFO  [signal_backtester] csv: open,1675188635,Short,403.509555,
2023-02-01T04:20:25.619Z INFO  [signal_backtester] csv: close,1675188645,Short,403.610445,DirectionChange
2023-02-01T04:20:25.619Z INFO  [signal_backtester] csv: open,1675188645,Long,403.610445,
2023-02-01T04:20:25.643Z INFO  [signal_backtester] csv: close,1675189136,Long,403.54955,DirectionChange
2023-02-01T04:20:25.643Z INFO  [signal_backtester] csv: open,1675189136,Short,403.54955,
2023-02-01T04:20:25.710Z INFO  [signal_backtester] csv: close,1675190456,Short,403.88047875,DirectionChange
2023-02-01T04:20:25.710Z INFO  [signal_backtester] csv: open,1675190456,Long,403.88047875,
2023-02-01T04:20:25.843Z INFO  [signal_backtester] csv: close,1675192791,Long,404.07948375,DirectionChange
2023-02-01T04:20:25.843Z INFO  [signal_backtester] csv: open,1675192791,Short,404.07948375,
2023-02-01T04:20:25.843Z INFO  [signal_backtester] csv: close,1675192795,Short,404.2305225,DirectionChange
2023-02-01T04:20:25.843Z INFO  [signal_backtester] csv: open,1675192795,Long,404.2305225,
2023-02-01T04:20:25.843Z INFO  [signal_backtester] csv: close,1675192800,Long,404.10948,DirectionChange
2023-02-01T04:20:25.843Z INFO  [signal_backtester] csv: open,1675192800,Short,404.10948,
2023-02-01T04:20:25.844Z INFO  [signal_backtester] csv: close,1675192805,Short,404.24052375,DirectionChange
2023-02-01T04:20:25.844Z INFO  [signal_backtester] csv: open,1675192805,Long,404.24052375,
2023-02-01T04:20:25.855Z INFO  [signal_backtester] csv: close,1675192996,Long,404.07948375,DirectionChange
2023-02-01T04:20:25.855Z INFO  [signal_backtester] csv: open,1675192996,Short,404.07948375,
2023-02-01T04:20:25.857Z INFO  [signal_backtester] csv: close,1675193021,Short,404.21052000000003,DirectionChange
2023-02-01T04:20:25.857Z INFO  [signal_backtester] csv: open,1675193021,Long,404.21052000000003,
2023-02-01T04:20:25.862Z INFO  [signal_backtester] csv: close,1675193105,Long,404.09948125,DirectionChange
2023-02-01T04:20:25.862Z INFO  [signal_backtester] csv: open,1675193105,Short,404.09948125,
2023-02-01T04:20:25.863Z INFO  [signal_backtester] csv: close,1675193120,Short,404.2305225,DirectionChange
2023-02-01T04:20:25.863Z INFO  [signal_backtester] csv: open,1675193120,Long,404.2305225,
2023-02-01T04:20:25.864Z INFO  [signal_backtester] csv: close,1675193140,Long,404.0894825,DirectionChange
2023-02-01T04:20:25.864Z INFO  [signal_backtester] csv: open,1675193140,Short,404.0894825,
2023-02-01T04:20:25.928Z INFO  [signal_backtester] csv: close,1675194145,Short,404.5505625,DirectionChange
2023-02-01T04:20:25.928Z INFO  [signal_backtester] csv: open,1675194145,Long,404.5505625,
2023-02-01T04:20:25.929Z INFO  [signal_backtester] csv: close,1675194160,Long,404.43953873749996,DirectionChange
2023-02-01T04:20:25.929Z INFO  [signal_backtester] csv: open,1675194160,Short,404.43953873749996,
2023-02-01T04:20:25.940Z INFO  [signal_backtester] csv: close,1675194320,Short,404.56056375,DirectionChange
2023-02-01T04:20:25.940Z INFO  [signal_backtester] csv: open,1675194320,Long,404.56056375,
2023-02-01T04:20:25.941Z INFO  [signal_backtester] csv: close,1675194330,Long,404.45943625,DirectionChange
2023-02-01T04:20:25.941Z INFO  [signal_backtester] csv: open,1675194330,Short,404.45943625,
2023-02-01T04:20:25.944Z INFO  [signal_backtester] csv: close,1675194380,Short,404.5896673875,DirectionChange
2023-02-01T04:20:25.944Z INFO  [signal_backtester] csv: open,1675194380,Long,404.5896673875,
2023-02-01T04:20:25.945Z INFO  [signal_backtester] csv: close,1675194390,Long,404.4403386375,DirectionChange
2023-02-01T04:20:25.945Z INFO  [signal_backtester] csv: open,1675194390,Short,404.4403386375,
2023-02-01T04:20:25.951Z INFO  [signal_backtester] csv: close,1675194485,Short,404.56056375,DirectionChange
2023-02-01T04:20:25.951Z INFO  [signal_backtester] csv: open,1675194485,Long,404.56056375,
2023-02-01T04:20:25.951Z INFO  [signal_backtester] csv: close,1675194490,Long,404.434439375,DirectionChange
2023-02-01T04:20:25.951Z INFO  [signal_backtester] csv: open,1675194490,Short,404.434439375,
2023-02-01T04:20:25.953Z INFO  [signal_backtester] csv: close,1675194515,Short,404.570565,DirectionChange
2023-02-01T04:20:25.953Z INFO  [signal_backtester] csv: open,1675194515,Long,404.570565,
2023-02-01T04:20:25.953Z INFO  [signal_backtester] csv: close,1675194520,Long,404.4494375,DirectionChange
2023-02-01T04:20:25.953Z INFO  [signal_backtester] csv: open,1675194520,Short,404.4494375,
2023-02-01T04:20:25.956Z INFO  [signal_backtester] csv: close,1675194550,Short,404.56056375,DirectionChange
2023-02-01T04:20:25.956Z INFO  [signal_backtester] csv: open,1675194550,Long,404.56056375,
2023-02-01T04:20:26.044Z INFO  [signal_backtester] csv: close,1675195855,Long,404.33945124999997,DirectionChange
2023-02-01T04:20:26.044Z INFO  [signal_backtester] csv: open,1675195855,Short,404.33945124999997,
2023-02-01T04:20:26.146Z INFO  [signal_backtester] csv: close,1675197265,Short,404.27552812500005,DirectionChange
2023-02-01T04:20:26.146Z INFO  [signal_backtester] csv: open,1675197265,Long,404.27552812500005,
2023-02-01T04:20:26.146Z INFO  [signal_backtester] csv: close,1675197270,Long,404.174471875,DirectionChange
2023-02-01T04:20:26.146Z INFO  [signal_backtester] csv: open,1675197270,Short,404.174471875,
2023-02-01T04:20:26.148Z INFO  [signal_backtester] csv: close,1675197285,Short,404.375540625,DirectionChange
2023-02-01T04:20:26.148Z INFO  [signal_backtester] csv: open,1675197285,Long,404.375540625,
2023-02-01T04:20:26.263Z INFO  [signal_backtester] csv: close,1675198791,Long,406.41919125000004,ProfitLimit
```