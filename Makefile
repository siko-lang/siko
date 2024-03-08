stdtest:
	./stdtest.sh

test: sikors/target/release/
	cd sikors && cargo run --release && cd .. && ./siko test.sk