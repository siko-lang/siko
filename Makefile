stdtest:
	./stdtest.sh

test: Siko/target/release/
	cd Siko && cargo run --release && cd .. && ./siko test.sk