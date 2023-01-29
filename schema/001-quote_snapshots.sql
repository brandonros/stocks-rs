CREATE TABLE IF NOT EXISTS quote_snapshots (
  symbol TEXT,
  scraped_at INTEGER,
  ask_price REAL,
  bid_price REAL,
  last_trade_price REAL,
  PRIMARY KEY (symbol, scraped_at)
);
