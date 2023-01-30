lint:
		cargo clippy --fix -- -A clippy::needless_return -A clippy::bool_comparison -W clippy::implicit_return -A clippy::new_without_default

scrape_quotes:
		while [ 1 ]; do cargo run --bin quote_scraper; sleep 1; done

scrape_candles:
		while [ 1 ]; do cargo run --bin candle_scraper; sleep 1; done

scrape_signals:
		while [ 1 ]; do cargo run --bin signal_scraper; sleep 1; done
