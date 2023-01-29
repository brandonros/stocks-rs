CREATE TABLE IF NOT EXISTS candles (
  symbol TEXT,
  resolution TEXT,
  scraped_at INTEGER,
  timestamp INTEGER,
  open REAL,
  high REAL,
  low REAL,
  close REAL,
  volume INTEGER,
  PRIMARY KEY (symbol, resolution, scraped_at, timestamp)
)