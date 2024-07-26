cargo_watch_test:
	cargo watch -q -c -x "test -- --nocapture"

cargo_fmt:
	cargo +nightly fmt --all