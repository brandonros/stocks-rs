# backtesting

## init_directories

~~~sh
mkdir data/
mkdir output/
~~~

## generate_dates

~~~sh
START_DATE='2022-01-01'
END_DATE='2023-03-03'
rust-script src/date_generator.rs $START_DATE $END_DATE > ./output/dates.txt
~~~

## scrape_polygon

~~~sh
# make sure POLYGON_API_KEY environment variable is set
SYMBOL='SPY'
RESOLUTION='5'
cat ./output/dates.txt | while read DATE
do
  FROM_TIMESTAMP=$(date -j -f "%Y-%m-%d %I:%M:%S %p" "$DATE 04:00:00 AM" "+%s")
  FROM_TIMESTAMP=$((FROM_TIMESTAMP * 1000))
  TO_TIMESTAMP=$((FROM_TIMESTAMP + 57600000)) # 60 * 16 hours = 57600 seconds = 8pm
  LIMIT=1000
  curl "https://api.polygon.io/v2/aggs/ticker/$SYMBOL/range/$RESOLUTION/minute/$FROM_TIMESTAMP/$TO_TIMESTAMP?adjusted=true&sort=asc&limit=$LIMIT&apiKey=$POLYGON_API_KEY" -o "./data/polygon-$SYMBOL-$RESOLUTION-$FROM_TIMESTAMP-$TO_TIMESTAMP.json"
  # TODO: jq result to make sure no errors
  sleep 15 # due to API request limits
done
~~~

## build_candles

~~~sh
rust-script src/candle_builder.rs > ./output/candles.csv
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
