# backtesting

## generate_dates

~~~sh
START_DATE='2022-01-01'
END_DATE='2023-03-03'
rust-script src/date_generator.rs $START_DATE $END_DATE > ./output/dates.txt
~~~

## scrape_dates

~~~sh
# make sure POLYGON_API_KEY environment variable is set
TICKER='SPY'
RESOLUTION='5'
mkdir data/
cat ./output/dates.txt | while read DATE
do
  ./scrape.sh $DATE $TICKER $RESOLUTION
  sleep 15 # due to API request limits
done
~~~

## transform

~~~sh
rust-script src/transform.rs > ./output/candles.csv
~~~

## generate_signals

~~~sh
CANDLES_FILE="./output/candles.csv"
rust-script src/signal_generator.rs $CANDLES_FILE > ./output/signals.csv
~~~

## generate_trades

~~~sh
SIGNALS_FILE="./output/signals.csv"
rust-script src/trade_generator.rs $SIGNALS_FILE > ./output/trades.csv
~~~

## backtest_trades

~~~sh
CANDLES_FILE="./output/candles.csv"
TRADES_FILE="./output/trades.csv"
rust-script src/trade_backtester.rs $CANDLES_FILE $TRADES_FILE > ./output/results.csv
~~~
