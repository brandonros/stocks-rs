lint:
		cargo clippy --fix -- -A clippy::needless_return -A clippy::bool_comparison -A clippy::new_without_default -A clippy::unnecessary_unwrap -A clippy::map_flatten -A clippy::comparison_to_empty -A clippy::len_zero -A clippy::to_string_in_format_args

scrape_quotes:
		while [ 1 ]; do cargo run --bin quote_scraper; sleep 1; done

scrape_candles:
		while [ 1 ]; do cargo run --bin candle_scraper; sleep 1; done

scrape_signals:
		while [ 1 ]; do cargo run --bin signal_scraper; sleep 1; done
